//! Autotest `.at` structure parser (GnuCOBOL suite).
//!
//! Parses the macro stream from [`m4::scan`] into groups (`AT_SETUP ... AT_CLEANUP`) with their
//! `AT_DATA` files, `AT_CHECK` steps, and skip/xfail conditions. Classification happens at
//! `AT_CHECK`-step level, never by filename. Constructs outside the known macro surface are
//! recorded (`unknown_macros`) and never silently guessed.

use super::m4::{scan, Item};
use std::path::Path;

/// One `AT_DATA([file], [content])` file creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtData {
    pub filename: String,
    pub content: String,
    pub line: usize,
}

/// One `AT_CHECK([command], [status], [stdout], [stderr])` step. The status/expected fields stay
/// raw strings (`"0"`, `"1"`, `"ignore"`, expected text) -- the oracle-contract interpretation is
/// the classifier's job, not the parser's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtCheck {
    pub command: String,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
    /// True for `AT_CHECK_UNQUOTED`.
    pub unquoted: bool,
    pub line: usize,
}

/// One parsed test group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtGroup {
    /// Family-relative source path (e.g. `testsuite.src/syn_move.at`).
    pub source_file: String,
    pub line: usize,
    pub title: String,
    pub keywords: Vec<String>,
    pub data_files: Vec<AtData>,
    pub checks: Vec<AtCheck>,
    pub xfail: Vec<String>,
    pub skip: Vec<String>,
    /// Files the harness captures for the evidence (`AT_CAPTURE_FILE`): generated-file
    /// expectations.
    pub capture_files: Vec<String>,
    /// Macro names used in this group that the parser does not interpret (recorded, not guessed).
    pub unknown_macros: Vec<String>,
}

impl AtGroup {
    /// Steps that the oracle contract declares invalid (expected nonzero status).
    pub fn expected_rejects(&self) -> Vec<&AtCheck> {
        self.checks
            .iter()
            .filter(|c| c.status.parse::<i32>().map(|s| s != 0).unwrap_or(false))
            .collect()
    }
}

/// One parsed suite source file: the ordered groups plus any file-level notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAtFile {
    pub path: String,
    pub groups: Vec<AtGroup>,
    /// `m4_include` directives in order (for the dependency graph).
    pub includes: Vec<String>,
    pub parse_errors: Vec<String>,
}

/// Helper macro expansions known to be used by the suite (fail-closed: anything else is recorded
/// as unknown). `MANUAL_CHECK` is the only `m4_define`d wrapper in the GnuCOBOL suite; it wraps
/// `AT_CHECK` with a `DESC=...` environment prefix for the manual/screen runner.
fn expand_known(name: &str, args: &[String], group_title: &str) -> Option<(String, Vec<String>)> {
    if name == "MANUAL_CHECK" {
        if args.len() == 4 {
            let cmd = format!("DESC=\"{group_title}\" $RUN_PROG_MANUAL {}", args[0]);
            Some((
                "AT_CHECK".to_string(),
                vec![cmd, args[1].clone(), args[2].clone(), args[3].clone()],
            ))
        } else {
            None
        }
    } else {
        None
    }
}

/// Parse one `.at` file (already assembled from `m4_include`s by the caller) into groups.
pub fn parse_at(source: &Path, text: &str) -> ParsedAtFile {
    let mut out = ParsedAtFile {
        path: source.display().to_string(),
        groups: Vec::new(),
        includes: Vec::new(),
        parse_errors: Vec::new(),
    };
    let items = match scan(text) {
        Ok(items) => items,
        Err(e) => {
            out.parse_errors.push(e);
            return out;
        }
    };
    let mut cur: Option<AtGroup> = None;
    let mut seen_unknown: Vec<String> = Vec::new();
    for item in &items {
        let (name, args, line) = match item {
            Item::Macro { name, args, line } => (name, args, *line),
            Item::Text(_) => continue, // top-level text: comments/whitespace already consumed
        };
        match name.as_str() {
            "m4_include" => {
                if let Some(a) = args.first() {
                    out.includes.push(a.clone());
                }
            }
            "AT_BANNER" | "AT_INIT" | "AT_COPYRIGHT" | "AT_TESTED" | "AT_COLOR_TESTS" => {}
            "AT_SETUP" => {
                // close any unclosed group (should not happen; record defensively)
                if let Some(g) = cur.take() {
                    out.groups.push(g);
                }
                let title = args.first().cloned().unwrap_or_default();
                cur = Some(AtGroup {
                    source_file: out.path.clone(),
                    line,
                    title,
                    keywords: Vec::new(),
                    data_files: Vec::new(),
                    checks: Vec::new(),
                    xfail: Vec::new(),
                    skip: Vec::new(),
                    capture_files: Vec::new(),
                    unknown_macros: Vec::new(),
                });
                seen_unknown = Vec::new();
            }
            "AT_CLEANUP" => {
                if let Some(mut g) = cur.take() {
                    g.unknown_macros = seen_unknown.clone();
                    out.groups.push(g);
                }
            }
            "AT_KEYWORDS" => {
                if let Some(g) = cur.as_mut() {
                    if let Some(a) = args.first() {
                        g.keywords = a.split_whitespace().map(|s| s.to_string()).collect();
                    }
                }
            }
            "AT_DATA" => {
                if let Some(g) = cur.as_mut() {
                    if args.len() >= 2 {
                        g.data_files.push(AtData {
                            filename: args[0].clone(),
                            content: args[1].clone(),
                            line,
                        });
                    } else {
                        out.parse_errors.push(format!(
                            "{:?}: AT_DATA with {} arg(s) at line {line}",
                            source,
                            args.len()
                        ));
                    }
                }
            }
            "AT_CHECK" | "AT_CHECK_UNQUOTED" => {
                if let Some(g) = cur.as_mut() {
                    if args.is_empty() {
                        out.parse_errors
                            .push(format!("{:?}: AT_CHECK with 0 args at line {line}", source));
                    } else {
                        // Autotest defaults: status -> 0, stdout/stderr -> ignore
                        let status = args.get(1).cloned().unwrap_or_else(|| "0".to_string());
                        let stdout = args.get(2).cloned().unwrap_or_else(|| "ignore".to_string());
                        let stderr = args.get(3).cloned().unwrap_or_else(|| "ignore".to_string());
                        g.checks.push(AtCheck {
                            command: args[0].clone(),
                            status,
                            stdout,
                            stderr,
                            unquoted: name == "AT_CHECK_UNQUOTED",
                            line,
                        });
                    }
                }
            }
            "AT_XFAIL_IF" => {
                if let Some(g) = cur.as_mut() {
                    if let Some(a) = args.first() {
                        g.xfail.push(a.clone());
                    }
                }
            }
            "AT_SKIP_IF" => {
                if let Some(g) = cur.as_mut() {
                    if let Some(a) = args.first() {
                        g.skip.push(a.clone());
                    }
                }
            }
            "AT_CAPTURE_FILE" => {
                if let Some(g) = cur.as_mut() {
                    if let Some(a) = args.first() {
                        g.capture_files.push(a.clone());
                    }
                }
            }
            other => {
                // known helper macro? (the only one in the suite is MANUAL_CHECK)
                let title = cur.as_ref().map(|g| g.title.as_str()).unwrap_or("");
                if let Some((_expanded_name, expanded_args)) = expand_known(other, args, title) {
                    if let Some(g) = cur.as_mut() {
                        if expanded_args.len() >= 4 {
                            g.checks.push(AtCheck {
                                command: expanded_args[0].clone(),
                                status: expanded_args[1].clone(),
                                stdout: expanded_args[2].clone(),
                                stderr: expanded_args[3].clone(),
                                unquoted: false,
                                line,
                            });
                        }
                    }
                } else {
                    // recorded, never guessed
                    if !seen_unknown.contains(&other.to_string()) {
                        seen_unknown.push(other.to_string());
                    }
                }
            }
        }
    }
    if let Some(g) = cur.take() {
        // an unterminated group (no AT_CLEANUP): fail closed -- record the group but flag it
        let mut g = g;
        g.unknown_macros = seen_unknown.clone();
        out.parse_errors.push(format!(
            "{:?}: group {:?} not terminated by AT_CLEANUP",
            source, g.title
        ));
        out.groups.push(g);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r##"
## comment
AT_SETUP([MOVE SPACE TO numeric item])
AT_KEYWORDS([move editing])

AT_DATA([prog.cob], [
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       PROCEDURE        DIVISION.
           MOVE SPACE TO X.
           STOP RUN.
])

AT_CHECK([$COMPILE_ONLY prog.cob], [1], [],
[prog.cob:9: error: MOVE of figurative constant SPACE to numeric item used
])

AT_CLEANUP

AT_SETUP([MOVE ZERO TO alphabetic item])
AT_DATA([prog.cob], [
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       PROCEDURE        DIVISION.
           MOVE ZERO TO A.
           STOP RUN.
])

AT_CHECK([$COMPILE prog.cob], [0], [], [])
AT_CHECK([./prog], [0], [OK
], [])
AT_CLEANUP
"##;

    #[test]
    fn parses_two_groups_with_steps() {
        let parsed = parse_at(Path::new("testsuite.src/syn_move.at"), SAMPLE);
        assert!(parsed.parse_errors.is_empty(), "{:?}", parsed.parse_errors);
        assert_eq!(parsed.groups.len(), 2);
        let g0 = &parsed.groups[0];
        assert_eq!(g0.title, "MOVE SPACE TO numeric item");
        assert_eq!(g0.keywords, vec!["move", "editing"]);
        assert_eq!(g0.data_files.len(), 1);
        assert_eq!(g0.data_files[0].filename, "prog.cob");
        assert!(g0.data_files[0].content.contains("MOVE SPACE TO X."));
        assert_eq!(g0.checks.len(), 1);
        assert_eq!(g0.checks[0].command, "$COMPILE_ONLY prog.cob");
        assert_eq!(g0.checks[0].status, "1");
        assert!(g0.checks[0].stderr.contains("error: MOVE"));
        assert!(g0.unknown_macros.is_empty());
        let g1 = &parsed.groups[1];
        assert_eq!(g1.checks.len(), 2);
        assert_eq!(g1.checks[1].command, "./prog");
        assert_eq!(g1.checks[1].stdout, "OK\n");
    }

    #[test]
    fn expected_rejects_only_nonzero() {
        let parsed = parse_at(Path::new("t.at"), SAMPLE);
        assert_eq!(parsed.groups[0].expected_rejects().len(), 1);
        assert!(parsed.groups[1].expected_rejects().is_empty());
    }

    #[test]
    fn unknown_macro_is_recorded() {
        let src = "AT_SETUP([t])\nAT_MYSTERY([x])\nAT_CHECK([c], [0], [], [])\nAT_CLEANUP\n";
        let parsed = parse_at(Path::new("t.at"), src);
        assert_eq!(parsed.groups[0].unknown_macros, vec!["AT_MYSTERY"]);
    }

    #[test]
    fn unterminated_group_fails_closed() {
        let src = "AT_SETUP([t])\nAT_CHECK([c], [0], [], [])\n";
        let parsed = parse_at(Path::new("t.at"), src);
        assert!(parsed.parse_errors.iter().any(|e| e.contains("AT_CLEANUP")));
        assert_eq!(parsed.groups.len(), 1);
    }

    #[test]
    fn manual_check_expands() {
        let src = "AT_SETUP([screen t])\nMANUAL_CHECK([prog], [0], [], [])\nAT_CLEANUP\n";
        let parsed = parse_at(Path::new("testsuite_manual.at"), src);
        assert_eq!(parsed.groups[0].checks.len(), 1);
        let c = &parsed.groups[0].checks[0];
        assert!(c.command.contains("$RUN_PROG_MANUAL prog"));
        assert!(c.command.contains("DESC=\"screen t\""));
    }
}
