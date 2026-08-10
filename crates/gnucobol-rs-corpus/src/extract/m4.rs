//! Minimal M4-aware scanner for Autotest `.at` sources.
//!
//! The GnuCOBOL suite sources are M4 with Autotest macros. A regex-only parse would mis-handle
//! multiline `AT_DATA` bodies and quoted content, so the extractor uses a small recursive-descent
//! scanner with the exact quoting rules that matter here:
//!
//! * `[...]` quoting with **nesting** (M4's bracket quotes nest; a `]` closes only the innermost
//!   open `[`);
//! * `#` comments to end-of-line at the top level (never inside a quote);
//! * `dnl` (the M4 "delete to next line" primitive) at the top level;
//! * macro invocations `NAME(arg1, arg2, ...)` with quoted or unquoted arguments.
//!
//! The scanner fails closed: an unterminated quote or unbalanced argument list is a typed error,
//! never a guess.

/// One scanned unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A macro invocation: `NAME` + already-split arguments (quoting stripped one level;
    /// nested `[...]` inside an argument stays bracketed, as M4 would keep it). `line` is the
    /// 1-based source line where the macro name starts.
    Macro {
        name: String,
        args: Vec<String>,
        line: usize,
    },
    /// Plain top-level text (comments and `dnl` lines are consumed, never emitted).
    Text(String),
}

/// A cursor over the source.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(src: &'a str) -> Reader<'a> {
        Reader { src, pos: 0 }
    }

    pub fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    pub fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Skip ASCII whitespace (spaces, tabs, newlines, CR).
    pub fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    /// Skip a `#` comment to the end of the line (top level only; never called inside quotes).
    pub fn skip_hash_comment(&mut self) {
        if self.peek() == Some('#') {
            while let Some(c) = self.peek() {
                if c == '\n' {
                    break;
                }
                self.bump();
            }
        }
    }

    /// Skip a `dnl` primitive (word `dnl` followed by whitespace/EOL): everything to EOL.
    pub fn skip_dnl(&mut self) {
        let rest = self.rest();
        if let Some(i) = rest.find("dnl") {
            let before = &rest[..i];
            if before.ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_') {
                return; // a longer identifier ending in dnl, e.g. "adnl" -- not the primitive
            }
            let after = &rest[i + 3..];
            let ok = after
                .chars()
                .next()
                .map(|c| c.is_ascii_whitespace())
                .unwrap_or(true);
            if ok {
                self.pos += i + 3;
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.bump();
                }
            }
        }
    }

    /// Read a quoted string `[ ... ]` with nesting; returns the inner content with ONE level of
    /// quoting removed. Nested `[...]` pairs inside remain bracketed. Fails closed on an
    /// unterminated quote.
    pub fn read_quoted(&mut self) -> Result<String, String> {
        if self.peek() != Some('[') {
            return Err(format!(
                "read_quoted: expected '[' at byte {}, got {:?}",
                self.pos,
                self.peek()
            ));
        }
        self.bump();
        let mut depth = 1usize;
        let start = self.pos;
        while self.pos < self.src.len() {
            let c = self.peek().expect("in bounds");
            match c {
                '[' => {
                    depth += 1;
                    self.bump();
                }
                ']' => {
                    depth -= 1;
                    self.bump();
                    if depth == 0 {
                        let end = self.pos - 1; // exclude the closing bracket
                        return Ok(self.src[start..end].to_string());
                    }
                }
                _ => {
                    self.bump();
                }
            }
        }
        Err(format!(
            "unterminated quoted string starting at byte {start} (depth {depth})"
        ))
    }

    /// Read an identifier `[A-Za-z_][A-Za-z0-9_]*`.
    pub fn read_ident(&mut self) -> Option<String> {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    /// At the current position, try to read a macro invocation `NAME(...)` -- or a zero-argument
    /// invocation of a known `AT_*` macro (`AT_CLEANUP`, `AT_COLOR_TESTS`), which M4 expands
    /// without parentheses. Returns `Ok(None)` when the position is not the start of a macro
    /// call. Arguments are split on top-level commas; each argument may be a `[...]` quote
    /// (read with nesting) or a bare token run. Fails closed on unbalanced arguments or quotes.
    pub fn read_macro(&mut self) -> Result<Option<(String, Vec<String>)>, String> {
        let save = self.pos;
        let name = match self.read_ident() {
            Some(n) => n,
            None => return Ok(None),
        };
        // only spaces/tabs may separate the name from its '(' (never a newline: an ident at the
        // end of a line is not a macro call)
        let mut j = self.pos;
        while j < self.src.len() && matches!(self.src.as_bytes()[j], b' ' | b'\t') {
            j += 1;
        }
        if !self.src[j..].starts_with('(') {
            if name.starts_with("AT_") {
                // zero-argument AT_* invocation (M4 expands AT_CLEANUP / AT_COLOR_TESTS etc.
                // without parens)
                return Ok(Some((name, Vec::new())));
            }
            self.pos = save;
            return Ok(None);
        }
        self.pos = j;
        self.bump(); // '('
        let mut args: Vec<String> = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(')') {
                self.bump();
                break;
            }
            if self.peek() == Some('[') {
                args.push(self.read_quoted()?);
            } else {
                // bare (unquoted) argument: read until a top-level ',' or ')' without
                // consuming either -- the outer match below consumes the separator
                let mut s = String::new();
                loop {
                    match self.peek() {
                        None => {
                            return Err(format!(
                                "unbalanced macro arguments for {name} (started at byte {save})"
                            ))
                        }
                        Some(')') | Some(',') => break,
                        Some(c) => {
                            s.push(c);
                            self.bump();
                        }
                    }
                }
                args.push(s);
            }
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                    continue;
                }
                Some(')') => {
                    self.bump();
                    break;
                }
                None => {
                    return Err(format!(
                        "unbalanced macro arguments for {name} (started at byte {save})"
                    ))
                }
                Some(c) => {
                    return Err(format!(
                        "unexpected {:?} in argument list of {name} at byte {}",
                        c, self.pos
                    ))
                }
            }
        }
        Ok(Some((name, args)))
    }
}

/// Scan the whole source into a flat item stream, consuming top-level comments/`dnl` and blank
/// text. Text between macros is kept as [`Item::Text`] so the AT parser can detect unexpected
/// content (fail-closed on constructs outside the known macro surface).
pub fn scan(src: &str) -> Result<Vec<Item>, String> {
    let mut r = Reader::new(src);
    let mut out = Vec::new();
    let mut text = String::new();
    let mut line = 1usize;
    while !r.eof() {
        let before = r.pos;
        // top-level trivia first (ws / # comment / dnl): consumed without emission, but only
        // leading trivia may drop the text buffer -- mid-run trivia belongs to the text run.
        r.skip_ws();
        r.skip_hash_comment();
        r.skip_dnl();
        if r.pos != before {
            line += src[before..r.pos].bytes().filter(|&b| b == b'\n').count();
            if text.trim().is_empty() {
                text = String::new();
            } else {
                text.push_str(&src[before..r.pos]);
            }
            continue;
        }
        let item_line = line;
        match r.read_macro()? {
            Some((name, args)) => {
                line += src[before..r.pos].bytes().filter(|&b| b == b'\n').count();
                if !text.trim().is_empty() {
                    out.push(Item::Text(text.trim_end().to_string()));
                    text = String::new();
                }
                out.push(Item::Macro {
                    name,
                    args,
                    line: item_line,
                });
            }
            None => {
                if let Some(c) = r.bump() {
                    if c == '\n' {
                        line += 1;
                    }
                    text.push(c);
                }
            }
        }
    }
    if !text.trim().is_empty() {
        out.push(Item::Text(text.trim_end().to_string()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_nesting() {
        let mut r = Reader::new("[a [b] c] tail");
        assert_eq!(r.read_quoted().unwrap(), "a [b] c");
        assert_eq!(r.rest(), " tail");
    }

    #[test]
    fn quoted_unterminated_fails_closed() {
        let mut r = Reader::new("[abc");
        let e = r.read_quoted().unwrap_err();
        assert!(e.contains("unterminated"), "{e}");
    }

    #[test]
    fn macro_with_quoted_args_multiline() {
        let src = "AT_DATA([prog.cob], [\n       IDENTIFICATION DIVISION.\n])\n";
        let items = scan(src).unwrap();
        assert_eq!(items.len(), 1);
        match &items[0] {
            Item::Macro { name, args, line } => {
                assert_eq!(name, "AT_DATA");
                assert_eq!(*line, 1);
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], "prog.cob");
                assert!(args[1].contains("IDENTIFICATION DIVISION"));
            }
            _ => panic!("expected macro"),
        }
    }

    #[test]
    fn check_four_args() {
        let src = "AT_CHECK([$COMPILE_ONLY prog.cob], [1], [],\n[prog.cob:9: error: bad\n])\n";
        let items = scan(src).unwrap();
        match &items[0] {
            Item::Macro { name, args, line } => {
                assert_eq!(name, "AT_CHECK");
                assert_eq!(*line, 1);
                assert_eq!(args.len(), 4);
                assert_eq!(args[0], "$COMPILE_ONLY prog.cob");
                assert_eq!(args[1], "1");
                assert_eq!(args[2], "");
                assert!(args[3].contains("error: bad"));
            }
            _ => panic!("expected macro"),
        }
    }

    #[test]
    fn hash_comment_and_dnl_skipped() {
        let src = "# a comment\ndnl gone\nAT_SETUP([t])\n";
        let items = scan(src).unwrap();
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0], Item::Macro { name, .. } if name == "AT_SETUP"));
        assert!(matches!(&items[0], Item::Macro { line: 3, .. }));
    }

    #[test]
    fn hash_inside_quote_is_content() {
        let src = "AT_DATA([f], [COBOL # not a comment\n])\n";
        let items = scan(src).unwrap();
        match &items[0] {
            Item::Macro { args, .. } => assert!(args[1].contains("# not a comment")),
            _ => panic!(),
        }
    }

    #[test]
    fn unquoted_args() {
        let mut r = Reader::new("M(a, b)");
        let (name, args) = r.read_macro().unwrap().unwrap();
        assert_eq!(name, "M");
        assert_eq!(args, vec!["a", "b"]);
    }
}
