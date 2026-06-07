//! Copybook `COPY` expansion (`GNURUST.5`) with `COPY ... REPLACING` (`GNURUST.6`): splice
//! `COPY name [REPLACING ==old== BY ==new== …].` copybooks into the source, recursively, with cycle
//! detection, depth/size limits, a **provenance map**, and GnuCOBOL-compatible **whole-text-word**
//! pseudo-text replacement. Verified against the GnuCOBOL preprocessor (`cobc -P`) at text-word
//! granularity.
//!
//! **Text-word fidelity (the senior rule).** Replacement is **not** string substitution. A text
//! word is a maximal run of `[_-0-9A-Za-z]` plus high bytes — exactly GnuCOBOL's `is_word`
//! (`cobc/replace.c:603`). `==AA==` therefore does **not** touch `AA-X` or `BAAB` (single words),
//! while the `:tag:` idiom works because `:` (a non-word char) splits `:PFX:` into its own word.
//! REPLACING **composes across nesting**: an outer `COPY`'s REPLACING does **not** alter a nested
//! `COPY`'s operands, but it **does** penetrate the text the nested copy brings in — applied after
//! the nested copy's own REPLACING (all verified against `cobc -P`).
//!
//! **Sealed subset:** line-oriented `COPY name.` and `COPY name REPLACING ==p== BY ==q== ….`
//! (pseudo-text operands), possibly spanning lines up to the terminating period. **Fail closed**
//! (typed [`CopyError`]) on: recursion, missing copybook, over-deep/over-large expansion, and
//! any non-pseudo-text REPLACING form (`LEADING`/`TRAILING`, identifier/literal operands,
//! unterminated `==`) — deferred, never half-applied.

/// Resolves a copybook name to its source text (the caller owns the search path / filesystem; the
/// expander itself is pure given a resolver).
pub trait CopyResolver {
    /// Return the copybook text for `name`, or `None` if it cannot be found.
    fn resolve(&self, name: &str) -> Option<String>;
}

/// Where an expanded line came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// `"<main>"` for the top-level source, else the copybook name.
    pub file: String,
    /// 1-based line number within `file`.
    pub line: usize,
}

/// The result of expansion: the spliced lines and a parallel provenance map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expanded {
    pub lines: Vec<String>,
    pub provenance: Vec<Provenance>,
}

impl Expanded {
    /// The expanded source as one string (lines joined by `\n`).
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

/// Why expansion failed (fail closed).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CopyError {
    /// A `COPY`ed copybook could not be resolved.
    Missing(String),
    /// A `COPY` cycle (a copybook that, directly or transitively, copies itself).
    Recursive(String),
    /// `COPY` nesting deeper than the limit.
    TooDeep,
    /// Expansion produced more than the total-size limit (resource guard, `GNURUST.DOS.0`).
    TooLarge,
    /// A `COPY ... REPLACING` form outside the sealed pseudo-text subset (`LEADING`/`TRAILING`,
    /// identifier/literal operands, unterminated `==`) — deferred, not half-applied.
    ReplacingDeferred,
    /// A `COPY` statement that could not be parsed (no name, missing `BY`, …).
    MalformedCopy,
}

impl core::fmt::Display for CopyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CopyError::Missing(n) => write!(f, "copybook '{n}' not found"),
            CopyError::Recursive(n) => write!(f, "recursive COPY of '{n}'"),
            CopyError::TooDeep => write!(f, "COPY nesting too deep"),
            CopyError::TooLarge => write!(f, "expansion exceeds size limit"),
            CopyError::ReplacingDeferred => {
                write!(f, "unsupported COPY ... REPLACING form (deferred)")
            }
            CopyError::MalformedCopy => write!(f, "malformed COPY statement"),
        }
    }
}

impl std::error::Error for CopyError {}

const MAX_DEPTH: usize = 50;
const MAX_LINES: usize = 1_000_000;

// --- Text words (GnuCOBOL `is_word`, cobc/replace.c:603) ---------------------------------------

#[inline]
fn is_word_char(c: char) -> bool {
    c == '_' || c == '-' || c.is_ascii_alphanumeric() || (c as u32) >= 128
}

/// A lexical token: a text word, or a single non-word character (separator/punctuation/space).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Word(String),
    Sep(char),
}

impl Tok {
    fn text(&self) -> String {
        match self {
            Tok::Word(w) => w.clone(),
            Tok::Sep(c) => c.to_string(),
        }
    }
    /// Match for REPLACING: words compare case-insensitively; separators compare exactly.
    fn matches(&self, other: &Tok) -> bool {
        match (self, other) {
            (Tok::Word(a), Tok::Word(b)) => a.eq_ignore_ascii_case(b),
            (Tok::Sep(a), Tok::Sep(b)) => a == b,
            _ => false,
        }
    }
}

/// Tokenize text into words (maximal word-char runs) and single-character separators.
fn tokenize(s: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut word = String::new();
    for c in s.chars() {
        if is_word_char(c) {
            word.push(c);
        } else {
            if !word.is_empty() {
                out.push(Tok::Word(std::mem::take(&mut word)));
            }
            out.push(Tok::Sep(c));
        }
    }
    if !word.is_empty() {
        out.push(Tok::Word(word));
    }
    out
}

/// Apply one REPLACING pair-list to a line's text, left-to-right, first-listed-pair-wins, at
/// text-word granularity. Adjacent words re-concatenate (GnuCOBOL's prequeue merge) simply because
/// tokens are emitted back-to-back.
fn apply_pairs(line: &str, pairs: &[(Vec<Tok>, Vec<Tok>)]) -> String {
    if pairs.is_empty() {
        return line.to_string();
    }
    let toks = tokenize(line);
    let mut out = String::new();
    let mut i = 0;
    'scan: while i < toks.len() {
        for (from, to) in pairs {
            if !from.is_empty() && i + from.len() <= toks.len() {
                let hit = from
                    .iter()
                    .zip(&toks[i..i + from.len()])
                    .all(|(p, t)| p.matches(t));
                if hit {
                    for t in to {
                        out.push_str(&t.text());
                    }
                    i += from.len();
                    continue 'scan;
                }
            }
        }
        out.push_str(&toks[i].text());
        i += 1;
    }
    out
}

/// Apply a chain of REPLACING pair-lists (innermost copybook first, then each ancestor) to a line.
/// The outer copy's REPLACING penetrates the text a nested copy brings in (after the nested copy's
/// own REPLACING), exactly as `cobc -P` does — but never the nested copy's operands.
fn apply_chain(line: &str, chain: &[&[(Vec<Tok>, Vec<Tok>)]]) -> String {
    let mut s = line.to_string();
    for pairs in chain {
        s = apply_pairs(&s, pairs);
    }
    s
}

// --- COPY statement parsing --------------------------------------------------------------------

struct CopyStmt {
    name: String,
    pairs: Vec<(Vec<Tok>, Vec<Tok>)>,
}

/// Does the first text word of `line` (ignoring leading spaces) equal `COPY`?
fn line_starts_copy(line: &str) -> bool {
    line.split_whitespace()
        .next()
        .is_some_and(|w| w.eq_ignore_ascii_case("COPY"))
}

/// Has the gathered COPY statement reached its terminating period (a `.` outside `==` pseudo-text)?
fn copy_terminated(buf: &str) -> bool {
    let b = buf.as_bytes();
    let mut in_pseudo = false;
    let mut i = 0;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'=' && b[i + 1] == b'=' {
            in_pseudo = !in_pseudo;
            i += 2;
            continue;
        }
        if !in_pseudo && b[i] == b'.' {
            return true;
        }
        i += 1;
    }
    false
}

/// Parse `COPY name [REPLACING ==p== BY ==q== …] .` from a gathered statement string.
fn parse_copy_statement(stmt: &str) -> Result<CopyStmt, CopyError> {
    let chars: Vec<char> = stmt.chars().collect();
    let mut i = 0;
    let skip_ws = |i: &mut usize| {
        while *i < chars.len() && chars[*i].is_whitespace() {
            *i += 1;
        }
    };
    let read_word = |i: &mut usize| -> String {
        let mut w = String::new();
        while *i < chars.len() && is_word_char(chars[*i]) {
            w.push(chars[*i]);
            *i += 1;
        }
        w
    };

    skip_ws(&mut i);
    let kw = read_word(&mut i);
    if !kw.eq_ignore_ascii_case("COPY") {
        return Err(CopyError::MalformedCopy);
    }
    skip_ws(&mut i);
    let name = read_word(&mut i);
    if name.is_empty() {
        return Err(CopyError::MalformedCopy);
    }
    skip_ws(&mut i);

    // `COPY name.`
    if i < chars.len() && chars[i] == '.' {
        return Ok(CopyStmt {
            name: name.to_ascii_uppercase(),
            pairs: Vec::new(),
        });
    }

    let next = read_word(&mut i);
    if !next.eq_ignore_ascii_case("REPLACING") {
        return Err(CopyError::MalformedCopy);
    }

    let mut pairs = Vec::new();
    loop {
        skip_ws(&mut i);
        if i < chars.len() && chars[i] == '.' {
            break;
        }
        // Only pseudo-text operands are sealed; anything else (LEADING/TRAILING/identifier) defers.
        let from = read_pseudo(&chars, &mut i).ok_or(CopyError::ReplacingDeferred)?;
        skip_ws(&mut i);
        let by = read_word(&mut i);
        if !by.eq_ignore_ascii_case("BY") {
            return Err(CopyError::ReplacingDeferred);
        }
        skip_ws(&mut i);
        let to = read_pseudo(&chars, &mut i).ok_or(CopyError::ReplacingDeferred)?;
        pairs.push((tokenize(&from), tokenize(&to)));
        if pairs.len() > 256 {
            return Err(CopyError::ReplacingDeferred);
        }
    }
    Ok(CopyStmt {
        name: name.to_ascii_uppercase(),
        pairs,
    })
}

/// Read a `== … ==` pseudo-text operand at `*i`, returning its inner content, or `None` if the
/// next operand is not pseudo-text (an unsupported REPLACING form).
fn read_pseudo(chars: &[char], i: &mut usize) -> Option<String> {
    if *i + 1 >= chars.len() || chars[*i] != '=' || chars[*i + 1] != '=' {
        return None;
    }
    *i += 2;
    let mut content = String::new();
    while *i + 1 < chars.len() {
        if chars[*i] == '=' && chars[*i + 1] == '=' {
            *i += 2;
            return Some(content);
        }
        content.push(chars[*i]);
        *i += 1;
    }
    None // unterminated ==
}

// --- Expansion ---------------------------------------------------------------------------------

/// Expand all `COPY` statements in `source` using `resolver`.
pub fn expand(source: &str, resolver: &impl CopyResolver) -> Result<Expanded, CopyError> {
    let mut out = Expanded {
        lines: Vec::new(),
        provenance: Vec::new(),
    };
    let mut stack: Vec<String> = Vec::new();
    expand_into("<main>", source, resolver, &mut out, &mut stack, &[])?;
    Ok(out)
}

fn expand_into(
    file: &str,
    text: &str,
    resolver: &impl CopyResolver,
    out: &mut Expanded,
    stack: &mut Vec<String>,
    chain: &[&[(Vec<Tok>, Vec<Tok>)]],
) -> Result<(), CopyError> {
    if stack.len() > MAX_DEPTH {
        return Err(CopyError::TooDeep);
    }
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if out.lines.len() > MAX_LINES {
            return Err(CopyError::TooLarge);
        }
        if line_starts_copy(lines[i]) {
            // Gather the (possibly multi-line) COPY statement up to its terminating period.
            let mut buf = lines[i].to_string();
            while !copy_terminated(&buf) && i + 1 < lines.len() {
                i += 1;
                buf.push('\n');
                buf.push_str(lines[i]);
            }
            let stmt = parse_copy_statement(&buf)?;
            if stack.iter().any(|n| n == &stmt.name) {
                return Err(CopyError::Recursive(stmt.name));
            }
            let body = resolver
                .resolve(&stmt.name)
                .ok_or_else(|| CopyError::Missing(stmt.name.clone()))?;
            stack.push(stmt.name.clone());
            // The nested copy is expanded with ITS OWN replacing innermost, then this chain — the
            // ancestor REPLACING penetrates the brought-in text, but not the nested operands.
            let mut new_chain: Vec<&[(Vec<Tok>, Vec<Tok>)]> = Vec::with_capacity(chain.len() + 1);
            new_chain.push(&stmt.pairs);
            new_chain.extend_from_slice(chain);
            expand_into(&stmt.name, &body, resolver, out, stack, &new_chain)?;
            stack.pop();
            i += 1;
        } else {
            out.lines.push(apply_chain(lines[i], chain));
            out.provenance.push(Provenance {
                file: file.to_string(),
                line: i + 1,
            });
            i += 1;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Map(HashMap<String, String>);
    impl CopyResolver for Map {
        fn resolve(&self, name: &str) -> Option<String> {
            self.0.get(name).cloned()
        }
    }
    fn map(pairs: &[(&str, &str)]) -> Map {
        Map(pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect())
    }

    #[test]
    fn basic_copy_with_provenance() {
        let src = "01 R.\n       COPY REC.\n01 S.";
        let r = map(&[("REC", "05 A PIC 9(3).\n05 B PIC X(5).")]);
        let e = expand(src, &r).unwrap();
        assert_eq!(
            e.lines,
            vec!["01 R.", "05 A PIC 9(3).", "05 B PIC X(5).", "01 S."]
        );
        assert_eq!(
            e.provenance[1],
            Provenance {
                file: "REC".into(),
                line: 1
            }
        );
    }

    #[test]
    fn replacing_is_whole_text_word() {
        // ==AA== must NOT touch AA-X / KEEP-AA (single words); :PFX: must (colon-delimited word).
        let r = map(&[(
            "T",
            "05 :PFX:-AA PIC 9.\n05 KEEP-AA PIC 9.\n05 BB-:PFX: PIC X.",
        )]);
        let e = expand(
            "COPY T REPLACING ==:PFX:== BY ==CUST== ==AA== BY ==ZZ==.",
            &r,
        )
        .unwrap();
        assert_eq!(
            e.lines,
            vec![
                "05 CUST-AA PIC 9.",
                "05 KEEP-AA PIC 9.",
                "05 BB-CUST PIC X."
            ]
        );
    }

    #[test]
    fn nested_replacing_composes_but_not_operands() {
        // The outer REPLACING does NOT alter the nested COPY's operands (`==:PFX:==` still matches T),
        // but it DOES penetrate the text the nested copy brings in (AMOUNT -> QQ), after the nested
        // copy's own replacing — exactly as `cobc -P` does.
        let r = map(&[
            (
                "OUT",
                "05 :PFX:-HEAD PIC X.\nCOPY T REPLACING ==:PFX:== BY ==INNER==.",
            ),
            ("T", "05 :PFX:-ID PIC 9.\n05 AMOUNT PIC 9."),
        ]);
        let e = expand(
            "COPY OUT REPLACING ==:PFX:== BY ==OUTER== ==AMOUNT== BY ==QQ==.",
            &r,
        )
        .unwrap();
        assert_eq!(
            e.lines,
            vec!["05 OUTER-HEAD PIC X.", "05 INNER-ID PIC 9.", "05 QQ PIC 9."]
        );
    }

    #[test]
    fn fails_closed() {
        let r = map(&[("SELF", "COPY SELF."), ("R", "05 A PIC X.")]);
        assert_eq!(
            expand("COPY SELF.", &r),
            Err(CopyError::Recursive("SELF".into()))
        );
        assert_eq!(
            expand("COPY NOPE.", &map(&[])),
            Err(CopyError::Missing("NOPE".into()))
        );
        // Unsupported REPLACING forms defer (not half-applied).
        assert_eq!(
            expand("COPY R REPLACING LEADING ==A== BY ==B==.", &r),
            Err(CopyError::ReplacingDeferred)
        );
        assert_eq!(
            expand("COPY R REPLACING AAA BY BBB.", &r),
            Err(CopyError::ReplacingDeferred)
        );
    }
}
