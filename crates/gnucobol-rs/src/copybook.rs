//! Copybook `COPY` expansion (`GNURUST.5`): splice `COPY name.` copybooks into the source,
//! recursively, with cycle detection, depth/size limits, and a **provenance map** (each expanded
//! line → the file and original line it came from). Verified against the GnuCOBOL preprocessor
//! (`cobc -P`) at text-word granularity.
//!
//! **Sealed subset:** a line-oriented `COPY <name>.` statement (the name a COBOL word; the whole
//! statement on its own line). Nested `COPY` is expanded; recursion, a missing copybook, excessive
//! depth, and excessive total size **fail closed** with a typed [`CopyError`]. `COPY ... REPLACING`
//! (the text-word replacement algorithm) is a separate future court (`GNURUST.6` / `GNURUST.REPLACEALG.0`)
//! and is **rejected** here, not half-applied.

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
    /// A `COPY ... REPLACING` statement — deferred to the REPLACING court (`GNURUST.6`).
    ReplacingDeferred,
}

impl core::fmt::Display for CopyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CopyError::Missing(n) => write!(f, "copybook '{n}' not found"),
            CopyError::Recursive(n) => write!(f, "recursive COPY of '{n}'"),
            CopyError::TooDeep => write!(f, "COPY nesting too deep"),
            CopyError::TooLarge => write!(f, "expansion exceeds size limit"),
            CopyError::ReplacingDeferred => {
                write!(f, "COPY ... REPLACING is deferred (GNURUST.6)")
            }
        }
    }
}

impl std::error::Error for CopyError {}

const MAX_DEPTH: usize = 50;
const MAX_LINES: usize = 1_000_000;

/// If `line` is a bare `COPY <name>.` statement, return the copybook name (uppercased). A trailing
/// `REPLACING` makes it a deferred form (handled by the caller via a separate check).
fn parse_copy(line: &str) -> Option<(String, bool)> {
    let t = line.trim();
    let mut words = t.split_whitespace();
    let first = words.next()?;
    if !first.eq_ignore_ascii_case("COPY") {
        return None;
    }
    let name = words.next()?;
    // name is a COBOL word optionally followed by '.', or the '.' is a separate token.
    let (name, had_dot) = match name.strip_suffix('.') {
        Some(n) => (n, true),
        None => (name, false),
    };
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }
    let rest: Vec<&str> = words.collect();
    let replacing = rest.iter().any(|w| w.eq_ignore_ascii_case("REPLACING"));
    if replacing {
        return Some((name.to_ascii_uppercase(), true)); // signal: REPLACING form
    }
    // accept `COPY NAME.`  or  `COPY NAME .`
    let terminated = had_dot || rest.contains(&".");
    if !terminated {
        return None;
    }
    Some((name.to_ascii_uppercase(), false))
}

/// Expand all `COPY` statements in `source` using `resolver`.
pub fn expand(source: &str, resolver: &impl CopyResolver) -> Result<Expanded, CopyError> {
    let mut out = Expanded {
        lines: Vec::new(),
        provenance: Vec::new(),
    };
    let mut stack: Vec<String> = Vec::new();
    expand_into("<main>", source, resolver, &mut out, &mut stack)?;
    Ok(out)
}

fn expand_into(
    file: &str,
    text: &str,
    resolver: &impl CopyResolver,
    out: &mut Expanded,
    stack: &mut Vec<String>,
) -> Result<(), CopyError> {
    if stack.len() > MAX_DEPTH {
        return Err(CopyError::TooDeep);
    }
    for (i, line) in text.lines().enumerate() {
        if out.lines.len() > MAX_LINES {
            return Err(CopyError::TooLarge);
        }
        match parse_copy(line) {
            Some((_name, true)) => return Err(CopyError::ReplacingDeferred),
            Some((name, false)) => {
                if stack.iter().any(|n| n == &name) {
                    return Err(CopyError::Recursive(name));
                }
                let body = resolver
                    .resolve(&name)
                    .ok_or_else(|| CopyError::Missing(name.clone()))?;
                stack.push(name.clone());
                expand_into(&name, &body, resolver, out, stack)?;
                stack.pop();
            }
            None => {
                out.lines.push(line.to_string());
                out.provenance.push(Provenance {
                    file: file.to_string(),
                    line: i + 1,
                });
            }
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
    fn basic_copy_splices_with_provenance() {
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
        assert_eq!(
            e.provenance[3],
            Provenance {
                file: "<main>".into(),
                line: 3
            }
        );
    }

    #[test]
    fn nested_copy_expands() {
        let r = map(&[
            ("OUTER", "05 A PIC 9.\nCOPY INNER."),
            ("INNER", "05 B PIC X."),
        ]);
        let e = expand("COPY OUTER.", &r).unwrap();
        assert_eq!(e.lines, vec!["05 A PIC 9.", "05 B PIC X."]);
    }

    #[test]
    fn fails_closed() {
        let r = map(&[("SELF", "COPY SELF.")]);
        assert_eq!(
            expand("COPY SELF.", &r),
            Err(CopyError::Recursive("SELF".into()))
        );
        assert_eq!(
            expand("COPY NOPE.", &map(&[])),
            Err(CopyError::Missing("NOPE".into()))
        );
        let rr = map(&[("R", "05 A PIC X.")]);
        assert_eq!(
            expand("COPY R REPLACING ==A== BY ==B==.", &rr),
            Err(CopyError::ReplacingDeferred)
        );
    }
}
