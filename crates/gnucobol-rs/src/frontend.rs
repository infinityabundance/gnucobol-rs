//! A minimal, CLEAN-ROOM COBOL front-end (lexer + parser + executor) over the ported libcob runtime
//! -- the piece that turns "Not a COBOL compiler" from a non-claim toward a positive. It is NOT
//! derived from `cobc` (GPL): it is a from-scratch reader for a small, explicit subset, and it
//! EXECUTES by calling the sealed runtime primitives ([`crate::pic`], [`crate::move_ops::cob_move`],
//! [`crate::arith`], [`crate::edited`], [`crate::termio::cob_display`]) -- no `cobc`, no `libcob`
//! linked. Correctness is judged the project's way: the stdout bytes must be byte-identical to the
//! admitted `cobc` for the same source (see `lab/oracle/cobol_run_sweep.sh`).
//!
//! ## The sealed subset (everything else fails closed -- never a silent mis-run)
//!
//! * `IDENTIFICATION DIVISION.` / `PROGRAM-ID.` (read, ignored).
//! * `DATA DIVISION.` `WORKING-STORAGE SECTION.` with `01`-level **elementary** items:
//!   `01 NAME PIC <pic> [VALUE <literal>].` -- numeric (`9 V S P`), alphanumeric (`X A`), or
//!   numeric-EDITED pictures (`Z * $ + - , . CR DB B 0 /`).
//! * `PROCEDURE DIVISION.` statements: `MOVE`, `ADD`, `SUBTRACT`, `MULTIPLY`, `DIVIDE` (the `TO` /
//!   `FROM` / `BY` / `INTO` / `GIVING` forms), `COMPUTE` (arithmetic expressions: `+ - * / **`,
//!   parentheses, unary minus, standard precedence), `DISPLAY`, `STOP RUN`.
//!
//! Group items, `OCCURS`/`REDEFINES`, level numbers other than `01`, `COMPUTE`, control flow
//! (`IF`/`PERFORM`/`EVALUATE`), `ACCEPT`, files, and any unlisted verb are out of subset and return a
//! [`RunError`] rather than guessing.

use crate::arith::{cob_arith, cob_divide, cob_divide_remainder, ArithError, Op, Round};
use crate::attr::{FieldAttr, COB_TYPE_NUMERIC_DISPLAY};
use crate::edited::{edited_size, encode_edited_cfg};
use crate::move_ops::{cob_move, cob_move_cfg};
use crate::pic::{build_field, Usage};
use crate::termio::{cob_display, DisplaySettings};
use crate::value::Decimal;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

/// The COBOL statement verbs the front-end actually EXECUTES (not merely recognizes as a boundary).
/// The generated parity tracker (`xtask cobol-parity`) reads this to report front-end coverage, so it
/// stays honest as the subset grows. Keep this in sync with the dispatch in `exec_stmt` + `run_block`.
pub const WIRED_STATEMENTS: &[&str] = &[
    "DISPLAY",
    "MOVE",
    "SET",
    "INITIALIZE",
    "INSPECT",
    "STRING",
    "UNSTRING",
    "ACCEPT",
    "ADD",
    "SUBTRACT",
    "MULTIPLY",
    "DIVIDE",
    "COMPUTE",
    "IF",
    "PERFORM",
    "STOP",
    "CONTINUE",
    "GOTO",
    "GOBACK",
    "EXIT",
    "CALL",
    "CANCEL",
    "EVALUATE",
    "SEARCH",
    "OPEN",
    "CLOSE",
    "READ",
    "WRITE",
    "REWRITE",
    "DELETE",
    "START",
    "UNLOCK",
    "COMMIT",
    "ROLLBACK",
    "SORT",
    "MERGE",
    "RELEASE",
    "RETURN",
    "JSON",
    "XML",
    "TRANSFORM",
    "RAISE",
    "VALIDATE",
    "DESTROY",
    "READY",
    "RESET",
    "EXHIBIT",
    "ALTER",
    "GENERATE",
    "INITIATE",
    "TERMINATE",
    "SUPPRESS",
    "EXAMINE",
    "ALLOCATE",
    "FREE",
    "USE",
];

/// The intrinsic functions the front-end evaluates in `FUNCTION ...` references (DISPLAY / COMPUTE /
/// MOVE / conditions), each dispatched to the ported `cob_intr_*` runtime and proven byte-identical to
/// cobc. Names are the canonical hyphenated COBOL spellings; the COBOL-PARITY tracker parses this marker
/// to count front-end intrinsic coverage. Keep in sync with `eval_intrinsic`.
pub const WIRED_FUNCTIONS: &[&str] = &[
    "LENGTH",
    "BYTE-LENGTH",
    "UPPER-CASE",
    "LOWER-CASE",
    "REVERSE",
    "TRIM",
    "NUMVAL",
    "NUMVAL-C",
    "NUMVAL-F",
    "INTEGER",
    "INTEGER-PART",
    "FRACTION-PART",
    "ABS",
    "ABSOLUTE-VALUE",
    "FACTORIAL",
    "SIGN",
    "ORD",
    "CHAR",
    "HEX-OF",
    "HEX-TO-CHAR",
    "BIT-OF",
    "BIT-TO-CHAR",
    "STORED-CHAR-LENGTH",
    "MOD",
    "REM",
    "MAX",
    "MIN",
    "SUM",
    "MEAN",
    "MEDIAN",
    "RANGE",
    "MIDRANGE",
    "ORD-MAX",
    "ORD-MIN",
    "VARIANCE",
    "STANDARD-DEVIATION",
    "ANNUITY",
    "PRESENT-VALUE",
    "CONCATENATE",
    "SUBSTITUTE",
    "SUBSTITUTE-CASE",
    "CURRENCY-SYMBOL",
    "SQRT",
    "EXP",
    "EXP10",
    "LOG",
    "LOG10",
    "SIN",
    "COS",
    "TAN",
    "ASIN",
    "ACOS",
    "ATAN",
    "PI",
    "E",
    "INTEGER-OF-DATE",
    "INTEGER-OF-DAY",
    "DATE-OF-INTEGER",
    "DAY-OF-INTEGER",
    "TEST-DATE-YYYYMMDD",
    "TEST-DAY-YYYYDDD",
    "TEST-NUMVAL",
    "TEST-NUMVAL-C",
    "TEST-NUMVAL-F",
    "LOWEST-ALGEBRAIC",
    "HIGHEST-ALGEBRAIC",
    "CURRENT-DATE",
    "COMBINED-DATETIME",
    "FORMATTED-DATE",
    "FORMATTED-TIME",
    "FORMATTED-DATETIME",
    "INTEGER-OF-FORMATTED-DATE",
    "TEST-FORMATTED-DATETIME",
    "SECONDS-FROM-FORMATTED-TIME",
    "FORMATTED-CURRENT-DATE",
    "YEAR-TO-YYYY",
    "DATE-TO-YYYYMMDD",
    "DAY-TO-YYYYDDD",
    "LOCALE-DATE",
    "LOCALE-TIME",
    "LOCALE-COMPARE",
    "MODULE-ID",
    "MODULE-CALLER-ID",
    "WHEN-COMPILED",
    "MODULE-DATE",
    "MODULE-TIME",
    "MODULE-FORMATTED-DATE",
    "MODULE-SOURCE",
    "EXCEPTION-STATUS",
    "EXCEPTION-STATEMENT",
    "EXCEPTION-LOCATION",
    "EXCEPTION-FILE",
    "CONTENT-OF",
    "CONTENT-LENGTH",
    "SECONDS-PAST-MIDNIGHT",
];

/// Why a program could not be run (fail closed -- the front-end never guesses).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// A token/keyword/structure outside the sealed subset, with a human note.
    Unsupported(String),
    /// A referenced data name was never declared in WORKING-STORAGE.
    UndefinedName(String),
    /// A runtime operation (move/arith/edit) failed (e.g. non-numeric operand).
    Runtime(String),
    /// A FATAL runtime error (libcob's `libcob: <file>:<line>: error: <msg>` shape, exit 1): the
    /// program must abort with the libcob-shaped diagnostic (e.g. "attempt to CANCEL active
    /// program"). The run boundary renders it distinctly from the adapter's own `cobrun:` notes.
    Fatal(String),
    /// An arithmetic SIZE ERROR condition (EC-SIZE-*): a divide-by-zero (or, in future, a result too
    /// large for the receiver). The receiver is left UNCHANGED and the statement's `ON SIZE ERROR`
    /// handler (if any) runs; with no handler, execution continues silently. Caught by `run_block` /
    /// the `exec_arith` / `exec_compute` wrappers -- it never propagates out as a fatal error.
    SizeError,
}

/// Build a [`RunError::Fatal`] carrying the source line for the libcob-shaped diagnostic
/// (`libcob: <file>:<line>: error: <msg>`). The line comes from the token map; the file is the
/// SOURCE_FILE set by the run boundary.
pub fn fatal_with_line(line: usize, msg: String) -> RunError {
    // libcob's runtime-error shape: `<file>:<line>: error: <msg>` (the run boundary prefixes
    // `libcob: ` and exits 1).
    RunError::Fatal(format!(
        "{}:{}: error: {}",
        crate::frontend::source_file_name(),
        line,
        msg
    ))
}

/// The current source file name (for fatal-error rendering), empty when unset.
pub fn source_file_name() -> String {
    SOURCE_FILE.with(|s| s.borrow().clone())
}

impl core::fmt::Display for RunError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RunError::Unsupported(s) => write!(f, "unsupported: {s}"),
            RunError::UndefinedName(s) => write!(f, "undefined data name: {s}"),
            RunError::Runtime(s) => write!(f, "runtime error: {s}"),
            RunError::Fatal(s) => write!(f, "runtime error: {s}"),
            RunError::SizeError => write!(f, "SIZE ERROR"),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Data model: each WORKING-STORAGE elementary item is one of three storage shapes.
// ---------------------------------------------------------------------------------------------

/// How a declared field is stored + operated on.
#[derive(Debug, Clone)]
enum Storage {
    /// A numeric `USAGE DISPLAY` field (zoned decimal): arithmetic-capable, carries its attr.
    Numeric(FieldAttr),
    /// An alphanumeric `PIC X/A` field: raw bytes, left-justified space-padded.
    Alpha(FieldAttr),
    /// A numeric-edited field: the bytes are the edited image; its PIC string drives editing, with the
    /// program's CURRENCY SIGN symbol (default `b'$'`) and DECIMAL-POINT IS COMMA flag (`SPECIAL-NAMES`).
    /// The trailing `bool` is `BLANK WHEN ZERO` (the whole field becomes spaces when the value is zero).
    Edited(String, u8, bool, bool),
    /// An `88`-level condition-name: true when its `parent` field's value equals any of `values` (a single
    /// value or a `lo THRU hi` range). Carries no storage of its own. `false_value` is the `WHEN SET TO
    /// FALSE <lit>` value (for `SET cond TO FALSE`), if declared.
    Condition {
        parent: String,
        values: Vec<CondVal>,
        false_value: Option<String>,
    },
    /// A group item: an ordered list of its elementary leaf field names. The group has no bytes of its own
    /// -- a read concatenates the leaves' current bytes, a write distributes the incoming bytes across them
    /// by length. (The leaves own their storage; the group is the aggregate view over the record.)
    Group { children: Vec<String> },
}

/// One value clause of an `88` condition-name: a single literal/identifier or an inclusive `lo THRU hi`
/// range. Stored as condition-comparison words (the `\u{1}`-marked form for string literals).
#[derive(Debug, Clone)]
enum CondVal {
    Single(String),
    Range(String, String),
}

/// A live field: its storage shape and its current bytes. For a scalar `bytes` is exactly the field's
/// size; for an `OCCURS n` table `bytes` is `n` element images concatenated and `occurs == n` (the
/// element size is `bytes.len() / occurs`), accessed by a subscript `NAME(i)`.
#[derive(Debug, Clone)]
struct Field {
    storage: Storage,
    bytes: Vec<u8>,
    occurs: usize,
    /// `NAME REDEFINES TARGET`: this field ALIASES `TARGET`'s storage (the C `cob_field` aliasable
    /// pointer). When set, the field carries no independent bytes -- every read/write reinterprets the
    /// target's bytes through this field's `storage` (so a MOVE into the redefining field is visible when
    /// the redefined field is read, and vice versa).
    redefines: Option<String>,
}

/// An alphanumeric literal/source attribute (`COB_TYPE_ALPHANUMERIC` = 0x21).
fn alnum_attr() -> FieldAttr {
    FieldAttr {
        field_type: 0x21,
        digits: 0,
        scale: 0,
        flags: 0,
    }
}

/// Build a numeric `USAGE DISPLAY` attr for an integer/decimal literal of `digits` digits and
/// `scale` fractional digits (sign per `signed`).
fn lit_num_attr(digits: u16, scale: i16, signed: bool) -> FieldAttr {
    let flags = if signed {
        crate::attr::COB_FLAG_HAVE_SIGN
    } else {
        0
    };
    FieldAttr {
        field_type: COB_TYPE_NUMERIC_DISPLAY,
        digits,
        scale,
        flags,
    }
}

// ---------------------------------------------------------------------------------------------
// Lexer: COBOL source -> a flat token stream. Strings are kept as single tokens; '.' that ends a
// sentence is emitted as its own "." token (a period glued to a word, like "RUN.", is split).
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Word(String),
    Str(Vec<u8>),
    /// `VALUE ALL "lit"` -- the repeating figurative literal (upstream 61479ba0c widened the
    /// SCREEN SECTION VALUE clause to ALL-literals; the candidate applies it to every VALUE clause).
    AllLiteral(Vec<u8>),
    Dot,
}

/// True if `bytes[i]` is a full-line comment INDICATOR: the first non-blank character of its line AND within
/// the fixed-format indicator area (column <= 7, i.e. at most 6 leading blanks -- the 6-column sequence area
/// then the column-7 indicator). The column bound matters for `/`: a line-leading `/` in the indicator area
/// is a fixed-format page-eject comment, but a deeply-indented `/` is the DIVISION operator continuing a
/// multi-line expression (`COMPUTE r = (a * 100)` <newline> `   / (b - 1)`), which must NOT be dropped.
fn line_blank_before(bytes: &[u8], i: usize) -> bool {
    let mut j = i;
    while j > 0 && bytes[j - 1] != b'\n' {
        if !bytes[j - 1].is_ascii_whitespace() {
            return false;
        }
        j -= 1;
    }
    i - j <= 6 // column (0-based) of `bytes[i]` is <= 6, i.e. column 7 or earlier
}

fn lex(src: &str) -> Vec<Tok> {
    let (toks, lines) = lex_with_lines(src);
    TOKEN_LINES.with(|l| *l.borrow_mut() = lines);
    toks
}

/// `lex` plus a per-token source LINE map (1-based), used by fatal runtime-error reporting
/// (libcob's `libcob: <file>:<line>: error: <msg>` shape). The line is the token's start line.
fn lex_with_lines(src: &str) -> (Vec<Tok>, Vec<usize>) {
    let mut toks = Vec::new();
    let mut lines = Vec::new();
    let mut line = 1usize;
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'*' && bytes.get(i + 1) == Some(&b'>') {
            // free-format inline comment `*>` (anywhere on the line, after indentation): skip to EOL.
            // Must be stripped BEFORE quote tokenization -- an apostrophe in the comment (e.g. "caller's")
            // would otherwise open a spurious string literal that swallows the rest of the source.
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if (c == b'*' || c == b'/') && line_blank_before(bytes, i) {
            // A full-line comment: `*` or `/` as the FIRST non-blank char of the line -- col-1 in free form
            // AND the fixed-format column-7 indicator (`      *...` / `      /...`). A `/` elsewhere (DIVIDE)
            // is not line-leading, so it tokenizes normally. Skip to end of line.
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_whitespace() {
            if c == b'\n' {
                line += 1;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' {
            let quote = c;
            i += 1;
            let mut s = Vec::new();
            while i < bytes.len() {
                if bytes[i] == quote {
                    // doubled quote -> literal quote, else end.
                    if i + 1 < bytes.len() && bytes[i + 1] == quote {
                        s.push(quote);
                        i += 2;
                        continue;
                    }
                    i += 1;
                    break;
                }
                s.push(bytes[i]);
                i += 1;
            }
            toks.push(Tok::Str(s));
            lines.push(line);
            continue;
        }
        if c == b'.' {
            // A `.` directly followed by a digit begins a numeric literal written with no leading zero
            // (`.08` = 0.08, which cobc accepts) -- fall through to word-scanning so the whole literal is one
            // token. Otherwise the `.` is a sentence/clause terminator.
            if !matches!(bytes.get(i + 1), Some(n) if n.is_ascii_digit()) {
                toks.push(Tok::Dot);
                lines.push(line);
                i += 1;
                continue;
            }
        }
        // a word: letters, digits, '-', '(', ')', '$', and PIC symbols up to whitespace/'.'/quote.
        let start = i;
        while i < bytes.len() {
            let b = bytes[i];
            if b.is_ascii_whitespace() || b == b'"' || b == b'\'' {
                break;
            }
            if b == b'.' {
                // a '.' inside a PIC (e.g. ZZ9.99) stays; a '.' that ends a sentence is followed by
                // whitespace/EOF. Peek: if the next char is a digit or PIC symbol, keep it in the word.
                let nxt = bytes.get(i + 1).copied();
                let kept = matches!(nxt, Some(n) if n.is_ascii_digit() || n == b'9' || n == b'Z' || n == b'*' || n == b'(');
                if !kept {
                    break;
                }
            }
            i += 1;
        }
        toks.push(Tok::Word(src[start..i].to_string()));
        lines.push(line);
    }
    (toks, lines)
}

// ---------------------------------------------------------------------------------------------
// Executor: walk the token stream, build the field table from WORKING-STORAGE, then run statements.
// ---------------------------------------------------------------------------------------------

/// Parse + execute a COBOL program from `source`, returning the exact stdout bytes it would write.
/// Fails closed with a [`RunError`] for anything outside the sealed subset. Runs under the default
/// dialect; [`run_program_dialect`] selects a `-std=` dialect (e.g. uninitialized-storage `defaultbyte`).
pub fn run_program(source: &str) -> Result<Vec<u8>, RunError> {
    run_program_dialect(source, crate::dialect::Dialect::DEFAULT)
}

/// As [`run_program_dialect_with_rc`] but returns only the stdout bytes (the common case; the sweep and
/// most callers ignore the exit code).
pub fn run_program_dialect(
    source: &str,
    dialect: crate::dialect::Dialect,
) -> Result<Vec<u8>, RunError> {
    run_program_dialect_with_rc(source, dialect).map(|(out, _rc)| out)
}

/// Convert FIXED-format COBOL source to free format (`-fixed` / `>>SOURCE FORMAT IS FIXED`): columns 1-6
/// are the sequence area (ignored), column 7 is the indicator (`*` or `/` = a full-line comment, dropped;
/// anything else = code), columns 8-72 are the code area, and columns 73+ are ignored. The result is the
/// free-format text [`run_program`] then lexes. (Tabs are not expanded -- the sealed corpus uses spaces.)
pub fn fixed_to_free(source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        let chars: Vec<char> = line.chars().collect();
        // A line with no indicator column (<= 6 chars: blank or sequence-only) contributes a blank line.
        if chars.len() < 7 {
            out.push('\n');
            continue;
        }
        match chars[6] {
            '*' | '/' => out.push('\n'), // comment / page-eject: drop the line
            '-' => {
                // Continuation line: the resumed text joins the PREVIOUS line flush (the CCVS85
                // corpus splits VALUE literals across lines). Columns 73+ (sequence area) are never
                // part of the text. Two cases, per the standard + cobc:
                //  * NONNUMERIC-LITERAL continuation (the previous line ended inside an open
                //    literal): the literal resumes at column 12, and the QUOTE at column 12 is the
                //    continuation marker -- not part of the value (skipped). Trailing spaces to
                //    column 72 ARE part of the literal.
                //  * WORD continuation: the word resumes at column 8, rejoined flush.
                let open_lit = {
                    let prev_trim = out.trim_end();
                    let q = prev_trim.chars().filter(|&c| c == '"').count();
                    q % 2 == 1
                };
                let end = chars.len().min(72);
                let text: String = if open_lit {
                    // resume at column 12 (index 11); a quote exactly at column 12 is the marker.
                    let mut s: String = chars.get(11..end).unwrap_or_default().iter().collect();
                    if s.starts_with('"') {
                        s.remove(0);
                    }
                    s
                } else {
                    chars[7..end].iter().collect()
                };
                if out.ends_with('\n') {
                    // drop the trailing newline of the previous line, then append flush
                    out.pop();
                    out.push_str(&text);
                    out.push('\n');
                } else {
                    out.push_str(&text);
                    out.push('\n');
                }
            }
            _ => {
                let end = chars.len().min(72); // columns 8..=72 (0-indexed 7..72); 73+ ignored
                out.extend(&chars[7..end]);
                out.push('\n');
            }
        }
    }
    out
}

/// The conditional-compilation preprocessor (`cobc`'s `>>` directives): resolve `>>DEFINE name [AS value]`,
/// `>>IF <cond>` / `>>ELSE` / `>>END-IF` line by line, emitting only the lines whose enclosing conditions
/// are all true. Directive lines are never emitted. Supported conditions: `[NOT] name DEFINED` and a plain
/// `name = value` equality against the defined value. Unrecognized `>>` directives (e.g. `>>SOURCE FORMAT`)
/// are passed through untouched so the rest of the pipeline can fail closed on them if needed.
fn preprocess(source: &str) -> String {
    let mut defines: HashMap<String, String> = HashMap::new();
    // one (include_this_branch, any_branch_taken) per open >>IF.
    let mut stack: Vec<(bool, bool)> = Vec::new();
    let including = |s: &[(bool, bool)]| s.iter().all(|&(inc, _)| inc);
    let mut out = String::new();
    for line in source.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix(">>").map(str::trim) {
            let up = rest.to_ascii_uppercase();
            if let Some(d) = up.strip_prefix("DEFINE ") {
                if including(&stack) {
                    let parts: Vec<&str> = d.split_whitespace().collect();
                    if let Some(name) = parts.first() {
                        let value = if parts.len() >= 3 && parts[1] == "AS" {
                            parts[2].to_string()
                        } else {
                            String::new()
                        };
                        defines.insert((*name).to_string(), value);
                    }
                }
                continue;
            }
            if let Some(c) = up.strip_prefix("IF ") {
                let parent = including(&stack);
                let cond = parent && eval_pp_cond(c, &defines);
                stack.push((cond, cond));
                continue;
            }
            if up == "ELSE" {
                if let Some(idx) = stack.len().checked_sub(1) {
                    let parent = stack[..idx].iter().all(|&(i, _)| i);
                    let taken = stack[idx].1;
                    stack[idx] = (parent && !taken, true);
                }
                continue;
            }
            if up == "END-IF" || up.starts_with("END-IF ") {
                stack.pop();
                continue;
            }
            // An unrecognized >> compiler directive (>>TURN ... CHECKING, >>SOURCE FORMAT, >>CALL-CONVENTION,
            // >>PAGE, >>LISTING, >>D debug line, ...) carries no runtime tokens -- drop it rather than leak it
            // into the statement stream. (Source FORMAT is governed by the -fixed/-free flag here.)
            continue;
        }
        if including(&stack) {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Evaluate a `>>IF` condition: `[NOT] name DEFINED`, or `name = value` (string-equal the defined value).
fn eval_pp_cond(c: &str, defines: &HashMap<String, String>) -> bool {
    // drop the optional `IS` so `name IS [NOT] DEFINED` normalises to `name [NOT] DEFINED`.
    let p: Vec<&str> = c.split_whitespace().filter(|w| *w != "IS").collect();
    // DEFINED test in any spelling: `NOT name DEFINED`, `name DEFINED`, `name NOT DEFINED`.
    if p.len() >= 3 && p[0] == "NOT" && p[2] == "DEFINED" {
        return !defines.contains_key(p[1]);
    }
    if p.len() >= 3 && p[1] == "NOT" && p[2] == "DEFINED" {
        return !defines.contains_key(p[0]);
    }
    if p.len() >= 2 && p[1] == "DEFINED" {
        return defines.contains_key(p[0]);
    }
    // equality against the defined value: `name = value`, `name NOT = value`.
    if p.len() >= 4 && p[1] == "NOT" && p[2] == "=" {
        return defines.get(p[0]).map(|v| v != p[3]).unwrap_or(true);
    }
    if p.len() >= 3 && p[1] == "=" {
        return defines.get(p[0]).map(|v| v == p[2]).unwrap_or(false);
    }
    false
}

/// As [`run_program_dialect`] but also returns the program's final `RETURN-CODE` as the process exit code
/// (`MOVE n TO RETURN-CODE` / `STOP RUN n`; default 0) -- the value a `cobrun` would `exit()` with.
pub fn run_program_dialect_with_rc(
    source: &str,
    dialect: crate::dialect::Dialect,
) -> Result<(Vec<u8>, i32), RunError> {
    let (out, _printer, rc) = run_program_redirected(source, dialect, false)?;
    Ok((out, rc))
}

/// A **prepared program** (spec 9.6): the full front-end (preprocess, lex, parse, resolution,
/// layout, check) is executed once and its immutable artifacts are embedded here. [`Self::run`]
/// re-executes the program WITHOUT reparsing the source -- repeated prepared execution never
/// touches the source text again. The representation is versioned via `compat` (bump on any
/// front-end artifact change) and carries the source/expanded hashes for corruption and stale
/// detection.
#[allow(missing_debug_implementations)]
pub struct PreparedProgram {
    /// Source + expanded hashes (cache-off equivalence and stale detection).
    pub source_hash: String,
    pub expanded_hash: String,
    pub dialect: crate::dialect::Dialect,
    /// Front-end version stamp (bump with the parser/checker).
    pub compat: &'static str,
    /// Phase probes of the prepare run (all ok when the program is accepted).
    pub probes: Vec<PhaseProbe>,
    // ---- embedded parse artifacts (never re-derived on run) ----
    program_map: HashMap<String, ProgramDef>,
    currency: u8,
    decimal_comma: bool,
    switches: SwitchEnv,
    collation: Option<[u8; 256]>,
    main_name: String,
    file_defs: HashMap<String, FileDef>,
    record_files: HashMap<String, String>,
    reports: HashMap<String, ReportDef>,
}

impl PreparedProgram {
    /// Re-execute the prepared program (no parse). Returns `(stdout, printer, exit_code)`.
    pub fn run(&self, redirect_printer: bool) -> Result<(Vec<u8>, Vec<u8>, i32), RunError> {
        let ctx = Ctx {
            programs: &self.program_map,
            dialect: self.dialect,
            currency: self.currency,
            decimal_comma: self.decimal_comma,
            collation: self.collation,
            switches: self.switches.clone(),
            print_redirect: redirect_printer,
            printer: RefCell::new(Vec::new()),
            stop_run: Cell::new(false),
            exit_perform: Cell::new(false),
            exit_cycle: Cell::new(false),
            next_sentence: Cell::new(false),
            call_state: RefCell::new(HashMap::new()),
            goto: RefCell::new(None),
            file_defs: self.file_defs.clone(),
            record_files: self.record_files.clone(),
            files: RefCell::new(HashMap::new()),
            reports: self.reports.clone(),
        };
        let main = ctx
            .programs
            .get(&self.main_name)
            .expect("main program is registered");
        let mut out = Vec::new();
        EXTERNAL_STORE.with(|m| m.borrow_mut().clear());
        POINTER_TARGETS.with(|m| m.borrow_mut().clear());
        ENV_NAME_REG.with(|r| r.borrow_mut().clear());
        ENV_OVERRIDE.with(|m| m.borrow_mut().clear());
        ARG_NUMBER_REG.with(|r| *r.borrow_mut() = 0);
        let mut fields = build_program_fields(main, &ctx)?;
        reset_exception();
        let env_get = |k: &str| std::env::var(k).ok();
        crate::profiling::prof_start(&crate::profiling::prof_config(&env_get));
        run_program_body(main, &self.main_name, &ctx, &mut fields, &mut out)?;
        let rc = read_return_code(&fields);
        let printer = ctx.printer.borrow().clone();
        dump_file_store(&ctx, &fields);
        crate::profiling::prof_report(&env_get, &source_file_name());
        Ok((out, printer, rc))
    }
}

/// Prepare a program: run the full front-end once and embed the parse artifacts (spec 9.6).
/// Rejected programs return the first failing probe's [`RunError`] -- never a silent accept.
pub fn prepare_program(
    source: &str,
    dialect: crate::dialect::Dialect,
) -> Result<PreparedProgram, RunError> {
    let pre = preprocess(source);
    let expanded_hash = crate::sha256::sha256_hex(pre.as_bytes());
    let up = uppercase_outside_quotes(&pre);
    let mut toks = lex(&up);
    let first_proc = find_seq(&toks, &["PROCEDURE", "DIVISION"])
        .ok_or_else(|| RunError::Unsupported("no PROCEDURE DIVISION".into()))?;
    let currency = parse_currency_sign(&toks, first_proc);
    let decimal_comma = parse_decimal_comma(&toks, first_proc);
    if decimal_comma {
        for t in toks.iter_mut() {
            if let Tok::Word(w) = t {
                if is_comma_decimal_literal(w) {
                    *w = w.replace(',', ".");
                }
            }
        }
    }
    let (main_name, program_map) = parse_programs(&toks)?;
    let switches = parse_switches(&toks, first_proc);
    let collation = parse_collation(&toks, first_proc);
    let file_defs: HashMap<String, FileDef> = program_map
        .get(&main_name)
        .map(|p| {
            p.files
                .iter()
                .map(|f| (f.name.clone(), f.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut record_files: HashMap<String, String> = HashMap::new();
    if let Some(p) = program_map.get(&main_name) {
        for f in &p.files {
            for r in &f.records {
                record_files
                    .entry(r.clone())
                    .or_insert_with(|| f.name.clone());
            }
        }
    }
    let reports: HashMap<String, ReportDef> = program_map
        .get(&main_name)
        .map(|p| p.reports.clone())
        .unwrap_or_default();
    // layout + check (same typed errors as check_program)
    {
        let ctx = Ctx {
            programs: &program_map,
            dialect,
            currency,
            decimal_comma,
            collation,
            switches: switches.clone(),
            print_redirect: false,
            printer: RefCell::new(Vec::new()),
            stop_run: Cell::new(false),
            exit_perform: Cell::new(false),
            exit_cycle: Cell::new(false),
            next_sentence: Cell::new(false),
            call_state: RefCell::new(HashMap::new()),
            goto: RefCell::new(None),
            file_defs: file_defs.clone(),
            record_files: record_files.clone(),
            files: RefCell::new(HashMap::new()),
            reports: reports.clone(),
        };
        let main = ctx.programs.get(&main_name).expect("main registered");
        let fields = build_program_fields(main, &ctx)?;
        CHECK_MODE.with(|c| c.set(true));
        let body_check: Result<(), RunError> = (|| {
            let names: Vec<String> = ctx.programs.keys().cloned().collect();
            for name in names {
                let prog = &ctx.programs[&name];
                let mut fields = if name == main_name {
                    fields.clone()
                } else {
                    build_program_fields(prog, &ctx)?
                };
                let mut p = 0usize;
                while p < prog.proc_toks.len() {
                    let before = p;
                    if run_block(
                        &prog.proc_toks,
                        &mut p,
                        &mut fields,
                        &mut Vec::new(),
                        false,
                        &ctx,
                    )? {
                        break;
                    }
                    if p == before {
                        p += 1;
                    }
                    if matches!(prog.proc_toks.get(p), Some(Tok::Dot)) {
                        p += 1;
                    }
                }
            }
            Ok(())
        })();
        CHECK_MODE.with(|c| c.set(false));
        body_check?;
    }
    Ok(PreparedProgram {
        source_hash: crate::sha256::sha256_hex(source.as_bytes()),
        expanded_hash,
        dialect,
        compat: "prepared-v1",
        probes: probe_phases(source, dialect, false),
        program_map,
        currency,
        decimal_comma,
        switches,
        collation,
        main_name,
        file_defs,
        record_files,
        reports,
    })
}

/// Run a program, optionally diverting `DISPLAY ... UPON PRINTER` to a separate `printer` stream (instead
/// of interleaving it into stdout) when `redirect_printer` is set -- the host (`cobrun`) supplies this
/// from `COB_DISPLAY_PRINT_FILE`/`_PIPE` and appends the returned printer bytes to that file, mirroring
/// libcob's `cob_display_print_file`. Returns `(stdout, printer, exit_code)`.
pub fn run_program_redirected(
    source: &str,
    dialect: crate::dialect::Dialect,
    redirect_printer: bool,
) -> Result<(Vec<u8>, Vec<u8>, i32), RunError> {
    // Conditional-compilation preprocessor: resolve >>DEFINE / >>IF / >>ELSE / >>END-IF before lexing.
    let pre = preprocess(source);
    let up = uppercase_outside_quotes(&pre);
    // EC-BOUND-SUBSCRIPT checking is per-run state (>>TURN ... CHECKING ON/OFF); default OFF like cobc.
    // Scan the RAW source, NOT the preprocessed text: the preprocessor drops unrecognized `>>`
    // directives (>>SOURCE FORMAT, >>PAGE, >>LISTING, ...) as no-runtime-token lines, and >>TURN
    // EC-BOUND-SUBSCRIPT is precisely one of them -- but it is NOT token-free for cobrun: it drives
    // this very flag. (Regression from the ELITE-REPLAY.2 directive-drop; the
    // `turn_ec_bound_subscript_is_honored` test guards it.)
    EC_BOUND_SUBSCRIPT_ON.with(|c| c.set(parse_ec_bound_check(&uppercase_outside_quotes(source))));
    let mut toks = lex(&up);

    // ENVIRONMENT DIVISION / SPECIAL-NAMES of the first program: CURRENCY SIGN IS "x" + DECIMAL-POINT IS
    // COMMA apply program-wide. (Scanned before the first PROCEDURE DIVISION.)
    let first_proc = find_seq(&toks, &["PROCEDURE", "DIVISION"])
        .ok_or_else(|| RunError::Unsupported("no PROCEDURE DIVISION".into()))?;
    let currency = parse_currency_sign(&toks, first_proc);
    let decimal_comma = parse_decimal_comma(&toks, first_proc);
    if decimal_comma {
        for t in toks.iter_mut() {
            if let Tok::Word(w) = t {
                if is_comma_decimal_literal(w) {
                    *w = w.replace(',', ".");
                }
            }
        }
    }

    // Split the source into its programs (a MAIN plus any CONTAINED / nested programs reachable by CALL).
    let (main_name, program_map) = parse_programs(&toks)?;
    let switches = parse_switches(&toks, first_proc);
    let collation = parse_collation(&toks, first_proc);
    set_collation(collation);
    let file_defs: HashMap<String, FileDef> = program_map
        .get(&main_name)
        .map(|p| {
            p.files
                .iter()
                .map(|f| (f.name.clone(), f.clone()))
                .collect()
        })
        .unwrap_or_default();
    // Pre-resolved FD record -> owning file (first declaration wins on a duplicate name, matching the
    // deterministic source order of the files list).
    let mut record_files: HashMap<String, String> = HashMap::new();
    if let Some(p) = program_map.get(&main_name) {
        for f in &p.files {
            for r in &f.records {
                record_files
                    .entry(r.clone())
                    .or_insert_with(|| f.name.clone());
            }
        }
    }
    let reports: HashMap<String, ReportDef> = program_map
        .get(&main_name)
        .map(|p| p.reports.clone())
        .unwrap_or_default();
    let ctx = Ctx {
        programs: &program_map,
        dialect,
        currency,
        decimal_comma,
        collation,
        switches,
        print_redirect: redirect_printer,
        printer: RefCell::new(Vec::new()),
        stop_run: Cell::new(false),
        exit_perform: Cell::new(false),
        exit_cycle: Cell::new(false),
        next_sentence: Cell::new(false),
        call_state: RefCell::new(HashMap::new()),
        goto: RefCell::new(None),
        file_defs,
        record_files,
        files: RefCell::new(HashMap::new()),
        reports,
    };
    let main = ctx
        .programs
        .get(&main_name)
        .expect("main program is registered");

    let mut out = Vec::new();
    EXTERNAL_STORE.with(|m| m.borrow_mut().clear()); // EXTERNAL storage is per run unit (before any build)
    POINTER_TARGETS.with(|m| m.borrow_mut().clear());
    ENV_NAME_REG.with(|r| r.borrow_mut().clear());
    ENV_OVERRIDE.with(|m| m.borrow_mut().clear());
    // NOTE: COMMAND_LINE is NOT cleared here -- the host (cobcrun) sets it BEFORE the run; a
    // launcher/`./prog` execution leaves it at its thread-local empty default.
    ARG_NUMBER_REG.with(|r| *r.borrow_mut() = 0);
    let mut fields = build_program_fields(main, &ctx)?;
    reset_exception(); // a fresh run starts with no raised exception
                       // Profiling (upstream 7b6995042): the runtime setting activates it; the paragraph hooks below
                       // are the interpreted equivalent of the generated cob_prof_function_call calls.
    let env_get = |k: &str| std::env::var(k).ok();
    crate::profiling::prof_start(&crate::profiling::prof_config(&env_get));
    run_program_body(main, &main_name, &ctx, &mut fields, &mut out)?;
    let rc = read_return_code(&fields);
    let printer = ctx.printer.borrow().clone();
    dump_file_store(&ctx, &fields);
    crate::profiling::prof_report(&env_get, &source_file_name());
    Ok((out, printer, rc))
}

/// Parse + semantically check a COBOL program WITHOUT executing it — the `-fsyntax-only` path for
/// the `cobc-rs` adapter. Runs the same phases as [`run_program_redirected`] (preprocessor, lexer,
/// program split, switches, and the full WORKING-STORAGE / FILE / REPORT layout construction) and
/// fails closed with the same [`RunError`]s on anything outside the sealed subset. The program body
/// is NOT executed and no runtime state is touched, so a program that would write files, loop, or
/// STOP at runtime is still only checked here.
pub fn check_program(source: &str, dialect: crate::dialect::Dialect) -> Result<(), RunError> {
    // Conditional-compilation preprocessor: resolve >>DEFINE / >>IF / >>ELSE / >>END-IF before lexing.
    let pre = preprocess(source);
    let up = uppercase_outside_quotes(&pre);
    // EC-BOUND-SUBSCRIPT is per-run state; for a syntax-only check it is irrelevant, but the raw
    // source scan keeps the flag consistent with a later run of the same source.
    EC_BOUND_SUBSCRIPT_ON.with(|c| c.set(parse_ec_bound_check(&uppercase_outside_quotes(source))));
    let mut toks = lex(&up);

    // ENVIRONMENT DIVISION / SPECIAL-NAMES of the first program: CURRENCY SIGN + DECIMAL-POINT.
    let first_proc = find_seq(&toks, &["PROCEDURE", "DIVISION"])
        .ok_or_else(|| RunError::Unsupported("no PROCEDURE DIVISION".into()))?;
    let currency = parse_currency_sign(&toks, first_proc);
    let decimal_comma = parse_decimal_comma(&toks, first_proc);
    if decimal_comma {
        for t in toks.iter_mut() {
            if let Tok::Word(w) = t {
                if is_comma_decimal_literal(w) {
                    *w = w.replace(',', ".");
                }
            }
        }
    }

    let (main_name, program_map) = parse_programs(&toks)?;
    let switches = parse_switches(&toks, first_proc);
    let collation = parse_collation(&toks, first_proc);
    set_collation(collation);
    let file_defs: HashMap<String, FileDef> = program_map
        .get(&main_name)
        .map(|p| {
            p.files
                .iter()
                .map(|f| (f.name.clone(), f.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut record_files: HashMap<String, String> = HashMap::new();
    if let Some(p) = program_map.get(&main_name) {
        for f in &p.files {
            for r in &f.records {
                record_files
                    .entry(r.clone())
                    .or_insert_with(|| f.name.clone());
            }
        }
    }
    let reports: HashMap<String, ReportDef> = program_map
        .get(&main_name)
        .map(|p| p.reports.clone())
        .unwrap_or_default();
    let ctx = Ctx {
        programs: &program_map,
        dialect,
        currency,
        decimal_comma,
        collation,
        switches,
        print_redirect: false,
        printer: RefCell::new(Vec::new()),
        stop_run: Cell::new(false),
        exit_perform: Cell::new(false),
        exit_cycle: Cell::new(false),
        next_sentence: Cell::new(false),
        call_state: RefCell::new(HashMap::new()),
        goto: RefCell::new(None),
        file_defs,
        record_files,
        files: RefCell::new(HashMap::new()),
        reports,
    };
    let main = ctx
        .programs
        .get(&main_name)
        .expect("main program is registered");
    // Build the full declaration/layout model (WORKING-STORAGE, FD records, reports) — this is the
    // deepest static phase; the program body is never run.
    let fields = build_program_fields(main, &ctx)?;
    // Walk EVERY program's procedure body in skip mode with CHECK_MODE set: the statement walker
    // validates the statement structure (IF/PERFORM/… scopes, operand collection) and rejects a
    // statement that starts with a non-verb token, WITHOUT executing anything. This is what makes
    // `cobc-rs -fsyntax-only` fail closed on unsupported syntax instead of accepting it.
    CHECK_MODE.with(|c| c.set(true));
    let body_check: Result<(), RunError> = (|| {
        let names: Vec<String> = ctx.programs.keys().cloned().collect();
        for name in names {
            let prog = &ctx.programs[&name];
            let mut fields = if name == main_name {
                fields.clone()
            } else {
                build_program_fields(prog, &ctx)?
            };
            let mut p = 0usize;
            while p < prog.proc_toks.len() {
                let before = p;
                if run_block(
                    &prog.proc_toks,
                    &mut p,
                    &mut fields,
                    &mut Vec::new(),
                    false,
                    &ctx,
                )? {
                    break;
                }
                if p == before {
                    // run_block parked on a scope ender (END-READ/END-IF/...) without advancing:
                    // skip it so the walk always makes progress.
                    p += 1;
                }
                if matches!(prog.proc_toks.get(p), Some(Tok::Dot)) {
                    p += 1;
                }
            }
        }
        Ok(())
    })();
    CHECK_MODE.with(|c| c.set(false));
    body_check?;
    Ok(())
}

/// The candidate front-end phases -- the phase-attribution vocabulary of the corpus subsystem.
/// Exactly one first failure per program profile; phases after the first failure are not probed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProbePhase {
    Preprocess,
    Lex,
    Parse,
    Resolution,
    Layout,
    Check,
    Prepare,
    Execute,
}

impl ProbePhase {
    pub fn as_str(self) -> &'static str {
        match self {
            ProbePhase::Preprocess => "preprocess",
            ProbePhase::Lex => "lex",
            ProbePhase::Parse => "parse",
            ProbePhase::Resolution => "resolution",
            ProbePhase::Layout => "layout",
            ProbePhase::Check => "check",
            ProbePhase::Prepare => "prepare",
            ProbePhase::Execute => "execute",
        }
    }

    /// The corpus first-failure vocabulary (spec 9.4): parse/resolution/layout/check/prepare/run
    /// are separate; `run` is the corpus name for the execute probe.
    pub fn corpus_phase(self) -> &'static str {
        match self {
            ProbePhase::Execute => "run",
            other => other.as_str(),
        }
    }
}

/// One probed phase outcome. `ok == false` marks the first failing phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseProbe {
    pub phase: String,
    pub ok: bool,
    pub diagnostic: String,
}

/// Probe the front-end phases separately and stop at the first failure.
///
/// Mirrors [`check_program`] (same private phase functions, same order, same typed [`RunError`]s)
/// and, when `run` is set, continues into the execution path exactly as
/// [`run_program_redirected`] does (prepare + execute). The corpus records the FIRST failing
/// phase per valid program package; the probe never guesses a phase from diagnostic text.
///
/// Consistency with the two existing entry points is guarded by tests (a probe that says a
/// source checks clean must agree with `check_program`, and a run probe must agree with
/// `run_program_redirected` on exit code / error).
pub fn probe_phases(source: &str, dialect: crate::dialect::Dialect, run: bool) -> Vec<PhaseProbe> {
    let mut out = Vec::new();
    let mut stopped = false;
    let mut push = |phase: ProbePhase, ok: bool, diagnostic: String| {
        if !stopped {
            out.push(PhaseProbe {
                phase: phase.as_str().to_string(),
                ok,
                diagnostic,
            });
            if !ok {
                stopped = true;
            }
        }
    };

    // Phase 1: preprocess (conditional compilation). Best-effort, cannot fail.
    let pre = preprocess(source);
    push(ProbePhase::Preprocess, true, String::new());

    // Phase 2: lex (uppercase outside quotes, tokenize).
    let up = uppercase_outside_quotes(&pre);
    let mut toks = lex(&up);
    push(ProbePhase::Lex, true, String::new());

    // Phase 3: parse (currency/decimal handling, program split).
    let first_proc = match find_seq(&toks, &["PROCEDURE", "DIVISION"]) {
        Some(i) => i,
        None => {
            push(
                ProbePhase::Parse,
                false,
                RunError::Unsupported("no PROCEDURE DIVISION".into()).to_string(),
            );
            return out;
        }
    };
    let currency = parse_currency_sign(&toks, first_proc);
    let decimal_comma = parse_decimal_comma(&toks, first_proc);
    if decimal_comma {
        for t in toks.iter_mut() {
            if let Tok::Word(w) = t {
                if is_comma_decimal_literal(w) {
                    *w = w.replace(',', ".");
                }
            }
        }
    }
    let (main_name, program_map) = match parse_programs(&toks) {
        Ok(p) => p,
        Err(e) => {
            push(ProbePhase::Parse, false, e.to_string());
            return out;
        }
    };
    push(ProbePhase::Parse, true, String::new());

    // Phase 4: resolution (switches, collation, FD record -> file ownership).
    let switches = parse_switches(&toks, first_proc);
    let collation = parse_collation(&toks, first_proc);
    set_collation(collation);
    let file_defs: HashMap<String, FileDef> = program_map
        .get(&main_name)
        .map(|p| {
            p.files
                .iter()
                .map(|f| (f.name.clone(), f.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut record_files: HashMap<String, String> = HashMap::new();
    if let Some(p) = program_map.get(&main_name) {
        for f in &p.files {
            for r in &f.records {
                record_files
                    .entry(r.clone())
                    .or_insert_with(|| f.name.clone());
            }
        }
    }
    let reports: HashMap<String, ReportDef> = program_map
        .get(&main_name)
        .map(|p| p.reports.clone())
        .unwrap_or_default();
    push(ProbePhase::Resolution, true, String::new());

    let ctx = Ctx {
        programs: &program_map,
        dialect,
        currency,
        decimal_comma,
        collation,
        switches,
        print_redirect: false,
        printer: RefCell::new(Vec::new()),
        stop_run: Cell::new(false),
        exit_perform: Cell::new(false),
        exit_cycle: Cell::new(false),
        next_sentence: Cell::new(false),
        call_state: RefCell::new(HashMap::new()),
        goto: RefCell::new(None),
        file_defs,
        record_files,
        files: RefCell::new(HashMap::new()),
        reports,
    };
    let main = ctx
        .programs
        .get(&main_name)
        .expect("main program is registered");

    // Phase 5: layout (WORKING-STORAGE / FD / REPORT model construction).
    let fields = match build_program_fields(main, &ctx) {
        Ok(f) => f,
        Err(e) => {
            push(ProbePhase::Layout, false, e.to_string());
            return out;
        }
    };
    push(ProbePhase::Layout, true, String::new());

    // Phase 6: semantic check (skip-mode body walk, exactly as check_program).
    CHECK_MODE.with(|c| c.set(true));
    let body_check: Result<(), RunError> = (|| {
        let names: Vec<String> = ctx.programs.keys().cloned().collect();
        for name in names {
            let prog = &ctx.programs[&name];
            let mut fields = if name == main_name {
                fields.clone()
            } else {
                build_program_fields(prog, &ctx)?
            };
            let mut p = 0usize;
            while p < prog.proc_toks.len() {
                let before = p;
                if run_block(
                    &prog.proc_toks,
                    &mut p,
                    &mut fields,
                    &mut Vec::new(),
                    false,
                    &ctx,
                )? {
                    break;
                }
                if p == before {
                    p += 1;
                }
                if matches!(prog.proc_toks.get(p), Some(Tok::Dot)) {
                    p += 1;
                }
            }
        }
        Ok(())
    })();
    CHECK_MODE.with(|c| c.set(false));
    match body_check {
        Ok(()) => push(ProbePhase::Check, true, String::new()),
        Err(e) => {
            push(ProbePhase::Check, false, e.to_string());
            return out;
        }
    }

    if run {
        // Phases 7-8: prepare + execute -- exactly the run path of run_program_redirected. The
        // static phases above already passed, so a run-mode error is the execution boundary.
        match run_program_redirected(source, dialect, false) {
            Ok((_out, _printer, rc)) => {
                push(ProbePhase::Prepare, true, String::new());
                push(ProbePhase::Execute, true, format!("exit {rc}"));
            }
            Err(e) => {
                push(ProbePhase::Prepare, true, String::new());
                push(ProbePhase::Execute, false, e.to_string());
            }
        }
    }
    out
}

/// Read the program's final `RETURN-CODE` register as the process exit code (`MOVE n TO RETURN-CODE` /
/// `STOP RUN n`). Defaults to 0.
fn read_return_code(fields: &HashMap<String, Field>) -> i32 {
    match fields.get("RETURN-CODE").map(|f| &f.storage) {
        Some(Storage::Numeric(a)) => {
            let f = &fields["RETURN-CODE"];
            source_to_decimal(&f.bytes, a)
                .ok()
                .map(|d| {
                    let mag: i64 = d.digits.iter().fold(0i64, |acc, &x| acc * 10 + x as i64);
                    if d.negative {
                        -mag as i32
                    } else {
                        mag as i32
                    }
                })
                .unwrap_or(0)
        }
        _ => 0,
    }
}

/// A program's parsed shape: WORKING-STORAGE + LINKAGE items, the `PROCEDURE DIVISION USING` parameter
/// names, and the procedure-body tokens.
struct ProgramDef {
    ws: Vec<ProgItem>,
    /// Parsed LINKAGE SECTION items (retained for structure/forensics; CALL binds USING by position, not by
    /// these declarations yet).
    #[allow(dead_code)]
    linkage: Vec<ProgItem>,
    using: Vec<String>,
    /// `SELECT ... ASSIGN` + `FD` declared files (the subset: sequential / line-sequential).
    files: Vec<FileDef>,
    /// `RD` report descriptions (REPORT SECTION) by report name.
    reports: HashMap<String, ReportDef>,
    proc_toks: Vec<Tok>,
    /// Per-token source line map of `proc_toks` (threaded through the subscript-glue and
    /// qualified-name transforms), for libcob-shaped fatal-error line reporting.
    proc_lines: Vec<usize>,
    /// `PROGRAM-ID. name IS INITIAL` -- the program's WORKING-STORAGE is re-initialized to its VALUE
    /// clauses on EVERY entry, rather than persisting (static) across CALLs.
    is_initial: bool,
    /// `PROGRAM-ID. name PROTOTYPE.` -- a prototype declaration (signature only; cobc warns
    /// 'handling of PROGRAM PROTOTYPE is unfinished'). It is never the run's main program and is
    /// not executable; CALL targets it as a typed boundary.
    is_prototype: bool,
}

/// A file's record organization (the subset). `LINE SEQUENTIAL` writes each record as a `\n`-terminated
/// line (trailing spaces trimmed by the oracle); `SEQUENTIAL` is fixed-length record-sequential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileOrg {
    LineSequential,
    Sequential,
    /// `ORGANIZATION RELATIVE` -- records addressed by a 1-based relative record number (the RELATIVE KEY).
    /// Modelled position-indexed in `FileState.records`, where an empty slot means absent/deleted.
    Relative,
    /// `ORGANIZATION INDEXED` -- records addressed by a RECORD KEY field within the record. Stored in
    /// `FileState.records`; READ NEXT / START present them in ascending RECORD KEY order.
    Indexed,
    /// A `SORT DESCRIPTION` (`SD` in the FILE SECTION) -- the work file of `SORT`/`MERGE` and the subject
    /// of `RELEASE` (input) / `RETURN` (output). Upstream requires the SD to be `SELECT`ed in FILE-CONTROL
    /// (an SD without a SELECT is "not defined"); SORT/MERGE/RETURN on any other organization is a
    /// compile-time error ("must be an SD filename"), and RELEASE of an FD record is rejected.
    Sort,
}

/// A declared file: its `SELECT` name, the `FD` record descriptions it reads/writes through, the optional
/// `FILE STATUS` field, the organization, and (for RELATIVE) the RELATIVE KEY field name.
#[derive(Debug, Clone)]
struct FileDef {
    name: String,
    /// The `ASSIGN TO` target -- the in-memory file store is keyed by this, so two SELECTs on the same
    /// physical name share records (a report written then re-read: the disk semantics the oracle has).
    assign: String,
    /// The `01`-level record descriptions declared beneath this file's `FD`/`SD`, in source order. A file
    /// may declare several ALTERNATIVE record descriptions; all of them share ONE record area (GnuCOBOL
    /// union semantics -- see the FD-record pass in `build_program_fields`), any of them may be the subject
    /// of `WRITE`/`REWRITE`/`RELEASE`, and the FIRST is the default receiving record for `READ`/`RETURN`.
    records: Vec<String>,
    status: Option<String>,
    org: FileOrg,
    rel_key: Option<String>,
    /// `RECORD KEY IS field` for an INDEXED file (the key field within the record).
    record_key: Option<String>,
    /// `RECORD IS VARYING ... DEPENDING ON field` -- `Some(field)` marks a variable-length record-sequential
    /// file (GnuCOBOL var-seq on-disk format: a 4-byte record header `[u16 BE length][0x0000]` before each
    /// record). On READ the field is set to the record's actual length; on disk-load each record is kept at
    /// its true length rather than padded to a fixed width.
    varying_dep: Option<String>,
    /// `ACCESS MODE IS RANDOM|DYNAMIC` (vs the default SEQUENTIAL): when true, a plain `READ` of a RELATIVE
    /// / INDEXED file is a keyed (random) read; when false every `READ` is sequential (next in key order).
    /// `READ NEXT` is always sequential regardless.
    access_random: bool,
}

impl FileDef {
    /// The default (first-declared) record -- the receiving record for `READ`/`RETURN`/`SORT` (and the
    /// record used to size fixed-length disk loads). Empty when the FD declared no record descriptions.
    fn primary_record(&self) -> &str {
        self.records.first().map(String::as_str).unwrap_or("")
    }
}

/// One printable report element: a `COLUMN n PIC p {SOURCE field | VALUE lit | SUM field}` entry.
#[derive(Debug, Clone)]
struct RElem {
    column: usize,
    pic: String,
    source: Option<String>,
    value: Option<Tok>,
    sum: Option<String>,
}

/// A report group line's vertical position: `LINE n` (absolute) or `LINE PLUS k` (relative to the current).
#[derive(Debug, Clone, Copy)]
enum LineSpec {
    Abs(usize),
    Plus(usize),
}

/// One `05 LINE ...` line within a report group: its vertical position + its column-placed elements.
#[derive(Debug, Clone)]
struct RLine {
    spec: LineSpec,
    elems: Vec<RElem>,
}

/// The `TYPE` of a report group (Report Writer). `ControlHeading`/`ControlFooting` carry the control name
/// (`FINAL` or a control data-name).
#[derive(Debug, Clone, PartialEq)]
enum GType {
    ReportHeading,
    PageHeading,
    Detail,
    ControlHeading(String),
    ControlFooting(String),
    PageFooting,
    ReportFooting,
}

/// One report group (an `01 [name] TYPE ...` entry): its name (DETAIL groups are GENERATEd by name), its
/// TYPE, and its lines.
#[derive(Debug, Clone)]
struct RGroup {
    name: Option<String>,
    gtype: GType,
    lines: Vec<RLine>,
}

/// A `RD` report description: the file, the PAGE LIMIT line geometry, the CONTROLS list, and the groups.
#[derive(Debug, Clone, Default)]
struct ReportDef {
    file: String,
    page_limit: usize,
    heading: usize,
    first_detail: usize,
    footing: usize,
    controls: Vec<String>,
    groups: Vec<RGroup>,
}

/// The live run state of an active report (between INITIATE and TERMINATE): the page line buffer, the
/// current vertical line, whether the report heading has been emitted, the SUM accumulators (keyed by the
/// SUM source data-name), and whether any detail has printed on the current page.
#[derive(Debug, Default)]
struct ReportRun {
    line: usize,
    page: Vec<Vec<u8>>,
    rh_done: bool,
    page_first: bool,
    // SUM accumulators keyed by (CONTROL FOOTING control name, SUM source data-name): each control level
    // has its own running total, reset when that control breaks (FINAL only at TERMINATE).
    sums: HashMap<(String, String), Decimal>,
    // The control data-name values as of the last GENERATE (to detect a control break).
    ctrl_prev: HashMap<String, Vec<u8>>,
}

/// The live state of an OPEN file: its logical records, the next READ position, and the open mode.
#[derive(Debug, Clone, Default)]
struct FileState {
    records: Vec<Vec<u8>>,
    read_pos: usize,
    /// 0 = closed, 1 = INPUT, 2 = OUTPUT, 3 = EXTEND, 4 = I-O, 5 = locked (CLOSE WITH LOCK).
    /// Since upstream 62b39805c, a LOCKED file behaves like CLOSED for CLOSE (status 42) and
    /// READ/START (status 30) and rejects a re-OPEN with status 38.
    mode: u8,
    /// `WRITE ... AFTER ADVANCING` leaves the line pending: GnuCOBOL emits a final `\n` at CLOSE
    /// (libcob `cob_file_close` / `flag_needs_nl`), so the in-memory store mirrors the disk bytes.
    pending_nl: bool,
}

/// One `01`-level elementary item (its name, PIC, and optional VALUE literal) -- the field is built at run
/// time (so a CALL can build the callee's fields under the same dialect).
#[derive(Clone)]
struct ProgItem {
    /// The COBOL level number (01..49, or 77). Determines group nesting; an item is a GROUP when the next
    /// item has a higher level number and this item has no PIC.
    level: u16,
    name: String,
    pic: String,
    value: Option<Tok>,
    /// `OCCURS n TIMES` element count (1 = a scalar). A 01-level table `01 E PIC 9 OCCURS 3` is `n` copies
    /// of the element, subscripted `E(i)`.
    occurs: usize,
    /// `NAME REDEFINES TARGET`: the field aliases `TARGET`'s storage (see [`Field::redefines`]).
    redefines: Option<String>,
    /// An `88`-level condition-name on a parent item: `Some((parent, values))`. A normal data item is None.
    condition: Option<(String, Vec<CondVal>, Option<String>)>,
    /// `OCCURS ... INDEXED BY idx [idx ...]` -- the table's index name(s). Each becomes an integer index
    /// field; the first is the table's implicit SEARCH index.
    indexed_by: Vec<String>,
    /// `USAGE [IS] <form>` as stated on THIS item (`None` = not stated). A group's USAGE is inherited by
    /// every nested item that does not state its own; `resolve_usage_inheritance` rewrites each `None` to
    /// the effective form before build, so by build time this is `Some(..)`.
    usage: Option<Usage>,
    /// `SIGN IS [LEADING|TRAILING] [SEPARATE]` for a signed DISPLAY numeric: `(separate, leading)`.
    /// Default `(false, false)` = the standard trailing overpunch.
    sign: (bool, bool),
    /// Extra `FieldAttr` flag bits set by clauses: `COB_FLAG_JUSTIFIED` (`JUSTIFIED RIGHT`) and
    /// `COB_FLAG_BLANK_ZERO` (`BLANK WHEN ZERO`). OR-ed into the built field's attr.
    extra_flags: u16,
    /// `USAGE COMP-1`/`COMP-2`: the IEEE float field type (`0x13`/`0x14`); `None` for a non-float item.
    float_kind: Option<u16>,
    /// `OCCURS min TO max DEPENDING ON counter`: the counter data-name (`occurs` holds the MAX). `None`
    /// for a fixed (or no) OCCURS.
    odo_counter: Option<String>,
    /// `66 name RENAMES start [THRU end]`: the `(start, end)` sibling range this item re-groups (an alias
    /// over their contiguous bytes). `None` for a normal item.
    renames: Option<(String, String)>,
    /// `SYNCHRONIZED` / `SYNC` -- align a binary/float item to its natural boundary (slack inserted before
    /// it in the group layout).
    sync: bool,
    /// `EXTERNAL` -- storage shared across the run unit (by name), zero-filled, VALUE ignored.
    external: bool,
    /// `OCCURS ... ASCENDING|DESCENDING KEY` sort direction for a `SEARCH ALL` (binary search) table:
    /// `Some(true)` = ascending, `Some(false)` = descending, `None` = no KEY clause (SEARCH ALL fails closed).
    occurs_key: Option<bool>,
}

/// Resolve `USAGE` group inheritance in place: a data item with no stated `USAGE` inherits the nearest
/// enclosing group's, defaulting to `DISPLAY`. Uses a level-keyed stack of groups that stated a usage.
fn resolve_usage_inheritance(items: &mut [ProgItem]) {
    let mut stack: Vec<(u16, Usage)> = Vec::new();
    for it in items.iter_mut() {
        // pop groups we have exited (a sibling or shallower item ends the group's scope).
        while let Some(&(lvl, _)) = stack.last() {
            if it.level <= lvl {
                stack.pop();
            } else {
                break;
            }
        }
        let stated = it.usage;
        let effective = stated
            .or_else(|| stack.last().map(|&(_, u)| u))
            .unwrap_or(Usage::Display);
        // a stated usage encloses this item's nested children (popped when a <= level item appears).
        if let Some(u) = stated {
            stack.push((it.level, u));
        }
        it.usage = Some(effective);
    }
}

/// Map a USAGE keyword (with or without the `USAGE [IS]` prefix) to the lib's [`Usage`]. Returns `None`
/// for a non-usage word; the COMP-1/COMP-2/POINTER/INDEX/NATIONAL forms the field model does not yet
/// carry are caught separately and fail closed (never silently treated as DISPLAY).
fn usage_from_kw(w: &str) -> Option<Usage> {
    match w {
        "COMP-3" | "COMPUTATIONAL-3" | "PACKED-DECIMAL" => Some(Usage::Comp3),
        "COMP" | "COMPUTATIONAL" | "COMP-4" | "COMPUTATIONAL-4" | "BINARY" => Some(Usage::Comp),
        "COMP-5" | "COMPUTATIONAL-5" => Some(Usage::Comp5),
        "COMP-X" | "COMPUTATIONAL-X" => Some(Usage::CompX),
        "COMP-6" | "COMPUTATIONAL-6" => Some(Usage::Comp6),
        "DISPLAY" => Some(Usage::Display),
        _ => None,
    }
}

/// `USAGE BINARY-CHAR` / `BINARY-SHORT` / `BINARY-LONG` / `BINARY-DOUBLE` -> (fixed byte width, signed
/// implied PIC, unsigned implied PIC). These COMP-5-family synonyms carry BOTH a usage and an implied
/// PIC: the byte width is fixed by the synonym (1/2/4/8), while the implied PIC gives the display digit
/// count (3/5/10/20 = the unsigned decimal capacity of that width). A trailing `SIGNED` (default) or
/// `UNSIGNED` selects the `S9(n)` vs `9(n)` PIC; the unsigned form drops the sign flag for full range.
fn binary_native_usage(w: &str) -> Option<(u8, &'static str, &'static str)> {
    match w {
        "BINARY-CHAR" => Some((1, "S9(3)", "9(3)")),
        "BINARY-SHORT" => Some((2, "S9(5)", "9(5)")),
        "BINARY-LONG" => Some((4, "S9(10)", "9(10)")),
        // BINARY-INT is GnuCOBOL's 4-byte C-int form (the suite's arithmetic tables declare
        // `FILLER USAGE BINARY-INT VALUE 0`); it maps to the BINARY-LONG width/PIC.
        "BINARY-INT" => Some((4, "S9(10)", "9(10)")),
        "BINARY-DOUBLE" => Some((8, "S9(20)", "9(20)")),
        _ => None,
    }
}

/// After a `BINARY-*` keyword at `toks[k]`, consume an optional `SIGNED`/`UNSIGNED` qualifier and return
/// the implied PIC to use (`UNSIGNED` -> the `9(n)` form, else the default signed `S9(n)`). Advances `k`.
fn binary_native_pic(
    toks: &[Tok],
    k: &mut usize,
    signed_pic: &'static str,
    unsigned_pic: &'static str,
) -> &'static str {
    if matches!(toks.get(*k), Some(Tok::Word(x)) if x == "UNSIGNED") {
        *k += 1;
        unsigned_pic
    } else {
        if matches!(toks.get(*k), Some(Tok::Word(x)) if x == "SIGNED") {
            *k += 1;
        }
        signed_pic
    }
}

/// A USAGE keyword for an opaque machine pointer: modelled as an 8-byte field (its value -- an address --
/// is a non-claim, never displayed deterministically), so the byte width is faithful.
fn is_pointer_usage(w: &str) -> bool {
    matches!(w, "POINTER" | "PROGRAM-POINTER" | "FUNCTION-POINTER")
}

/// A synthetic PIC for a USAGE form with no PIC that the front-end models with an equivalent display
/// field: POINTER -> an opaque 8-byte item; INDEX -> a signed binary integer cobc DISPLAYs as `S9(9)`.
fn synthetic_usage_pic(w: &str) -> Option<&'static str> {
    if is_pointer_usage(w) || w == "HANDLE" {
        Some("X(8)")
    } else if w == "INDEX" {
        Some("S9(9)")
    } else {
        None
    }
}

/// The IEEE field type for a `COMP-1` (32-bit float) / `COMP-2` (64-bit double) usage keyword.
fn float_usage_kind(w: &str) -> Option<u16> {
    match w {
        "COMP-1" | "COMPUTATIONAL-1" | "FLOAT-SHORT" => Some(crate::attr::COB_TYPE_NUMERIC_FLOAT),
        "COMP-2" | "COMPUTATIONAL-2" | "FLOAT-LONG" | "FLOAT-DOUBLE" | "FLOAT-EXTENDED" => {
            Some(crate::attr::COB_TYPE_NUMERIC_DOUBLE)
        }
        _ => None,
    }
}

/// A USAGE keyword the field model does not yet carry (fails closed rather than mis-modelling): NATIONAL
/// (UTF-16).
fn unsupported_usage_kw(w: &str) -> bool {
    matches!(w, "NATIONAL")
}

/// A named compile-time CONSTANT (`78 name VALUE x.` or `01 name CONSTANT [GLOBAL] x.`): stored as an
/// Alpha field carrying the literal's display bytes. Frontend-local bit (libcob's attr bits 0x0008 /
/// 0x0010 are COB_FLAG_BLANK_ZERO / COB_FLAG_JUSTIFIED; 0x2000 is free in this model).
const FLAG_CONSTANT: u16 = 0x2000;

/// Build a `COMP-1` (4-byte float) / `COMP-2` (8-byte double) field. The IEEE bytes drive display
/// (`cob_display_common` reads the f32/f64 directly) and decimal<->float conversion (`cob_move`); a
/// VALUE is encoded through the same path.
fn make_float_field(kind: u16, value: Option<&Tok>) -> Result<Field, RunError> {
    let size = if kind == crate::attr::COB_TYPE_NUMERIC_DOUBLE {
        8
    } else {
        4
    };
    // digits/scale are unused by the float display path; a generous decimal width lets the float operand
    // round-trip through the wide-decimal arithmetic intermediate (to_arith_operand).
    let attr = FieldAttr {
        field_type: kind,
        digits: 18,
        scale: 9,
        flags: crate::attr::COB_FLAG_HAVE_SIGN,
    };
    let mut f = Field {
        storage: Storage::Numeric(attr),
        bytes: vec![0u8; size],
        occurs: 1,
        redefines: None,
    };
    if let Some(v) = value {
        init_value(&mut f, v)?;
    }
    Ok(f)
}

/// The shared execution context: the program registry (for CALL resolution) + the dialect / SPECIAL-NAMES
/// needed to build any program's fields, and the UPSI switch environment.
struct Ctx<'a> {
    programs: &'a HashMap<String, ProgramDef>,
    dialect: crate::dialect::Dialect,
    currency: u8,
    decimal_comma: bool,
    /// `PROGRAM COLLATING SEQUENCE IS <ebcdic-alphabet>` -- the byte->weight table alphanumeric comparisons
    /// go through (None = native ASCII byte order). Currently the EBCDIC alphabet (the common case).
    collation: Option<[u8; 256]>,
    switches: SwitchEnv,
    /// `DISPLAY ... UPON PRINTER` is diverted here (instead of stdout) when the print redirect is active
    /// (`COB_DISPLAY_PRINT_FILE`/`_PIPE` set), mirroring libcob's `cob_display_print_file`. The host
    /// (`cobrun`) appends this stream to the redirect file; with no redirect it stays empty and UPON
    /// PRINTER interleaves into stdout, as the oracle does.
    print_redirect: bool,
    printer: RefCell<Vec<u8>>,
    /// `STOP RUN` reached anywhere -- including inside a CALLed contained program -- halts the WHOLE run
    /// (the libcob `longjmp(return_jmp_buf, stop_run)` to the run boundary), whereas `GOBACK` / `EXIT
    /// PROGRAM` only end the current program body and return to the caller. The flag carries the STOP-RUN
    /// decision back across the CALL boundary, where the plain "this body ended" bool cannot distinguish them.
    stop_run: Cell<bool>,
    /// `EXIT PERFORM` / `EXIT PERFORM CYCLE` signals. An inline PERFORM body that executes one of these
    /// returns like a halt (`Ok(true)`); the nearest enclosing PERFORM loop absorbs the signal -- BREAK the
    /// loop (`exit_perform`) or skip to its next iteration (`exit_cycle`) -- so it does not end the program.
    exit_perform: Cell<bool>,
    exit_cycle: Cell<bool>,
    /// `NEXT SENTENCE` signal: the executing block returns like a halt (`Ok(true)`); the enclosing
    /// paragraph/range loop skips to the statement AFTER the next period (not merely past the END-IF, which
    /// is what CONTINUE does).
    next_sentence: Cell<bool>,
    /// Each CALLed contained program's persisted WORKING-STORAGE (COBOL static storage: a subprogram's WS
    /// survives between CALLs). Keyed by program name; absent = never called or CANCELed (next CALL rebuilds
    /// from VALUE clauses). INITIAL programs are never stored here. CANCEL removes the entry.
    call_state: RefCell<HashMap<String, HashMap<String, Field>>>,
    /// A pending `GO TO <paragraph>`: set when a GO TO executes, it makes the enclosing block return like a
    /// halt; the program-body loop then resumes at the named paragraph instead of ending. Resolved + cleared
    /// per program body (a GO TO never targets a label outside its own program), so it does not cross a CALL.
    goto: RefCell<Option<String>>,
    /// Declared file metadata (SELECT/FD) by file name, and the live OPEN state of each file. The front-end
    /// models files logically in memory (a self-contained WRITE-then-READ round-trips), so a program's file
    /// I/O is deterministic on stdout without touching the host filesystem.
    file_defs: HashMap<String, FileDef>,
    /// Pre-resolved record ownership: `FD record name -> owning SELECT name`, built ONCE at run start. A
    /// multi-record FD declares several records for one file; `WRITE`/`REWRITE`/`RELEASE` resolve the
    /// owning file through this map in O(1) -- never by scanning every file at runtime.
    record_files: HashMap<String, String>,
    files: RefCell<HashMap<String, FileState>>,
    /// `RD` report descriptions by report name (from the main program), for INITIATE/GENERATE/TERMINATE.
    reports: HashMap<String, ReportDef>,
}

/// The UPSI switch environment: the live switch states (from `COB_SWITCH_n`) and the `SPECIAL-NAMES`
/// `SWITCH-n ON/OFF STATUS IS <name>` condition-name map.
struct SwitchEnv {
    /// `cob_switch[n]` -- index `n` from `SWITCH-n`; on/off. RefCell so `SET <mnemonic> TO ON|OFF` can toggle
    /// a switch at runtime and the condition-name predicates read the live state.
    states: std::cell::RefCell<[bool; crate::common_misc::COB_SWITCH_COUNT]>,
    /// condition-name -> (switch index, expected ON when true).
    conds: HashMap<String, (usize, bool)>,
    /// `SWITCH-n IS <mnemonic>` -- the user mnemonic name -> switch index (the `SET <mnemonic> TO ON|OFF` target).
    mnemonics: HashMap<String, usize>,
}

impl Default for SwitchEnv {
    fn default() -> Self {
        SwitchEnv {
            states: std::cell::RefCell::new([false; crate::common_misc::COB_SWITCH_COUNT]),
            conds: HashMap::new(),
            mnemonics: HashMap::new(),
        }
    }
}

impl Clone for SwitchEnv {
    fn clone(&self) -> Self {
        SwitchEnv {
            states: std::cell::RefCell::new(*self.states.borrow()),
            conds: self.conds.clone(),
            mnemonics: self.mnemonics.clone(),
        }
    }
}

/// Parse the `SPECIAL-NAMES` switch declarations (`SWITCH-n [ON STATUS IS a] [OFF STATUS IS b]`) before
/// `before`, and load the switch states from the `COB_SWITCH_n` environment (`ON`/`1` -> on, else off --
/// the default is off), mirroring `cob_init`.
fn parse_switches(toks: &[Tok], before: usize) -> SwitchEnv {
    let mut conds: HashMap<String, (usize, bool)> = HashMap::new();
    let mut mnemonics: HashMap<String, usize> = HashMap::new();
    let mut i = 0;
    while i < before {
        if let Some(Tok::Word(w)) = toks.get(i) {
            if let Some(n) = w
                .strip_prefix("SWITCH-")
                .and_then(|s| s.parse::<usize>().ok())
            {
                let mut k = i + 1;
                // optional `IS <mnemonic>` right after SWITCH-n (the SET target name).
                if matches!(toks.get(k), Some(Tok::Word(x)) if x == "IS") {
                    if let Some(Tok::Word(m)) = toks.get(k + 1) {
                        if m != "ON" && m != "OFF" {
                            if n < crate::common_misc::COB_SWITCH_COUNT {
                                mnemonics.insert(m.clone(), n);
                            }
                            k += 2;
                        }
                    }
                }
                while k < before {
                    match toks.get(k) {
                        Some(Tok::Dot) => break,
                        Some(Tok::Word(x)) if x.starts_with("SWITCH-") => break,
                        Some(Tok::Word(x)) if x == "ON" || x == "OFF" => {
                            let on = x == "ON";
                            let mut j = k + 1;
                            if matches!(toks.get(j), Some(Tok::Word(y)) if y == "STATUS") {
                                j += 1;
                            }
                            if matches!(toks.get(j), Some(Tok::Word(y)) if y == "IS") {
                                j += 1;
                            }
                            if let Some(Tok::Word(name)) = toks.get(j) {
                                if n < crate::common_misc::COB_SWITCH_COUNT {
                                    conds.insert(name.clone(), (n, on));
                                }
                                k = j + 1;
                                continue;
                            }
                            k += 1;
                        }
                        _ => k += 1,
                    }
                }
            }
        }
        i += 1;
    }
    let mut states = [false; crate::common_misc::COB_SWITCH_COUNT];
    for (n, slot) in states.iter_mut().enumerate() {
        if let Ok(v) = std::env::var(format!("COB_SWITCH_{n}")) {
            let v = v.trim().to_ascii_uppercase();
            *slot = v == "ON" || v == "1";
        }
    }
    SwitchEnv {
        states: std::cell::RefCell::new(states),
        conds,
        mnemonics,
    }
}

/// Parse `PROGRAM COLLATING SEQUENCE IS <alphabet>` (OBJECT-COMPUTER / SPECIAL-NAMES) before `before`,
/// resolving the alphabet against any `ALPHABET <name> IS EBCDIC` declaration. Returns the EBCDIC
/// collating-weight table when the program's collating sequence is EBCDIC, else None (native ASCII order).
fn parse_collation(toks: &[Tok], before: usize) -> Option<[u8; 256]> {
    // ALPHABET <name> [IS] EBCDIC -> <name> denotes the EBCDIC ordering.
    let mut ebcdic_alphabets: Vec<String> = Vec::new();
    let mut i = 0;
    while i + 2 < before {
        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "ALPHABET") {
            if let Some(Tok::Word(name)) = toks.get(i + 1) {
                let mut j = i + 2;
                if matches!(toks.get(j), Some(Tok::Word(w)) if w == "IS") {
                    j += 1;
                }
                if matches!(toks.get(j), Some(Tok::Word(w)) if w == "EBCDIC") {
                    ebcdic_alphabets.push(name.clone());
                }
            }
        }
        i += 1;
    }
    // PROGRAM COLLATING SEQUENCE [IS] <name>.
    let mut sel: Option<String> = None;
    let mut i = 0;
    while i + 1 < before {
        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "COLLATING")
            && matches!(toks.get(i + 1), Some(Tok::Word(w)) if w == "SEQUENCE")
        {
            let mut j = i + 2;
            if matches!(toks.get(j), Some(Tok::Word(w)) if w == "IS") {
                j += 1;
            }
            if let Some(Tok::Word(name)) = toks.get(j) {
                sel = Some(name.clone());
            }
        }
        i += 1;
    }
    let sel = sel?;
    if sel == "EBCDIC" || ebcdic_alphabets.iter().any(|a| a == &sel) {
        Some(crate::ebcdic::ebcdic_collation())
    } else {
        None
    }
}

/// Uppercase COBOL source -- keywords and user names are case-insensitive -- while PRESERVING the bytes
/// inside string literals (`"..."` / `'...'`, COBOL's doubled-quote escape kept inside): `DISPLAY "hello"`
/// must print `hello`, and `IF "a" < "A"` must compare distinct bytes (not both folded to `A`).
fn uppercase_outside_quotes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut quote: Option<char> = None;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                out.push(c);
                if c == q {
                    if chars.peek() == Some(&q) {
                        // doubled quote -> an escaped quote, stays inside the literal.
                        out.push(chars.next().unwrap());
                    } else {
                        quote = None;
                    }
                }
            }
            None => {
                if c == '"' || c == '\'' {
                    quote = Some(c);
                    out.push(c);
                } else {
                    out.extend(c.to_uppercase());
                }
            }
        }
    }
    out
}

/// Split the token stream at `PROGRAM-ID` boundaries into a registry of programs; the first is the MAIN.
/// A static qualified-name resolution index built per program: maps each declared data-name to the list of
/// `(canonical_key, parent_chain)` it can refer to (parent chain immediate-first). Used by
/// [`collapse_qualified`] to rewrite `name OF group [OF group...]` into a single resolved field key, and to
/// disambiguate duplicate child names across record layouts (the foundation for `MOVE/ADD/SUBTRACT CORR`).
struct NameIndex {
    by_name: std::collections::HashMap<String, Vec<(String, Vec<String>)>>,
}

/// The bare data-name underlying a (possibly canonicalized) field key: a colliding leaf is stored under
/// `"name\u{1}parent\u{1}..."`, so the bare name is the prefix before the first `\u{1}`.
fn bare_name(key: &str) -> &str {
    key.split('\u{1}').next().unwrap_or(key)
}

/// Whether `needles` appear in `hay` as an ordered subsequence (used to match `OF` qualifiers against an
/// item's parent chain: `A OF G1 OF GG` requires G1 then GG to appear, in containing order).
fn is_ordered_subseq(needles: &[String], hay: &[String]) -> bool {
    let mut hi = hay.iter();
    needles.iter().all(|n| hi.any(|h| h == n))
}

/// Assign canonical field keys + build the qualified-name index for a program's WORKING-STORAGE. A
/// data-name that collides with another (two record layouts sharing a child name) is renamed to a unique
/// key `"name\u{1}<parent chain>"` -- BUT only when every occurrence is a simple elementary scalar and the
/// name is not referenced by REDEFINES / OCCURS DEPENDING / RENAMES / INDEXED BY / an 88 (those stay bare,
/// preserving today's behavior). A program with no duplicate names is returned UNCHANGED (key == name), so
/// existing behavior is byte-identical. Returns the (possibly rewritten) items and the resolution index.
fn canonicalize_ws(ws: &[ProgItem]) -> (Vec<ProgItem>, NameIndex) {
    use std::collections::{HashMap, HashSet};
    // Parent chain per item (immediate-first), via a level stack.
    let mut chains: Vec<Vec<String>> = Vec::with_capacity(ws.len());
    let mut stack: Vec<(u16, String)> = Vec::new();
    for it in ws {
        if it.level == 88 {
            chains.push(Vec::new());
            continue;
        }
        while matches!(stack.last(), Some(&(lvl, _)) if it.level <= lvl) {
            stack.pop();
        }
        chains.push(stack.iter().rev().map(|(_, n)| n.clone()).collect());
        if it.level != 66 {
            stack.push((it.level, it.name.clone()));
        }
    }
    // Names that collide (appear on >1 field-producing item).
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for it in ws {
        if it.level != 88 {
            *counts.entry(it.name.as_str()).or_insert(0) += 1;
        }
    }
    // Names pinned to their bare key (referenced by a cross-clause that uses the bare name).
    let mut pinned: HashSet<String> = HashSet::new();
    for it in ws {
        if let Some(t) = &it.redefines {
            pinned.insert(t.clone());
        }
        if let Some(c) = &it.odo_counter {
            pinned.insert(c.clone());
        }
        if let Some((s, e)) = &it.renames {
            pinned.insert(s.clone());
            pinned.insert(e.clone());
        }
        for ib in &it.indexed_by {
            pinned.insert(ib.clone());
        }
        if it.level == 88 {
            if let Some((p, _, _)) = &it.condition {
                pinned.insert(p.clone());
            }
        }
    }
    let simple = |it: &ProgItem| {
        it.level != 88
            && it.level != 66
            && !it.pic.is_empty()
            && it.occurs <= 1
            && it.redefines.is_none()
            && it.float_kind.is_none()
            && it.indexed_by.is_empty()
            && it.odo_counter.is_none()
    };
    let mut out = ws.to_vec();
    let mut idx = NameIndex {
        by_name: HashMap::new(),
    };
    for (i, it) in ws.iter().enumerate() {
        if it.level == 88 {
            continue;
        }
        let colliding = counts.get(it.name.as_str()).copied().unwrap_or(0) > 1;
        let key = if it.name == "FILLER" {
            // FILLER is never referenced by name, but two FILLERs under the SAME group share both the bare
            // name AND the parent chain -- so a chain-based key still collides in the flat field map, which
            // corrupts a group's byte distribution when the FILLERs differ in size (one field's length is
            // used for every FILLER slot). Give each a UNIQUE key (bare_name stays "FILLER" via the \u{1}).
            format!("FILLER\u{1}\u{4}{i}")
        } else if colliding && !pinned.contains(&it.name) && simple(it) && !chains[i].is_empty() {
            format!("{}\u{1}{}", it.name, chains[i].join("\u{1}"))
        } else {
            it.name.clone()
        };
        out[i].name = key.clone();
        idx.by_name
            .entry(it.name.clone())
            .or_default()
            .push((key, chains[i].clone()));
    }
    (out, idx)
}

/// Re-glue a multi-subscript reference that the lexer split on internal whitespace (`N(I, J)` lexes as
/// `N(I,` + `J)`). Only a token whose prefix before the first `(` is a declared DATA-NAME is glued -- a
/// `FUNCTION name(...)` call keeps its split form (resolve_functions tracks paren depth across tokens).
fn glue_subscripts(toks: &[Tok], lines: &[usize], idx: &NameIndex) -> (Vec<Tok>, Vec<usize>) {
    let unbalanced = |w: &str| w.matches('(').count() > w.matches(')').count();
    // Any '(' may be a subscript needing a glue (a split `C(2`/`3)`, OR a `name (WS-SUB)` whose balanced
    // subscript token follows a space) -- so process whenever the stream has a paren at all.
    if !toks
        .iter()
        .any(|t| matches!(t, Tok::Word(w) if w.contains('(')))
    {
        return (toks.to_vec(), lines.to_vec());
    }
    // Glue `name(... )` (or `name (...)`) whose subscript list the lexer split on internal whitespace, into a
    // single token. A SPACE is inserted between the joined fragments so space-separated subscripts survive
    // (`C(2` + `3)` -> `C(2 3)`, NOT `C(23)`); the name stays attached to its `(`.
    let mut out = Vec::with_capacity(toks.len());
    let mut out_lines = Vec::with_capacity(toks.len());
    let mut i = 0;
    while i < toks.len() {
        if let Tok::Word(w) = &toks[i] {
            // Case A: `NAME(...` -- the name and opening paren are in this token; it is unbalanced.
            // Case B: `NAME (...` -- this token is a bare known name and the NEXT token opens the subscript.
            let case_a = unbalanced(w) && {
                let prefix = &w[..w.find('(').unwrap()];
                !prefix.is_empty() && idx.by_name.contains_key(prefix)
            };
            // Case B: a bare known name, then a separate token opening the subscript -- whether the lexer
            // split it unbalanced (`(1`/`1)`) or kept it balanced (`(WS-SUB)`) after a space.
            let case_b = !w.contains('(')
                && idx.by_name.contains_key(w.as_str())
                && matches!(toks.get(i + 1), Some(Tok::Word(w2)) if w2.starts_with('('));
            if case_a || case_b {
                // Seed `glued` with the name+paren start (case A: this token; case B: name directly + next token).
                let (mut glued, mut j) = if case_a {
                    (w.clone(), i + 1)
                } else {
                    let mut g = w.clone();
                    if let Some(Tok::Word(w2)) = toks.get(i + 1) {
                        g.push_str(w2);
                    }
                    (g, i + 2)
                };
                let mut depth =
                    glued.matches('(').count() as i64 - glued.matches(')').count() as i64;
                while depth > 0 && j < toks.len() {
                    if let Tok::Word(w2) = &toks[j] {
                        glued.push(' ');
                        glued.push_str(w2);
                        depth += w2.matches('(').count() as i64 - w2.matches(')').count() as i64;
                        j += 1;
                    } else {
                        break;
                    }
                }
                out.push(Tok::Word(glued));
                out_lines.push(lines[i]);
                i = j;
                continue;
            }
        }
        out.push(toks[i].clone());
        out_lines.push(lines[i]);
        i += 1;
    }
    (out, out_lines)
}

/// Rewrite every `name OF group [OF group...]` (and `IN`) reference in a token stream into a single resolved
/// field-key token, using the static [`NameIndex`]. A bare name with one candidate is left as-is (its key
/// equals the name); an unresolvable / ambiguous qualified reference is left untouched (it errors downstream
/// as before). No-op when the program has no duplicate names AND the stream has no `OF`/`IN`.
fn collapse_qualified(toks: &[Tok], lines: &[usize], idx: &NameIndex) -> (Vec<Tok>, Vec<usize>) {
    let has_renames = idx
        .by_name
        .iter()
        .any(|(b, v)| v.iter().any(|(k, _)| k != b));
    let has_of = toks
        .iter()
        .any(|t| matches!(t, Tok::Word(w) if w == "OF" || w == "IN"));
    if !has_renames && !has_of {
        return (toks.to_vec(), lines.to_vec());
    }
    let mut out = Vec::with_capacity(toks.len());
    let mut out_lines = Vec::with_capacity(toks.len());
    let mut i = 0;
    while i < toks.len() {
        if let Tok::Word(w) = &toks[i] {
            // Strip any leading `(` the lexer glued on (a parenthesised operand `(SALES-AMOUNT OF REC ...`);
            // it is re-attached after the qualified name is resolved, so the arithmetic parser still sees it.
            let lp = w.bytes().take_while(|&b| b == b'(').count();
            let (base0, base_sub) = split_subscript(&w[lp..]);
            let base = base0.to_string();
            // The subscript of a qualified reference syntactically follows the LAST qualifier:
            // `LEAF OF GROUP (sub)` (glued by glue_subscripts to `... GROUP(sub)`) -- so a subscript carried
            // by a qualifier token belongs to the whole resolved reference, not to that group name.
            let mut sub: Option<String> = base_sub.map(|s| s.to_string());
            let mut quals: Vec<String> = Vec::new();
            let mut j = i + 1;
            while matches!(toks.get(j), Some(Tok::Word(q)) if q == "OF" || q == "IN") {
                if let Some(Tok::Word(q2)) = toks.get(j + 1) {
                    let (qbase, qsub) = split_subscript(q2);
                    quals.push(qbase.to_string());
                    if let Some(qs) = qsub {
                        sub = Some(qs.to_string());
                    }
                    j += 2;
                } else {
                    break;
                }
            }
            if let Some(cands) = idx.by_name.get(base.as_str()) {
                let resolved: Option<String> = if !quals.is_empty() {
                    let m: Vec<&String> = cands
                        .iter()
                        .filter(|(_, chain)| is_ordered_subseq(&quals, chain))
                        .map(|(k, _)| k)
                        .collect();
                    if m.len() == 1 {
                        Some(m[0].clone())
                    } else {
                        None
                    }
                } else if cands.len() == 1 {
                    Some(cands[0].0.clone())
                } else {
                    None
                };
                if let Some(key) = resolved {
                    let inner = match &sub {
                        Some(s) => format!("{key}({s})"),
                        None => key,
                    };
                    let nw = if lp > 0 {
                        format!("{}{}", "(".repeat(lp), inner)
                    } else {
                        inner
                    };
                    out.push(Tok::Word(nw));
                    out_lines.push(lines[i]);
                    i = j;
                    continue;
                }
            }
        }
        out.push(toks[i].clone());
        out_lines.push(lines[i]);
        i += 1;
    }
    (out, out_lines)
}

/// The elementary leaf children of a group field, as `(canonical_key, bare_name)` pairs (skipping nested
/// groups and SYNC slack FILLERs) -- the candidate set for a `CORRESPONDING` match.
fn corr_leaves(
    fields: &HashMap<String, Field>,
    group: &str,
) -> Result<Vec<(String, String)>, RunError> {
    match fields.get(group).map(|f| &f.storage) {
        Some(Storage::Group { children }) => Ok(children
            .iter()
            .filter(|c| {
                !c.starts_with('\u{3}')
                    && !matches!(
                        fields.get(*c).map(|f| &f.storage),
                        Some(Storage::Group { .. })
                    )
            })
            // FILLER never participates in CORRESPONDING (cobc excludes unnamed items) -- so a separator
            // FILLER in the target keeps its own value (e.g. the `-` in a `yyyy-mm-dd` trailer date).
            .filter(|c| bare_name(c) != "FILLER")
            .map(|c| (c.clone(), bare_name(c).to_string()))
            .collect()),
        Some(_) => Err(RunError::Unsupported(format!(
            "CORRESPONDING operand `{group}` is not a group item"
        ))),
        None => Err(RunError::UndefinedName(group.to_string())),
    }
}

/// The `(src_key, dst_key)` pairs for `CORRESPONDING src dst`: elementary leaves present in BOTH groups
/// under the same bare name (matched in src declaration order).
fn corr_pairs(
    fields: &HashMap<String, Field>,
    src: &str,
    dst: &str,
) -> Result<Vec<(String, String)>, RunError> {
    let sc = corr_leaves(fields, src)?;
    let dc = corr_leaves(fields, dst)?;
    let mut pairs = Vec::new();
    for (sk, sb) in &sc {
        if let Some((dk, _)) = dc.iter().find(|(_, db)| db == sb) {
            pairs.push((sk.clone(), dk.clone()));
        }
    }
    Ok(pairs)
}

fn parse_programs(toks: &[Tok]) -> Result<(String, HashMap<String, ProgramDef>), RunError> {
    let starts: Vec<usize> = toks
        .iter()
        .enumerate()
        .filter(|(_, t)| matches!(t, Tok::Word(w) if w == "PROGRAM-ID"))
        .map(|(i, _)| i)
        .collect();
    if starts.is_empty() {
        return Err(RunError::Unsupported("no PROGRAM-ID".into()));
    }
    let mut map = HashMap::new();
    let mut main_name = None;
    for (idx, &s) in starts.iter().enumerate() {
        let end = starts.get(idx + 1).copied().unwrap_or(toks.len());
        let (name, def) = parse_one_program(toks, s, end)?;
        // A PROTOTYPE is a signature-only declaration: never the run's main program (cobc compiles
        // a prototype-containing source with the FIRST executable program as the entry point).
        if main_name.is_none() && !def.is_prototype {
            main_name = Some(name.clone());
        }
        map.insert(name, def);
    }
    let main_name = main_name.ok_or_else(|| {
        RunError::Unsupported("no executable program (only PROTOTYPE declarations)".into())
    })?;
    Ok((main_name, map))
}

/// Parse one program from `toks[start..end]` (start is its `PROGRAM-ID`).
fn parse_one_program(
    toks: &[Tok],
    start: usize,
    end: usize,
) -> Result<(String, ProgramDef), RunError> {
    // PROGRAM-ID. NAME.
    let mut k = start + 1;
    if matches!(toks.get(k), Some(Tok::Dot)) {
        k += 1;
    }
    let name = match toks.get(k) {
        Some(Tok::Word(w)) => w.clone(),
        _ => {
            return Err(RunError::Unsupported(
                "expected program name after PROGRAM-ID".into(),
            ))
        }
    };
    // PROGRAM-ID. name [IS] [INITIAL | COMMON | RECURSIVE]. -- scan the paragraph (to its '.') for INITIAL.
    let mut is_initial = false;
    let mut is_prototype = false;
    let mut q = k + 1;
    while let Some(t) = toks.get(q) {
        match t {
            Tok::Dot => break,
            Tok::Word(w) if w == "INITIAL" => {
                is_initial = true;
                break;
            }
            // `PROGRAM-ID. name PROTOTYPE.` -- a signature-only declaration (upstream 14f0d0908's
            // grammar surface; cobc warns 'handling of PROGRAM PROTOTYPE is unfinished').
            Tok::Word(w) if w == "PROTOTYPE" => {
                is_prototype = true;
                break;
            }
            _ => q += 1,
        }
    }
    let proc_at = find_seq_in(toks, &["PROCEDURE", "DIVISION"], start, end)
        .ok_or_else(|| RunError::Unsupported(format!("{name}: no PROCEDURE DIVISION")))?;
    let ws_at = find_seq_in(toks, &["WORKING-STORAGE", "SECTION"], start, proc_at);
    let link_at = find_seq_in(toks, &["LINKAGE", "SECTION"], start, proc_at);

    // WORKING-STORAGE items: from WS SECTION to LINKAGE SECTION (or PROCEDURE).
    let mut ws = match ws_at {
        Some(w) => parse_items(toks, w + 2, link_at.unwrap_or(proc_at))?,
        None => Vec::new(),
    };
    // ENVIRONMENT FILE-CONTROL (SELECT ... ) + DATA FILE SECTION (FD + 01 record). The FD record items are
    // added to the field table; each file's metadata becomes a FileDef.
    let file_control = parse_file_control(toks, start, proc_at);
    let (mut file_recs, file_rec, report_file, file_varying, sort_files) =
        parse_file_section(toks, start, proc_at)?;
    let mut files: Vec<FileDef> = file_control
        .into_iter()
        .map(
            |(name, assign, org, status, rel_key, record_key, access_random)| {
                let records = file_rec.get(&name).cloned().unwrap_or_default();
                let varying_dep = file_varying.get(&name).cloned();
                FileDef {
                    name,
                    assign,
                    records,
                    status,
                    org,
                    rel_key,
                    record_key,
                    varying_dep,
                    access_random,
                }
            },
        )
        .collect();
    // `SD` files (FILE SECTION sort descriptions) become sort-organization files. Upstream requires an SD
    // to be `SELECT`ed in FILE-CONTROL too (an SD without a SELECT errors "'name' is not defined" at use),
    // so only SDs that have a SELECT get a FileDef; the others stay undeclared and fail closed when used.
    for sf in &sort_files {
        if let Some(f) = files.iter_mut().find(|f| &f.name == sf) {
            f.org = FileOrg::Sort;
        }
    }
    let reports = parse_report_section(toks, start, proc_at, &report_file);
    ws.append(&mut file_recs);
    let linkage = match link_at {
        Some(l) => parse_items(toks, l + 2, proc_at)?,
        None => Vec::new(),
    };

    // PROCEDURE DIVISION [USING name ...].
    let mut p = proc_at + 2;
    let mut using = Vec::new();
    if matches!(toks.get(p), Some(Tok::Word(w)) if w == "USING") {
        p += 1;
        while p < end {
            match toks.get(p) {
                Some(Tok::Word(w))
                    if w == "BY" || w == "REFERENCE" || w == "CONTENT" || w == "VALUE" =>
                {
                    p += 1;
                }
                Some(Tok::Word(w)) => {
                    using.push(w.clone());
                    p += 1;
                }
                _ => break,
            }
        }
    }
    if matches!(toks.get(p), Some(Tok::Dot)) {
        p += 1;
    }
    // proc body: from here to END PROGRAM (or the range end).
    let body_end = find_seq_in(toks, &["END", "PROGRAM"], p, end).unwrap_or(end);
    let proc_toks = toks[p..body_end].to_vec();
    let proc_lines = TOKEN_LINES.with(|l| l.borrow().get(p..body_end).unwrap_or(&[]).to_vec());

    // Statically canonicalize duplicate WORKING-STORAGE data-names and collapse `name OF group` qualifiers in
    // the procedure body ONCE (qualification is purely a function of the declarations). A program with no
    // duplicate names and no OF/IN is returned unchanged, so existing behavior is byte-identical.
    let (ws, idx) = canonicalize_ws(&ws);
    let (proc_toks, proc_lines) = glue_subscripts(&proc_toks, &proc_lines, &idx);
    let (proc_toks, proc_lines) = collapse_qualified(&proc_toks, &proc_lines, &idx);

    Ok((
        name,
        ProgramDef {
            ws,
            linkage,
            using,
            files,
            reports,
            proc_toks,
            proc_lines,
            is_initial,
            is_prototype,
        },
    ))
}

/// Parse `FILE-CONTROL` `SELECT name ASSIGN ... [ORGANIZATION [IS] {LINE SEQUENTIAL|SEQUENTIAL}]
/// [FILE STATUS [IS] status]` entries -> `(name, org, status)`. Unknown clauses are skipped.
#[allow(clippy::type_complexity)]
fn parse_file_control(
    toks: &[Tok],
    start: usize,
    end: usize,
) -> Vec<(
    String,
    String,
    FileOrg,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
)> {
    let fc = match find_seq_in(toks, &["FILE-CONTROL"], start, end) {
        Some(i) => i + 1,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    let mut i = fc;
    while i < end {
        match toks.get(i) {
            Some(Tok::Word(w)) if w == "DATA" => break,
            Some(Tok::Word(w)) if w == "SELECT" => {
                i += 1;
                let name = match toks.get(i) {
                    Some(Tok::Word(w)) => w.clone(),
                    _ => break,
                };
                i += 1;
                let mut org = FileOrg::Sequential;
                let mut status = None;
                let mut rel_key = None;
                let mut record_key = None;
                let mut access_random = false; // ACCESS RANDOM/DYNAMIC -> a plain READ is keyed; default (SEQUENTIAL) = next
                let mut assign = name.clone();
                while i < end {
                    match toks.get(i) {
                        Some(Tok::Dot) => {
                            i += 1;
                            break;
                        }
                        // ASSIGN [TO] [DYNAMIC] {"path" | word} -- the physical file the store keys on.
                        Some(Tok::Word(w)) if w == "ASSIGN" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "TO") {
                                i += 1;
                            }
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "DYNAMIC" || w == "EXTERNAL")
                            {
                                i += 1;
                            }
                            match toks.get(i) {
                                Some(Tok::Str(s)) => {
                                    assign = String::from_utf8_lossy(s).to_string();
                                    i += 1;
                                }
                                Some(Tok::Word(w)) => {
                                    assign = w.clone();
                                    i += 1;
                                }
                                _ => {}
                            }
                        }
                        Some(Tok::Word(w)) if w == "ORGANIZATION" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") {
                                i += 1;
                            }
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "LINE") {
                                org = FileOrg::LineSequential;
                                i += 1;
                                if matches!(toks.get(i), Some(Tok::Word(w)) if w == "SEQUENTIAL") {
                                    i += 1;
                                }
                            } else if matches!(toks.get(i), Some(Tok::Word(w)) if w == "RELATIVE") {
                                org = FileOrg::Relative;
                                i += 1;
                            } else if matches!(toks.get(i), Some(Tok::Word(w)) if w == "INDEXED") {
                                org = FileOrg::Indexed;
                                i += 1;
                            } else if matches!(toks.get(i), Some(Tok::Word(w)) if w == "SEQUENTIAL")
                            {
                                org = FileOrg::Sequential;
                                i += 1;
                            }
                        }
                        // RELATIVE KEY [IS] field
                        Some(Tok::Word(w)) if w == "RELATIVE" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "KEY") {
                                i += 1;
                            }
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") {
                                i += 1;
                            }
                            if let Some(Tok::Word(w)) = toks.get(i) {
                                rel_key = Some(w.clone());
                                i += 1;
                            }
                        }
                        // RECORD KEY [IS] field  (INDEXED)
                        Some(Tok::Word(w)) if w == "RECORD" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "KEY") {
                                i += 1;
                            }
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") {
                                i += 1;
                            }
                            if let Some(Tok::Word(w)) = toks.get(i) {
                                record_key = Some(w.clone());
                                i += 1;
                            }
                        }
                        Some(Tok::Word(w)) if w == "STATUS" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") {
                                i += 1;
                            }
                            if let Some(Tok::Word(w)) = toks.get(i) {
                                status = Some(w.clone());
                                i += 1;
                            }
                        }
                        // ACCESS [MODE] [IS] {SEQUENTIAL|RANDOM|DYNAMIC} -- only RANDOM/DYNAMIC make a plain
                        // READ keyed; SEQUENTIAL (the default) makes every READ a next-in-key-order read.
                        Some(Tok::Word(w)) if w == "ACCESS" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "MODE") {
                                i += 1;
                            }
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") {
                                i += 1;
                            }
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "RANDOM" || w == "DYNAMIC")
                            {
                                access_random = true;
                                i += 1;
                            } else if matches!(toks.get(i), Some(Tok::Word(w)) if w == "SEQUENTIAL")
                            {
                                i += 1;
                            }
                        }
                        _ => i += 1,
                    }
                }
                out.push((
                    name,
                    assign,
                    org,
                    status,
                    rel_key,
                    record_key,
                    access_random,
                ));
            }
            None => break,
            _ => i += 1,
        }
    }
    out
}

/// Parse the `FILE SECTION` `FD name [clauses]. 01 record ...` / `SD name [clauses]. 01 record ...` entries
/// -> (the record items to add to the field table, a file-name -> record-names map, and the set of `SD`
/// (sort-description) file names). A file may declare SEVERAL `01` record descriptions (alternative layouts
/// over one shared record area); all of them are collected, in source order.
#[allow(clippy::type_complexity)]
fn parse_file_section(
    toks: &[Tok],
    start: usize,
    end: usize,
) -> Result<
    (
        Vec<ProgItem>,
        HashMap<String, Vec<String>>,
        HashMap<String, String>,
        HashMap<String, String>,
        HashSet<String>,
    ),
    RunError,
> {
    let fs = match find_seq_in(toks, &["FILE", "SECTION"], start, end) {
        Some(i) => i + 2,
        None => {
            return Ok((
                Vec::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashSet::new(),
            ))
        }
    };
    // the FILE SECTION ends at the next section (WORKING-STORAGE, LOCAL-STORAGE, LINKAGE, or REPORT).
    let ws_at = [
        find_seq_in(toks, &["WORKING-STORAGE", "SECTION"], fs, end),
        find_seq_in(toks, &["REPORT", "SECTION"], fs, end),
        find_seq_in(toks, &["LINKAGE", "SECTION"], fs, end),
    ]
    .into_iter()
    .flatten()
    .min()
    .unwrap_or(end);
    let mut recs = Vec::new();
    let mut file_rec = HashMap::new();
    let mut report_file = HashMap::new();
    let mut file_varying = HashMap::new();
    let mut sort_files = HashSet::new();
    let mut i = fs;
    while i < ws_at {
        match toks.get(i) {
            Some(Tok::Word(w)) if w == "FD" || w == "SD" => {
                let is_sort = w == "SD";
                i += 1;
                let fname = match toks.get(i) {
                    Some(Tok::Word(w)) => w.clone(),
                    _ => break,
                };
                if is_sort {
                    sort_files.insert(fname.clone());
                }
                i += 1;
                // scan the FD clauses to the period; capture `REPORT[S] [IS|ARE] r1 [r2 ...]` and a
                // `RECORD [IS] VARYING [IN SIZE] [FROM n] [TO m] [CHARACTERS] [DEPENDING [ON] field]` clause.
                while i < ws_at && !matches!(toks.get(i), Some(Tok::Dot)) {
                    if matches!(toks.get(i), Some(Tok::Word(w)) if w == "REPORT" || w == "REPORTS")
                    {
                        i += 1;
                        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS" || w == "ARE") {
                            i += 1;
                        }
                        while let Some(Tok::Word(r)) = toks.get(i) {
                            report_file.insert(r.clone(), fname.clone());
                            i += 1;
                        }
                        continue;
                    }
                    if matches!(toks.get(i), Some(Tok::Word(w)) if w == "DEPENDING") {
                        i += 1;
                        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "ON") {
                            i += 1;
                        }
                        if let Some(Tok::Word(field)) = toks.get(i) {
                            file_varying.insert(fname.clone(), field.clone());
                            i += 1;
                        }
                        continue;
                    }
                    i += 1;
                }
                if matches!(toks.get(i), Some(Tok::Dot)) {
                    i += 1;
                }
                let rec_start = i;
                let mut rec_end = i;
                while rec_end < ws_at
                    && !matches!(toks.get(rec_end), Some(Tok::Word(w)) if w == "FD" || w == "SD")
                {
                    rec_end += 1;
                }
                let items = parse_items(toks, rec_start, rec_end)?;
                // Every 01-level record description beneath this FD/SD belongs to the file. The items list
                // is flat (each 01 followed by its subordinate levels), so the record names are the level-1
                // entries in order; the subordinates attach to their nearest 01 during field building.
                let record_names: Vec<String> = items
                    .iter()
                    .filter(|it| it.level == 1)
                    .map(|it| it.name.clone())
                    .collect();
                if !record_names.is_empty() {
                    file_rec.insert(fname, record_names);
                }
                recs.extend(items);
                i = rec_end;
            }
            _ => i += 1,
        }
    }
    Ok((recs, file_rec, report_file, file_varying, sort_files))
}

/// Parse the `REPORT SECTION` `RD r1. 01 group [TYPE ...]. ... COLUMN n PIC p {SOURCE id | VALUE lit} ...`
/// into report definitions. Minimal subset: each report group is a set of column-placed elements, one
/// output line per `LINE` clause (a group with no LINE clause is one line).
fn parse_report_section(
    toks: &[Tok],
    start: usize,
    end: usize,
    report_file: &HashMap<String, String>,
) -> HashMap<String, ReportDef> {
    let mut reports: HashMap<String, ReportDef> = HashMap::new();
    let rs = match find_seq_in(toks, &["REPORT", "SECTION"], start, end) {
        Some(i) => i + 2,
        None => return reports,
    };
    let word = |i: usize| -> Option<&str> {
        if let Some(Tok::Word(w)) = toks.get(i) {
            Some(w.as_str())
        } else {
            None
        }
    };
    let mut cur_report: Option<String> = None;
    let mut i = rs;
    while i < end {
        let w = match word(i) {
            Some(w) => w.to_string(),
            None => {
                i += 1;
                continue;
            }
        };
        if w == "PROCEDURE" {
            break;
        }
        if w == "RD" {
            i += 1;
            let rname = word(i).map(String::from).unwrap_or_default();
            let file = report_file.get(&rname).cloned().unwrap_or_default();
            // RD geometry defaults: no PAGE LIMIT (page_limit 0 -> the page is flushed at its high-water line,
            // no blank padding -- matching cobc when no PAGE clause is given), heading 1, details from line 1.
            let mut def = ReportDef {
                file,
                page_limit: 0,
                heading: 1,
                first_detail: 1,
                footing: 0,
                controls: Vec::new(),
                groups: Vec::new(),
            };
            i += 1;
            // Parse the RD clauses up to the terminating '.'.
            while i < end && !matches!(toks.get(i), Some(Tok::Dot)) {
                match word(i) {
                    Some("PAGE") => {
                        i += 1;
                        while matches!(word(i), Some("LIMIT" | "LIMITS" | "IS" | "ARE")) {
                            i += 1;
                        }
                        if let Some(n) = word(i).and_then(|s| s.parse::<usize>().ok()) {
                            def.page_limit = n;
                            i += 1;
                        }
                        while matches!(word(i), Some("LINE" | "LINES")) {
                            i += 1;
                        }
                    }
                    Some("HEADING") => {
                        i += 1;
                        if let Some(n) = word(i).and_then(|s| s.parse().ok()) {
                            def.heading = n;
                            i += 1;
                        }
                    }
                    Some("FIRST") => {
                        i += 1;
                        while matches!(word(i), Some("DETAIL" | "DE")) {
                            i += 1;
                        }
                        if let Some(n) = word(i).and_then(|s| s.parse().ok()) {
                            def.first_detail = n;
                            i += 1;
                        }
                    }
                    Some("LAST") => {
                        i += 1;
                        while matches!(word(i), Some("DETAIL" | "DE" | "CONTROL" | "HEADING")) {
                            i += 1;
                        }
                        if word(i).and_then(|s| s.parse::<usize>().ok()).is_some() {
                            i += 1;
                        }
                    }
                    Some("FOOTING") => {
                        i += 1;
                        if let Some(n) = word(i).and_then(|s| s.parse().ok()) {
                            def.footing = n;
                            i += 1;
                        }
                    }
                    Some("CONTROL" | "CONTROLS") => {
                        i += 1;
                        while matches!(word(i), Some("IS" | "ARE")) {
                            i += 1;
                        }
                        while let Some(c) = word(i) {
                            if matches!(c, "PAGE" | "HEADING" | "FIRST" | "LAST" | "FOOTING") {
                                break;
                            }
                            def.controls.push(c.to_string());
                            i += 1;
                        }
                    }
                    _ => i += 1,
                }
            }
            i += 1; // past '.'
            reports.insert(rname.clone(), def);
            cur_report = Some(rname);
            continue;
        }
        // A group header: `01 [name] TYPE [IS] <type> [control].`
        if w == "01" {
            i += 1;
            // optional group data-name (absent when the very next token is TYPE).
            let mut name: Option<String> = None;
            if let Some(n) = word(i) {
                if n != "TYPE" {
                    name = Some(n.to_string());
                    i += 1;
                }
            }
            let mut gtype = GType::Detail;
            // scan the rest of the header line for the TYPE clause.
            let hstart = i;
            while i < end && !matches!(toks.get(i), Some(Tok::Dot)) {
                if matches!(word(i), Some("TYPE")) {
                    i += 1;
                    if matches!(word(i), Some("IS")) {
                        i += 1;
                    }
                    gtype = match word(i) {
                        Some("REPORT") if matches!(word(i + 1), Some("HEADING")) => {
                            i += 2;
                            GType::ReportHeading
                        }
                        Some("REPORT") if matches!(word(i + 1), Some("FOOTING")) => {
                            i += 2;
                            GType::ReportFooting
                        }
                        Some("RH") => {
                            i += 1;
                            GType::ReportHeading
                        }
                        Some("RF") => {
                            i += 1;
                            GType::ReportFooting
                        }
                        Some("PAGE") if matches!(word(i + 1), Some("HEADING")) => {
                            i += 2;
                            GType::PageHeading
                        }
                        Some("PAGE") if matches!(word(i + 1), Some("FOOTING")) => {
                            i += 2;
                            GType::PageFooting
                        }
                        Some("PH") => {
                            i += 1;
                            GType::PageHeading
                        }
                        Some("PF") => {
                            i += 1;
                            GType::PageFooting
                        }
                        Some("DETAIL") | Some("DE") => {
                            i += 1;
                            GType::Detail
                        }
                        Some("CONTROL") if matches!(word(i + 1), Some("HEADING" | "CH")) => {
                            i += 2;
                            let c = word(i)
                                .filter(|c| *c != ".")
                                .map(String::from)
                                .unwrap_or_else(|| "FINAL".into());
                            if word(i).is_some() {
                                i += 1;
                            }
                            GType::ControlHeading(c)
                        }
                        Some("CONTROL") if matches!(word(i + 1), Some("FOOTING" | "CF")) => {
                            i += 2;
                            let c = word(i).map(String::from).unwrap_or_else(|| "FINAL".into());
                            i += 1;
                            GType::ControlFooting(c)
                        }
                        Some("CH") => {
                            i += 1;
                            let c = word(i).map(String::from).unwrap_or_else(|| "FINAL".into());
                            i += 1;
                            GType::ControlHeading(c)
                        }
                        Some("CF") => {
                            i += 1;
                            let c = word(i).map(String::from).unwrap_or_else(|| "FINAL".into());
                            i += 1;
                            GType::ControlFooting(c)
                        }
                        _ => GType::Detail,
                    };
                } else {
                    i += 1;
                }
            }
            let _ = hstart;
            i += 1; // past '.'
            if let Some(rep) = &cur_report {
                if let Some(rd) = reports.get_mut(rep) {
                    rd.groups.push(RGroup {
                        name,
                        gtype,
                        lines: Vec::new(),
                    });
                }
            }
            continue;
        }
        // A `05 [name] LINE [NUMBER IS] {n | PLUS k}` -> a new line in the current group.
        if matches!(word(i), Some(lvl) if lvl.parse::<u16>().map(|n| (2..=49).contains(&n)).unwrap_or(false))
            && (matches!(word(i + 1), Some("LINE")) || matches!(word(i + 2), Some("LINE")))
        {
            // skip the level and optional name to LINE
            i += 1;
            while !matches!(word(i), Some("LINE"))
                && i < end
                && !matches!(toks.get(i), Some(Tok::Dot))
            {
                i += 1;
            }
            i += 1; // past LINE
            while matches!(word(i), Some("NUMBER" | "IS")) {
                i += 1;
            }
            let spec = if matches!(word(i), Some("PLUS")) {
                i += 1;
                LineSpec::Plus(word(i).and_then(|s| s.parse().ok()).unwrap_or(1))
            } else {
                LineSpec::Abs(word(i).and_then(|s| s.parse().ok()).unwrap_or(1))
            };
            i += 1;
            if let Some(rep) = &cur_report {
                if let Some(rd) = reports.get_mut(rep) {
                    if let Some(g) = rd.groups.last_mut() {
                        g.lines.push(RLine {
                            spec,
                            elems: Vec::new(),
                        });
                    }
                }
            }
            continue;
        }
        // A `COLUMN n ... PIC p ... {SOURCE id | VALUE lit | SUM id}` element.
        if w == "COLUMN" || w == "COL" {
            i += 1;
            while matches!(word(i), Some("NUMBER" | "IS" | "PLUS")) {
                i += 1;
            }
            let column: usize = word(i).and_then(|s| s.parse().ok()).unwrap_or(1);
            i += 1;
            let (mut pic, mut source, mut value, mut sum) = (String::new(), None, None, None);
            while i < end && !matches!(toks.get(i), Some(Tok::Dot)) {
                match word(i) {
                    Some("PIC" | "PICTURE") => {
                        i += 1;
                        if matches!(word(i), Some("IS")) {
                            i += 1;
                        }
                        if let Some(p) = word(i) {
                            pic = p.trim_end_matches([',', ';']).to_string();
                            i += 1;
                        }
                    }
                    Some("SOURCE") => {
                        i += 1;
                        if matches!(word(i), Some("IS")) {
                            i += 1;
                        }
                        if let Some(s) = word(i) {
                            source = Some(s.to_string());
                            i += 1;
                        }
                    }
                    Some("SUM") => {
                        i += 1;
                        if let Some(s) = word(i) {
                            sum = Some(s.to_string());
                            i += 1;
                        }
                    }
                    Some("VALUE") => {
                        i += 1;
                        if matches!(word(i), Some("IS")) {
                            i += 1;
                        }
                        value = toks.get(i).cloned();
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
            if let Some(rep) = &cur_report {
                if let Some(rd) = reports.get_mut(rep) {
                    if let Some(g) = rd.groups.last_mut() {
                        if g.lines.is_empty() {
                            g.lines.push(RLine {
                                spec: LineSpec::Plus(1),
                                elems: Vec::new(),
                            });
                        }
                        g.lines.last_mut().unwrap().elems.push(RElem {
                            column,
                            pic,
                            source,
                            value,
                            sum,
                        });
                    }
                }
            }
            continue;
        }
        i += 1;
    }
    reports
}

/// Parse the `01`-level elementary items in `toks[start..end]` (a WORKING-STORAGE or LINKAGE section body).
fn parse_items(toks: &[Tok], start: usize, end: usize) -> Result<Vec<ProgItem>, RunError> {
    let mut items = Vec::new();
    let mut last_item: Option<String> = None; // the most recent data item (parent for an 88 condition-name)
    let mut k = start;
    if matches!(toks.get(k), Some(Tok::Dot)) {
        k += 1; // skip the '.' after SECTION
    }
    while k < end {
        let level = match toks.get(k) {
            Some(Tok::Word(w)) => w.clone(),
            _ => {
                k += 1;
                continue;
            }
        };
        // stop if we reach a new DIVISION/SECTION header word.
        if level == "PROCEDURE" || level == "LINKAGE" || level == "DATA" || level == "REPORT" {
            break;
        }
        // A `78`-level named constant: `78 name [GLOBAL] VALUE [IS] lit.` (a compile-time constant,
        // usable wherever a literal is). Also produced by `01 name CONSTANT [GLOBAL] lit.` below.
        if level == "78" {
            k += 1;
            let name = match toks.get(k) {
                Some(Tok::Word(w)) => w.clone(),
                _ => {
                    return Err(RunError::Unsupported(
                        "expected constant name after 78".into(),
                    ))
                }
            };
            k += 1;
            while matches!(toks.get(k), Some(Tok::Word(w)) if w == "GLOBAL" || w == "CONSTANT") {
                k += 1;
            }
            if matches!(toks.get(k), Some(Tok::Word(w)) if w == "VALUE" || w == "VALUES") {
                k += 1;
                if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS" || w == "ARE") {
                    k += 1;
                }
            }
            let value = toks.get(k).cloned();
            while k < end && !matches!(toks.get(k), Some(Tok::Dot)) {
                k += 1;
            }
            if matches!(toks.get(k), Some(Tok::Dot)) {
                k += 1;
            }
            items.push(ProgItem {
                level: 78,
                name,
                pic: String::new(),
                value,
                occurs: 1,
                redefines: None,
                condition: None,
                indexed_by: Vec::new(),
                usage: None,
                sign: (false, false),
                extra_flags: FLAG_CONSTANT,
                float_kind: None,
                odo_counter: None,
                renames: None,
                sync: false,
                external: false,
                occurs_key: None,
            });
            continue;
        }
        // A `66`-level `RENAMES start [THRU|THROUGH end]` regrouping alias.
        if level == "66" {
            k += 1;
            let rname = match toks.get(k) {
                Some(Tok::Word(w)) => w.clone(),
                _ => return Err(RunError::Unsupported("expected data-name after 66".into())),
            };
            k += 1;
            if matches!(toks.get(k), Some(Tok::Word(w)) if w == "RENAMES") {
                k += 1;
            } else {
                return Err(RunError::Unsupported("66 level without RENAMES".into()));
            }
            let start = match toks.get(k) {
                Some(Tok::Word(w)) => w.clone(),
                _ => {
                    return Err(RunError::Unsupported(
                        "RENAMES without a start data-name".into(),
                    ))
                }
            };
            k += 1;
            let end = if matches!(toks.get(k), Some(Tok::Word(w)) if w == "THRU" || w == "THROUGH")
            {
                k += 1;
                match toks.get(k) {
                    Some(Tok::Word(w)) => {
                        let e = w.clone();
                        k += 1;
                        e
                    }
                    _ => {
                        return Err(RunError::Unsupported(
                            "RENAMES THRU without an end data-name".into(),
                        ))
                    }
                }
            } else {
                start.clone()
            };
            if matches!(toks.get(k), Some(Tok::Dot)) {
                k += 1;
            }
            items.push(ProgItem {
                level: 66,
                name: rname,
                pic: String::new(),
                value: None,
                occurs: 1,
                redefines: None,
                condition: None,
                indexed_by: Vec::new(),
                usage: None,
                sign: (false, false),
                extra_flags: 0,
                float_kind: None,
                odo_counter: None,
                renames: Some((start, end)),
                sync: false,
                external: false,
                occurs_key: None,
            });
            continue;
        }
        // An `88`-level condition-name on the most recent data item.
        if level == "88" {
            k += 1;
            let cname = match toks.get(k) {
                Some(Tok::Word(w)) => w.clone(),
                _ => {
                    return Err(RunError::Unsupported(
                        "expected condition-name after 88".into(),
                    ))
                }
            };
            k += 1;
            let parent = last_item.clone().ok_or_else(|| {
                RunError::Unsupported("88 condition-name with no parent item".into())
            })?;
            // VALUE [IS] v [THRU h] [v2 [THRU h2] ...] .
            if matches!(toks.get(k), Some(Tok::Word(w)) if w == "VALUE" || w == "VALUES") {
                k += 1;
                if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS" || w == "ARE") {
                    k += 1;
                }
            }
            let mut values: Vec<CondVal> = Vec::new();
            let mut false_value: Option<String> = None;
            while k < end {
                match toks.get(k) {
                    Some(Tok::Dot) => {
                        k += 1;
                        break;
                    }
                    // `WHEN SET TO FALSE <lit>` -- the value `SET cond TO FALSE` stores into the parent.
                    Some(Tok::Word(w)) if w == "WHEN" => {
                        k += 1;
                        for kw in ["SET", "TO", "FALSE"] {
                            if matches!(toks.get(k), Some(Tok::Word(x)) if x == kw) {
                                k += 1;
                            }
                        }
                        if let Some(t) = toks.get(k) {
                            false_value = Some(tok_to_cond_word(t));
                            k += 1;
                        }
                    }
                    Some(t) => {
                        let lo = tok_to_cond_word(t);
                        k += 1;
                        if matches!(toks.get(k), Some(Tok::Word(w)) if w == "THRU" || w == "THROUGH")
                        {
                            k += 1;
                            if let Some(ht) = toks.get(k) {
                                values.push(CondVal::Range(lo, tok_to_cond_word(ht)));
                                k += 1;
                            }
                        } else {
                            values.push(CondVal::Single(lo));
                        }
                    }
                    None => break,
                }
            }
            items.push(ProgItem {
                level: 88,
                name: cname,
                pic: String::new(),
                value: None,
                occurs: 1,
                redefines: None,
                condition: Some((parent, values, false_value)),
                indexed_by: Vec::new(),
                usage: None,
                sign: (false, false),
                extra_flags: 0,
                float_kind: None,
                odo_counter: None,
                renames: None,
                sync: false,
                external: false,
                occurs_key: None,
            });
            continue;
        }
        let lvl: u16 = level.parse().unwrap_or(0);
        if lvl == 0 || (lvl > 49 && lvl != 77) {
            // 01..49 group/elementary levels and 77 (independent elementary) are supported; 66 (RENAMES)
            // and other forms fail closed.
            return Err(RunError::Unsupported(format!(
                "unsupported level number {level}"
            )));
        }
        k += 1;
        let name = match toks.get(k) {
            Some(Tok::Word(w)) => w.clone(),
            _ => {
                return Err(RunError::Unsupported(
                    "expected data name after a level number".into(),
                ))
            }
        };
        last_item = Some(name.clone());
        k += 1;
        // `01 name CONSTANT [GLOBAL] lit.` -- a named compile-time constant (78-style): the value
        // literal follows the clause (no VALUE keyword), then the '.'.
        if matches!(toks.get(k), Some(Tok::Word(w)) if w == "CONSTANT") {
            k += 1;
            while matches!(toks.get(k), Some(Tok::Word(w)) if w == "GLOBAL") {
                k += 1;
            }
            let value = toks.get(k).cloned();
            while k < end && !matches!(toks.get(k), Some(Tok::Dot)) {
                k += 1;
            }
            if matches!(toks.get(k), Some(Tok::Dot)) {
                k += 1;
            }
            items.push(ProgItem {
                level: 78,
                name,
                pic: String::new(),
                value,
                occurs: 1,
                redefines: None,
                condition: None,
                indexed_by: Vec::new(),
                usage: None,
                sign: (false, false),
                extra_flags: FLAG_CONSTANT,
                float_kind: None,
                odo_counter: None,
                renames: None,
                sync: false,
                external: false,
                occurs_key: None,
            });
            continue;
        }
        // `NAME REDEFINES TARGET` -- the item aliases TARGET's storage.
        let mut redefines: Option<String> = None;
        if matches!(toks.get(k), Some(Tok::Word(w)) if w == "REDEFINES") {
            k += 1;
            if let Some(Tok::Word(t)) = toks.get(k) {
                redefines = Some(t.clone());
                k += 1;
            }
        }
        let mut pic: Option<String> = None;
        let mut value: Option<Tok> = None;
        let mut occurs: usize = 1;
        let mut indexed: Vec<String> = Vec::new();
        // None = no USAGE stated here (inherit a group's, else DISPLAY); Some = stated on this item.
        let mut usage: Option<Usage> = None;
        // a USAGE form with no PIC modelled via a synthetic PIC (POINTER -> X(8), INDEX -> S9(9)).
        let mut synthetic: Option<&'static str> = None;
        // SIGN IS [LEADING|TRAILING] [SEPARATE]: (separate, leading).
        let mut sign: (bool, bool) = (false, false);
        // JUSTIFIED / BLANK WHEN ZERO -> extra attr flag bits.
        let mut extra_flags: u16 = 0;
        // USAGE COMP-1/COMP-2 -> an IEEE float field type.
        let mut float_kind: Option<u16> = None;
        // OCCURS ... DEPENDING ON counter.
        let mut odo_counter: Option<String> = None;
        let mut occurs_key: Option<bool> = None;
        // SYNCHRONIZED alignment.
        let mut sync = false;
        // EXTERNAL shared storage.
        let mut external = false;
        while k < end {
            match toks.get(k) {
                Some(Tok::Dot) => {
                    k += 1;
                    break;
                }
                Some(Tok::Word(w)) if w == "OCCURS" => {
                    k += 1;
                    if let Some(Tok::Word(n)) = toks.get(k) {
                        occurs = n.parse::<usize>().map_err(|_| {
                            RunError::Unsupported(format!("OCCURS count {n} is not an integer"))
                        })?;
                        k += 1;
                    }
                    // `OCCURS min TO max` -- the physical size is the MAX.
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "TO") {
                        k += 1;
                        if let Some(Tok::Word(m)) = toks.get(k) {
                            occurs = m.parse::<usize>().map_err(|_| {
                                RunError::Unsupported(format!("OCCURS max {m} is not an integer"))
                            })?;
                            k += 1;
                        }
                    }
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "TIMES") {
                        k += 1;
                    }
                    // `DEPENDING [ON] counter` -- the variable-length counter.
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "DEPENDING") {
                        k += 1;
                        if matches!(toks.get(k), Some(Tok::Word(w)) if w == "ON") {
                            k += 1;
                        }
                        if let Some(Tok::Word(c)) = toks.get(k) {
                            odo_counter = Some(c.clone());
                            k += 1;
                        }
                    }
                }
                // `ASCENDING|DESCENDING [KEY] [IS] keyname...` -- the SEARCH ALL sort direction. Record the
                // direction (the binary search reads the key from its WHEN condition); skip the key names.
                Some(Tok::Word(w)) if w == "ASCENDING" || w == "DESCENDING" => {
                    occurs_key = Some(w == "ASCENDING");
                    k += 1;
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "KEY") {
                        k += 1;
                    }
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS") {
                        k += 1;
                    }
                    while let Some(Tok::Word(nm)) = toks.get(k) {
                        if matches!(
                            nm.as_str(),
                            "PIC"
                                | "PICTURE"
                                | "VALUE"
                                | "OCCURS"
                                | "REDEFINES"
                                | "TIMES"
                                | "INDEXED"
                                | "ASCENDING"
                                | "DESCENDING"
                        ) {
                            break;
                        }
                        k += 1;
                    }
                }
                // `INDEXED BY idx [idx ...]` -- read the index name(s) until the next clause/period.
                Some(Tok::Word(w)) if w == "INDEXED" => {
                    k += 1;
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "BY") {
                        k += 1;
                    }
                    while let Some(Tok::Word(nm)) = toks.get(k) {
                        if matches!(
                            nm.as_str(),
                            "PIC" | "PICTURE" | "VALUE" | "OCCURS" | "REDEFINES" | "TIMES"
                        ) {
                            break;
                        }
                        indexed.push(nm.clone());
                        k += 1;
                    }
                }
                Some(Tok::Word(w)) if w == "PIC" || w == "PICTURE" => {
                    k += 1;
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS") {
                        k += 1;
                    }
                    if let Some(Tok::Word(p)) = toks.get(k) {
                        // a trailing ',' / ';' is a CLAUSE SEPARATOR, never picture editing (an insertion
                        // comma must sit between digit positions) -- e.g. `PIC 9, BLANK WHEN ZERO`.
                        pic = Some(p.trim_end_matches([',', ';']).to_string());
                        k += 1;
                    }
                }
                Some(Tok::Word(w)) if w == "VALUE" => {
                    k += 1;
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS") {
                        k += 1;
                    }
                    value = toks.get(k).cloned();
                    k += 1;
                    // `VALUE ALL "lit"` -- the repeating figurative literal (oracle: PIC X(5)
                    // VALUE ALL "ab" -> "ababa"; even into a numeric field with a warning).
                    if matches!(&value, Some(Tok::Word(w)) if w == "ALL") {
                        if let Some(Tok::Str(s)) = toks.get(k) {
                            value = Some(Tok::AllLiteral(s.clone()));
                            k += 1;
                        }
                    }
                }
                // `USAGE [IS] <form>` -- the explicit clause.
                Some(Tok::Word(w)) if w == "USAGE" => {
                    k += 1;
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS") {
                        k += 1;
                    }
                    match toks.get(k) {
                        // BINARY-CHAR/SHORT/LONG/DOUBLE set BOTH the usage (fixed-width native binary) and
                        // the implied PIC (display digits) -- the only USAGE form that needs both.
                        Some(Tok::Word(u)) if binary_native_usage(u).is_some() => {
                            let (width, spic, upic) = binary_native_usage(u).unwrap();
                            k += 1;
                            synthetic = Some(binary_native_pic(toks, &mut k, spic, upic));
                            usage = Some(Usage::CompNative(width));
                        }
                        Some(Tok::Word(u)) if usage_from_kw(u).is_some() => {
                            usage = usage_from_kw(u);
                            k += 1;
                        }
                        Some(Tok::Word(u)) if synthetic_usage_pic(u).is_some() => {
                            synthetic = synthetic_usage_pic(u);
                            k += 1;
                        }
                        Some(Tok::Word(u)) if float_usage_kind(u).is_some() => {
                            float_kind = float_usage_kind(u);
                            k += 1;
                        }
                        Some(Tok::Word(u)) if unsupported_usage_kw(u) => {
                            return Err(RunError::Unsupported(format!("USAGE {u}: cobc 3.2 leaves NATIONAL unfinished (-Wunfinished) -- a non-claim, not a buildable front-end gap")));
                        }
                        Some(Tok::Word(u)) => {
                            return Err(RunError::Unsupported(format!("unrecognized USAGE {u}")));
                        }
                        _ => return Err(RunError::Unsupported("USAGE with no form".into())),
                    }
                }
                // a bare usage keyword (no `USAGE` prefix), e.g. `PIC S9(5) COMP-3` or `BINARY-LONG`.
                Some(Tok::Word(w)) if binary_native_usage(w).is_some() => {
                    let (width, spic, upic) = binary_native_usage(w).unwrap();
                    k += 1;
                    synthetic = Some(binary_native_pic(toks, &mut k, spic, upic));
                    usage = Some(Usage::CompNative(width));
                }
                Some(Tok::Word(w)) if usage_from_kw(w).is_some() => {
                    usage = usage_from_kw(w);
                    k += 1;
                }
                Some(Tok::Word(w)) if synthetic_usage_pic(w).is_some() => {
                    synthetic = synthetic_usage_pic(w);
                    k += 1;
                }
                Some(Tok::Word(w)) if float_usage_kind(w).is_some() => {
                    float_kind = float_usage_kind(w);
                    k += 1;
                }
                Some(Tok::Word(w)) if unsupported_usage_kw(w) => {
                    return Err(RunError::Unsupported(format!("USAGE {w}: cobc 3.2 leaves NATIONAL unfinished (-Wunfinished) -- a non-claim, not a buildable front-end gap")));
                }
                // `JUSTIFIED [RIGHT]` / `JUST [RIGHT]` -- alphanumeric right-justification.
                Some(Tok::Word(w)) if w == "JUSTIFIED" || w == "JUST" => {
                    extra_flags |= crate::attr::COB_FLAG_JUSTIFIED;
                    k += 1;
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "RIGHT") {
                        k += 1;
                    }
                }
                // `EXTERNAL` -- run-unit-shared storage (VALUE ignored, zero-filled).
                Some(Tok::Word(w)) if w == "EXTERNAL" => {
                    external = true;
                    k += 1;
                }
                // `SYNCHRONIZED [LEFT|RIGHT]` / `SYNC` -- natural-boundary alignment.
                Some(Tok::Word(w)) if w == "SYNCHRONIZED" || w == "SYNC" => {
                    sync = true;
                    k += 1;
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "LEFT" || w == "RIGHT") {
                        k += 1;
                    }
                }
                // `BLANK [WHEN] ZERO` -- the field becomes spaces when its value is zero.
                Some(Tok::Word(w)) if w == "BLANK" => {
                    extra_flags |= crate::attr::COB_FLAG_BLANK_ZERO;
                    k += 1;
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "WHEN") {
                        k += 1;
                    }
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "ZERO" || w == "ZEROS" || w == "ZEROES")
                    {
                        k += 1;
                    }
                }
                // `SIGN [IS] [LEADING|TRAILING] [SEPARATE [CHARACTER]]` -- sets the (separate, leading) form.
                Some(Tok::Word(w)) if w == "SIGN" => {
                    k += 1;
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS") {
                        k += 1;
                    }
                    // LEADING/TRAILING, then optional SEPARATE [CHARACTER], in any of the allowed orders.
                    while let Some(Tok::Word(w)) = toks.get(k) {
                        match w.as_str() {
                            "LEADING" => {
                                sign.1 = true;
                                k += 1;
                            }
                            "TRAILING" => {
                                sign.1 = false;
                                k += 1;
                            }
                            "SEPARATE" => {
                                sign.0 = true;
                                k += 1;
                            }
                            "CHARACTER" => {
                                k += 1;
                            }
                            _ => break,
                        }
                    }
                }
                _ => k += 1,
            }
        }
        // a PIC-less item is a GROUP (its children follow at higher level numbers); resolved in build.
        // a USAGE POINTER item has no PIC -- model it as an opaque 8-byte field (the libcob pointer width).
        let pic = match pic {
            Some(p) => p,
            None => synthetic.map(|s| s.to_string()).unwrap_or_default(),
        };
        items.push(ProgItem {
            level: lvl,
            name,
            pic,
            value,
            occurs,
            redefines,
            condition: None,
            indexed_by: indexed,
            usage,
            sign,
            extra_flags,
            float_kind,
            odo_counter,
            renames: None,
            sync,
            external,
            occurs_key,
        });
    }
    resolve_usage_inheritance(&mut items);
    Ok(items)
}

/// `find_seq` restricted to the window `toks[from..to]`.
fn find_seq_in(toks: &[Tok], seq: &[&str], from: usize, to: usize) -> Option<usize> {
    let to = to.min(toks.len());
    'outer: for i in from..to.saturating_sub(seq.len().saturating_sub(1)) {
        for (j, s) in seq.iter().enumerate() {
            match toks.get(i + j) {
                Some(Tok::Word(w)) if w == s => {}
                _ => continue 'outer,
            }
        }
        return Some(i);
    }
    None
}

/// Build a program's runtime field table (its WORKING-STORAGE items + the RETURN-CODE special register).
/// LINKAGE fields are NOT created here -- a CALL fills them from the caller's arguments.
/// A laid-out leaf of a (possibly multi-dimension) table: `(name, byte offset within ONE element of the
/// subtree root, leaf size, dims)` where `dims` is `(occurs, stride)` outermost-first for the OCCURS levels
/// at or below this subtree root that enclose the leaf.
type LeafDesc = (String, usize, usize, Vec<(usize, usize)>);

/// Recursively lay out the subtree rooted at `ws[idx]` over the already-built elementary `fields`, returning
/// `(block_size, leaves)` -- `block_size` is the byte size of ONE instance of `ws[idx]` INCLUDING its own
/// OCCURS, and each leaf's offset/dims are relative to that block. Immediate children are the items at the
/// first deeper level; deeper items are consumed by recursion. (Scope: elementary leaves + sub-groups; the
/// caller's gate keeps REDEFINES/ODO/SYNC/88 out of the subtree.)
fn nested_layout(
    ws: &[ProgItem],
    idx: usize,
    fields: &HashMap<String, Field>,
) -> (usize, Vec<LeafDesc>) {
    let it = &ws[idx];
    let is_group = it.pic.is_empty() && it.float_kind.is_none();
    if !is_group {
        let f = fields.get(&it.name);
        let occ = f.map(|f| f.occurs.max(1)).unwrap_or(it.occurs.max(1));
        let esz = f.map(|f| f.bytes.len() / occ).unwrap_or(0);
        if it.occurs > 1 {
            (
                esz * it.occurs,
                vec![(it.name.clone(), 0, esz, vec![(it.occurs, esz)])],
            )
        } else {
            (esz, vec![(it.name.clone(), 0, esz, vec![])])
        }
    } else {
        let mut elem = 0usize;
        let mut leaves: Vec<LeafDesc> = Vec::new();
        let mut max_align = 1usize; // largest SYNCHRONIZED alignment among descendants (1 = none)
        let mut child_level: Option<u16> = None;
        let mut child_off: HashMap<String, usize> = HashMap::new(); // child name -> start offset (for REDEFINES)
        let mut j = idx + 1;
        while j < ws.len() && ws[j].level > it.level {
            if ws[j].level == 88 || ws[j].level == 66 {
                j += 1;
                continue;
            }
            let cl = *child_level.get_or_insert(ws[j].level);
            if ws[j].level == cl {
                // A SYNCHRONIZED elementary child aligns the running offset (slack before it) to its natural
                // boundary, and bumps the element's max alignment so the element size is padded below -- so
                // every occurrence of a multi-dimension table keeps its SYNC fields aligned (cobc layout).
                if ws[j].sync {
                    let align = fields.get(&ws[j].name).map(sync_align).unwrap_or(1);
                    if align > 1 {
                        max_align = max_align.max(align);
                        if ws[j].redefines.is_none() {
                            elem = elem.div_ceil(align) * align;
                        }
                    }
                }
                // A REDEFINES child overlays its target at the SAME offset and does NOT advance the element
                // size (cobc requires the redefining item to be no larger). Others lay out sequentially.
                let start = match &ws[j].redefines {
                    Some(tgt) => *child_off.get(tgt).unwrap_or(&elem),
                    None => elem,
                };
                let (cblock, cls) = nested_layout(ws, j, fields);
                for (n, off, sz, dims) in cls {
                    leaves.push((n, start + off, sz, dims));
                }
                child_off.insert(ws[j].name.clone(), start);
                if ws[j].redefines.is_none() {
                    elem += cblock;
                }
                // skip past this child's whole subtree
                j += 1;
                while j < ws.len() && ws[j].level > cl {
                    j += 1;
                }
            } else {
                j += 1;
            }
        }
        // Pad the element to its largest SYNC alignment so each occurrence stays aligned (no-op when none).
        if max_align > 1 {
            elem = elem.div_ceil(max_align) * max_align;
        }
        if it.occurs > 1 {
            let occ = it.occurs;
            let leaves2 = leaves
                .into_iter()
                .map(|(n, off, sz, mut dims)| {
                    let mut d = vec![(occ, elem)];
                    d.append(&mut dims);
                    (n, off, sz, d)
                })
                .collect();
            (elem * occ, leaves2)
        } else {
            (elem, leaves)
        }
    }
}

fn build_program_fields(prog: &ProgramDef, ctx: &Ctx) -> Result<HashMap<String, Field>, RunError> {
    let mut fields = HashMap::new();
    ODO_TABLES.with(|m| m.borrow_mut().clear()); // fresh per program build
    REPORT_STATE.with(|m| m.borrow_mut().clear());
    GROUP_OCCURS.with(|m| m.borrow_mut().clear());
    GROUP_CHILD.with(|m| m.borrow_mut().clear());
    NESTED_LEAF.with(|m| m.borrow_mut().clear());
    ALPHABETIC_FIELDS.with(|m| m.borrow_mut().clear());
    FIELD_VALUES.with(|m| m.borrow_mut().clear());
    REDEF_VIEW.with(|m| m.borrow_mut().clear());
    for (gi, it) in prog.ws.iter().enumerate() {
        // An 88-level condition-name carries no storage -- record its parent + values for cond_rel.
        if let Some((parent, values, false_value)) = &it.condition {
            fields.insert(
                it.name.clone(),
                Field {
                    storage: Storage::Condition {
                        parent: parent.clone(),
                        values: values.clone(),
                        false_value: false_value.clone(),
                    },
                    bytes: Vec::new(),
                    occurs: 1,
                    redefines: None,
                },
            );
            continue;
        }
        // A named CONSTANT (78 / 01 CONSTANT): an Alpha field initialized to the literal's display
        // bytes (numeric constants keep their written digits, so arithmetic/display read them back).
        if it.level == 78 || (it.extra_flags & FLAG_CONSTANT) != 0 {
            let bytes = match &it.value {
                Some(Tok::Str(s)) => s.clone(),
                Some(Tok::Word(w)) => w.as_bytes().to_vec(),
                _ => Vec::new(),
            };
            fields.insert(
                it.name.clone(),
                Field {
                    storage: Storage::Alpha(alnum_attr()),
                    bytes,
                    occurs: 1,
                    redefines: None,
                },
            );
            continue;
        }
        // A group item (no PIC) is built after its leaves exist (second pass below) -- but a COMP-1/COMP-2
        // item also has no PIC yet is an elementary float leaf, so it must build here.
        if it.pic.is_empty() && it.float_kind.is_none() {
            // A GROUP with OCCURS is a table of group items, built as an interleaved buffer in pass 2.
            // Admit ONLY the oracle-confirmed subset (single level, fixed count, all-elementary children);
            // fail CLOSED on anything that would need a richer model than the flat field store provides.
            if it.occurs > 1 {
                // A FLAT group-OCCURS supports OCCURS DEPENDING ON (built at MAX, truncated to the counter in
                // pass 2 / read_field). Multi-dimension AND group-of-group tables go through the recursive
                // nested layout (which has no var-length model -> ODO on a NESTED table fails closed there).
                // Still out of subset everywhere: any REDEFINES / ODO / SYNCHRONIZED DESCENDANT in the subtree.
                for sib in &prog.ws[gi + 1..] {
                    if sib.level <= it.level {
                        break;
                    }
                    if sib.level == 88 {
                        continue;
                    }
                    // An OCCURS DEPENDING ON item INSIDE a fixed table is a cobc COMPILE ERROR ("'<group>'
                    // cannot have an OCCURS clause due to '<item>'") -- a table of variable-length items is
                    // not allowed; we fail closed the same way (validation, not a feature gap).
                    if sib.odo_counter.is_some() {
                        return Err(RunError::Unsupported(format!(
                            "group-OCCURS `{}` cannot contain an OCCURS DEPENDING ON item -- cobc rejects a table of variable-length items", it.name
                        )));
                    }
                    // (SYNCHRONIZED descendants are handled by the FLAT branch's per-element alignment slack +
                    // trailing element padding below; the MULTI-DIMENSION branch -- whose nested layout has no
                    // alignment model -- guards SYNC itself.)
                }
                // eligible: the interleaved buffer + per-leaf nested layout is built in pass 2.
            }
            continue;
        }
        let mut f = match it.float_kind {
            Some(kind) => make_float_field(kind, it.value.as_ref())?,
            None => make_field(
                &it.pic,
                it.value.as_ref(),
                ctx.currency,
                ctx.decimal_comma,
                ctx.dialect,
                it.usage.unwrap_or(Usage::Display),
                it.sign,
                it.extra_flags,
            )?,
        };
        // Retain the compile-time ALPHABETIC category (PIC A, no X) for INITIALIZE ... REPLACING.
        if matches!(f.storage, Storage::Alpha(_)) && pic_is_alphabetic(&it.pic) {
            ALPHABETIC_FIELDS.with(|m| m.borrow_mut().insert(it.name.clone()));
        }
        if it.occurs > 1 {
            // A 01-level OCCURS table: replicate the element image `occurs` times (each element initialized
            // identically, per its VALUE or the dialect fill).
            let elem = f.bytes.clone();
            // OCCURS DEPENDING ON: physical size is MAX; record (counter, element size) for the live length.
            if let Some(counter) = &it.odo_counter {
                ODO_TABLES.with(|m| {
                    m.borrow_mut()
                        .insert(it.name.clone(), (counter.clone(), elem.len()))
                });
            }
            f.bytes = elem.repeat(it.occurs);
            f.occurs = it.occurs;
        }
        // A REDEFINES item aliases its target's storage: it keeps its own `storage` (the reinterpretation
        // shape) but reads/writes the target's bytes -- `f.bytes` here is only a size reference.
        f.redefines = it.redefines.clone();
        // OCCURS ... INDEXED BY idx: each index is an integer field (modelled S9(9) DISPLAY, init 0); the
        // first index is recorded as the table's implicit SEARCH index.
        for (n, idx) in it.indexed_by.iter().enumerate() {
            fields.insert(idx.clone(), make_return_code(0));
            if n == 0 {
                TABLE_INDEX.with(|m| m.borrow_mut().insert(it.name.clone(), idx.clone()));
            }
        }
        // OCCURS ... ASCENDING|DESCENDING KEY: the table's sort direction, for SEARCH ALL (binary search).
        if let Some(asc) = it.occurs_key {
            TABLE_KEY.with(|m| m.borrow_mut().insert(it.name.clone(), asc));
        }
        // Capture the VALUE image (scalars only) for INITIALIZE ... ALL TO VALUE.
        // Capture the per-ELEMENT VALUE image (the full bytes for a scalar; one element for an OCCURS table)
        // for INITIALIZE ... TO VALUE -- which restores each occurrence to its VALUE.
        if it.value.is_some() {
            let occ = f.occurs.max(1);
            let esz = f.bytes.len() / occ;
            FIELD_VALUES.with(|m| {
                m.borrow_mut()
                    .insert(it.name.clone(), f.bytes[..esz.min(f.bytes.len())].to_vec())
            });
        }
        fields.insert(it.name.clone(), f);
    }
    // Second pass: group items (no PIC). A group's IMMEDIATE children are the items that follow it at the
    // first (smallest) level below it, up to the next item at <= its level; deeper items belong to a child.
    // SYNCHRONIZED children get a synthetic slack FILLER inserted before them to reach their natural
    // boundary (offset relative to the group start -- the flat-record case).
    let mut slack_n = 0usize;
    // Sub-group items consumed by a nested-table build (descendants of a multi-dimension group-OCCURS) are
    // skipped: their leaves are addressed via NESTED_LEAF, so the intermediate groups own no field.
    let mut consumed: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (i, it) in prog.ws.iter().enumerate() {
        if it.level == 88
            || it.level == 78
            || (it.extra_flags & FLAG_CONSTANT) != 0
            || !it.pic.is_empty()
            || it.float_kind.is_some()
            || consumed.contains(&i)
        {
            continue;
        }
        let mut children = Vec::new();
        // group-OCCURS: each immediate child's (name, offset within the element, size) -- for the views.
        let mut child_views: Vec<(String, usize, usize)> = Vec::new();
        let mut child_level: Option<u16> = None;
        let mut offset = 0usize;
        let mut has_occurs_child = false;
        let mut has_subgroup_child = false;
        let mut max_align = 1usize; // largest SYNCHRONIZED alignment among children (1 = none)
        for sib in &prog.ws[i + 1..] {
            if sib.level <= it.level {
                break;
            }
            if sib.level == 88 {
                continue;
            }
            let cl = *child_level.get_or_insert(sib.level);
            if sib.level == cl {
                if sib.occurs > 1 {
                    has_occurs_child = true;
                }
                if sib.pic.is_empty() && sib.float_kind.is_none() {
                    has_subgroup_child = true;
                }
                let csize = fields.get(&sib.name).map(|f| f.bytes.len()).unwrap_or(0);
                if sib.sync {
                    let align = fields.get(&sib.name).map(sync_align).unwrap_or(1);
                    max_align = max_align.max(align);
                    let rem = offset % align;
                    if rem != 0 {
                        let slack = align - rem;
                        let sname = format!("\u{3}SYNC{slack_n}");
                        slack_n += 1;
                        fields.insert(
                            sname.clone(),
                            Field {
                                storage: Storage::Alpha(alnum_attr()),
                                bytes: vec![0u8; slack],
                                occurs: 1,
                                redefines: None,
                            },
                        );
                        children.push(sname);
                        offset += slack;
                    }
                }
                // A REDEFINES child overlays its target at the SAME element offset and does NOT advance the
                // element size; others are placed sequentially.
                let this_off = match &sib.redefines {
                    Some(tgt) => child_views
                        .iter()
                        .find(|(n, _, _)| n == tgt)
                        .map(|(_, o, _)| *o)
                        .unwrap_or(offset),
                    None => offset,
                };
                children.push(sib.name.clone());
                child_views.push((sib.name.clone(), this_off, csize));
                if sib.redefines.is_none() {
                    offset += csize;
                }
            }
        }
        // A multi-dimension / group-of-group table: an immediate child has its OWN OCCURS (`C(i,j)`) or is
        // itself a sub-group (`A(i)` reaching a deeper leaf). Build the interleaved buffer via the recursive
        // layout and register each leaf with its full (occurs, stride) dims in NESTED_LEAF.
        if it.occurs > 1 && (has_occurs_child || has_subgroup_child) {
            // (SYNCHRONIZED descendants are now handled by nested_layout, which inserts per-element slack.)
            // Skip the intermediate sub-groups in this subtree (their leaves are addressed via NESTED_LEAF).
            let mut k = i + 1;
            while k < prog.ws.len() && prog.ws[k].level > it.level {
                consumed.insert(k);
                k += 1;
            }
            let (block, leaves) = nested_layout(&prog.ws, i, &fields);
            let stride = block / it.occurs.max(1);
            let mut buf = vec![b' '; block];
            for (name, off, sz, dims) in &leaves {
                // the leaf's single-element VALUE/default image (pass-1 built it; an occurs leaf is tiled).
                let img = fields
                    .get(name)
                    .map(|f| {
                        let occ = f.occurs.max(1);
                        let esz = (f.bytes.len() / occ).max(*sz);
                        f.bytes
                            .get(..esz.min(f.bytes.len()))
                            .map(|s| s.to_vec())
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();
                let occs: Vec<usize> = dims.iter().map(|&(o, _)| o).collect();
                let strides: Vec<usize> = dims.iter().map(|&(_, s)| s).collect();
                let total: usize = occs.iter().product::<usize>().max(1);
                for combo in 0..total {
                    let mut rem = combo;
                    let mut cell = *off;
                    for d in (0..dims.len()).rev() {
                        cell += (rem % occs[d]) * strides[d];
                        rem /= occs[d];
                    }
                    let n = (*sz).min(img.len());
                    if cell + n <= buf.len() {
                        buf[cell..cell + n].copy_from_slice(&img[..n]);
                    }
                }
                NESTED_LEAF.with(|m| {
                    m.borrow_mut()
                        .insert(name.clone(), (it.name.clone(), *off, *sz, dims.clone()))
                });
                if let Some(cf) = fields.get_mut(name) {
                    cf.bytes.clear(); // the leaf is now a view into the parent buffer
                    cf.occurs = 1;
                }
            }
            GROUP_OCCURS.with(|m| m.borrow_mut().insert(it.name.clone(), (stride, it.occurs)));
            // OCCURS DEPENDING ON on the OUTER dimension of a multi-dimension group: the buffer is built at
            // MAX (above) and element addressing uses the fixed MAX strides; the LIVE image is counter*stride
            // (so LENGTH / group reads truncate). The inner fixed dimension(s) are part of the stride.
            if let Some(counter) = &it.odo_counter {
                ODO_TABLES.with(|m| {
                    m.borrow_mut()
                        .insert(it.name.clone(), (counter.clone(), stride))
                });
            }
            for (n, idx) in it.indexed_by.iter().enumerate() {
                fields
                    .entry(idx.clone())
                    .or_insert_with(|| make_return_code(0));
                if n == 0 {
                    TABLE_INDEX.with(|m| m.borrow_mut().insert(it.name.clone(), idx.clone()));
                }
            }
            if let Some(asc) = it.occurs_key {
                TABLE_KEY.with(|m| m.borrow_mut().insert(it.name.clone(), asc));
            }
            fields.insert(
                it.name.clone(),
                Field {
                    storage: Storage::Group { children },
                    bytes: buf,
                    occurs: it.occurs,
                    redefines: None,
                },
            );
            continue;
        }
        // A group with OCCURS n: build the live INTERLEAVED buffer [elem]*n and demote children to strided
        // views into it (the children own no bytes). The pass-1 gate already restricted this to the
        // supported subset (single level, fixed count, all-elementary children).
        if it.occurs > 1 {
            // The element size, padded up to the largest SYNCHRONIZED alignment so every occurrence keeps its
            // SYNC fields aligned (cobc: an element with a 4-byte-SYNC field is padded to a multiple of 4).
            let stride = if max_align > 1 {
                offset.div_ceil(max_align) * max_align
            } else {
                offset
            };
            let mut elem_image = Vec::with_capacity(stride);
            for (cname, _o, csz) in &child_views {
                let cb = fields
                    .get(cname)
                    .map(|f| f.bytes.clone())
                    .unwrap_or_else(|| vec![b' '; *csz]);
                elem_image.extend_from_slice(&cb);
            }
            elem_image.resize(stride, b' ');
            let buf = elem_image.repeat(it.occurs); // interleaved, n elements (built at MAX for ODO)
            GROUP_OCCURS.with(|m| m.borrow_mut().insert(it.name.clone(), (stride, it.occurs)));
            // OCCURS DEPENDING ON on the group: the live image length is counter*stride (built at MAX above).
            if let Some(counter) = &it.odo_counter {
                ODO_TABLES.with(|m| {
                    m.borrow_mut()
                        .insert(it.name.clone(), (counter.clone(), stride))
                });
            }
            for (cname, coff, csz) in &child_views {
                GROUP_CHILD.with(|m| {
                    m.borrow_mut()
                        .insert(cname.clone(), (it.name.clone(), *coff, *csz))
                });
                if let Some(cf) = fields.get_mut(cname) {
                    cf.bytes.clear(); // the child is now a view into the parent buffer
                }
            }
            // INDEXED BY / ASCENDING|DESCENDING KEY are keyed on the GROUP name (for SEARCH / SEARCH ALL).
            for (n, idx) in it.indexed_by.iter().enumerate() {
                fields
                    .entry(idx.clone())
                    .or_insert_with(|| make_return_code(0));
                if n == 0 {
                    TABLE_INDEX.with(|m| m.borrow_mut().insert(it.name.clone(), idx.clone()));
                }
            }
            if let Some(asc) = it.occurs_key {
                TABLE_KEY.with(|m| m.borrow_mut().insert(it.name.clone(), asc));
            }
            fields.insert(
                it.name.clone(),
                Field {
                    storage: Storage::Group { children }, // children retained for category/DISPLAY resolution
                    bytes: buf,                           // AUTHORITATIVE interleaved buffer
                    occurs: it.occurs, // n (not 1) -- unblocks SEARCH's occurs > 1 gate
                    redefines: None,
                },
            );
            continue;
        }
        fields.insert(
            it.name.clone(),
            Field {
                storage: Storage::Group { children },
                bytes: Vec::new(),
                occurs: 1,
                redefines: None,
            },
        );
    }
    // Third pass: `66 RENAMES start [THRU end]` regrouping -- an alias over the contiguous leaf fields
    // from `start` to `end`, modelled as a Group so reads/writes distribute across them.
    for it in &prog.ws {
        let Some((start, end)) = &it.renames else {
            continue;
        };
        let s_idx = prog.ws.iter().position(|x| &x.name == start);
        let e_idx = prog.ws.iter().position(|x| &x.name == end);
        if let (Some(s), Some(e)) = (s_idx, e_idx) {
            let children: Vec<String> = prog.ws[s..=e.max(s)]
                .iter()
                .filter(|x| x.level != 88 && x.level != 66
                    && group_child_lookup(&x.name).is_none() // skip group-OCCURS child views (zero-byte)
                    && fields.get(&x.name).is_some_and(|f| !matches!(f.storage, Storage::Group { .. })))
                .map(|x| x.name.clone())
                .collect();
            fields.insert(
                it.name.clone(),
                Field {
                    storage: Storage::Group { children },
                    bytes: Vec::new(),
                    occurs: 1,
                    redefines: None,
                },
            );
        }
    }
    // REDEFINES over a GROUP: register each authoritative descendant store of a redefining GROUP so reads /
    // writes alias the redefined target at the descendant's offset (e.g. a group-OCCURS table that REDEFINES
    // a VALUE-bearing group -- the classic "table initialised via a redefinition" idiom). Only DIRECT
    // children that are a group-OCCURS buffer or an elementary leaf are mapped (the common shape); a deeper
    // plain sub-group is left as-is (no regression, just not aliased).
    for (i, it) in prog.ws.iter().enumerate() {
        let Some(target) = &it.redefines else {
            continue;
        };
        if !it.pic.is_empty() || it.float_kind.is_some() {
            continue; // only a GROUP redefinition propagates to descendants
        }
        let mut child_level: Option<u16> = None;
        let mut off = 0usize;
        for sib in &prog.ws[i + 1..] {
            if sib.level <= it.level {
                break;
            }
            if sib.level == 88 {
                continue;
            }
            let cl = *child_level.get_or_insert(sib.level);
            if sib.level != cl {
                continue;
            }
            let csz = field_len(&sib.name, &fields);
            let is_group_occurs = group_occurs_lookup(&sib.name).is_some();
            let is_leaf = fields
                .get(&sib.name)
                .is_some_and(|f| !matches!(f.storage, Storage::Group { .. }));
            if (is_group_occurs || is_leaf)
                && group_child_lookup(&sib.name).is_none()
                && nested_leaf_lookup(&sib.name).is_none()
            {
                REDEF_VIEW.with(|m| {
                    m.borrow_mut()
                        .insert(sib.name.clone(), (target.clone(), off))
                });
            }
            // A nested REDEFINES child overlays at the same offset and does not advance it.
            if sib.redefines.is_none() {
                off += csz;
            }
        }
    }
    // FD/SD record-area union: every `01` record description beneath ONE FD shares a single record area
    // (GnuCOBOL semantics -- the file's records are ALTERNATIVE VIEWS of one buffer, sized to the LARGEST
    // record; verified against the pinned oracle: `MOVE "11111" TO A` then `WRITE B` writes "11111", and
    // `MOVE` into one record is visible through every other). Each non-owner record aliases the largest
    // record's storage: an elementary record via `Field.redefines` (reads/writes reinterpret the owner's
    // bytes through its own shape), a group record by mapping its DIRECT leaf children into REDEF_VIEW at
    // their record-relative offsets (the same shape the REDEFINES-group pass maps; a deeper plain
    // sub-group stays unaliased -- the documented REDEFINES limitation). `WRITE` of the NAMED record then
    // emits the shared area sized to that record's own length, exactly like the oracle.
    for file in &prog.files {
        if file.records.len() < 2 {
            continue;
        }
        // total live size of each record (elementary: its bytes; group: the concatenated leaves)
        let sizes: Vec<(String, usize)> = file
            .records
            .iter()
            .map(|r| (r.clone(), field_len(r, &fields)))
            .collect();
        let owner = sizes
            .iter()
            .max_by_key(|(_, s)| *s)
            .map(|(n, _)| n.clone())
            .unwrap_or_default();
        if owner.is_empty() {
            continue;
        }
        for (rec, _) in &sizes {
            if rec == &owner {
                continue;
            }
            let Some(f) = fields.get(rec) else { continue };
            if let Storage::Group { children } = &f.storage {
                // Direct elementary / group-OCCURS children of the alternative record alias the owner's
                // area at their record-relative offsets (each record layout starts at offset 0 of the
                // shared area).
                let mut off = 0usize;
                for c in children {
                    let csz = field_len(c, &fields);
                    let is_leaf = fields
                        .get(c)
                        .is_some_and(|x| !matches!(x.storage, Storage::Group { .. }));
                    let is_group_occurs = group_occurs_lookup(c).is_some();
                    if (is_leaf || is_group_occurs)
                        && group_child_lookup(c).is_none()
                        && nested_leaf_lookup(c).is_none()
                    {
                        REDEF_VIEW.with(|m| m.borrow_mut().insert(c.clone(), (owner.clone(), off)));
                    }
                    off += csz;
                }
            } else if let Some(f) = fields.get_mut(rec) {
                // Elementary alternative: alias the owner's storage through Field::redefines.
                f.redefines = Some(owner.clone());
            }
        }
    }
    // EXTERNAL items: VALUE is ignored and the storage is run-unit-shared by name (zero-filled on first
    // use). Load the shared value if another program already created it, else zero-fill + register.
    for it in &prog.ws {
        if !it.external {
            continue;
        }
        let size = field_len(&it.name, &fields);
        let bytes = EXTERNAL_STORE
            .with(|m| m.borrow().get(&it.name).cloned())
            .unwrap_or_else(|| vec![0u8; size]);
        set_item_bytes(&it.name, bytes, &mut fields);
        let cur = read_field(&fields, &it.name)
            .ok()
            .flatten()
            .map(|f| f.bytes)
            .unwrap_or_default();
        EXTERNAL_STORE.with(|m| m.borrow_mut().insert(it.name.clone(), cur));
    }
    // RETURN-CODE: the signed special register, initialised to 0 (modelled as S9(9) DISPLAY).
    fields.insert("RETURN-CODE".to_string(), make_return_code(0));
    // TALLY: the EXAMINE count register (unsigned 9(5) DISPLAY).
    if let Ok(t) = make_field(
        "9(5)",
        None,
        ctx.currency,
        ctx.decimal_comma,
        ctx.dialect,
        Usage::Display,
        (false, false),
        0,
    ) {
        fields.entry("TALLY".to_string()).or_insert(t);
    }
    Ok(fields)
}

/// The concatenated current bytes of a group's immediate children (a sub-group recurses via `read_field`).
fn group_bytes(children: &[String], fields: &HashMap<String, Field>) -> Vec<u8> {
    let mut b = Vec::new();
    for c in children {
        if let Ok(Some(f)) = read_field(fields, c) {
            b.extend_from_slice(&f.bytes);
        }
    }
    b
}

/// The total byte length of a field (a group recurses through `read_field`).
fn field_len(name: &str, fields: &HashMap<String, Field>) -> usize {
    read_field(fields, name)
        .ok()
        .flatten()
        .map(|f| f.bytes.len())
        .unwrap_or(0)
}

/// Distribute `bytes` (space-padded/truncated to the group's total length) across its immediate children by
/// length (a sub-group child distributes recursively via `write_field`).
fn put_group_bytes(children: &[String], mut bytes: Vec<u8>, fields: &mut HashMap<String, Field>) {
    let lens: Vec<usize> = children.iter().map(|c| field_len(c, fields)).collect();
    let total: usize = lens.iter().sum();
    bytes.resize(total, b' ');
    let mut off = 0;
    for (c, len) in children.iter().zip(lens) {
        let slice = bytes[off..off + len].to_vec();
        let _ = write_field(fields, c, |f| {
            f.bytes = slice;
            Ok(())
        });
        off += len;
    }
}

// The directory the in-memory file store is materialized into when the run ends. A HOST diagnostic
// (`cobrun --dump-files <dir>`): it writes each ASSIGN target as a file whose bytes mirror the
// GnuCOBOL on-disk format, so a differential court can compare the program's file output. It is
// never part of program semantics (the in-memory store remains the authoritative run model).
thread_local! {
    static FILE_DUMP_DIR: std::cell::RefCell<Option<std::path::PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

/// Point the file-store materializer at `dir` for this run (see `FILE_DUMP_DIR`).
pub fn set_file_dump_dir(dir: std::path::PathBuf) {
    FILE_DUMP_DIR.with(|d| *d.borrow_mut() = Some(dir));
}

/// Materialize the in-memory file store into the dump dir (when one is set): each ASSIGN target
/// becomes a file whose bytes mirror the GnuCOBOL on-disk format for the file's organization --
/// line-sequential = records joined by LF; fixed sequential = concatenated records; variable-length
/// (`RECORD IS VARYING` or differing declared record widths) = the var-seq 4-byte header framing
/// `[u16 BE length][0x0000]`; relative/indexed = the records in store order (gaps omitted).
fn dump_file_store(ctx: &Ctx, fields: &HashMap<String, Field>) {
    FILE_DUMP_DIR.with(|d| {
        let guard = d.borrow();
        let Some(dir) = guard.as_ref() else { return };
        let _ = std::fs::create_dir_all(dir);
        for (assign, st) in ctx.files.borrow().iter() {
            let def = ctx.file_defs.values().find(|f| &f.assign == assign);
            let mut out_bytes: Vec<u8> = Vec::new();
            let (org, var_seq) = match def {
                Some(f) => {
                    let widths: Vec<usize> =
                        f.records.iter().map(|r| field_len(r, fields)).collect();
                    let differing = widths.iter().any(|&w| w != widths[0]);
                    (f.org, f.varying_dep.is_some() || differing)
                }
                None => (FileOrg::Sequential, false),
            };
            for r in &st.records {
                if var_seq {
                    let len = (r.len().min(u16::MAX as usize) as u16).to_be_bytes();
                    out_bytes.extend_from_slice(&len);
                    out_bytes.extend_from_slice(&[0, 0]);
                }
                out_bytes.extend_from_slice(r);
                if org == FileOrg::LineSequential && !var_seq {
                    out_bytes.push(b'\n');
                }
            }
            let _ = std::fs::write(dir.join(assign), &out_bytes);
        }
    });
}

thread_local! {
    /// `OCCURS ... INDEXED BY` table -> its implicit SEARCH index name, populated as each program's fields
    /// are built. `SEARCH table` (without `VARYING`) varies this index.
    static TABLE_INDEX: std::cell::RefCell<HashMap<String, String>> = std::cell::RefCell::new(HashMap::new());
    /// `OCCURS ... ASCENDING|DESCENDING KEY` table -> sort direction (`true` = ascending). Read by
    /// `SEARCH ALL` to narrow the binary search. Absent = no KEY clause (SEARCH ALL fails closed).
    static TABLE_KEY: std::cell::RefCell<HashMap<String, bool>> = std::cell::RefCell::new(HashMap::new());
    /// A group-OCCURS GROUP name -> (group element stride, occurs n). The group `Field` holds the live
    /// INTERLEAVED buffer `[c0 c1 ...][c0 c1 ...]...` as its real bytes, occurs = n.
    static GROUP_OCCURS: std::cell::RefCell<HashMap<String, (usize, usize)>> = std::cell::RefCell::new(HashMap::new());
    /// A group-OCCURS CHILD name -> (parent group, offset within the element, child size). The child owns
    /// NO bytes; it is a strided view into the parent's interleaved buffer (read/written at stride).
    static GROUP_CHILD: std::cell::RefCell<HashMap<String, (String, usize, usize)>> = std::cell::RefCell::new(HashMap::new());
    /// A MULTI-DIMENSION leaf name -> (base buffer group, byte offset at all-1 subscripts, leaf size,
    /// dims). `dims` is `(occurs, stride)` per subscript, OUTERMOST first, so the address of `LEAF(s1..sk)`
    /// is `offset + sum_d (s_d - 1) * stride_d` into the base group's interleaved buffer. Used for the 2-D
    /// `C(i,j)` shape (outer group-OCCURS with an inner elementary-OCCURS child); the flat single-subscript
    /// group-OCCURS stays on GROUP_CHILD. `dims.len()` is the required subscript count.
    static NESTED_LEAF: std::cell::RefCell<HashMap<String, (String, usize, usize, Vec<(usize, usize)>)>> = std::cell::RefCell::new(HashMap::new());
    /// Names of fields whose PICTURE is ALPHABETIC (`PIC A...`, no `X`). At runtime PIC A and PIC X are both
    /// `Storage::Alpha` / `COB_TYPE_ALPHANUMERIC` (libcob has no distinct alphabetic type); cobc decides the
    /// category at COMPILE time from the PIC. We retain it here so `INITIALIZE ... REPLACING ALPHABETIC` vs
    /// `REPLACING ALPHANUMERIC` can target the right leaves (an `ALPHANUMERIC` replace must skip PIC A).
    static ALPHABETIC_FIELDS: std::cell::RefCell<std::collections::HashSet<String>> = std::cell::RefCell::new(std::collections::HashSet::new());
    /// The VALUE-clause image (initial bytes) of each elementary field that declared a `VALUE`, keyed by
    /// field name. Captured at field-build time so `INITIALIZE ... ALL TO VALUE` can restore each leaf to
    /// its VALUE (a leaf with no VALUE is absent here and left unchanged, matching cobc). Scalars only
    /// (OCCURS tables are not captured; INITIALIZE TO VALUE over a table fails closed).
    static FIELD_VALUES: std::cell::RefCell<HashMap<String, Vec<u8>>> = std::cell::RefCell::new(HashMap::new());
    /// The stack of executing PROGRAM-IDs (top = current). Pushed/popped around each program body so
    /// `FUNCTION MODULE-ID` reads the running program and `FUNCTION MODULE-CALLER-ID` reads its caller.
    static PROGRAM_STACK: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    /// `DISPLAY x UPON ENVIRONMENT-NAME` sets this register; `DISPLAY y UPON ENVIRONMENT-VALUE` and
    /// `ACCEPT z FROM ENVIRONMENT-VALUE` then act on the variable it names (none of these write stdout).
    static ENV_NAME_REG: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    /// The per-run command line (set by the run boundary for `cobcrun module args...`): `ACCEPT ... FROM
    /// COMMAND-LINE` / `ARGUMENT-VALUE n` / `ARGUMENT-NUMBER` read it. Empty = no program arguments.
    static COMMAND_LINE: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    /// The ARGUMENT-NUMBER register (index of the argument ARGUMENT-VALUE reads; DISPLAY n UPON
    /// ARGUMENT-NUMBER sets it, 1-based in GnuCOBOL).
    static ARG_NUMBER_REG: std::cell::RefCell<usize> = const { std::cell::RefCell::new(0) };
    /// Per-token source line map from the most recent `lex` (a fresh top-level run lexes once; the
    /// map is reset at the run start). Used for libcob-shaped fatal-error line reporting.
    static TOKEN_LINES: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
    /// Per-run environment-variable overrides set via `DISPLAY ... UPON ENVIRONMENT-VALUE`; consulted before
    /// the real process environment by `ACCEPT ... FROM ENVIRONMENT [-VALUE]` so a set-then-read round-trips
    /// deterministically without mutating the host env.
    static ENV_OVERRIDE: std::cell::RefCell<HashMap<String, Vec<u8>>> = std::cell::RefCell::new(HashMap::new());
    /// REDEFINES over a GROUP: an authoritative descendant store of a redefining group (a group-OCCURS buffer
    /// or an elementary leaf) -> (redefined target name, byte offset within that target). Reads and writes of
    /// the descendant go through the target's live image at the offset, so a group-OCCURS table that
    /// REDEFINES a VALUE-bearing group (the classic "table initialised via a redefinition" idiom) sees the
    /// real values -- and a write through the redefinition lands in the shared storage.
    static REDEF_VIEW: std::cell::RefCell<HashMap<String, (String, usize)>> = std::cell::RefCell::new(HashMap::new());
}

/// The current PROGRAM-ID (top of the program stack), empty outside any program body.
fn current_program_id() -> String {
    PROGRAM_STACK
        .with(|s| s.borrow().last().cloned())
        .unwrap_or_default()
}

thread_local! {
    /// The source-file path the host is running, for `FUNCTION MODULE-SOURCE` (cobc embeds the source
    /// name it was given; the interpreter knows the `.cob` it was invoked with). Set by the host
    /// (`set_source_file`) before running; empty otherwise.
    static SOURCE_FILE: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// Record the source-file path for `FUNCTION MODULE-SOURCE` (the host sets this before running a program).
pub fn set_source_file(path: &str) {
    SOURCE_FILE.with(|s| *s.borrow_mut() = path.to_string());
}

/// The per-run program collating sequence (upstream `COB_MODULE_PTR->collating_sequence`), set by
/// the run boundary from `parse_collation`; `None` = no sequence (the identity CHAR/ORD path).
/// Used by FUNCTION CHAR / ORD (upstream 5bb0fbe1b) and the collating comparisons.
thread_local! {
    static COLLATION: std::cell::RefCell<Option<[u8; 256]>> = const { std::cell::RefCell::new(None) };
}

pub fn set_collation(col: Option<[u8; 256]>) {
    COLLATION.with(|c| *c.borrow_mut() = col);
}

fn current_collation() -> Option<[u8; 256]> {
    COLLATION.with(|c| *c.borrow())
}

/// Record the program command line (the run boundary sets it for `cobcrun module args...`; cleared
/// at the start of each fresh top-level run). `ACCEPT ... FROM COMMAND-LINE` / `ARGUMENT-VALUE` /
/// `ARGUMENT-NUMBER` read it.
pub fn set_command_line(args: &[String]) {
    COMMAND_LINE.with(|c| *c.borrow_mut() = args.to_vec());
}

thread_local! {
    /// The last raised arithmetic exception condition name (an `EC-SIZE-*`), STICKY: set when a SIZE ERROR
    /// occurs (divide-by-zero, or a result with more integer digits than the receiver holds) and NOT
    /// cleared by a later successful statement -- only overwritten by the next exception, exactly as
    /// libcob's `FUNCTION EXCEPTION-STATUS` behaves (proven: a clean COMPUTE/MOVE after a fault does not
    /// reset it). Reset only at the start of a top-level run.
    static EXCEPTION_CODE: std::cell::Cell<&'static str> = const { std::cell::Cell::new("") };
    /// The PROGRAM-ID where the last exception was raised, for `FUNCTION EXCEPTION-LOCATION`.
    static EXCEPTION_PROGRAM: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    /// The LAST file operation's `(2-char status, SELECT name)`, for `FUNCTION EXCEPTION-FILE`. Updated on
    /// every I/O (success too -- it reflects the last op, not just exceptions); `None` before any I/O.
    static FILE_EXCEPTION: std::cell::RefCell<Option<(String, String)>> = const { std::cell::RefCell::new(None) };
}

/// Record the last file operation for `FUNCTION EXCEPTION-FILE`.
fn set_file_exception(code: &str, select: &str) {
    FILE_EXCEPTION.with(|c| *c.borrow_mut() = Some((code.to_string(), select.to_string())));
}

/// The `FUNCTION EXCEPTION-FILE` field: the last I/O `<status><SELECT>`, or `"00"` before any I/O.
fn exception_file_field() -> (Vec<u8>, FieldAttr) {
    FILE_EXCEPTION.with(|c| match c.borrow().as_ref() {
        Some((code, name)) => {
            crate::intrinsic::cob_intr_exception_file(Some((code.as_bytes(), name.as_bytes())))
        }
        None => crate::intrinsic::cob_intr_exception_file(None),
    })
}

/// Set the current exception condition (sticky until the next exception), recording the raising program.
fn set_exception(code: &'static str) {
    EXCEPTION_CODE.with(|c| c.set(code));
    let prog = current_program_id();
    EXCEPTION_PROGRAM.with(|p| *p.borrow_mut() = prog);
}

/// Clear the exception register (called once at the start of a top-level run).
fn reset_exception() {
    EXCEPTION_CODE.with(|c| c.set(""));
    EXCEPTION_PROGRAM.with(|p| p.borrow_mut().clear());
    FILE_EXCEPTION.with(|c| *c.borrow_mut() = None);
}

/// The `FUNCTION EXCEPTION-LOCATION` field: `"<prog>; ; 0"` once an exception has been raised (no
/// paragraph/section and line 0 without `>>TURN EC ... CHECKING`, which the sealed subset omits), or a
/// single space before any exception -- matching libcob's default.
fn exception_location_field() -> (Vec<u8>, FieldAttr) {
    let code = EXCEPTION_CODE.with(|c| c.get());
    if code.is_empty() {
        return crate::intrinsic::cob_intr_exception_location(None);
    }
    let prog = EXCEPTION_PROGRAM.with(|p| p.borrow().clone());
    crate::intrinsic::cob_intr_exception_location(Some((prog.as_bytes(), None, None, 0)))
}

/// The `FUNCTION EXCEPTION-STATUS` field: the current condition name in a 31-byte field, or spaces.
fn exception_status_field() -> (Vec<u8>, FieldAttr) {
    let code = EXCEPTION_CODE.with(|c| c.get());
    crate::intrinsic::cob_intr_exception_status(if code.is_empty() {
        None
    } else {
        Some(code.as_bytes())
    })
}

/// The integer-digit capacity of an arithmetic receiver (digit positions left of the implied decimal
/// point), or `None` for a receiver that does not raise a SIZE ERROR (alphanumeric / non-sized).
fn receiver_int_digits(f: &Field) -> Option<usize> {
    match &f.storage {
        // Float receivers (COMP-1/COMP-2) do NOT size-check on decimal digit count: the IEEE range is
        // the boundary (the oracle fires at ~2^127 for COMP-1), and the field's decimal "digits" are a
        // conversion-intermediate width, not a storage capacity. Treating them as 9 digits raised a
        // spurious EC-SIZE-OVERFLOW at ~2^30 (GnuCOBOL `FLOAT-SHORT with SIZE ERROR` divergence).
        Storage::Numeric(a)
            if a.field_type == crate::attr::COB_TYPE_NUMERIC_FLOAT
                || a.field_type == crate::attr::COB_TYPE_NUMERIC_DOUBLE =>
        {
            None
        }
        Storage::Numeric(a) => Some((a.digits as i32 - a.scale as i32).max(0) as usize),
        _ => None,
    }
}

/// Whether the wide arithmetic result `(val, attr)` has more significant integer digits than `cap` --
/// i.e. storing it into a receiver of `cap` integer digits would lose high-order digits (SIZE ERROR).
fn arith_overflows(val: &[u8], attr: &FieldAttr, cap: usize) -> bool {
    let dec = match source_to_decimal(val, attr) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let intlen = dec.digits.len().saturating_sub(dec.scale.max(0) as usize);
    let sig = dec.digits[..intlen].iter().skip_while(|&&d| d == 0).count();
    sig > cap
}

/// Store an arithmetic result into receiver `f`, detecting a SIZE ERROR overflow. Returns `true` if a
/// size error occurred. On overflow WITH a handler present the receiver is left UNCHANGED (the handler
/// runs); WITHOUT a handler it is truncated; both set `EXCEPTION-STATUS` to `EC-SIZE-OVERFLOW`.
fn store_arith_result(
    f: &mut Field,
    val: &[u8],
    attr: &FieldAttr,
    has_handler: bool,
    decimal_comma: bool,
) -> Result<bool, RunError> {
    if let Some(cap) = receiver_int_digits(f) {
        if arith_overflows(val, attr, cap) {
            set_exception("EC-SIZE-OVERFLOW");
            if has_handler {
                return Ok(true); // receiver unchanged; the ON SIZE ERROR handler runs
            }
            move_into(f, val, attr, decimal_comma)?; // no handler: truncate-store, but still a size error
            return Ok(true);
        }
    }
    move_into(f, val, attr, decimal_comma)?;
    Ok(false)
}

/// The caller's PROGRAM-ID (the entry below the top), or `None` at the top-level program.
fn caller_program_id() -> Option<String> {
    PROGRAM_STACK.with(|s| {
        let b = s.borrow();
        (b.len() >= 2).then(|| b[b.len() - 2].clone())
    })
}

/// The implicit SEARCH index for an `OCCURS ... INDEXED BY` table, if one was declared.
fn table_index_lookup(table: &str) -> Option<String> {
    TABLE_INDEX.with(|m| m.borrow().get(table).cloned())
}

thread_local! {
    /// `OCCURS min TO max DEPENDING ON counter` tables -> `(counter name, single-element byte size)`.
    /// The field is built at MAX physical size; the CURRENT length is `counter_value * elem`, computed at
    /// read time so a group's live image and `FUNCTION LENGTH` reflect the DEPENDING counter.
    static ODO_TABLES: std::cell::RefCell<HashMap<String, (String, usize)>> = std::cell::RefCell::new(HashMap::new());
    /// Active report run state by report name (Report Writer), between INITIATE and TERMINATE.
    static REPORT_STATE: std::cell::RefCell<HashMap<String, ReportRun>> = std::cell::RefCell::new(HashMap::new());
}

/// The `(counter, elem-size)` of an `OCCURS DEPENDING ON` table, if `name` is one.
fn odo_lookup(name: &str) -> Option<(String, usize)> {
    ODO_TABLES.with(|m| m.borrow().get(name).cloned())
}

/// `(group element stride, occurs)` if `name` is a group-OCCURS group, else `None`.
fn group_occurs_lookup(name: &str) -> Option<(usize, usize)> {
    GROUP_OCCURS.with(|m| m.borrow().get(name).cloned())
}

/// `(parent group, offset within element, child size)` if `name` is a group-OCCURS child view, else `None`.
fn group_child_lookup(name: &str) -> Option<(String, usize, usize)> {
    GROUP_CHILD.with(|m| m.borrow().get(name).cloned())
}

/// `(base buffer group, offset, leaf size, dims)` if `name` is a multi-dimension leaf, else `None`.
#[allow(clippy::type_complexity)]
fn nested_leaf_lookup(name: &str) -> Option<(String, usize, usize, Vec<(usize, usize)>)> {
    NESTED_LEAF.with(|m| m.borrow().get(name).cloned())
}

/// The comma-separated subscripts of a reference inner (`"I,J"` -> `["I","J"]`); a single subscript
/// (`"I"`) -> `["I"]`. Whitespace around each is trimmed.
fn subscripts(inner: &str) -> Vec<&str> {
    // Subscripts are separated by commas OR spaces (`C(1, 2)` / `C(1 2)` / `C(I J)`). A single relative
    // subscript `C(I + 1)` contains a `+`/`-` operator -- keep it whole (it has no comma and is one
    // subscript) rather than splitting its spaces.
    if inner.contains(',') {
        inner
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    } else if inner.split_whitespace().any(|t| t == "+" || t == "-") {
        vec![inner.trim()]
    } else {
        inner.split_whitespace().collect()
    }
}

/// Resolve a multi-dimension leaf reference `LEAF(s1,..,sk)` to its `(base_group, byte_offset, size)` in the
/// base buffer. The subscript count must equal `dims.len()`; an out-of-range subscript follows the same
/// EC-BOUND-SUBSCRIPT policy as [`table_element`] (caller returns a default element when checking is OFF).
fn nested_addr(
    offset: usize,
    dims: &[(usize, usize)],
    subs: &[&str],
    fields: &HashMap<String, Field>,
) -> Result<Option<usize>, RunError> {
    if subs.len() != dims.len() {
        return Err(RunError::Unsupported(format!(
            "multi-dimension leaf needs {} subscript(s), got {}",
            dims.len(),
            subs.len()
        )));
    }
    let mut off = offset;
    for (s, &(occ, stride)) in subs.iter().zip(dims.iter()) {
        let idx = resolve_int(s, fields)
            .ok_or_else(|| RunError::Unsupported(format!("subscript '{s}' is not an integer")))?;
        if idx < 1 || idx as usize > occ {
            if EC_BOUND_SUBSCRIPT_ON.with(|c| c.get()) {
                return Err(RunError::Runtime(format!(
                    "subscript out of bounds: {idx} (maximum: {occ})"
                )));
            }
            return Ok(None); // suppressed OOB -> caller substitutes a default element
        }
        off += (idx as usize - 1) * stride;
    }
    Ok(Some(off))
}

thread_local! {
    /// `SET ptr TO ADDRESS OF field` -- a USAGE POINTER's target field name, for `FUNCTION CONTENT-OF` /
    /// `CONTENT-LENGTH` (which dereference the pointer). Cleared at a fresh top-level run.
    static POINTER_TARGETS: std::cell::RefCell<HashMap<String, String>> = std::cell::RefCell::new(HashMap::new());
}

thread_local! {
    /// `EXTERNAL` data items: the run-unit-shared storage, keyed by item name. Persists across program
    /// builds and CALLs (cleared only at a fresh top-level run); zero-filled on first use.
    static EXTERNAL_STORE: std::cell::RefCell<HashMap<String, Vec<u8>>> = std::cell::RefCell::new(HashMap::new());
}

/// Set an item's bytes -- distributing into a group's leaves, or replacing an elementary field's bytes.
fn set_item_bytes(name: &str, bytes: Vec<u8>, fields: &mut HashMap<String, Field>) {
    if let Some(Field {
        storage: Storage::Group { children },
        ..
    }) = fields.get(name)
    {
        let children = children.clone();
        put_group_bytes(&children, bytes, fields);
    } else if let Some(f) = fields.get_mut(name) {
        f.bytes = bytes;
    }
}

/// Copy every EXTERNAL item present in `fields` into the shared store (a program's current values out).
fn sync_external_to_store(fields: &HashMap<String, Field>) {
    let names: Vec<String> = EXTERNAL_STORE.with(|m| m.borrow().keys().cloned().collect());
    for name in names {
        if let Ok(Some(f)) = read_field(fields, &name) {
            EXTERNAL_STORE.with(|m| m.borrow_mut().insert(name, f.bytes));
        }
    }
}

/// Copy every EXTERNAL item from the shared store into `fields` (the shared values in).
fn sync_store_to_external(fields: &mut HashMap<String, Field>) {
    let entries: Vec<(String, Vec<u8>)> = EXTERNAL_STORE.with(|m| {
        m.borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    });
    for (name, bytes) in entries {
        if fields.contains_key(&name) {
            set_item_bytes(&name, bytes, fields);
        }
    }
}

/// The natural-boundary alignment (bytes) of a SYNCHRONIZED item: its size for a binary/float
/// (COMP/COMP-5/COMP-X/COMP-1/COMP-2) field, capped at 8; 1 (no effect) for display/packed/alnum.
fn sync_align(f: &Field) -> usize {
    if let Storage::Numeric(a) = &f.storage {
        if matches!(a.field_type, 0x11 | 0x1B | 0x13 | 0x14 | 0x15) {
            return f.bytes.len().clamp(1, 8);
        }
    }
    1
}

/// Whether `name` is variable-length: an `OCCURS DEPENDING ON` table itself, or a group whose subtree
/// contains one. cobc computes `FUNCTION LENGTH` of such an item at runtime (not a compile-time constant).
fn is_variable_length(name: &str, fields: &HashMap<String, Field>) -> bool {
    let base = split_subscript(name).0;
    if odo_lookup(base).is_some() {
        return true;
    }
    if let Some(Field {
        storage: Storage::Group { children },
        ..
    }) = fields.get(base)
    {
        return children.iter().any(|c| is_variable_length(c, fields));
    }
    false
}

thread_local! {
    /// The currently-executing program body's paragraphs as `(name, start_token)` plus the body length,
    /// used by out-of-line `PERFORM para [THRU para2]` to find the token range to run. Saved/restored
    /// around each program body so a CALL does not clobber the caller's paragraphs.
    static CUR_PARAS: std::cell::RefCell<(Vec<(String, usize)>, usize)> = const { std::cell::RefCell::new((Vec::new(), 0)) };
    /// The current program body's tokens (`proc_toks`), so a verb that runs a paragraph range (SORT
    /// INPUT/OUTPUT PROCEDURE) can reach them. Saved/restored around each program body.
    static CUR_PROC: std::cell::RefCell<Vec<Tok>> = const { std::cell::RefCell::new(Vec::new()) };
    /// The current program body's per-token source lines (threaded through the proc_toks transforms),
    /// for libcob-shaped fatal-error line reporting. Saved/restored around each program body.
    static CUR_PROC_LINES: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
    /// `ALTER`ed GO TO targets: the token index of a `GO` verb -> the paragraph it now proceeds to. Set by
    /// ALTER, consulted by the GO TO executor. Saved/restored per program body.
    static ALTERED: std::cell::RefCell<HashMap<usize, String>> = std::cell::RefCell::new(HashMap::new());
    /// DECLARATIVES `USE ... ON file` error handlers: file name -> the handler's `[start, end)` token range
    /// to run when a file op on that file returns an error status. Saved/restored per program body.
    static USE_PROCS: std::cell::RefCell<HashMap<String, (usize, usize)>> = std::cell::RefCell::new(HashMap::new());
}

/// Parse a PROCEDURE DIVISION's DECLARATIVES (if present): map each `USE ... ON file` file to the `[start,
/// end)` token range of its handler paragraph (the first label after the USE statement, up to the next
/// label or `END DECLARATIVES`), and return the token index where normal execution starts. No DECLARATIVES
/// -> empty.
fn parse_declaratives(
    proc: &[Tok],
    labels: &HashMap<String, usize>,
) -> (HashMap<String, (usize, usize)>, usize) {
    let mut use_procs = HashMap::new();
    let decl = match proc
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "DECLARATIVES"))
    {
        Some(d) => d,
        None => return (use_procs, 0),
    };
    let end_decl =
        find_seq_in(proc, &["END", "DECLARATIVES"], decl, proc.len()).unwrap_or(proc.len());
    let mut i = decl;
    while i < end_decl {
        if matches!(proc.get(i), Some(Tok::Word(w)) if w == "USE") {
            let mut j = i + 1;
            while j < end_decl && !matches!(proc.get(j), Some(Tok::Word(w)) if w == "ON") {
                j += 1;
            }
            j += 1;
            let mut files = Vec::new();
            while j < end_decl && !matches!(proc.get(j), Some(Tok::Dot)) {
                if let Some(Tok::Word(f)) = proc.get(j) {
                    if !matches!(f.as_str(), "FILE" | "INPUT" | "OUTPUT" | "I-O" | "EXTEND") {
                        files.push(f.clone());
                    }
                }
                j += 1;
            }
            // handler range: the first label after the USE statement, to the next label or END DECLARATIVES.
            if let Some(&start) = labels.values().filter(|&&p| p > j).min() {
                let end = labels
                    .values()
                    .copied()
                    .filter(|&p| p > start)
                    .min()
                    .unwrap_or(end_decl)
                    .min(end_decl);
                for f in files {
                    use_procs.insert(f, (start, end));
                }
            }
        }
        i += 1;
    }
    let mut start = end_decl + 2;
    if matches!(proc.get(start), Some(Tok::Dot)) {
        start += 1;
    }
    (use_procs, start)
}

/// Whether `name` is a paragraph/section label in the current program body.
fn para_exists(name: &str) -> bool {
    CUR_PARAS.with(|c| c.borrow().0.iter().any(|(n, _)| n == name))
}

/// The `[start, end)` token range of `PERFORM p1 [THRU p2]`: from p1's first statement to the start of the
/// paragraph following p2 (or the body end).
fn para_range(p1: &str, p2: &str) -> Option<(usize, usize)> {
    CUR_PARAS.with(|c| {
        let (paras, plen) = &*c.borrow();
        let start = paras.iter().find(|(n, _)| n == p1).map(|(_, s)| *s)?;
        let p2start = paras.iter().find(|(n, _)| n == p2).map(|(_, s)| *s)?;
        let end = paras
            .iter()
            .map(|(_, s)| *s)
            .filter(|&s| s > p2start)
            .min()
            .unwrap_or(*plen);
        Some((start, end))
    })
}

/// Run the statements in `toks[start..end)` (a performed paragraph range). Returns `Ok(true)` if a halt
/// (STOP RUN / GOBACK / EXIT PROGRAM / pending GO TO) propagated out.
fn run_range(
    toks: &[Tok],
    start: usize,
    end: usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    let mut pos = start;
    while pos < end {
        if matches!(toks.get(pos), Some(Tok::Dot)) {
            pos += 1;
            continue;
        }
        let before = pos;
        if run_block(toks, &mut pos, fields, out, true, ctx)? {
            // NEXT SENTENCE: skip to the statement after the next period instead of halting.
            if ctx.next_sentence.get() {
                ctx.next_sentence.set(false);
                while pos < end && !matches!(toks.get(pos), Some(Tok::Dot)) {
                    pos += 1;
                }
                if pos < end {
                    pos += 1;
                }
                continue;
            }
            return Ok(true);
        }
        if pos == before {
            // run_block parked on a scope ender (e.g. a malformed EVALUATE WHEN clause leaves an
            // `ELSE`/`WHEN` at the top) without advancing -- skip it so the range ALWAYS progresses
            // (fail closed on the surrounding statement, never an infinite loop).
            pos += 1;
        }
        if matches!(toks.get(pos), Some(Tok::Dot)) {
            pos += 1;
        }
    }
    Ok(false)
}

/// Like [`run_range`] but GO-TO-aware WITHIN `[start, end)`: a pending `GO TO` to a paragraph whose label
/// falls inside the range resumes there (so a paragraph that loops via `GO TO` -- e.g. a SORT INPUT/OUTPUT
/// PROCEDURE's `READ ... GO TO same-para` gather loop -- iterates instead of running once). A real halt
/// (STOP RUN / GOBACK / EXIT PROGRAM) or a GO TO that targets a label OUTSIDE the range ends the runner
/// (the latter is non-conforming for a sort procedure -- control may not leave its range -- so its pending
/// jump is dropped). Returns `true` if a real halt propagated.
fn run_range_goto(
    toks: &[Tok],
    start: usize,
    end: usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    // paragraph label -> token index, restricted to those inside this range (the legal GO TO targets here).
    let in_range: HashMap<String, usize> = CUR_PARAS.with(|c| {
        c.borrow()
            .0
            .iter()
            .filter(|(_, ix)| *ix >= start && *ix < end)
            .map(|(n, ix)| (n.clone(), *ix))
            .collect()
    });
    let mut pos = start;
    let mut guard = 0u64;
    while pos < end {
        if matches!(toks.get(pos), Some(Tok::Dot)) {
            pos += 1;
            continue;
        }
        let before = pos;
        if run_block(toks, &mut pos, fields, out, true, ctx)? {
            if ctx.next_sentence.get() {
                ctx.next_sentence.set(false);
                while pos < end && !matches!(toks.get(pos), Some(Tok::Dot)) {
                    pos += 1;
                }
                if pos < end {
                    pos += 1;
                }
                continue;
            }
            // A pending GO TO into this range resumes there; STOP/GOBACK/EXIT or an out-of-range jump ends it.
            let target = ctx.goto.borrow().clone();
            if let Some(label) = target {
                if let Some(&ix) = in_range.get(&label) {
                    ctx.goto.borrow_mut().take();
                    pos = ix;
                    guard += 1;
                    if guard > 10_000_000 {
                        return Err(RunError::Runtime("GO TO exceeded 1e7 jumps".into()));
                    }
                    continue;
                }
                ctx.goto.borrow_mut().take(); // out-of-range jump from a sort procedure: drop it, end the runner
                return Ok(false);
            }
            return Ok(true); // real halt (STOP RUN / GOBACK / EXIT PROGRAM)
        }
        if pos == before {
            // progress guarantee (see run_range): never park on a scope ender.
            pos += 1;
        }
        if matches!(toks.get(pos), Some(Tok::Dot)) {
            pos += 1;
        }
    }
    Ok(false)
}

/// Run a PERFORMed paragraph range with COBOL's performed-range GO TO semantics: a `GO TO` to a
/// paragraph INSIDE the range resumes there (control stays within the performed procedure -- the
/// classic CCVS85 `PERFORM X THRU X-EXIT` ... `GO TO X-EXIT` report idiom), so the PERFORM still
/// returns to the statement after it when the range end is reached; a `GO TO` to a label OUTSIDE the
/// range abandons the PERFORM and propagates as a body-level jump; a real halt (STOP RUN / GOBACK /
/// EXIT PROGRAM) and the EXIT PERFORM / EXIT PERFORM CYCLE signals also propagate. Returns
/// `Ok(true)` when the range must NOT complete normally (the caller inspects `ctx.goto` /
/// `perform_flow`); `Ok(false)` when the range ran to its end.
fn run_range_perform(
    toks: &[Tok],
    start: usize,
    end: usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    // paragraph label -> token index, restricted to those inside this range (the legal in-range GO TO targets).
    let in_range: HashMap<String, usize> = CUR_PARAS.with(|c| {
        c.borrow()
            .0
            .iter()
            .filter(|(_, ix)| *ix >= start && *ix < end)
            .map(|(n, ix)| (n.clone(), *ix))
            .collect()
    });
    let mut pos = start;
    let mut guard = 0u64;
    loop {
        if run_range(toks, pos, end, fields, out, ctx)? {
            let target = ctx.goto.borrow().clone();
            if let Some(label) = target {
                if let Some(&ix) = in_range.get(&label) {
                    ctx.goto.borrow_mut().take();
                    pos = ix;
                    guard += 1;
                    if guard > 10_000_000 {
                        return Err(RunError::Runtime("GO TO exceeded 1e7 jumps".into()));
                    }
                    continue;
                }
            }
            return Ok(true); // real halt, EXIT PERFORM/CYCLE, or an out-of-range GO TO -- propagate
        }
        return Ok(false); // the range ran to its end (perform iteration complete)
    }
}

/// Execute a program's PROCEDURE DIVISION against `fields`, writing output to `out`. Returns when the body
/// ends (`STOP RUN` / `GOBACK` / `EXIT PROGRAM` / falling off the end).
fn run_program_body(
    prog: &ProgramDef,
    prog_id: &str,
    ctx: &Ctx,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
) -> Result<(), RunError> {
    let proc = &prog.proc_toks;
    let labels = paragraph_labels(proc);
    // Publish this PROGRAM-ID for MODULE-ID / MODULE-CALLER-ID (popped at the normal return below).
    PROGRAM_STACK.with(|s| s.borrow_mut().push(prog_id.to_string()));
    // Publish this body's paragraph ranges for out-of-line PERFORM, saving the caller's (CALL nesting).
    let paras_vec: Vec<(String, usize)> = labels.iter().map(|(n, s)| (n.clone(), *s)).collect();
    let prev_paras = CUR_PARAS.with(|c| c.replace((paras_vec, proc.len())));
    let prev_proc = CUR_PROC.with(|c| c.replace(proc.clone()));
    let prev_proc_lines = CUR_PROC_LINES.with(|c| c.replace(prog.proc_lines.clone()));
    let prev_altered = ALTERED.with(|c| c.replace(HashMap::new()));
    // DECLARATIVES: register the USE error handlers and begin normal execution after END DECLARATIVES.
    let (use_procs, decl_start) = parse_declaratives(proc, &labels);
    let prev_use = USE_PROCS.with(|c| c.replace(use_procs));
    let mut pos = decl_start;
    if pos == 0 && matches!(proc.first(), Some(Tok::Dot)) {
        pos = 1;
    }
    let mut guard = 0u64;
    while pos < proc.len() {
        if matches!(proc.get(pos), Some(Tok::Dot)) {
            pos += 1;
            continue;
        }
        let halted = run_block(proc, &mut pos, fields, out, true, ctx)?;
        if halted {
            // NEXT SENTENCE: skip to the statement after the next period; not a real halt.
            if ctx.next_sentence.get() {
                ctx.next_sentence.set(false);
                while pos < proc.len() && !matches!(proc.get(pos), Some(Tok::Dot)) {
                    pos += 1;
                }
                if pos < proc.len() {
                    pos += 1;
                }
                continue;
            }
            // A pending GO TO is not a real halt: resume at the named paragraph. STOP/GOBACK/EXIT leave
            // `goto` clear and genuinely end the body.
            let target = ctx.goto.borrow_mut().take();
            if let Some(label) = target {
                pos = *labels.get(&label).ok_or_else(|| {
                    RunError::Unsupported(format!("GO TO unknown paragraph `{label}`"))
                })?;
                guard += 1;
                if guard > 10_000_000 {
                    return Err(RunError::Runtime("GO TO exceeded 1e7 jumps".into()));
                }
                continue;
            }
            break; // STOP RUN / GOBACK / EXIT PROGRAM
        }
        if matches!(proc.get(pos), Some(Tok::Dot)) {
            pos += 1;
        }
    }
    // restore the caller's paragraph table (normal return; an error aborts the whole run anyway).
    prof_end_body();
    CUR_PARAS.with(|c| {
        *c.borrow_mut() = prev_paras;
    });
    CUR_PROC.with(|c| {
        *c.borrow_mut() = prev_proc;
    });
    CUR_PROC_LINES.with(|c| {
        *c.borrow_mut() = prev_proc_lines;
    });
    ALTERED.with(|c| {
        *c.borrow_mut() = prev_altered;
    });
    USE_PROCS.with(|c| {
        *c.borrow_mut() = prev_use;
    });
    PROGRAM_STACK.with(|s| {
        s.borrow_mut().pop();
    });
    Ok(())
}

/// The currently-executing paragraph for profiling (upstream 7b6995042's procedure stack, the
/// interpreted equivalent). `prof_switch` fires at each paragraph label in the run stream; the
/// runtime setting (COB_PROF_ENABLE) decides whether anything accumulates.
thread_local! {
    static PROF_CUR: std::cell::RefCell<Option<crate::profiling::ProfProc>> =
        const { std::cell::RefCell::new(None) };
}

/// A paragraph label in the run stream: leave the current paragraph (crediting its time), enter
/// the new one (upstream cob_prof_enter_procedure / cob_prof_exit_procedure).
fn prof_switch(name: &str, pos: usize) {
    let new = crate::profiling::ProfProc {
        module: PROGRAM_STACK
            .with(|s| s.borrow().last().cloned())
            .unwrap_or_default(),
        paragraph: name.to_string(),
        file: source_file_name(),
        line: CUR_PROC_LINES
            .with(|l| l.borrow().get(pos).copied())
            .unwrap_or(0),
    };
    if let Some(cur) = PROF_CUR.with(|c| c.borrow().clone()) {
        if cur != new {
            crate::profiling::prof_exit(&cur);
            PROF_CUR.with(|c| *c.borrow_mut() = Some(new.clone()));
            crate::profiling::prof_enter(new);
        }
    } else {
        PROF_CUR.with(|c| *c.borrow_mut() = Some(new.clone()));
        crate::profiling::prof_enter(new);
    }
}

/// A program body ended: close the current paragraph's accumulation.
fn prof_end_body() {
    if let Some(cur) = PROF_CUR.with(|c| c.borrow_mut().take()) {
        crate::profiling::prof_exit(&cur);
    }
}

/// Map each PROCEDURE DIVISION paragraph/section label -> the token index of its first statement. A label
/// is a non-verb word at statement start (program start or just after a `.`) followed by `.` (paragraph) or
/// by `SECTION .` (section). This is the GO TO jump table; `run_block` skips the same labels while running.
fn paragraph_labels(proc: &[Tok]) -> HashMap<String, usize> {
    let mut m = HashMap::new();
    let mut i = 0;
    let mut at_start = true;
    while i < proc.len() {
        match &proc[i] {
            Tok::Dot => {
                at_start = true;
                i += 1;
            }
            Tok::Word(w)
                if at_start
                    && matches!(proc.get(i + 1), Some(Tok::Dot))
                    && !STMT_VERBS.contains(&w.as_str())
                    && !SCOPE_ENDERS.contains(&w.as_str()) =>
            {
                m.entry(w.clone()).or_insert(i + 2);
                i += 2;
                at_start = true;
            }
            Tok::Word(w)
                if at_start
                    && matches!(proc.get(i + 1), Some(Tok::Word(s)) if s == "SECTION")
                    && matches!(proc.get(i + 2), Some(Tok::Dot)) =>
            {
                m.entry(w.clone()).or_insert(i + 3);
                i += 3;
                at_start = true;
            }
            _ => {
                at_start = false;
                i += 1;
            }
        }
    }
    m
}

/// Statement verbs that begin a new statement (so an operand list ends when one is seen).
const STMT_VERBS: &[&str] = &[
    "MOVE",
    "SET",
    "INITIALIZE",
    "INSPECT",
    "STRING",
    "UNSTRING",
    "ADD",
    "SUBTRACT",
    "MULTIPLY",
    "DIVIDE",
    "COMPUTE",
    "DISPLAY",
    "IF",
    "PERFORM",
    "STOP",
    "CONTINUE",
    "ACCEPT",
    "GO",
    "EVALUATE",
    "SEARCH",
    "CALL",
    "GOBACK",
    "EXIT",
    "CANCEL",
    "OPEN",
    "CLOSE",
    "READ",
    "WRITE",
    "REWRITE",
    "DELETE",
    "START",
    "UNLOCK",
    "COMMIT",
    "ROLLBACK",
    "SORT",
    "MERGE",
    "RELEASE",
    "RETURN",
    "JSON",
    "XML",
    "TRANSFORM",
    "RAISE",
    "VALIDATE",
    "DESTROY",
    "READY",
    "RESET",
    "EXHIBIT",
    "ALTER",
    "GENERATE",
    "INITIATE",
    "TERMINATE",
    "SUPPRESS",
    "EXAMINE",
    "ALLOCATE",
    "FREE",
    "USE",
];
/// Scope terminators that end a block.
const SCOPE_ENDERS: &[&str] = &[
    "ELSE",
    "END-IF",
    "END-PERFORM",
    "WHEN",
    "END-EVALUATE",
    "END-SEARCH",
    "END-READ",
    "END-RETURN",
];

/// Explicit per-verb scope terminators that cobrun does not NEED (each verb parses its operands/handlers up
/// to a boundary or the period already), so a standalone `END-X` is a no-op: it bounds the preceding verb's
/// operand scan and is then skipped. Unlike SCOPE_ENDERS it does NOT end the enclosing block.
const STMT_ENDERS: &[&str] = &[
    "END-DISPLAY",
    "END-ACCEPT",
    "END-STRING",
    "END-UNSTRING",
    "END-COMPUTE",
    "END-ADD",
    "END-SUBTRACT",
    "END-MULTIPLY",
    "END-DIVIDE",
    "END-CALL",
    "END-WRITE",
    "END-REWRITE",
    "END-DELETE",
    "END-START",
    "END-UNLOCK",
    "END-DISABLE",
];

/// Is `verb` a statement verb the front-end accepts (a real dispatch, a verified no-op, or a
/// typed boundary rejection in [`exec_stmt`])? Used by the syntax-only check
/// ([`check_program`]) to fail closed on a statement that starts with a non-verb token (e.g. a
/// bare identifier like `NOT A STATEMENT.`) — the runtime rejects the same verb when executed.
fn known_statement_verb(verb: &str) -> bool {
    matches!(
        verb,
        "DISPLAY"
            | "MOVE"
            | "SET"
            | "INITIALIZE"
            | "INSPECT"
            | "STRING"
            | "UNSTRING"
            | "ACCEPT"
            | "OPEN"
            | "CLOSE"
            | "WRITE"
            | "REWRITE"
            | "DELETE"
            | "SORT"
            | "MERGE"
            | "RELEASE"
            | "JSON"
            | "XML"
            | "TRANSFORM"
            | "EXAMINE"
            | "EXHIBIT"
            | "ALTER"
            | "GENERATE"
            | "INITIATE"
            | "TERMINATE"
            | "SUPPRESS"
            | "RAISE"
            | "VALIDATE"
            | "DESTROY"
            | "READY"
            | "RESET"
            | "UNLOCK"
            | "COMMIT"
            | "ROLLBACK"
            | "CALL"
            | "STOP"
            | "SEND"
            | "RECEIVE"
            | "PURGE"
            | "ENABLE"
            | "DISABLE"
            | "MODIFY"
            | "INQUIRE"
            | "ALLOCATE"
            | "FREE"
            | "ENTRY"
            | "ADD"
            | "SUBTRACT"
            | "MULTIPLY"
            | "DIVIDE"
            | "COMPUTE"
            | "IF"
            | "PERFORM"
            | "EVALUATE"
            | "SEARCH"
            | "READ"
            | "START"
            | "RETURN"
            | "GOBACK"
            | "EXIT"
            | "CONTINUE"
            | "NEXT"
            | "GO"
            | "CANCEL"
    )
}

thread_local! {
    /// Syntax-only ([`check_program`]) mode: the statement walker rejects unknown statement verbs
    /// even while SKIPPING branches, so the check fails closed on any statement the runtime could
    /// not execute. Never set during an actual run (the runtime rejects unknown verbs only in the
    /// branch it executes; a skipped branch is not executed and must not fail the program).
    static CHECK_MODE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn is_boundary(w: &str) -> bool {
    STMT_VERBS.contains(&w) || SCOPE_ENDERS.contains(&w) || STMT_ENDERS.contains(&w)
}

/// Execute (or, when `exec` is false, SKIP) a block of statements starting at `*pos`, stopping -- WITHOUT
/// consuming -- at a `.`, a scope terminator, or end of input. Returns `Ok(true)` if `STOP RUN` halted
/// the program. This is the spine of control flow: `IF`/`PERFORM` recurse into it for their branches.
fn run_block(
    toks: &[Tok],
    pos: &mut usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    exec: bool,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    loop {
        match toks.get(*pos) {
            None | Some(Tok::Dot) => return Ok(false),
            Some(Tok::Word(w)) if SCOPE_ENDERS.contains(&w.as_str()) => return Ok(false),
            // A stray explicit scope terminator (END-DISPLAY, END-ADD, ...) -- the verb it closed already
            // ran (cobrun parses to a boundary), so skip it and continue this block.
            Some(Tok::Word(w)) if STMT_ENDERS.contains(&w.as_str()) => {
                *pos += 1;
                continue;
            }
            Some(Tok::Word(w)) => {
                // A paragraph label `NAME.` or section label `NAME SECTION.` in the run stream: skip it.
                // The following period ends this (empty) block; the program-body loop resumes after it.
                if matches!(toks.get(*pos + 1), Some(Tok::Dot))
                    && !STMT_VERBS.contains(&w.as_str())
                    && !SCOPE_ENDERS.contains(&w.as_str())
                {
                    prof_switch(w, *pos);
                    *pos += 1;
                    return Ok(false);
                }
                if matches!(toks.get(*pos + 1), Some(Tok::Word(s)) if s == "SECTION")
                    && matches!(toks.get(*pos + 2), Some(Tok::Dot))
                {
                    *pos += 2;
                    return Ok(false);
                }
                let verb = w.clone();
                let verb_pos = *pos; // token index of this verb (for ALTERed GO TO lookup)
                *pos += 1;
                match verb.as_str() {
                    "IF" => {
                        if exec_if(toks, pos, fields, out, exec, ctx)? {
                            return Ok(true);
                        }
                    }
                    "PERFORM" => {
                        if exec_perform(toks, pos, fields, out, exec, ctx)? {
                            return Ok(true);
                        }
                    }
                    "EVALUATE" => {
                        if exec_evaluate(toks, pos, fields, out, exec, ctx)? {
                            return Ok(true);
                        }
                    }
                    "SEARCH" => {
                        if exec_search(toks, pos, fields, out, exec, ctx)? {
                            return Ok(true);
                        }
                    }
                    "READ" => {
                        if exec_read(toks, pos, fields, out, exec, ctx)? {
                            return Ok(true);
                        }
                    }
                    "START" => {
                        if exec_start(toks, pos, fields, out, exec, ctx)? {
                            return Ok(true);
                        }
                    }
                    "RETURN" => {
                        if exec_return(toks, pos, fields, out, exec, ctx)? {
                            return Ok(true);
                        }
                    }
                    // STOP RUN halts the WHOLE run -- even from inside a CALLed program it unwinds to the run
                    // boundary (libcob longjmp(stop_run)); GOBACK / EXIT PROGRAM only end the current body and
                    // return to the caller. Both end this body (Ok(true)); STOP additionally sets ctx.stop_run
                    // so the decision survives the CALL boundary (see the CALL propagation below).
                    "STOP" | "GOBACK" => {
                        let rest = collect_operands(toks, pos); // consume RUN / trailing words
                        if exec {
                            // STOP RUN <n> / GOBACK <n>: set the exit code (RETURN-CODE) to the integer.
                            if let Some(n) = rest.iter().find_map(|t| match t {
                                Tok::Word(w) if w != "RUN" => w.parse::<i64>().ok(),
                                _ => None,
                            }) {
                                fields.insert("RETURN-CODE".to_string(), make_return_code(n));
                            }
                            if verb == "STOP" {
                                ctx.stop_run.set(true);
                            }
                            return Ok(true);
                        }
                    }
                    "EXIT" => {
                        // EXIT PROGRAM ends the body; EXIT PERFORM [CYCLE] signals the nearest PERFORM loop;
                        // a bare EXIT / EXIT PARAGRAPH / EXIT SECTION is a no-op (paragraph fall-through).
                        // (Peeked directly: PERFORM is a STMT_VERB, so collect_operands would stop before it.)
                        let qual = toks.get(*pos).and_then(|t| match t {
                            Tok::Word(w) => Some(w.as_str()),
                            _ => None,
                        });
                        match qual {
                            Some("PROGRAM") => {
                                *pos += 1;
                                if exec {
                                    return Ok(true);
                                }
                            }
                            Some("PERFORM") => {
                                *pos += 1;
                                let cycle =
                                    matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "CYCLE");
                                if cycle {
                                    *pos += 1;
                                }
                                if exec {
                                    if cycle {
                                        ctx.exit_cycle.set(true);
                                    } else {
                                        ctx.exit_perform.set(true);
                                    }
                                    return Ok(true);
                                }
                            }
                            Some("PARAGRAPH") | Some("SECTION") => {
                                *pos += 1; // no-op (fall through to the end of the paragraph/section)
                            }
                            _ => { /* bare EXIT: no-op */ }
                        }
                    }
                    "CONTINUE" => { /* no-op */ }
                    "NEXT" => {
                        // NEXT SENTENCE: control transfers to the statement after the next period. Consume
                        // SENTENCE, then (when executing) signal the paragraph loop and end this block.
                        let is_sentence =
                            matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "SENTENCE");
                        if is_sentence {
                            *pos += 1;
                        }
                        if exec && is_sentence {
                            ctx.next_sentence.set(true);
                            return Ok(true);
                        }
                    }
                    // GO TO <paragraph>: set the pending-jump and end this block like a halt; the program
                    // body loop resolves the label and resumes there. `GO TO ... DEPENDING ON` is out of subset.
                    "GO" => {
                        let rest = collect_operands(toks, pos);
                        if exec {
                            // GO TO l1 l2 ... lN DEPENDING ON id: jump to the id-th label (1-based); if id
                            // is < 1 or > N, fall through to the next statement (no jump), per the standard.
                            if let Some(dep) = rest
                                .iter()
                                .position(|t| matches!(t, Tok::Word(w) if w == "DEPENDING"))
                            {
                                let labels: Vec<String> = rest[..dep]
                                    .iter()
                                    .filter_map(|t| match t {
                                        Tok::Word(w) if w != "TO" => Some(w.clone()),
                                        _ => None,
                                    })
                                    .collect();
                                let id = rest[dep + 1..].iter().find_map(|t| match t {
                                    Tok::Word(w) if w != "ON" => Some(w.clone()),
                                    _ => None,
                                });
                                let idx = id.as_deref().and_then(|w| resolve_int(w, fields));
                                if let Some(i) = idx {
                                    if i >= 1 && (i as usize) <= labels.len() {
                                        ctx.goto
                                            .borrow_mut()
                                            .replace(labels[(i - 1) as usize].clone());
                                        return Ok(true);
                                    }
                                }
                                // out of range / unresolved -> fall through (continue with the next statement).
                                continue;
                            }
                            // an ALTERed GO TO (this verb's position is in the override map) proceeds to the
                            // altered target; otherwise the written target.
                            let altered = ALTERED.with(|c| c.borrow().get(&verb_pos).cloned());
                            let label = altered.or_else(|| {
                                rest.iter().find_map(|t| match t {
                                    Tok::Word(w) if w != "TO" => Some(w.clone()),
                                    _ => None,
                                })
                            });
                            match label {
                                Some(l) => {
                                    ctx.goto.borrow_mut().replace(l);
                                    return Ok(true);
                                }
                                None => {
                                    return Err(RunError::Unsupported(
                                        "GO TO without a target paragraph".into(),
                                    ))
                                }
                            }
                        }
                    }
                    // CANCEL "NAME" ... -- drop each named program's persisted WORKING-STORAGE, so its
                    // next CALL rebuilds from VALUE (libcob un-initializes + unloads the module).
                    // CANCELing an ACTIVE (in-call-chain) program that is not INITIAL is a fatal
                    // runtime error in libcob ("attempt to CANCEL active program").
                    "CANCEL" => {
                        let rest = collect_operands(toks, pos);
                        if exec {
                            for t in &rest {
                                let nm = match t {
                                    Tok::Str(s) => Some(String::from_utf8_lossy(s).to_string()),
                                    Tok::Word(w) => Some(w.clone()),
                                    Tok::AllLiteral(_) => None, // not a program name
                                    Tok::Dot => None,
                                };
                                if let Some(nm) = nm {
                                    let active = PROGRAM_STACK.with(|s| {
                                        s.borrow().iter().any(|p| p.eq_ignore_ascii_case(&nm))
                                    });
                                    let initial = ctx
                                        .programs
                                        .get(&nm)
                                        .or_else(|| ctx.programs.get(&nm.to_uppercase()))
                                        .map(|p| p.is_initial)
                                        .unwrap_or(false);
                                    if active && !initial {
                                        let line = CUR_PROC_LINES
                                            .with(|l| l.borrow().get(*pos).copied().unwrap_or(0));
                                        return Err(fatal_with_line(
                                            line,
                                            "attempt to CANCEL active program".to_string(),
                                        ));
                                    }
                                    ctx.call_state.borrow_mut().remove(&nm);
                                }
                            }
                        }
                    }
                    // Arithmetic verbs carry optional ON SIZE ERROR / NOT ON SIZE ERROR handler blocks
                    // (+ END-verb), so they are parsed here rather than via collect_operands/exec_stmt.
                    // STRING carries optional ON OVERFLOW / NOT ON OVERFLOW handler blocks (+ END-STRING),
                    // so it is parsed here (like the arithmetic verbs) rather than via exec_stmt.
                    "STRING" => {
                        let stmt = collect_arith_operands(toks, pos);
                        let on_of = parse_on_overflow_handler(toks, pos, false);
                        let not_of = parse_on_overflow_handler(toks, pos, true);
                        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-STRING") {
                            *pos += 1;
                        }
                        if exec {
                            let overflow = exec_string(&stmt, fields, ctx.decimal_comma)?;
                            let handler = if overflow { &on_of } else { &not_of };
                            if let Some(block) = handler {
                                if run_handler(block, fields, out, ctx)? {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                    // UNSTRING carries optional ON OVERFLOW / NOT ON OVERFLOW handler blocks (+ END-UNSTRING),
                    // parsed here like STRING. OVERFLOW = source characters remain after every receiver fills.
                    "UNSTRING" => {
                        let stmt = collect_arith_operands(toks, pos);
                        let on_of = parse_on_overflow_handler(toks, pos, false);
                        let not_of = parse_on_overflow_handler(toks, pos, true);
                        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-UNSTRING") {
                            *pos += 1;
                        }
                        if exec {
                            let overflow = exec_unstring(&stmt, fields)?;
                            let handler = if overflow { &on_of } else { &not_of };
                            if let Some(block) = handler {
                                if run_handler(block, fields, out, ctx)? {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                    // JSON / XML GENERATE: the SUPPRESS clause keyword is ALSO a Report-Writer verb, so the
                    // generic collect_operands would stop there. Collect the whole statement (NAME / SUPPRESS
                    // / COUNT clauses) up to the period / END-JSON / END-XML / a real boundary verb instead.
                    "JSON" | "XML" => {
                        let mut stmt = Vec::new();
                        while let Some(t) = toks.get(*pos) {
                            match t {
                                Tok::Dot => break,
                                Tok::Word(w) if w == "END-JSON" || w == "END-XML" => break,
                                // ON EXCEPTION / NOT ON EXCEPTION begin the handler section (parsed below).
                                Tok::Word(w)
                                    if w == "ON"
                                        && matches!(toks.get(*pos + 1), Some(Tok::Word(x)) if x == "EXCEPTION") =>
                                {
                                    break
                                }
                                Tok::Word(w)
                                    if w == "NOT"
                                        && matches!(toks.get(*pos + 1), Some(Tok::Word(x)) if x == "ON") =>
                                {
                                    break
                                }
                                // GENERATE/PARSE are the JSON/XML sub-verbs, SUPPRESS a clause keyword -- none ends the statement here.
                                // SUPPRESS / GENERATE / PARSE are sub-verbs/clauses, and WHEN is the SUPPRESS
                                // ... WHEN <fig> qualifier (a SCOPE_ENDER elsewhere) -- none ends the statement.
                                Tok::Word(w)
                                    if is_boundary(w)
                                        && !matches!(
                                            w.as_str(),
                                            "SUPPRESS" | "GENERATE" | "PARSE" | "WHEN"
                                        ) =>
                                {
                                    break
                                }
                                _ => {
                                    stmt.push(t.clone());
                                    *pos += 1;
                                }
                            }
                        }
                        let on_exc = parse_ml_exception_handler(toks, pos, false);
                        let not_exc = parse_ml_exception_handler(toks, pos, true);
                        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-JSON" || w == "END-XML")
                        {
                            *pos += 1;
                        }
                        if exec {
                            // The supported JSON/XML GENERATE subset does not raise an exception. On success
                            // cobc runs the NOT ON EXCEPTION handler ONLY when it is the SOLE handler; when an
                            // ON EXCEPTION clause is also present cobc runs NEITHER branch (a 3.2 quirk we match).
                            exec_stmt(&verb, &stmt, fields, out, ctx)?;
                            if ctx.stop_run.get() {
                                return Ok(true);
                            }
                            if on_exc.is_none() {
                                if let Some(block) = &not_exc {
                                    if run_handler(block, fields, out, ctx)? {
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                    }
                    "ADD" | "SUBTRACT" | "MULTIPLY" | "DIVIDE" | "COMPUTE" => {
                        let stmt = collect_arith_operands(toks, pos);
                        let end_kw = format!("END-{verb}");
                        let on_size = parse_on_size_handler(toks, pos, false, &end_kw);
                        let not_size = parse_on_size_handler(toks, pos, true, &end_kw);
                        if matches!(toks.get(*pos), Some(Tok::Word(w)) if *w == end_kw) {
                            *pos += 1;
                        }
                        if exec {
                            let has_handler = on_size.is_some();
                            let size_err = if verb == "COMPUTE" {
                                exec_compute(&stmt, fields, has_handler)?
                            } else {
                                exec_arith(&verb, &stmt, fields, has_handler)?
                            };
                            let handler = if size_err { &on_size } else { &not_size };
                            if let Some(block) = handler {
                                if run_handler(block, fields, out, ctx)? {
                                    return Ok(true); // STOP RUN / GOBACK inside the handler
                                }
                            }
                        }
                    }
                    _ => {
                        let stmt = collect_operands(toks, pos);
                        if exec {
                            exec_stmt(&verb, &stmt, fields, out, ctx)?;
                            // A CALLed program that hit STOP RUN unwinds the whole run, not just its own
                            // body: propagate the halt up past the CALL (GOBACK/EXIT PROGRAM leave the flag
                            // clear, so a normal return continues here).
                            if ctx.stop_run.get() {
                                return Ok(true);
                            }
                        } else if CHECK_MODE.with(|c| c.get()) && !known_statement_verb(&verb) {
                            return Err(RunError::Unsupported(format!("verb {verb}")));
                        } else if CHECK_MODE.with(|c| c.get())
                            && matches!(verb.as_str(), "SORT" | "MERGE" | "RELEASE" | "RETURN")
                        {
                            // Upstream 23f850352/277a07c2e: SORT/MERGE/RETURN must name an `SD` file and
                            // RELEASE a record of an `SD` file -- compile-time errors in cobc; mirror the
                            // accept/reject here so `-fsyntax-only` fails the same programs.
                            check_sort_statements(&verb, &stmt, ctx)?;
                        } else if CHECK_MODE.with(|c| c.get()) && verb == "INSPECT" {
                            // Upstream validate_inspect (04614ac7a): REPLACING/CONVERTING operand
                            // size/identity checks happen at compile time; mirror the accept/reject.
                            validate_inspect_operands(&stmt, fields)?;
                        }
                    }
                }
            }
            Some(_) => {
                *pos += 1;
            }
        }
    }
}

/// Syntax-only ([`check_program`]) validation of the SD operand rules (upstream 23f850352/277a07c2e):
/// SORT/MERGE work files and RETURN targets must be `SD` (sort-description) files, and RELEASE must name
/// a record of an `SD` file. Mirrors the runtime checks so `-fsyntax-only` rejects the same programs cobc
/// rejects at compile time.
fn check_sort_statements(verb: &str, stmt: &[Tok], ctx: &Ctx) -> Result<(), RunError> {
    let first = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Ok(()), // a structural shape error is the runtime's typed failure
    };
    match verb {
        "SORT" | "MERGE" => match ctx.file_defs.get(&first) {
            None => {
                return Err(RunError::Unsupported(format!(
                    "{verb}: `{first}` is not a declared file"
                )))
            }
            Some(d) if d.org != FileOrg::Sort => {
                return Err(RunError::Unsupported(format!(
                    "{verb}: `{first}` must be an SD filename"
                )))
            }
            _ => {}
        },
        "RETURN" => match ctx.file_defs.get(&first) {
            None => {
                return Err(RunError::Unsupported(format!(
                    "RETURN: `{first}` is not a declared file"
                )))
            }
            Some(d) if d.org != FileOrg::Sort => {
                return Err(RunError::Unsupported(format!(
                    "RETURN: `{first}` must be an SD filename"
                )))
            }
            _ => {}
        },
        "RELEASE" => {
            // RELEASE names a RECORD (not a file); resolve the owning file and require it to be an SD.
            let fname = match ctx.record_files.get(&first) {
                Some(f) => f.clone(),
                None => {
                    return Err(RunError::Unsupported(format!(
                        "RELEASE `{first}`: not an SD/FD record"
                    )))
                }
            };
            match ctx.file_defs.get(&fname) {
                None => {
                    return Err(RunError::Unsupported(format!(
                        "RELEASE `{first}`: not an SD/FD record"
                    )))
                }
                Some(d) if d.org != FileOrg::Sort => {
                    return Err(RunError::Unsupported(format!(
                        "RELEASE `{first}`: record is not a sort-file record"
                    )))
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

/// Collect a simple statement's operand tokens from `*pos` until the next statement verb, scope
/// terminator, or `.`.
fn collect_operands(toks: &[Tok], pos: &mut usize) -> Vec<Tok> {
    let mut v = Vec::new();
    while let Some(t) = toks.get(*pos) {
        match t {
            Tok::Dot => break,
            Tok::Word(w) if is_boundary(w) => break,
            _ => {
                v.push(t.clone());
                *pos += 1;
            }
        }
    }
    v
}

/// Collect an arithmetic statement's operand tokens (up to an `ON`/`NOT` SIZE-ERROR clause, an `END-verb`,
/// a `.`, or the next statement boundary).
fn collect_arith_operands(toks: &[Tok], pos: &mut usize) -> Vec<Tok> {
    let start = *pos;
    while let Some(t) = toks.get(*pos) {
        match t {
            Tok::Dot => break,
            Tok::Word(w) if w == "ON" || w == "NOT" || w.starts_with("END-") || is_boundary(w) => {
                break
            }
            _ => *pos += 1,
        }
    }
    toks[start..*pos].to_vec()
}

/// Words that open a scoped statement with an `END-<w>` terminator. While scanning an imperative
/// handler block, an occurrence of one of these only opens a scope when its matching `END-<w>`
/// actually appears before the handler ends (see [`scoped_end_before`]); a bare/inline use (e.g.
/// `PERFORM PASS` or a bare `READ f`) opens nothing.
const SCOPE_OPENERS: &[&str] = &[
    "IF", "EVALUATE", "PERFORM", "STRING", "UNSTRING", "SEARCH", "READ", "RETURN", "SORT", "START",
    "REWRITE", "DELETE", "JSON", "XML", "CALL",
];

/// Is the construct opened by `w` at `from` scoped — i.e. does a matching `END-<w>` appear before
/// the handler would end (`handler_ends`)? Returns the matching `END-<w>` position, else `None` (an
/// inline/imperative use such as `PERFORM PASS` or a bare `READ f`). Nested uses of the SAME verb
/// (`PERFORM` inside `PERFORM … END-PERFORM`) are counted, so only the OUTERMOST `END-<w>` matches.
fn scoped_end_before(
    toks: &[Tok],
    from: usize,
    w: &str,
    handler_ends: impl Fn(usize) -> bool,
) -> Option<usize> {
    let end = format!("END-{w}");
    let mut depth = 1usize;
    let mut p = from + 1;
    while p < toks.len() {
        match toks.get(p) {
            None | Some(Tok::Dot) => return None,
            Some(Tok::Word(x)) => {
                let x = x.as_str();
                if x == w && SCOPE_OPENERS.contains(&x) {
                    depth += 1;
                } else if x == end {
                    depth -= 1;
                    if depth == 0 {
                        return Some(p);
                    }
                } else if depth == 1 && handler_ends(p) {
                    return None;
                }
            }
            _ => {}
        }
        p += 1;
    }
    None
}

/// Scan an imperative handler's statement block (`ON SIZE ERROR`, `AT END`, `ON OVERFLOW`,
/// `ON EXCEPTION`, …) from `start`. Returns the index AT the block's terminator: the enclosing
/// statement's `END-<verb>` (`end_at`), the other handler clause (`other_clause`), a period (ends
/// the whole sentence), a depth-0 scope ender, or end of input. Nested scoped statements are
/// consumed as balanced blocks, so a nested `END-IF` / `END-PERFORM` / `END-READ` … is never
/// mistaken for the handler's own end (a real defect: `ON SIZE ERROR … IF … END-IF` truncated the
/// handler at the nested `END-IF`, misaligning the enclosing PERFORM scope into a non-terminating
/// loop — the GnuCOBOL `FLOAT-SHORT with SIZE ERROR` hang).
fn scan_handler_block(
    toks: &[Tok],
    start: usize,
    end_at: impl Fn(&str) -> bool,
    other_clause: impl Fn(usize) -> bool,
) -> usize {
    let mut stack: Vec<String> = Vec::new();
    let mut p = start;
    while p < toks.len() {
        match toks.get(p) {
            None | Some(Tok::Dot) => break,
            Some(Tok::Word(w)) => {
                let w = w.as_str();
                if w.starts_with("END-") {
                    if stack.last().map(String::as_str) == Some(w) {
                        stack.pop();
                    } else if end_at(w) {
                        // the handler's own END-<verb> (only legal at depth 0; a depth>0 hit is
                        // malformed COBOL and fails closed here).
                        break;
                    } else if STMT_ENDERS.contains(&w) {
                        // A nested NON-scoped statement's own terminator (END-DISPLAY / END-ADD /
                        // END-STRING ...) inside the handler (at ANY depth): it closes that
                        // statement, not the handler -- skip it and keep scanning.
                    } else {
                        // an unmatched scope ender (END-IF/END-PERFORM/...) whose opener was not
                        // tracked: malformed COBOL -- fail closed.
                        break;
                    }
                } else if SCOPE_OPENERS.contains(&w) {
                    let handler_ends = |q: usize| {
                        other_clause(q) || matches!(toks.get(q), Some(Tok::Word(x)) if end_at(x))
                    };
                    if scoped_end_before(toks, p, w, handler_ends).is_some() {
                        stack.push(format!("END-{w}"));
                    }
                } else if stack.is_empty()
                    && (end_at(w) || other_clause(p) || SCOPE_ENDERS.contains(&w))
                {
                    break;
                }
            }
            _ => {}
        }
        p += 1;
    }
    p
}

/// Parse an `[NOT] ON SIZE ERROR <statements>` handler at `*pos` (when `is_not`, the `NOT ON SIZE ERROR`
/// form). Returns the handler statement tokens and advances `*pos` past them; `None` if the clause is absent.
/// Parse a `[NOT] ON EXCEPTION <imperative>` handler block for JSON/XML GENERATE, mirroring
/// [`parse_on_size_handler`]. The block runs to END-JSON / END-XML / `.` / a scope terminator / the other
/// (`NOT ON`) handler.
fn parse_ml_exception_handler(toks: &[Tok], pos: &mut usize, is_not: bool) -> Option<Vec<Tok>> {
    let mut p = *pos;
    if is_not {
        if !matches!(toks.get(p), Some(Tok::Word(w)) if w == "NOT") {
            return None;
        }
        p += 1;
    }
    if !(matches!(toks.get(p), Some(Tok::Word(w)) if w == "ON")
        && matches!(toks.get(p + 1), Some(Tok::Word(w)) if w == "EXCEPTION"))
    {
        return None;
    }
    p += 2;
    let start = p;
    let other = |q: usize| {
        matches!(toks.get(q), Some(Tok::Word(w)) if w == "NOT")
            && matches!(toks.get(q + 1), Some(Tok::Word(w)) if w == "ON")
    };
    let end_at = |w: &str| w == "END-JSON" || w == "END-XML";
    p = scan_handler_block(toks, start, end_at, other);
    let block = toks[start..p].to_vec();
    *pos = p;
    Some(block)
}

fn parse_on_size_handler(
    toks: &[Tok],
    pos: &mut usize,
    is_not: bool,
    end_kw: &str,
) -> Option<Vec<Tok>> {
    let mut p = *pos;
    if is_not {
        if !matches!(toks.get(p), Some(Tok::Word(w)) if w == "NOT") {
            return None;
        }
        p += 1;
    }
    if !(matches!(toks.get(p), Some(Tok::Word(w)) if w == "ON")
        && matches!(toks.get(p + 1), Some(Tok::Word(w)) if w == "SIZE")
        && matches!(toks.get(p + 2), Some(Tok::Word(w)) if w == "ERROR"))
    {
        return None;
    }
    p += 3;
    let start = p;
    let other = |q: usize| {
        matches!(toks.get(q), Some(Tok::Word(w)) if w == "NOT")
            && matches!(toks.get(q + 1), Some(Tok::Word(w)) if w == "ON")
    };
    let end_at = move |w: &str| w == end_kw;
    p = scan_handler_block(toks, start, end_at, other);
    let block = toks[start..p].to_vec();
    *pos = p;
    Some(block)
}

/// Parse an `[NOT] ON OVERFLOW <statements>` handler at `*pos` (the STRING overflow clause). Returns the
/// handler tokens and advances `*pos`; `None` if the clause is absent.
fn parse_on_overflow_handler(toks: &[Tok], pos: &mut usize, is_not: bool) -> Option<Vec<Tok>> {
    let mut p = *pos;
    if is_not {
        if !matches!(toks.get(p), Some(Tok::Word(w)) if w == "NOT") {
            return None;
        }
        p += 1;
    }
    if !(matches!(toks.get(p), Some(Tok::Word(w)) if w == "ON")
        && matches!(toks.get(p + 1), Some(Tok::Word(w)) if w == "OVERFLOW"))
    {
        return None;
    }
    p += 2;
    let start = p;
    let other = |q: usize| {
        matches!(toks.get(q), Some(Tok::Word(w)) if w == "NOT")
            && matches!(toks.get(q + 1), Some(Tok::Word(w)) if w == "ON")
    };
    let end_at = |w: &str| w == "END-STRING" || w == "END-UNSTRING";
    p = scan_handler_block(toks, start, end_at, other);
    let block = toks[start..p].to_vec();
    *pos = p;
    Some(block)
}

/// Run an arithmetic SIZE-ERROR handler block (its own statement sequence). Returns `true` on `STOP RUN` /
/// `GOBACK` inside it.
fn run_handler(
    block: &[Tok],
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    let mut p = 0;
    while p < block.len() {
        if matches!(block.get(p), Some(Tok::Dot)) {
            p += 1;
            continue;
        }
        if run_block(block, &mut p, fields, out, true, ctx)? {
            return Ok(true);
        }
        if matches!(block.get(p), Some(Tok::Dot)) {
            p += 1;
        }
    }
    Ok(false)
}

/// `IF <cond> [THEN] <stmts> [ELSE <stmts>] [END-IF]` -- evaluate the condition, run the taken branch,
/// skip the other. The IF scope ends at `END-IF` or, in the period form, at the sentence `.`.
fn exec_if(
    toks: &[Tok],
    pos: &mut usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    exec: bool,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    // condition tokens: from here until a statement verb, THEN, scope terminator, or '.'.
    let mut cond = Vec::new();
    while let Some(t) = toks.get(*pos) {
        match t {
            Tok::Dot => break,
            Tok::Word(w)
                if w == "THEN"
                    || w == "NEXT"
                    || STMT_VERBS.contains(&w.as_str())
                    || SCOPE_ENDERS.contains(&w.as_str()) =>
            {
                break
            }
            _ => {
                cond.push(t.clone());
                *pos += 1;
            }
        }
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "THEN") {
        *pos += 1;
    }
    let truth = if exec {
        eval_cond(&cond, fields, &ctx.switches, ctx.collation.as_ref())?
    } else {
        false
    };

    // THEN branch.
    let halted = run_block(toks, pos, fields, out, exec && truth, ctx)?;
    if halted {
        return Ok(true);
    }
    // ELSE branch.
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "ELSE") {
        *pos += 1;
        let halted = run_block(toks, pos, fields, out, exec && !truth, ctx)?;
        if halted {
            return Ok(true);
        }
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-IF") {
        *pos += 1;
    }
    Ok(false)
}

/// A `Tok` as a condition/comparison word (the `\u{1}`-marked form for a string literal), so the EVALUATE
/// subject/WHEN operands route through the same `eval_cond` / `cond_compare` machinery as `IF`.
fn tok_to_cond_word(t: &Tok) -> String {
    match t {
        Tok::Word(w) => w.clone(),
        Tok::Str(s) => format!("\u{1}{}", String::from_utf8_lossy(s)),
        Tok::AllLiteral(s) => format!("\u{1}{}", String::from_utf8_lossy(s)),
        Tok::Dot => ".".into(),
    }
}

/// `EVALUATE <subject> (WHEN <object> <stmts>)+ [WHEN OTHER <stmts>] END-EVALUATE` -- the COBOL case
/// statement. The subject is either `TRUE` (each WHEN object is a CONDITION) or a value (each WHEN object
/// is a value, optionally `v1 THRU v2`); the FIRST matching WHEN's statements run, the rest are skipped.
fn exec_evaluate(
    toks: &[Tok],
    pos: &mut usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    exec: bool,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    // subject: tokens from here until the first WHEN.
    let mut subject: Vec<Tok> = Vec::new();
    while let Some(t) = toks.get(*pos) {
        match t {
            Tok::Word(w) if w == "WHEN" => break,
            Tok::Dot => break,
            _ => {
                subject.push(t.clone());
                *pos += 1;
            }
        }
    }
    let is_true =
        subject.len() == 1 && matches!(&subject[0], Tok::Word(w) if w == "TRUE" || w == "ANY");
    let subject_word = subject.first().map(tok_to_cond_word);
    let mut matched = false;

    while matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "WHEN") {
        *pos += 1;
        let is_other = matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "OTHER");
        let mut object: Vec<Tok> = Vec::new();
        if is_other {
            *pos += 1;
        } else {
            while let Some(t) = toks.get(*pos) {
                match t {
                    Tok::Dot => break,
                    Tok::Word(w) if w == "WHEN" || is_boundary(w) => break,
                    _ => {
                        object.push(t.clone());
                        *pos += 1;
                    }
                }
            }
        }
        // does this WHEN match? (only meaningful when executing and nothing matched yet).
        let clause_matches = if !exec || matched {
            false
        } else if is_other {
            true
        } else if is_true {
            eval_cond(&object, fields, &ctx.switches, ctx.collation.as_ref())?
        } else {
            evaluate_value_match(subject_word.as_deref(), &object, fields, ctx)?
        };
        // run (or skip) this WHEN's statements -- run_block stops at the next WHEN / END-EVALUATE.
        let halted = run_block(toks, pos, fields, out, exec && clause_matches, ctx)?;
        if halted {
            return Ok(true);
        }
        if clause_matches {
            matched = true;
        }
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-EVALUATE") {
        *pos += 1;
    }
    Ok(false)
}

/// `SEARCH table [VARYING idx] [AT END imperative] {WHEN cond imperative}... END-SEARCH` -- a serial (linear)
/// search: from the index's current value it tests each WHEN in order at each element; the first WHEN true
/// runs its imperative and ends, the index advancing by 1 otherwise; running off the end runs AT END. The
/// index is the table's `INDEXED BY` index (or the explicit `VARYING` one). Binary `SEARCH ALL` is out of
/// subset (fails closed).
fn exec_search(
    toks: &[Tok],
    pos: &mut usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    exec: bool,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    // `SEARCH ALL table` -- binary search over a sorted (ASCENDING/DESCENDING KEY) table.
    let is_all = matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "ALL");
    if is_all {
        *pos += 1;
    }
    let table = match toks.get(*pos) {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("SEARCH: missing table name".into())),
    };
    *pos += 1;
    let mut varying: Option<String> = None;
    if !is_all && matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "VARYING") {
        *pos += 1;
        if let Some(Tok::Word(w)) = toks.get(*pos) {
            varying = Some(w.clone());
            *pos += 1;
        }
    }
    let occurs = fields
        .get(&table)
        .map(|f| f.occurs)
        .filter(|&o| o > 1)
        .ok_or_else(|| RunError::Unsupported(format!("SEARCH `{table}` is not an OCCURS table")))?;
    let idx_name = varying
        .or_else(|| table_index_lookup(&table))
        .ok_or_else(|| {
            RunError::Unsupported(format!("SEARCH `{table}`: no INDEXED BY or VARYING index"))
        })?;
    // parse (do not yet run) the optional AT END block and the WHEN clauses, recording token ranges.
    let mut at_end: Option<usize> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "AT") {
        *pos += 1;
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END") {
            *pos += 1;
        }
        at_end = Some(*pos);
        let mut scan = *pos;
        let _ = run_block(toks, &mut scan, fields, out, false, ctx)?;
        *pos = scan;
    }
    let mut whens: Vec<(Vec<Tok>, usize)> = Vec::new();
    while matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "WHEN") {
        *pos += 1;
        let mut cond = Vec::new();
        while let Some(t) = toks.get(*pos) {
            match t {
                Tok::Dot => break,
                Tok::Word(w) if is_boundary(w) => break,
                _ => {
                    cond.push(t.clone());
                    *pos += 1;
                }
            }
        }
        let block_start = *pos;
        let mut scan = *pos;
        let _ = run_block(toks, &mut scan, fields, out, false, ctx)?;
        *pos = scan;
        whens.push((cond, block_start));
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-SEARCH") {
        *pos += 1;
    }
    if !exec {
        return Ok(false);
    }
    if is_all {
        return exec_search_all(
            toks, &idx_name, &table, occurs, at_end, &whens, fields, out, ctx,
        );
    }
    // serial search: vary the index from its current value until a WHEN matches or it runs off the table.
    loop {
        let iv = resolve_int(&idx_name, fields).unwrap_or(0);
        if iv < 1 || iv as usize > occurs {
            if let Some(s) = at_end {
                let mut p = s;
                if run_block(toks, &mut p, fields, out, true, ctx)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        for (cond, bstart) in &whens {
            if eval_cond(cond, fields, &ctx.switches, ctx.collation.as_ref())? {
                let mut p = *bstart;
                return run_block(toks, &mut p, fields, out, true, ctx);
            }
        }
        let mv = vec![
            Tok::Word((iv + 1).to_string()),
            Tok::Word("TO".to_string()),
            Tok::Word(idx_name.clone()),
        ];
        exec_move(&mv, fields, ctx.decimal_comma)?;
    }
}

/// `SEARCH ALL table [AT END imp] WHEN key=value [AND key2=value2...] imp END-SEARCH` -- a **binary
/// search** over a table sorted by its `OCCURS ... ASCENDING|DESCENDING KEY`. From the WHEN key-equality
/// condition: at each probe `mid` the index is set to `mid` and the full condition is tested for a match;
/// the first key's `key < value` comparison narrows the half (combined with the sort direction). On no
/// match, `AT END` runs. The WHEN must be key-equality (`=`), per the standard.
#[allow(clippy::too_many_arguments)]
fn exec_search_all(
    toks: &[Tok],
    idx_name: &str,
    table: &str,
    occurs: usize,
    at_end: Option<usize>,
    whens: &[(Vec<Tok>, usize)],
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    let (cond, bstart) = whens
        .first()
        .ok_or_else(|| RunError::Unsupported("SEARCH ALL: missing WHEN".into()))?;
    let asc = TABLE_KEY
        .with(|m| m.borrow().get(table).copied())
        .ok_or_else(|| {
            RunError::Unsupported(format!("SEARCH ALL `{table}`: no ASCENDING/DESCENDING KEY"))
        })?;
    // the narrowing comparison: the first key's `=` turned into `<`.
    let less = search_all_less(cond).ok_or_else(|| {
        RunError::Unsupported("SEARCH ALL: WHEN must be a key equality (key = value)".into())
    })?;
    let mut lo = 1i64;
    let mut hi = occurs as i64;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let mv = vec![
            Tok::Word(mid.to_string()),
            Tok::Word("TO".to_string()),
            Tok::Word(idx_name.to_string()),
        ];
        exec_move(&mv, fields, ctx.decimal_comma)?;
        if eval_cond(cond, fields, &ctx.switches, ctx.collation.as_ref())? {
            // match: the index is left at `mid`; run the WHEN imperative.
            let mut p = *bstart;
            return run_block(toks, &mut p, fields, out, true, ctx);
        }
        // key(mid) < value ? Under ascending, true => search the upper half; descending reverses.
        let key_less = eval_cond(&less, fields, &ctx.switches, ctx.collation.as_ref())?;
        if key_less == asc {
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    // not found: run AT END if present (the index value is unspecified per the standard).
    if let Some(s) = at_end {
        let mut p = s;
        if run_block(toks, &mut p, fields, out, true, ctx)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Turn a `SEARCH ALL` WHEN condition into its first-key `<` narrowing comparison: take the first
/// `AND`-segment and replace its equality (`=`/`EQUAL`) with `<`. `None` if the segment has no equality.
fn search_all_less(cond: &[Tok]) -> Option<Vec<Tok>> {
    let end = cond
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "AND"))
        .unwrap_or(cond.len());
    let mut seg: Vec<Tok> = cond[..end].to_vec();
    for t in seg.iter_mut() {
        if let Tok::Word(w) = t {
            if w == "=" || w == "EQUAL" || w == "EQUALS" {
                *w = "<".to_string();
                return Some(seg);
            }
        }
    }
    None
}

/// Whether the EVALUATE subject value matches a WHEN object: a single value (`subject = object`) or a
/// range (`object = lo THRU hi`, inclusive).
fn evaluate_value_match(
    subject: Option<&str>,
    object: &[Tok],
    fields: &HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    use std::cmp::Ordering;
    let subj = subject.ok_or_else(|| RunError::Unsupported("EVALUATE without a subject".into()))?;
    let col = ctx.collation.as_ref();
    let words: Vec<String> = object.iter().map(tok_to_cond_word).collect();
    if let Some(i) = words.iter().position(|w| w == "THRU" || w == "THROUGH") {
        let lo = words
            .first()
            .ok_or_else(|| RunError::Unsupported("THRU without lower bound".into()))?;
        let hi = words
            .get(i + 1)
            .ok_or_else(|| RunError::Unsupported("THRU without upper bound".into()))?;
        let ge = cond_compare(subj, lo, fields, col)? != Ordering::Less;
        let le = cond_compare(subj, hi, fields, col)? != Ordering::Greater;
        return Ok(ge && le);
    }
    let val = words
        .first()
        .ok_or_else(|| RunError::Unsupported("WHEN without a value".into()))?;
    Ok(cond_compare(subj, val, fields, col)? == Ordering::Equal)
}

/// `exec_perform` runs all wired forms: out-of-line `PERFORM para [THRU para2]` (with `n TIMES` / `UNTIL` /
/// `VARYING`), and the inline `PERFORM [n TIMES | UNTIL cond | VARYING ...] <stmts> END-PERFORM` -- including
/// the bare `PERFORM <stmts> END-PERFORM` that runs the body exactly once.
/// Parse `VARYING <id> FROM <x> BY <y> UNTIL <cond>` (the cursor is at `VARYING`). Returns the loop
/// variable, the FROM / BY operand tokens, and the UNTIL condition tokens. Nested `AFTER` varying is
/// not in the subset (fails closed).
type VaryingClause = (String, Tok, Tok, Vec<Tok>);

/// Parse the `VARYING id FROM x BY y UNTIL cond [AFTER id2 FROM ... UNTIL ...]...` chain (cursor at
/// `VARYING`). Returns one clause per VARYING/AFTER, outermost first; the last (innermost) varies fastest.
fn parse_varying_clauses(toks: &[Tok], pos: &mut usize) -> Result<Vec<VaryingClause>, RunError> {
    let word = |p: usize| matches!(toks.get(p), Some(Tok::Word(_)) | Some(Tok::Str(_)));
    let expect = |toks: &[Tok], pos: &mut usize, kw: &str| -> Result<(), RunError> {
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == kw) {
            *pos += 1;
            Ok(())
        } else {
            Err(RunError::Unsupported(format!(
                "PERFORM VARYING: expected {kw}"
            )))
        }
    };
    let mut clauses = Vec::new();
    loop {
        *pos += 1; // skip the VARYING / AFTER keyword
        let id = match toks.get(*pos) {
            Some(Tok::Word(w)) => w.clone(),
            _ => {
                return Err(RunError::Unsupported(
                    "PERFORM VARYING: missing loop variable".into(),
                ))
            }
        };
        *pos += 1;
        expect(toks, pos, "FROM")?;
        if !word(*pos) {
            return Err(RunError::Unsupported(
                "PERFORM VARYING: missing FROM value".into(),
            ));
        }
        let from = toks[*pos].clone();
        *pos += 1;
        expect(toks, pos, "BY")?;
        if !word(*pos) {
            return Err(RunError::Unsupported(
                "PERFORM VARYING: missing BY value".into(),
            ));
        }
        let by = toks[*pos].clone();
        *pos += 1;
        expect(toks, pos, "UNTIL")?;
        let mut cond = Vec::new();
        let mut after = false;
        while let Some(t) = toks.get(*pos) {
            match t {
                Tok::Dot => break,
                Tok::Word(w) if w == "AFTER" => {
                    after = true;
                    break;
                }
                Tok::Word(w)
                    if STMT_VERBS.contains(&w.as_str()) || SCOPE_ENDERS.contains(&w.as_str()) =>
                {
                    break
                }
                _ => {
                    cond.push(t.clone());
                    *pos += 1;
                }
            }
        }
        clauses.push((id, from, by, cond));
        if !after {
            return Ok(clauses);
        }
        // loop: parse the next AFTER clause (the `*pos += 1` at the top skips the AFTER keyword).
    }
}

/// Run a (possibly nested) `PERFORM VARYING ... AFTER ...`: at each level set the var to its FROM, test
/// UNTIL (TEST BEFORE), recurse into the next level (or run the body at the innermost), then step by BY.
/// Returns `true` if the body halted the program (STOP RUN).
/// How a PERFORM loop should react to a body that returned `Ok(true)` (a halt-like signal): an `EXIT
/// PERFORM` breaks the loop, `EXIT PERFORM CYCLE` skips to the next iteration, anything else (STOP RUN /
/// GO TO / GOBACK) is a real halt that propagates. The exit signals are consumed here.
enum PerfFlow {
    Break,
    Continue,
    Halt,
}
fn perform_flow(ctx: &Ctx) -> PerfFlow {
    if ctx.exit_perform.get() {
        ctx.exit_perform.set(false);
        PerfFlow::Break
    } else if ctx.exit_cycle.get() {
        ctx.exit_cycle.set(false);
        PerfFlow::Continue
    } else {
        PerfFlow::Halt
    }
}

fn run_varying_nested(
    clauses: &[VaryingClause],
    level: usize,
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
    test_after: bool,
    run_body: &mut dyn FnMut(&mut HashMap<String, Field>) -> Result<bool, RunError>,
) -> Result<bool, RunError> {
    let (id, from, by, cond) = &clauses[level];
    varying_set(id, from, fields)?;
    let mut guard = 0u32;
    loop {
        // WITH TEST BEFORE (default): test UNTIL before the body. WITH TEST AFTER: run the body first,
        // then test (so the loop variable's final value is also processed once).
        if !test_after && eval_cond(cond, fields, &ctx.switches, ctx.collation.as_ref())? {
            break;
        }
        let halted = if level + 1 < clauses.len() {
            run_varying_nested(clauses, level + 1, fields, ctx, test_after, run_body)?
        } else {
            run_body(fields)?
        };
        if halted {
            // EXIT PERFORM CYCLE at THIS (innermost) level: skip to this loop's next iteration. EXIT PERFORM
            // and real halts propagate (the top-level exec_perform absorbs EXIT PERFORM).
            if level + 1 == clauses.len() && ctx.exit_cycle.get() {
                ctx.exit_cycle.set(false);
            } else {
                return Ok(true);
            }
        }
        if test_after && eval_cond(cond, fields, &ctx.switches, ctx.collation.as_ref())? {
            break;
        }
        varying_step(id, by, fields)?;
        guard += 1;
        if guard > 1_000_000 {
            return Err(RunError::Runtime(
                "PERFORM VARYING exceeded 1e6 iterations".into(),
            ));
        }
    }
    Ok(false)
}

/// Consume an optional `[WITH] TEST {BEFORE|AFTER}` phrase (after `PERFORM [proc]`, before
/// `VARYING`/`UNTIL`). Returns `true` for TEST AFTER, `false` otherwise (TEST BEFORE is the default).
fn parse_with_test(toks: &[Tok], pos: &mut usize) -> bool {
    let mut p = *pos;
    if matches!(toks.get(p), Some(Tok::Word(w)) if w == "WITH") {
        p += 1;
    }
    if matches!(toks.get(p), Some(Tok::Word(w)) if w == "TEST") {
        p += 1;
        if matches!(toks.get(p), Some(Tok::Word(w)) if w == "AFTER") {
            *pos = p + 1;
            return true;
        }
        if matches!(toks.get(p), Some(Tok::Word(w)) if w == "BEFORE") {
            *pos = p + 1;
            return false;
        }
    }
    false // no WITH TEST phrase (cursor unmoved)
}

/// Set the VARYING loop variable to its FROM value.
fn varying_set(name: &str, src: &Tok, fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
    let (b, a) = operand_value(src, fields)?;
    let f = fields
        .get_mut(name)
        .ok_or_else(|| RunError::UndefinedName(name.to_string()))?;
    move_into(f, &b, &a, false)
}

/// Step the VARYING loop variable by its BY increment (`id = id + by`).
fn varying_step(name: &str, by: &Tok, fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
    let (idb, ida) = operand_value(&Tok::Word(name.to_string()), fields)?;
    let (byb, bya) = operand_value(by, fields)?;
    let (rb, ra) = wide_op(Op::Add, &idb, &ida, &byb, &bya)?;
    let f = fields
        .get_mut(name)
        .ok_or_else(|| RunError::UndefinedName(name.to_string()))?;
    store_arith_result(f, &rb, &ra, false, false)?;
    Ok(())
}

fn exec_perform(
    toks: &[Tok],
    pos: &mut usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    exec: bool,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    // The performed range always runs against the CURRENT program body's tokens (CUR_PROC), never the
    // `toks` we were handed: a PERFORM inside an imperative handler (SIZE ERROR / OVERFLOW / AT END /
    // INVALID KEY) is dispatched from a copied token BLOCK, whose indexes do not match the body --
    // para_range resolves body indexes, so running them against the copy mis-indexed (an unbounded
    // loop in the CCVS85 `ON SIZE ERROR PERFORM PASS GO TO ...` idiom). In the normal flow `toks` IS
    // the body, so this is identical there.
    let body = CUR_PROC.with(|c| c.borrow().clone());
    // out-of-line form: PERFORM para [THRU para2] [ n TIMES | UNTIL cond ] -- run a named paragraph range.
    if let Some(Tok::Word(w)) = toks.get(*pos) {
        if para_exists(w) {
            let p1 = w.clone();
            *pos += 1;
            let mut p2 = p1.clone();
            if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "THRU" || w == "THROUGH") {
                *pos += 1;
                if let Some(Tok::Word(w)) = toks.get(*pos) {
                    p2 = w.clone();
                    *pos += 1;
                }
            }
            // optional `WITH TEST {BEFORE|AFTER}` (applies to the UNTIL/VARYING condition placement).
            let test_after = parse_with_test(toks, pos);
            // PERFORM para [THRU para2] [WITH TEST x] VARYING id FROM x BY y UNTIL cond [AFTER ...].
            if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "VARYING") {
                let clauses = parse_varying_clauses(toks, pos)?;
                if !exec {
                    return Ok(false);
                }
                let (start, end) = para_range(&p1, &p2).ok_or_else(|| {
                    RunError::Unsupported(format!("PERFORM: unknown paragraph `{p1}`/`{p2}`"))
                })?;
                let mut body = |fields: &mut HashMap<String, Field>| {
                    run_range_perform(&body, start, end, fields, out, ctx)
                };
                if run_varying_nested(&clauses, 0, fields, ctx, test_after, &mut body)? {
                    if let PerfFlow::Halt = perform_flow(ctx) {
                        return Ok(true);
                    } // EXIT PERFORM absorbed
                }
                return Ok(false);
            }
            let mut times: Option<String> = None;
            let mut ucond: Vec<Tok> = Vec::new();
            let mut until = false;
            if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "UNTIL") {
                until = true;
                *pos += 1;
                while let Some(t) = toks.get(*pos) {
                    match t {
                        Tok::Dot => break,
                        Tok::Word(w)
                            if STMT_VERBS.contains(&w.as_str())
                                || SCOPE_ENDERS.contains(&w.as_str()) =>
                        {
                            break
                        }
                        _ => {
                            ucond.push(t.clone());
                            *pos += 1;
                        }
                    }
                }
            } else if matches!(toks.get(*pos), Some(Tok::Word(_)))
                && matches!(toks.get(*pos + 1), Some(Tok::Word(t)) if t == "TIMES")
            {
                if let Some(Tok::Word(w)) = toks.get(*pos) {
                    times = Some(w.clone());
                }
                *pos += 2;
            }
            if !exec {
                return Ok(false);
            }
            let (start, end) = para_range(&p1, &p2).ok_or_else(|| {
                RunError::Unsupported(format!("PERFORM: unknown paragraph `{p1}`/`{p2}`"))
            })?;
            if until {
                let mut guard = 0u32;
                loop {
                    if !test_after
                        && eval_cond(&ucond, fields, &ctx.switches, ctx.collation.as_ref())?
                    {
                        break;
                    }
                    if run_range_perform(&body, start, end, fields, out, ctx)? {
                        match perform_flow(ctx) {
                            PerfFlow::Break => break,
                            PerfFlow::Continue => {}
                            PerfFlow::Halt => return Ok(true),
                        }
                    }
                    if test_after
                        && eval_cond(&ucond, fields, &ctx.switches, ctx.collation.as_ref())?
                    {
                        break;
                    }
                    guard += 1;
                    if guard > 1_000_000 {
                        return Err(RunError::Runtime(
                            "PERFORM UNTIL exceeded 1e6 iterations".into(),
                        ));
                    }
                }
            } else {
                let n = times
                    .as_deref()
                    .and_then(|w| resolve_int(w, fields))
                    .unwrap_or(1);
                for _ in 0..n.max(0) {
                    if run_range_perform(&body, start, end, fields, out, ctx)? {
                        match perform_flow(ctx) {
                            PerfFlow::Break => break,
                            PerfFlow::Continue => continue,
                            PerfFlow::Halt => return Ok(true),
                        }
                    }
                }
            }
            return Ok(false);
        }
    }
    // inline: PERFORM VARYING id FROM x BY y UNTIL cond [AFTER ...] ... END-PERFORM (TEST BEFORE).
    // optional `WITH TEST {BEFORE|AFTER}` before the inline VARYING/UNTIL form.
    let test_after = parse_with_test(toks, pos);
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "VARYING") {
        let clauses = parse_varying_clauses(toks, pos)?;
        let body_start = *pos;
        let mut scan = *pos;
        let _ = run_block(toks, &mut scan, fields, out, false, ctx)?;
        let body_end = scan;
        if exec {
            let mut body = |fields: &mut HashMap<String, Field>| {
                let mut p = body_start;
                run_block(toks, &mut p, fields, out, true, ctx)
            };
            if run_varying_nested(&clauses, 0, fields, ctx, test_after, &mut body)? {
                if let PerfFlow::Halt = perform_flow(ctx) {
                    return Ok(true);
                } // EXIT PERFORM absorbed
            }
        }
        *pos = body_end;
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-PERFORM") {
            *pos += 1;
        }
        return Ok(false);
    }
    // forms: PERFORM <n> TIMES ... END-PERFORM ; PERFORM UNTIL <cond> ... END-PERFORM
    let is_until = matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "UNTIL");
    let mut times_word: Option<String> = None;
    let mut cond = Vec::new();
    if is_until {
        *pos += 1;
        while let Some(t) = toks.get(*pos) {
            match t {
                Tok::Dot => break,
                Tok::Word(w)
                    if STMT_VERBS.contains(&w.as_str()) || SCOPE_ENDERS.contains(&w.as_str()) =>
                {
                    break
                }
                _ => {
                    cond.push(t.clone());
                    *pos += 1;
                }
            }
        }
    } else if matches!(toks.get(*pos), Some(Tok::Word(_)))
        && matches!(toks.get(*pos + 1), Some(Tok::Word(w)) if w == "TIMES")
    {
        // PERFORM <n> TIMES ... END-PERFORM
        if let Some(Tok::Word(w)) = toks.get(*pos) {
            times_word = Some(w.clone());
        }
        *pos += 2; // skip the count and TIMES
    } else {
        // Bare inline `PERFORM <body> END-PERFORM` -- run the body exactly once (times_word stays None,
        // which the executor reads as a count of 1). The cursor is already at the body's first token.
    }

    // record the body's start; we re-run it per iteration.
    let body_start = *pos;
    let body_end;
    // first pass: if not executing, just skip the body once to find END-PERFORM.
    {
        let mut scan = *pos;
        let _ = run_block(toks, &mut scan, fields, out, false, ctx)?;
        body_end = scan;
    }

    if exec {
        if is_until {
            // PERFORM UNTIL: TEST BEFORE (default) tests before each iteration; TEST AFTER runs the body
            // first, then tests (so the body always runs at least once).
            let mut guard = 0u32;
            loop {
                if !test_after && eval_cond(&cond, fields, &ctx.switches, ctx.collation.as_ref())? {
                    break;
                }
                let mut p = body_start;
                if run_block(toks, &mut p, fields, out, true, ctx)? {
                    match perform_flow(ctx) {
                        PerfFlow::Break => break,
                        PerfFlow::Continue => {}
                        PerfFlow::Halt => return Ok(true),
                    }
                }
                if test_after && eval_cond(&cond, fields, &ctx.switches, ctx.collation.as_ref())? {
                    break;
                }
                guard += 1;
                if guard > 1_000_000 {
                    return Err(RunError::Runtime(
                        "PERFORM UNTIL exceeded 1e6 iterations".into(),
                    ));
                }
            }
        } else {
            // No times_word -> the bare inline form runs the body once.
            let n = match &times_word {
                Some(w) => resolve_int(w, fields).ok_or_else(|| {
                    RunError::Unsupported("PERFORM TIMES count not an integer".into())
                })?,
                None => 1,
            };
            for _ in 0..n {
                let mut p = body_start;
                if run_block(toks, &mut p, fields, out, true, ctx)? {
                    match perform_flow(ctx) {
                        PerfFlow::Break => break,
                        PerfFlow::Continue => continue,
                        PerfFlow::Halt => return Ok(true),
                    }
                }
            }
        }
    }

    // advance past the body + END-PERFORM.
    *pos = body_end;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-PERFORM") {
        *pos += 1;
    }
    Ok(false)
}

/// Resolve a token to an integer count (a numeric literal, or a numeric field's integer value).
fn resolve_int(w: &str, fields: &HashMap<String, Field>) -> Option<i64> {
    if let Some(f) = fields.get(w) {
        if let Storage::Numeric(a) = &f.storage {
            // decode_numeric_source (NOT the raw zoned source_to_decimal) so a COMP/COMP-5 (binary) or
            // COMP-3 (packed) counter -- e.g. an OCCURS DEPENDING ON counter, subscript, or TIMES count --
            // is decoded by value, not misread nibble-by-nibble as zoned display.
            let dec = decode_numeric_source(&f.bytes, a).ok()?;
            let mut v: i64 = 0;
            for d in &dec.digits {
                v = v.checked_mul(10)?.checked_add(*d as i64)?;
            }
            // ignore fractional digits for a TIMES count (integer part only).
            for _ in 0..dec.scale.max(0) {
                v /= 10;
            }
            return Some(if dec.negative { -v } else { v });
        }
        None
    } else {
        w.parse::<i64>().ok()
    }
}

/// Evaluate a condition to a boolean. Grammar: `or := and (OR and)*`, `and := rel (AND rel)*`,
/// `rel := operand [IS] [NOT] (= | > | < | >= | <= | <> | GREATER [THAN] | LESS [THAN] | EQUAL [TO])
/// operand`. Numeric operands compare by value; alphanumeric by space-padded bytes. The combined word
/// forms ("GREATER THAN OR EQUAL TO") and class/sign/88-level conditions are not in the subset.
fn eval_cond(
    t: &[Tok],
    fields: &mut HashMap<String, Field>,
    sw: &SwitchEnv,
    col: Option<&[u8; 256]>,
) -> Result<bool, RunError> {
    // resolve any FUNCTION reference into a temp field, then evaluate through the ordinary operand paths.
    let t = resolve_functions(t, fields)?;
    let t = &t[..];
    let fields = &*fields;
    let words0: Vec<String> = t
        .iter()
        .map(|tok| match tok {
            Tok::Word(w) => w.clone(),
            Tok::Str(s) => format!("\u{1}{}", String::from_utf8_lossy(s)), // mark string literal
            Tok::AllLiteral(s) => format!("\u{1}{}", String::from_utf8_lossy(s)),
            Tok::Dot => ".".into(),
        })
        .collect();
    // Collapse the figurative repeat `ALL <literal>` into a single operand token `\u{2}<unit-bytes>` -- it
    // compares as the unit cycled to the OTHER operand's width (`IF d = ALL "9"`). `ALL <figurative>` (ALL
    // SPACES) is just the figurative. A lone ALL is left as-is.
    let mut words: Vec<String> = Vec::with_capacity(words0.len());
    let mut wi = 0;
    while wi < words0.len() {
        if words0[wi] == "ALL" && wi + 1 < words0.len() {
            let nxt = &words0[wi + 1];
            if figurative_kind(nxt).is_some() {
                words.push(nxt.clone()); // ALL SPACES == SPACES
            } else if let Some(s) = nxt.strip_prefix('\u{1}') {
                words.push(format!("\u{5}{s}")); // ALL "lit" -> repeat-unit marker (\u{5}; \u{2} is the FN temp)
            } else {
                words.push(format!("\u{5}{nxt}")); // ALL <other literal token>
            }
            wi += 2;
            continue;
        }
        words.push(words0[wi].clone());
        wi += 1;
    }
    let mut p = 0;
    // `ctx` carries the last-stated (subject, operator, negation) for abbreviated combined conditions
    // (`A = 1 OR 2`, `A > B AND < C`); it threads left-to-right through the whole condition.
    let mut ctx: Option<(String, Rel, bool)> = None;
    let r = cond_or(&words, &mut p, fields, sw, col, &mut ctx)?;
    if p != words.len() {
        return Err(RunError::Unsupported(format!(
            "trailing tokens in condition at {}",
            words[p]
        )));
    }
    Ok(r)
}

type CondCtx = Option<(String, Rel, bool)>;

fn cond_or(
    w: &[String],
    p: &mut usize,
    f: &HashMap<String, Field>,
    sw: &SwitchEnv,
    col: Option<&[u8; 256]>,
    ctx: &mut CondCtx,
) -> Result<bool, RunError> {
    let mut acc = cond_and(w, p, f, sw, col, ctx)?;
    while w.get(*p).map(|s| s.as_str()) == Some("OR") {
        *p += 1;
        let r = cond_and(w, p, f, sw, col, ctx)?;
        acc = acc || r;
    }
    Ok(acc)
}

fn cond_and(
    w: &[String],
    p: &mut usize,
    f: &HashMap<String, Field>,
    sw: &SwitchEnv,
    col: Option<&[u8; 256]>,
    ctx: &mut CondCtx,
) -> Result<bool, RunError> {
    let mut acc = cond_rel(w, p, f, sw, col, ctx)?;
    while w.get(*p).map(|s| s.as_str()) == Some("AND") {
        *p += 1;
        let r = cond_rel(w, p, f, sw, col, ctx)?;
        acc = acc && r;
    }
    Ok(acc)
}

/// Consume a relational operator at `w[*p]` (`=` `>` `<` `>=` `<=` `<>`, or the worded `GREATER [THAN]` /
/// `LESS [THAN]` / `EQUAL [TO]`), advancing `*p` past it. Returns `None` (without advancing) if `w[*p]` is
/// not a relational operator.
fn parse_relop(w: &[String], p: &mut usize) -> Option<Rel> {
    let r = match w.get(*p).map(|s| s.as_str())? {
        "=" => Rel::Eq,
        ">" => Rel::Gt,
        "<" => Rel::Lt,
        ">=" => Rel::Ge,
        "<=" => Rel::Le,
        "<>" => Rel::Ne,
        "GREATER" => {
            *p += 1;
            if w.get(*p).map(|s| s.as_str()) == Some("THAN") {
                *p += 1;
            }
            return Some(Rel::Gt);
        }
        "LESS" => {
            *p += 1;
            if w.get(*p).map(|s| s.as_str()) == Some("THAN") {
                *p += 1;
            }
            return Some(Rel::Lt);
        }
        "EQUAL" => {
            *p += 1;
            if w.get(*p).map(|s| s.as_str()) == Some("TO") {
                *p += 1;
            }
            return Some(Rel::Eq);
        }
        _ => return None,
    };
    *p += 1;
    Some(r)
}

/// `IS NUMERIC` for a packed (COMP-3) field: every digit nibble is 0-9 and the trailing sign nibble is a
/// valid sign (C/D positive/negative, F unsigned). A field built by the runtime is always valid; this guards
/// raw/REDEFINES'd bytes.
fn packed_is_numeric(bytes: &[u8]) -> bool {
    let Some((last, rest)) = bytes.split_last() else {
        return false;
    };
    if rest.iter().any(|b| (b >> 4) > 9 || (b & 0x0f) > 9) {
        return false;
    }
    (last >> 4) <= 9 && matches!(last & 0x0f, 0x0c | 0x0d | 0x0f)
}

fn cond_rel(
    w: &[String],
    p: &mut usize,
    f: &HashMap<String, Field>,
    sw: &SwitchEnv,
    col: Option<&[u8; 256]>,
    ctx: &mut CondCtx,
) -> Result<bool, RunError> {
    // A leading NOT negates the whole relation term (`IF NOT A = 5`; abbreviated `... AND NOT 2`).
    let mut neg = false;
    if w.get(*p).map(|s| s.as_str()) == Some("NOT") {
        neg = true;
        *p += 1;
    }
    // Operator-first abbreviation `[NOT] op object` -- reuse the last subject (`A > B AND < C`).
    if ctx.is_some() {
        let mut q = *p;
        if let Some(op) = parse_relop(w, &mut q) {
            let subject = ctx.as_ref().unwrap().0.clone();
            let object = w
                .get(q)
                .ok_or_else(|| RunError::Unsupported("condition: missing right operand".into()))?
                .clone();
            *p = q + 1;
            *ctx = Some((subject.clone(), op, neg));
            return Ok(rel_holds(op, cond_compare(&subject, &object, f, col)?, neg));
        }
    }
    let left = w
        .get(*p)
        .ok_or_else(|| RunError::Unsupported("condition: missing left operand".into()))?
        .clone();
    *p += 1;
    // A UPSI switch condition-name (SPECIAL-NAMES `SWITCH-n ON/OFF STATUS IS <name>`): its truth is the
    // switch's state matching the declared ON/OFF sense. No relational operator follows.
    if let Some(&(idx, on)) = sw.conds.get(&left) {
        return Ok(neg ^ (sw.states.borrow()[idx] == on));
    }
    // An 88-level condition-name: true when its parent's value equals any listed value or range.
    if let Some(Field {
        storage: Storage::Condition { parent, values, .. },
        ..
    }) = f.get(&left)
    {
        let mut hit = false;
        for v in values {
            hit = match v {
                CondVal::Single(val) => {
                    cond_compare(parent, val, f, col)? == std::cmp::Ordering::Equal
                }
                CondVal::Range(lo, hi) => {
                    cond_compare(parent, lo, f, col)? != std::cmp::Ordering::Less
                        && cond_compare(parent, hi, f, col)? != std::cmp::Ordering::Greater
                }
            };
            if hit {
                break;
            }
        }
        return Ok(neg ^ hit);
    }
    if w.get(*p).map(|s| s.as_str()) == Some("IS") {
        *p += 1;
    }
    if w.get(*p).map(|s| s.as_str()) == Some("NOT") {
        neg = !neg;
        *p += 1;
    }
    // Sign condition: `IF identifier [IS] [NOT] {POSITIVE | NEGATIVE | ZERO}` -- a unary test of the
    // operand's numeric value against 0 (no right operand follows).
    if let Some(signw) = w.get(*p).map(|s| s.as_str()) {
        if matches!(signw, "POSITIVE" | "NEGATIVE" | "ZERO") {
            *p += 1;
            let ord = cond_compare(&left, "0", f, col)?;
            let base = match signw {
                "POSITIVE" => ord == std::cmp::Ordering::Greater,
                "NEGATIVE" => ord == std::cmp::Ordering::Less,
                _ => ord == std::cmp::Ordering::Equal, // ZERO
            };
            return Ok(if neg { !base } else { base });
        }
    }
    // Class condition: `IF identifier [IS] [NOT] {NUMERIC | ALPHABETIC | ALPHABETIC-UPPER | ALPHABETIC-LOWER}`
    // -- a byte predicate over the operand's raw storage (the sealed `class` module). The NUMERIC variant is
    // chosen by the operand's usage: binary is always numeric; packed validates its nibbles; a signed DISPLAY
    // uses the trailing-overpunch rule; everything else is the plain digit-string test.
    if let Some(classw) = w.get(*p).map(|s| s.as_str()) {
        if matches!(
            classw,
            "NUMERIC" | "ALPHABETIC" | "ALPHABETIC-UPPER" | "ALPHABETIC-LOWER"
        ) {
            *p += 1;
            let field =
                read_field(f, &left)?.ok_or_else(|| RunError::UndefinedName(left.clone()))?;
            let bytes = &field.bytes;
            let base = match classw {
                "ALPHABETIC" => crate::class::is_alphabetic(bytes),
                "ALPHABETIC-UPPER" => crate::class::is_alphabetic_upper(bytes),
                "ALPHABETIC-LOWER" => crate::class::is_alphabetic_lower(bytes),
                _ => match &field.storage {
                    Storage::Numeric(a) => match a.field_type {
                        crate::attr::COB_TYPE_NUMERIC_BINARY => true,
                        crate::attr::COB_TYPE_NUMERIC_PACKED => packed_is_numeric(bytes),
                        _ if a.flags & crate::attr::COB_FLAG_HAVE_SIGN != 0 => {
                            // a signed DISPLAY field -- pick the predicate for its sign convention.
                            match (a.sign_separate(), a.sign_leading()) {
                                (true, true) => {
                                    crate::class::is_numeric_sign_leading_separate(bytes)
                                }
                                (true, false) => {
                                    crate::class::is_numeric_sign_trailing_separate(bytes)
                                }
                                (false, true) => crate::class::is_numeric_sign_leading(bytes),
                                (false, false) => crate::class::is_numeric_signed_trailing(bytes),
                            }
                        }
                        _ => crate::class::is_numeric(bytes),
                    },
                    _ => crate::class::is_numeric(bytes), // alphanumeric / group / edited: digit-string test
                },
            };
            return Ok(if neg { !base } else { base });
        }
    }
    // Full relation `subject [IS] [NOT] op object`.
    if let Some(op) = parse_relop(w, p) {
        let right = w
            .get(*p)
            .ok_or_else(|| RunError::Unsupported("condition: missing right operand".into()))?
            .clone();
        *p += 1;
        *ctx = Some((left.clone(), op, neg));
        return Ok(rel_holds(op, cond_compare(&left, &right, f, col)?, neg));
    }
    // Bare-object abbreviation `object` -- reuse the last subject AND operator (`A = 1 OR 2`); a local NOT
    // toggles the reused negation (`A NOT = 1 AND 2` -> both negated).
    if let Some((subject, op, prev_neg)) = ctx.clone() {
        return Ok(rel_holds(
            op,
            cond_compare(&subject, &left, f, col)?,
            prev_neg ^ neg,
        ));
    }
    Err(RunError::Unsupported(
        "condition: unrecognized relational operator (expected = > < >= <= <> GREATER LESS EQUAL)"
            .into(),
    ))
}

/// Apply a relational operator to a comparison ordering, then the negation flag.
fn rel_holds(op: Rel, ord: std::cmp::Ordering, neg: bool) -> bool {
    use std::cmp::Ordering::{Equal, Greater, Less};
    let base = match op {
        Rel::Eq => ord == Equal,
        Rel::Ne => ord != Equal,
        Rel::Gt => ord == Greater,
        Rel::Lt => ord == Less,
        Rel::Ge => ord != Less,
        Rel::Le => ord != Greater,
    };
    if neg {
        !base
    } else {
        base
    }
}

#[derive(Clone, Copy)]
enum Rel {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

/// Compare two condition operands (each a word: a field name, a numeric literal, or a `\u{1}`-marked
/// string literal). If BOTH resolve to numeric values, compare by value; otherwise compare the display
/// bytes space-padded to equal length (the COBOL alphanumeric collation).
fn cond_compare(
    a: &str,
    b: &str,
    f: &HashMap<String, Field>,
    col: Option<&[u8; 256]>,
) -> Result<std::cmp::Ordering, RunError> {
    let na = cond_numeric(a, f);
    let nb = cond_numeric(b, f);
    if let (Some(da), Some(db)) = (&na, &nb) {
        return Ok(dec_cmp(da, db));
    }
    // alphanumeric compare: space-pad the shorter, byte compare. Under PROGRAM COLLATING SEQUENCE the
    // bytes are weighted through `col` first (e.g. EBCDIC order: lowercase < uppercase < digits). A
    // figurative operand (SPACES/HIGH-VALUE/...) fills the OTHER operand's width with its byte; an
    // `ALL <literal>` repeat operand (\u{2}unit) fills it by cycling the unit bytes.
    let (fa, fb) = (figurative_kind(a), figurative_kind(b));
    let (ra, rb) = (a.strip_prefix('\u{5}'), b.strip_prefix('\u{5}'));
    let ba = if fa.is_some() || ra.is_some() {
        Vec::new()
    } else {
        cond_bytes(a, f)
    };
    let bb = if fb.is_some() || rb.is_some() {
        Vec::new()
    } else {
        cond_bytes(b, f)
    };
    let width = ba.len().max(bb.len()).max(1);
    let repeat = |u: &str, w: usize| -> Vec<u8> {
        let u = u.as_bytes();
        if u.is_empty() {
            vec![b' '; w]
        } else {
            (0..w).map(|i| u[i % u.len()]).collect()
        }
    };
    let sa = match (fa, ra) {
        (Some(fig), _) => vec![fig_byte(fig); width],
        (_, Some(u)) => repeat(u, width),
        _ => ba,
    };
    let sb = match (fb, rb) {
        (Some(fig), _) => vec![fig_byte(fig); width],
        (_, Some(u)) => repeat(u, width),
        _ => bb,
    };
    let n = sa.len().max(sb.len());
    for i in 0..n {
        let ca = sa.get(i).copied().unwrap_or(b' ');
        let cb = sb.get(i).copied().unwrap_or(b' ');
        let (wa, wb) = match col {
            Some(t) => (t[ca as usize], t[cb as usize]),
            None => (ca, cb),
        };
        if wa != wb {
            return Ok(wa.cmp(&wb));
        }
    }
    Ok(std::cmp::Ordering::Equal)
}

/// If a condition operand is numeric (a numeric field or a numeric literal), decode it to a [`Decimal`].
/// Decode a numeric field to a [`Decimal`] respecting its USAGE: a zoned DISPLAY field decodes directly,
/// while a packed (COMP-3) / binary (COMP/COMP-5/COMP-X) field is normalised to zoned DISPLAY via
/// `cob_move` first (so a comparison like `IF comp-3-field = 5` is correct, not a raw-byte mis-read).
fn field_to_decimal(field: &Field) -> Option<Decimal> {
    let Storage::Numeric(a) = &field.storage else {
        return None;
    };
    if a.field_type == COB_TYPE_NUMERIC_DISPLAY {
        return source_to_decimal(&field.bytes, a).ok();
    }
    let digits = a.digits.max(1);
    let signed = a.flags & crate::attr::COB_FLAG_HAVE_SIGN != 0;
    let disp = lit_num_attr(digits, a.scale, signed);
    let mut buf = vec![b'0'; digits as usize];
    cob_move(&field.bytes, a, &mut buf, &disp).ok()?;
    source_to_decimal(&buf, &disp).ok()
}

fn cond_numeric(w: &str, f: &HashMap<String, Field>) -> Option<Decimal> {
    if let Some(field) = read_field(f, w).ok().flatten() {
        return field_to_decimal(&field);
    }
    if w.starts_with('\u{1}') || w.starts_with('\u{5}') {
        return None; // string literal / ALL-repeat -> alphanumeric
    }
    // figurative ZERO is the numeric value 0 (so `IF n = ZERO` compares numerically for any numeric usage).
    if matches!(w, "ZERO" | "ZEROS" | "ZEROES") {
        return Some(Decimal {
            negative: false,
            digits: vec![0],
            scale: 0,
        });
    }
    parse_num_literal(w).ok()
}

/// The fill byte for a figurative constant (used when it fills another operand's width).
fn fig_byte(fig: Fig) -> u8 {
    match fig {
        Fig::Space => b' ',
        Fig::Zero => b'0',
        Fig::HighValue => 0xFF,
        Fig::LowValue => 0x00,
        Fig::Quote => b'"',
    }
}

/// The display bytes of a condition operand for alphanumeric comparison.
fn cond_bytes(w: &str, f: &HashMap<String, Field>) -> Vec<u8> {
    if let Some(field) = read_field(f, w).ok().flatten() {
        return field.bytes.clone();
    }
    if let Some(rest) = w.strip_prefix('\u{1}') {
        return rest.as_bytes().to_vec();
    }
    w.as_bytes().to_vec()
}

/// Compare two decimals by value (scale-aligned).
fn dec_cmp(a: &Decimal, b: &Decimal) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let za = a.digits.iter().all(|&d| d == 0);
    let zb = b.digits.iter().all(|&d| d == 0);
    let na = a.negative && !za;
    let nb = b.negative && !zb;
    if na != nb {
        return if na {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    // align scales, compare integer magnitudes as digit strings.
    let scale = a.scale.max(b.scale).max(0);
    let ma = scaled_digits(a, scale);
    let mb = scaled_digits(b, scale);
    // magnitude order: more significant digits (after leading-zero strip) is larger, then lexicographic.
    let mag = ma.len().cmp(&mb.len()).then_with(|| ma.cmp(&mb));
    if na {
        mag.reverse()
    } else {
        mag
    }
}

/// The magnitude digit string of a decimal scaled to `scale` fractional digits (leading zeros kept for
/// length-aligned comparison).
fn scaled_digits(d: &Decimal, scale: i16) -> Vec<u8> {
    let mut digits = d.digits.clone();
    let extra = (scale - d.scale.max(0)).max(0);
    for _ in 0..extra {
        digits.push(0);
    }
    // strip leading zeros but keep at least one digit; then left-pad both to equal length at call site.
    while digits.len() > 1 && digits[0] == 0 {
        digits.remove(0);
    }
    digits
}

/// Build a [`Field`] from its PIC + optional VALUE literal. Edited pictures (which [`build_field`]
/// rejects as unsupported symbols) are stored as their edited image.
fn make_field(
    pic: &str,
    value: Option<&Tok>,
    currency: u8,
    decimal_comma: bool,
    dialect: crate::dialect::Dialect,
    usage: Usage,
    sign: (bool, bool),
    extra_flags: u16,
) -> Result<Field, RunError> {
    match build_field(pic, usage, sign.0, sign.1) {
        Ok(pf) => {
            let is_alpha = !pf.attr.is_numeric();
            // Uninitialized storage (no VALUE) is filled per the dialect's `defaultbyte`: the category
            // default ('0'/space) under the default dialect, a single byte (0x00 ibm/mvs, space mf) else.
            // A VALUE clause always overrides the fill.
            let fill = dialect.defaultbyte.byte(is_alpha);
            let bytes = vec![fill; pf.size];
            // JUSTIFIED / BLANK WHEN ZERO ride in the attr flags (COB_FLAG_JUSTIFIED / _BLANK_ZERO).
            let mut attr = pf.attr;
            attr.flags |= extra_flags;
            let storage = if is_alpha {
                Storage::Alpha(attr)
            } else {
                Storage::Numeric(attr)
            };
            let mut field = Field {
                storage,
                bytes,
                occurs: 1,
                redefines: None,
            };
            // Uninitialized binary/packed/float zero is NOT '0' chars (0x30...) -- that decodes as garbage.
            // Encode a proper zero for those usages (cobc default-inits COMP/COMP-3/COMP-1/2 to numeric 0).
            let non_display_numeric = !is_alpha && attr.field_type != COB_TYPE_NUMERIC_DISPLAY;
            if value.is_none() && non_display_numeric {
                let zd = lit_num_attr(attr.digits.max(1), attr.scale.max(0), attr.have_sign());
                let zsrc = vec![b'0'; zd.digits as usize];
                let mut dst = field.bytes.clone();
                if crate::move_ops::cob_move(&zsrc, &zd, &mut dst, &attr).is_ok() {
                    field.bytes = dst;
                }
            }
            if let Some(v) = value {
                init_value(&mut field, v)?;
            }
            Ok(field)
        }
        Err(crate::pic::PicError::UnsupportedSymbol(_))
        | Err(crate::pic::PicError::MixedCategory) => {
            // treat as numeric-edited: storage is the edited image, sized by edited_size. A non-'$'
            // CURRENCY SIGN is normalized to '$' for the size computation (the width is the same; the
            // '.'/',' role swap of DECIMAL-POINT IS COMMA is width-invariant too).
            let cur = (currency as char).to_ascii_uppercase();
            let pic_norm: String = if cur == '$' {
                pic.to_string()
            } else {
                pic.chars()
                    .map(|c| {
                        if c.to_ascii_uppercase() == cur {
                            '$'
                        } else {
                            c
                        }
                    })
                    .collect()
            };
            let size = edited_size(&pic_norm)
                .map_err(|e| RunError::Unsupported(format!("PIC {pic}: {e:?}")))?;
            let blank_zero = extra_flags & crate::attr::COB_FLAG_BLANK_ZERO != 0;
            let mut field = Field {
                storage: Storage::Edited(pic.to_string(), currency, decimal_comma, blank_zero),
                bytes: vec![b' '; size],
                occurs: 1,
                redefines: None,
            };
            if let Some(v) = value {
                init_value(&mut field, v)?;
            }
            Ok(field)
        }
        Err(e) => Err(RunError::Unsupported(format!("PIC {pic}: {e:?}"))),
    }
}

/// Initialize a field from a VALUE literal (a numeric literal word, or a string).
/// A figurative constant -- a value that fills its receiver to the receiver's full width.
#[derive(Clone, Copy)]
enum Fig {
    Space,
    Zero,
    HighValue,
    LowValue,
    Quote,
}

/// Recognise a figurative-constant word (singular + plural spellings). Figuratives are reserved words,
/// so they never collide with a data name.
fn figurative_kind(w: &str) -> Option<Fig> {
    match w {
        "SPACE" | "SPACES" => Some(Fig::Space),
        "ZERO" | "ZEROS" | "ZEROES" => Some(Fig::Zero),
        "HIGH-VALUE" | "HIGH-VALUES" => Some(Fig::HighValue),
        "LOW-VALUE" | "LOW-VALUES" => Some(Fig::LowValue),
        "QUOTE" | "QUOTES" => Some(Fig::Quote),
        _ => None,
    }
}

/// Fill `f` with a figurative constant across its full width: SPACE / HIGH-VALUE / LOW-VALUE / QUOTE are
/// raw byte fills (0x20 / 0xFF / 0x00 / 0x22); ZERO is numeric 0 into a numeric/edited receiver, else a
/// `'0'` fill (e.g. PIC X).
fn fill_figurative(f: &mut Field, fig: Fig, decimal_comma: bool) -> Result<(), RunError> {
    let n = f.bytes.len();
    match fig {
        Fig::Space => f.bytes = vec![b' '; n],
        Fig::HighValue => f.bytes = vec![0xFFu8; n],
        Fig::LowValue => f.bytes = vec![0x00u8; n],
        Fig::Quote => f.bytes = vec![b'"'; n],
        Fig::Zero => match &f.storage {
            Storage::Numeric(_) | Storage::Edited(..) => {
                return move_into(f, b"0", &lit_num_attr(1, 0, false), decimal_comma);
            }
            _ => f.bytes = vec![b'0'; n],
        },
    }
    Ok(())
}

fn init_value(field: &mut Field, v: &Tok) -> Result<(), RunError> {
    match v {
        Tok::AllLiteral(pat) => {
            // Oracle (cobc 3.2.0): VALUE ALL "lit" repeats the literal to fill the field
            // (`PIC X(5) VALUE ALL "ab"` -> "ababa"), truncating a too-long pattern. Applied to
            // every storage kind (the numeric case compiles with a warning and stores the raw
            // bytes).
            if pat.is_empty() {
                return Err(RunError::Unsupported("empty VALUE ALL".into()));
            }
            let n = field.bytes.len();
            let mut b = Vec::with_capacity(n);
            while b.len() < n {
                b.extend_from_slice(pat);
            }
            b.truncate(n);
            field.bytes = b;
            Ok(())
        }
        Tok::Str(s) => {
            let src = s.clone();
            store_alnum(field, &src)
        }
        Tok::Word(w) => {
            // a figurative constant (SPACES/ZEROS/HIGH-VALUE/...) fills the field; else a numeric literal.
            if let Some(fig) = figurative_kind(w) {
                return fill_figurative(field, fig, false);
            }
            let dec = parse_num_literal(w)?;
            store_decimal(field, &dec)
        }
        Tok::Dot => Err(RunError::Unsupported("empty VALUE".into())),
    }
}

/// Parse a numeric literal like `-12.34` (or E-notation `+1.23E-2`, `1.7014118E+38`) into a
/// [`Decimal`]. The E form's exponent is folded into the scale/digits exactly like the runtime's
/// decimal parse of a floating literal (cob_decimal.c), so `12.3E-2` == `0.123`.
fn parse_num_literal(w: &str) -> Result<Decimal, RunError> {
    let negative = w.starts_with('-');
    let mut body = w.trim_start_matches(['+', '-']).to_string();
    // E-notation: split off the exponent, then fold it into the scale (a positive exponent shifts
    // the decimal point right, a negative one left).
    let mut exp: i64 = 0;
    if let Some(e) = body.find(['e', 'E']) {
        // a trailing 'E' with no exponent digits is not a numeric literal
        let after = body.as_bytes().get(e + 1).copied();
        let neg_exp = after == Some(b'-');
        let exp_part = body[e + 1..].trim_start_matches(['+', '-']);
        // parse the exponent magnitude
        let mag: i64 = exp_part
            .parse()
            .map_err(|_| RunError::Unsupported(format!("not a numeric literal: {w}")))?;
        exp = if neg_exp { -mag } else { mag };
        body = body[..e].to_string();
    }
    if body.is_empty() || !body.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Err(RunError::Unsupported(format!("not a numeric literal: {w}")));
    }
    let (int_p, frac_p) = body.split_once('.').unwrap_or((body.as_str(), ""));
    let mut digits: Vec<u8> = Vec::new();
    for c in int_p.chars().chain(frac_p.chars()) {
        if c.is_ascii_digit() {
            digits.push(c as u8 - b'0');
        }
    }
    let mut scale = frac_p.chars().filter(|c| c.is_ascii_digit()).count() as i64 - exp;
    // a positive exponent shifts the point right: pad trailing zeros so scale never goes negative
    // (`1.7014118E+38` -> digits with 30 trailing zeros, scale 0).
    if scale < 0 {
        for _ in 0..(-scale) {
            digits.push(0);
        }
        scale = 0;
    }
    Ok(Decimal {
        negative,
        digits,
        scale: scale as i16,
    })
}

/// Whether a decimal's magnitude is zero (all digits zero).
fn dec_is_zero(dec: &Decimal) -> bool {
    dec.digits.iter().all(|&d| d == 0)
}

/// Place `src` into a `len`-byte alphanumeric field honoring `JUSTIFIED RIGHT` (right-aligned, truncated
/// on the LEFT) vs the default (left-aligned, space-padded, truncated on the right).
fn alnum_justified_or_left(src: &[u8], len: usize, justified: bool) -> Vec<u8> {
    let mut dst = vec![b' '; len];
    if justified {
        if src.len() >= len {
            dst.copy_from_slice(&src[src.len() - len..]);
        } else {
            dst[len - src.len()..].copy_from_slice(src);
        }
    } else {
        let m = src.len().min(len);
        dst[..m].copy_from_slice(&src[..m]);
    }
    dst
}

/// Store a [`Decimal`] into a field (numeric -> zoned via the runtime move; edited -> encode).
fn store_decimal(field: &mut Field, dec: &Decimal) -> Result<(), RunError> {
    match &field.storage {
        Storage::Edited(pic, currency, decimal_comma, blank_zero) => {
            let pic = pic.clone();
            let cur = *currency;
            let dc = *decimal_comma;
            let blank = *blank_zero;
            // BLANK WHEN ZERO: a zero value blanks the whole edited field.
            field.bytes = if blank && dec_is_zero(dec) {
                vec![b' '; field.bytes.len()]
            } else {
                encode_edited_cfg(&pic, dec, cur, dc)
                    .map_err(|e| RunError::Runtime(format!("{e:?}")))?
            };
            Ok(())
        }
        Storage::Numeric(attr) => {
            // build a literal source field (zoned display) holding the decimal, then move it in.
            let attr = *attr;
            // BLANK WHEN ZERO on a numeric receiver: a zero value blanks the field.
            if attr.blank_when_zero() && dec_is_zero(dec) {
                field.bytes = vec![b' '; field.bytes.len()];
                return Ok(());
            }
            let (src, src_attr) = decimal_as_display(dec);
            let mut dst = field.bytes.clone();
            cob_move(&src, &src_attr, &mut dst, &attr)
                .map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            field.bytes = dst;
            Ok(())
        }
        Storage::Alpha(_) => {
            // numeric VALUE into alphanumeric: the digits as characters.
            let s: Vec<u8> = dec.digits.iter().map(|d| d + b'0').collect();
            store_alnum(field, &s)
        }
        Storage::Group { .. } => Err(RunError::Unsupported(
            "a group MOVE is distributed across its leaves by write_field".into(),
        )),
        Storage::Condition { .. } => Err(RunError::Unsupported(
            "cannot MOVE into an 88 condition-name".into(),
        )),
    }
}

/// The receiver scale (fractional digit count) of a numeric or numeric-edited field -- what a `ROUNDED`
/// store rounds the arithmetic result to. Alphanumeric receivers have no scale (0).
fn receiver_scale(f: &Field) -> i16 {
    match &f.storage {
        Storage::Numeric(a) => a.scale,
        Storage::Edited(pic, ..) => crate::edited::edited_scale(pic).unwrap_or(0),
        _ => 0,
    }
}

/// Increment a big-endian decimal magnitude by 1, growing on all-nines carry (`[9,9]` -> `[1,0,0]`).
fn inc_magnitude(mut d: Vec<u8>) -> Vec<u8> {
    let mut i = d.len();
    loop {
        if i == 0 {
            d.insert(0, 1);
            break;
        }
        i -= 1;
        if d[i] == 9 {
            d[i] = 0;
        } else {
            d[i] += 1;
            break;
        }
    }
    d
}

/// Round a decimal value to `target_scale` fractional digits using COBOL's default `ROUNDED` mode --
/// NEAREST, ties **away from zero** (the libcob `COB_STORE_ROUND` default). A value already at or below
/// the target scale is returned unchanged (the store then zero-extends).
fn round_decimal(dec: &Decimal, target_scale: i16) -> Decimal {
    let ts = target_scale.max(0);
    if dec.scale <= ts {
        return dec.clone();
    }
    let drop = (dec.scale - ts) as usize;
    let keep = dec.digits.len().saturating_sub(drop);
    // `keep <= len-1` (drop >= 1), so `digits[keep]` is the most-significant dropped digit.
    let round_up = dec.digits[keep] >= 5;
    let mut kept: Vec<u8> = dec.digits[..keep].to_vec();
    if round_up {
        kept = inc_magnitude(kept);
    }
    if kept.is_empty() {
        kept.push(0);
    }
    Decimal {
        negative: dec.negative,
        digits: kept,
        scale: ts,
    }
}

/// Render a [`Decimal`] as a zoned `USAGE DISPLAY` source `(bytes, attr)` with a trailing sign
/// overpunch (the form arithmetic + move accept).
fn decimal_as_display(dec: &Decimal) -> (Vec<u8>, FieldAttr) {
    let mut digits = dec.digits.clone();
    if digits.is_empty() {
        digits.push(0);
    }
    let mut bytes: Vec<u8> = digits.iter().map(|d| d + b'0').collect();
    let signed = dec.negative;
    if signed {
        // trailing overpunch for negative (zoned): last digit's zone -> 0x70..0x79.
        if let Some(last) = bytes.last_mut() {
            *last = 0x70 | (*last - b'0');
        }
    }
    let attr = lit_num_attr(digits.len() as u16, dec.scale, signed);
    (bytes, attr)
}

/// The RETURN-CODE special register: a signed `S9(9)` DISPLAY field. (cobc renders it with a LEADING sign
/// + 9 zero-padded digits, e.g. `+000000042`, reproduced by [`display_return_code`].)
fn make_return_code(value: i64) -> Field {
    let attr = lit_num_attr(9, 0, true);
    let mut f = Field {
        storage: Storage::Numeric(attr),
        bytes: vec![b'0'; 9],
        occurs: 1,
        redefines: None,
    };
    let mag: Vec<u8> = value
        .unsigned_abs()
        .to_string()
        .bytes()
        .map(|b| b - b'0')
        .collect();
    let _ = store_decimal(
        &mut f,
        &Decimal {
            negative: value < 0,
            digits: mag,
            scale: 0,
        },
    );
    f
}

/// Format the RETURN-CODE register the way cobc DISPLAYs it: a leading `+`/`-` then 9 zero-padded digits
/// (`+000000042`, `+000000000`, `-000000007`).
fn display_return_code(f: &Field) -> Vec<u8> {
    let dec = match &f.storage {
        Storage::Numeric(a) => source_to_decimal(&f.bytes, a).ok(),
        _ => None,
    }
    .unwrap_or(Decimal {
        negative: false,
        digits: vec![0],
        scale: 0,
    });
    let mag_full: String = dec.digits.iter().map(|d| (d + b'0') as char).collect();
    let mag = mag_full.trim_start_matches('0');
    let mag = if mag.is_empty() { "0" } else { mag };
    let sign = if dec.negative && mag != "0" { '-' } else { '+' };
    format!("{sign}{mag:0>9}").into_bytes()
}

/// Store alphanumeric source bytes into a field (left-justified, space-padded/truncated, or numeric
/// receiver via the runtime move).
fn store_alnum(field: &mut Field, src: &[u8]) -> Result<(), RunError> {
    let src_attr = alnum_attr();
    match &field.storage {
        Storage::Edited(..) => {
            // alphanumeric into edited: just place left-justified (rare path).
            let n = field.bytes.len();
            let mut b = vec![b' '; n];
            let m = src.len().min(n);
            b[..m].copy_from_slice(&src[..m]);
            field.bytes = b;
            Ok(())
        }
        Storage::Alpha(attr) if attr.justified() => {
            // JUSTIFIED RIGHT alphanumeric receiver: right-align the source bytes.
            field.bytes = alnum_justified_or_left(src, field.bytes.len(), true);
            Ok(())
        }
        Storage::Alpha(attr) | Storage::Numeric(attr) => {
            let attr = *attr;
            let mut dst = field.bytes.clone();
            cob_move(src, &src_attr, &mut dst, &attr)
                .map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            field.bytes = dst;
            Ok(())
        }
        Storage::Group { .. } => Err(RunError::Unsupported(
            "a group MOVE is distributed across its leaves by write_field".into(),
        )),
        Storage::Condition { .. } => Err(RunError::Unsupported(
            "cannot MOVE into an 88 condition-name".into(),
        )),
    }
}

/// Execute one statement (its verb + the tokens up to the terminating period).
fn exec_stmt(
    verb: &str,
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    match verb {
        "DISPLAY" => exec_display(stmt, fields, out, ctx),
        "MOVE" => exec_move(stmt, fields, ctx.decimal_comma),
        "SET" => exec_set(stmt, fields, ctx.decimal_comma, &ctx.switches),
        "INITIALIZE" => exec_initialize(stmt, fields, ctx.decimal_comma),
        "INSPECT" => exec_inspect(stmt, fields, ctx.decimal_comma),
        // STRING is normally dispatched in run_block (for its ON OVERFLOW handler); this arm is a
        // type-safe fallback for any bare STRING reaching exec_stmt (the overflow flag is unused here).
        "STRING" => exec_string(stmt, fields, ctx.decimal_comma).map(|_| ()),
        // UNSTRING is normally dispatched in run_block (for its ON OVERFLOW handler); this arm is a
        // type-safe fallback for any bare UNSTRING reaching exec_stmt (the overflow flag is unused here).
        "UNSTRING" => exec_unstring(stmt, fields).map(|_| ()),
        "ACCEPT" => exec_accept(stmt, fields),
        "OPEN" => exec_open(stmt, fields, out, ctx),
        "CLOSE" => exec_close(stmt, fields, ctx),
        "WRITE" => exec_write(stmt, fields, ctx),
        "REWRITE" => exec_rewrite(stmt, fields, ctx),
        "DELETE" => exec_delete(stmt, fields, ctx),
        // MERGE of already-ordered inputs yields the same globally-ordered output as SORT over the union.
        "SORT" | "MERGE" => exec_sort(stmt, fields, out, ctx),
        "RELEASE" => exec_release(stmt, fields, ctx),
        "JSON" => match stmt.first() {
            Some(Tok::Word(w)) if w == "GENERATE" => exec_ml_generate(stmt, fields, ctx, false),
            Some(Tok::Word(w)) if w == "PARSE" => exec_ml_parse_noop(),
            _ => Err(RunError::Unsupported("JSON: expected GENERATE/PARSE".into())),
        },
        "XML" => match stmt.first() {
            Some(Tok::Word(w)) if w == "GENERATE" => exec_ml_generate(stmt, fields, ctx, true),
            Some(Tok::Word(w)) if w == "PARSE" => exec_ml_parse_noop(),
            _ => Err(RunError::Unsupported("XML: expected GENERATE".into())),
        },
        "TRANSFORM" => exec_transform(stmt, fields),
        "EXAMINE" => exec_examine(stmt, fields, ctx.decimal_comma),
        "EXHIBIT" => exec_exhibit(stmt, fields, out, ctx),
        "ALTER" => exec_alter(stmt),
        "GENERATE" => exec_generate(stmt, fields, ctx),
        "INITIATE" => exec_initiate(stmt, ctx),
        "TERMINATE" => exec_terminate(stmt, fields, ctx),
        // SUPPRESS (Report Writer DETAIL suppression) is a no-op over the current subset.
        "SUPPRESS" => Ok(()),
        // GnuCOBOL 3.2 compiles these as "not implemented" (or with no stdout effect under the default
        // runtime); the oracle-first front-end accepts them and matches that exactly as a no-op.
        "RAISE" | "VALIDATE" | "DESTROY" | "READY" | "RESET" => Ok(()),
        "UNLOCK" => exec_unlock(stmt, fields, ctx),
        // COMMIT / ROLLBACK are no-ops without a transactional backend (as libcob is for sequential files).
        "COMMIT" | "ROLLBACK" => Ok(()),
        "CALL" => exec_call(stmt, fields, out, ctx),
        "STOP" => Ok(()), // STOP RUN
        // ADD/SUBTRACT/MULTIPLY/DIVIDE/COMPUTE are handled in run_block (they carry ON SIZE ERROR clauses).
        // The remaining verbs are explicit boundary non-claims: GnuCOBOL itself needs a data-division
        // section the front-end's WORKING-STORAGE/FILE/REPORT model does not include, or the result is
        // nondeterministic. Each fails closed with the specific reason (not a lazy placeholder).
        // The COMMUNICATION SECTION (message control) is NOT IMPLEMENTED in the admitted GnuCOBOL 3.2
        // itself (`warning: COMMUNICATION SECTION is not implemented [-Wpending]` + "CD record missing"):
        // the oracle cannot compile/run these, so there is no output to be byte-identical to.
        "SEND" | "RECEIVE" | "PURGE" | "ENABLE" | "DISABLE" =>
            Err(RunError::Unsupported(format!("{verb}: GnuCOBOL 3.2 does not implement the COMMUNICATION SECTION -- the oracle itself cannot run it (boundary non-claim)"))),
        // MODIFY / INQUIRE are ACUCOBOL GUI verbs absent from the admitted GnuCOBOL 3.2 grammar (a syntax
        // error in the oracle) -- there is nothing to reproduce.
        "MODIFY" | "INQUIRE" =>
            Err(RunError::Unsupported(format!("{verb}: an ACUCOBOL screen/GUI verb absent from the GnuCOBOL 3.2 grammar -- the oracle itself rejects it (boundary non-claim)"))),
        "ALLOCATE" => exec_allocate(stmt, fields, ctx.decimal_comma),
        "FREE" => Ok(()), // FREE [ADDRESS OF] id -- release based storage; a no-op in the logical model.
        "ENTRY" =>
            Err(RunError::Unsupported("ENTRY: an alternate entry point that is invalid in a nested program -- it requires separately-compiled units, while the front-end runs one source with contained programs".into())),
        other => Err(RunError::Unsupported(format!("verb {other}"))),
    }
}

/// `CALL "NAME" [USING [BY REFERENCE|CONTENT] arg ...]` to a CONTAINED (nested) program. The callee runs
/// in its own field table: each `BY REFERENCE` (default) argument is copied into the callee's matching
/// `PROCEDURE DIVISION USING` parameter and copied BACK afterwards (so the caller sees the callee's
/// updates); `BY CONTENT` is copied in only. RETURN-CODE is shared (copied in, then back). A CALL to a
/// name that is not a contained program fails closed (an external `.so` CALL is the declared dlopen
/// boundary -- never silently no-op'd).
fn exec_call(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let name = match stmt.first() {
        Some(Tok::Str(s)) => String::from_utf8_lossy(s).to_string(),
        Some(Tok::Word(w)) => w.clone(), // CALL <identifier> -- treat the literal word as the name
        _ => return Err(RunError::Unsupported("CALL without a program name".into())),
    };
    // Program-ids are case-insensitive: the registry keys are uppercased at parse, but a `CALL "literal"`
    // keeps the literal's case, so match the name as-is and then uppercased.
    let callee = ctx
        .programs
        .get(&name)
        .or_else(|| ctx.programs.get(&name.to_uppercase()))
        .ok_or_else(|| {
            RunError::Unsupported(format!(
                "CALL \"{name}\": not a contained program (external CALL is a boundary)"
            ))
        })?;
    // A PROTOTYPE is a signature-only declaration: executing it is a typed boundary (cobc's own
    // prototype handling is unfinished; the ANY LENGTH / BY VALUE argument checks upstream
    // 14f0d0908 guards are inside that boundary).
    if callee.is_prototype {
        return Err(RunError::Unsupported(format!(
            "CALL \"{name}\": program is a PROTOTYPE declaration (signature-only; execution is a typed boundary)"
        )));
    }

    // Parse the USING argument list with optional BY REFERENCE/CONTENT modifiers.
    let mut args: Vec<(String, bool)> = Vec::new(); // (caller field name, by_reference)
    let mut by_ref = true;
    let mut seen_using = false;
    for t in &stmt[1..] {
        if let Tok::Word(w) = t {
            match w.as_str() {
                "USING" => seen_using = true,
                "BY" => {}
                "REFERENCE" => by_ref = true,
                "CONTENT" | "VALUE" => by_ref = false,
                _ if seen_using => args.push((w.clone(), by_ref)),
                _ => {}
            }
        }
    }

    // EXTERNAL storage is run-unit-shared: publish the caller's current values before the callee builds
    // (its build loads them) and reads them back after.
    sync_external_to_store(fields);
    // The callee's fields: restore its PERSISTED WORKING-STORAGE (COBOL static storage -- a subprogram's WS
    // survives between CALLs) when it has been called before and not CANCELed; otherwise build fresh from
    // the VALUE clauses. An INITIAL program is always rebuilt (re-initialized every entry).
    let mut cfields = if !callee.is_initial {
        match ctx.call_state.borrow_mut().remove(&name) {
            Some(saved) => saved,
            None => build_program_fields(callee, ctx)?,
        }
    } else {
        build_program_fields(callee, ctx)?
    };
    sync_store_to_external(&mut cfields); // the callee sees the shared EXTERNAL values (fresh or persisted)
                                          // RETURN-CODE is shared: seed the callee with the caller's current value.
    if let Some(rc) = fields.get("RETURN-CODE") {
        cfields.insert("RETURN-CODE".to_string(), rc.clone());
    }
    for (idx, param) in callee.using.iter().enumerate() {
        let (argname, _) = args.get(idx).ok_or_else(|| {
            RunError::Unsupported(format!(
                "CALL \"{name}\": fewer USING args than the {} parameters",
                callee.using.len()
            ))
        })?;
        let argf = fields
            .get(argname)
            .ok_or_else(|| RunError::UndefinedName(argname.clone()))?
            .clone();
        cfields.insert(param.clone(), argf); // copy-in (the LINKAGE field takes the caller's value)
    }

    run_program_body(callee, &name, ctx, &mut cfields, out)?;

    // EXTERNAL: the callee's (possibly modified) shared values flow back to the store, then to the caller.
    sync_external_to_store(&cfields);
    sync_store_to_external(fields);

    // Copy-out: BY REFERENCE arguments receive the callee's (possibly modified) parameter value back.
    for (idx, param) in callee.using.iter().enumerate() {
        if let Some((argname, true)) = args.get(idx) {
            if let Some(updated) = cfields.get(param) {
                fields.insert(argname.clone(), updated.clone());
            }
        }
    }
    // RETURN-CODE propagates back to the caller.
    if let Some(rc) = cfields.get("RETURN-CODE") {
        fields.insert("RETURN-CODE".to_string(), rc.clone());
    }
    // Persist the callee's WORKING-STORAGE for the next CALL (static storage). INITIAL programs do not
    // persist -- they re-initialize from VALUE each entry.
    if !callee.is_initial {
        ctx.call_state.borrow_mut().insert(name, cfields);
    }
    Ok(())
}

/// `COMPUTE r1 [r2 ...] [ROUNDED] = <expr>` -- evaluate an arithmetic expression and store the result
/// into each receiver. The expression grammar (standard precedence): `expr := term (('+'|'-') term)*`,
/// `term := factor (('*'|'/') factor)*`, `factor := primary ('**' factor)?`, `primary := '(' expr ')'
/// | '-' primary | operand`. Each binary op is computed via a WIDE numeric intermediate (so a long
/// expression keeps precision); the per-receiver store is the truncation/edit point. `ROUNDED` and any
/// non-integer `**` exponent fail closed (not yet in the sealed envelope).
/// Map an arithmetic error: a divide-by-zero becomes a recoverable [`RunError::SizeError`] (the ON SIZE
/// ERROR path -- the receiver is left unchanged); everything else is a fatal runtime error.
fn map_arith_err(e: ArithError) -> RunError {
    match e {
        ArithError::DivideByZero => RunError::SizeError,
        other => RunError::Runtime(format!("{other:?}")),
    }
}

/// `COMPUTE` -- returns `true` if a SIZE ERROR (e.g. divide-by-zero) occurred, in which case the receiver
/// is left UNCHANGED (the move never runs). The caller dispatches the `ON SIZE ERROR` handler.
fn exec_compute(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    has_handler: bool,
) -> Result<bool, RunError> {
    match exec_compute_inner(stmt, fields, has_handler) {
        Ok(size_err) => Ok(size_err),
        Err(RunError::SizeError) => {
            set_exception("EC-SIZE-ZERO-DIVIDE");
            Ok(true)
        }
        Err(e) => Err(e),
    }
}

fn exec_compute_inner(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    has_handler: bool,
) -> Result<bool, RunError> {
    let eq = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "="))
        .ok_or_else(|| RunError::Unsupported("COMPUTE without '='".into()))?;
    // receivers = the names before '='; a `ROUNDED` keyword rounds every receiver's store to its own
    // scale (COBOL default mode: NEAREST, ties away from zero).
    // receivers = the names before `ROUNDED` (or `=`); `ROUNDED [MODE [IS] <mode>]` rounds every receiver's
    // store to its own scale (default mode: NEAREST-AWAY-FROM-ZERO).
    let round_mode = round_mode_of(&stmt[..eq]);
    let mut receivers = Vec::new();
    for t in &stmt[..eq] {
        match t {
            Tok::Word(w) if w == "ROUNDED" => break, // ROUNDED + the MODE phrase that follows are not receivers
            Tok::Word(w) => receivers.push(w.clone()),
            _ => {}
        }
    }
    if receivers.is_empty() {
        return Err(RunError::Unsupported("COMPUTE with no receiver".into()));
    }
    // resolve any FUNCTION reference in the expression into a temp field (referenced by name below).
    let expr = resolve_functions(&stmt[eq + 1..], fields)?;
    // tokenize the expression: split parentheses glued to operands; operators are space-separated.
    let mut etoks: Vec<String> = Vec::new();
    for t in &expr {
        match t {
            Tok::Word(w) => split_parens(w, &mut etoks),
            Tok::Str(_) => return Err(RunError::Unsupported("string in COMPUTE".into())),
            Tok::AllLiteral(_) => return Err(RunError::Unsupported("string in COMPUTE".into())),
            Tok::Dot => {}
        }
    }
    let mut pos = 0;
    let (val, attr) = parse_expr(&etoks, &mut pos, fields)?;
    if pos != etoks.len() {
        return Err(RunError::Unsupported(format!(
            "trailing tokens in COMPUTE expr at {}",
            etoks[pos]
        )));
    }
    let mut size_err = false;
    for r in receivers {
        // Store through write_field so a subscripted / multi-dimension receiver (`E(I)`, `N(I,J)`) works,
        // not just a scalar. COMPUTE result is an already-decoded numeric value -> separator-independent
        // store. With ROUNDED, round to THIS receiver's scale before storing (default mode: ties away).
        let mut se = false;
        write_field(fields, &r, |f| {
            if let Some(mode) = round_mode {
                let dec = source_to_decimal(&val, &attr)?;
                let (rdec, prohibited) = round_decimal_mode(&dec, receiver_scale(f), mode);
                if prohibited {
                    se = true; // MODE PROHIBITED + a dropped non-zero digit -> size error, receiver unchanged
                } else {
                    let (rval, rattr) = decimal_as_display(&rdec);
                    se = store_arith_result(f, &rval, &rattr, has_handler, false)?;
                }
            } else {
                se = store_arith_result(f, &val, &attr, has_handler, false)?;
            }
            Ok(())
        })?;
        size_err |= se;
    }
    Ok(size_err)
}

/// Split a word into expression tokens, peeling leading `(` and trailing `)` (which may glue to an
/// operand, e.g. `(A` or `B)`); `**` / `+` / `-` / `*` / `/` and bare names pass through.
fn split_parens(w: &str, out: &mut Vec<String>) {
    // Peel a leading unary sign glued to the operand (`-A`, `+5`, `-(A-B)`): a name cannot begin with a sign,
    // so a leading `-`/`+` on a multi-char word is always unary. (A lone `-`/`+` is the binary operator and
    // is left intact.) parse_primary then applies the unary minus/plus.
    if w.len() > 1 && (w.starts_with('-') || w.starts_with('+')) {
        out.push(w[..1].to_string());
        return split_parens(&w[1..], out);
    }
    // Peel a leading GROUPING '(' then RECURSE, so a sign or further paren after it is re-handled
    // (`(-A)`, `-(-(-3))`).
    if let Some(rest) = w.strip_prefix('(') {
        out.push("(".into());
        return split_parens(rest, out);
    }
    let mut s = w;
    // If what remains is a name-prefixed subscript/refmod `NAME(...)`, keep that operand WHOLE -- its own
    // parens belong to it, not to grouping -- so parse_primary -> operand_value resolves the element; any
    // parens after its matching `)` are trailing grouping closes.
    if let Some(open) = s.find('(') {
        if open > 0 {
            let bytes = s.as_bytes();
            let mut depth = 0i32;
            let mut end = None;
            for (i, &c) in bytes.iter().enumerate().skip(open) {
                if c == b'(' {
                    depth += 1;
                } else if c == b')' {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
            }
            if let Some(e) = end {
                out.push(s[..=e].to_string()); // the subscripted operand, e.g. `E(1)`
                let mut tail = &s[e + 1..];
                let mut close = 0;
                while let Some(t) = tail.strip_suffix(')') {
                    close += 1;
                    tail = t;
                }
                for _ in 0..close {
                    out.push(")".into());
                }
                return;
            }
        }
    }
    // No subscript: peel trailing grouping ')'.
    let mut close = 0;
    while s.ends_with(')') {
        close += 1;
        s = &s[..s.len() - 1];
    }
    if !s.is_empty() {
        out.push(s.to_string());
    }
    for _ in 0..close {
        out.push(")".into());
    }
}

/// `expr := term (('+'|'-') term)*`.
fn parse_expr(
    t: &[String],
    pos: &mut usize,
    f: &HashMap<String, Field>,
) -> Result<(Vec<u8>, FieldAttr), RunError> {
    let (mut acc, mut aattr) = parse_term(t, pos, f)?;
    while let Some(op) = t.get(*pos).map(|s| s.as_str()) {
        let o = match op {
            "+" => Op::Add,
            "-" => Op::Subtract,
            _ => break,
        };
        *pos += 1;
        let (b, battr) = parse_term(t, pos, f)?;
        let (r, ra) = wide_op(o, &acc, &aattr, &b, &battr)?;
        acc = r;
        aattr = ra;
    }
    Ok((acc, aattr))
}

/// `term := factor (('*'|'/') factor)*`.
fn parse_term(
    t: &[String],
    pos: &mut usize,
    f: &HashMap<String, Field>,
) -> Result<(Vec<u8>, FieldAttr), RunError> {
    let (mut acc, mut aattr) = parse_factor(t, pos, f)?;
    while let Some(op) = t.get(*pos).map(|s| s.as_str()) {
        match op {
            "*" => {
                *pos += 1;
                let (b, battr) = parse_factor(t, pos, f)?;
                let (r, ra) = wide_op(Op::Multiply, &acc, &aattr, &b, &battr)?;
                acc = r;
                aattr = ra;
            }
            "/" => {
                *pos += 1;
                let (b, battr) = parse_factor(t, pos, f)?;
                // normalize binary/float operands to DISPLAY for cob_divide's decoder (handles DISPLAY+PACKED).
                let (an, anattr) = to_arith_operand(&acc, &aattr)?;
                let (bn, bnattr) = to_arith_operand(&b, &battr)?;
                let wide = lit_num_attr(36, 18, true); // generous quotient scale; receiver store truncates.
                acc = cob_divide(&an, &anattr, &bn, &bnattr, &wide, Round::Truncate)
                    .map_err(map_arith_err)?;
                aattr = wide;
            }
            _ => break,
        }
    }
    Ok((acc, aattr))
}

/// `factor := primary ('**' factor)?` -- exponentiation, RIGHT-associative (`2 ** 3 ** 2` = `2 ** (3 ** 2)`
/// = 512). A non-negative integer exponent uses exact repeated multiply (the sealed path); anything else
/// (fractional like 0.5, negative, or an identifier exponent) goes through the sealed cob_decimal_pow engine.
fn parse_factor(
    t: &[String],
    pos: &mut usize,
    f: &HashMap<String, Field>,
) -> Result<(Vec<u8>, FieldAttr), RunError> {
    let (base, battr) = parse_primary(t, pos, f)?;
    if t.get(*pos).map(|s| s.as_str()) == Some("**") {
        *pos += 1;
        let (eb, ea) = parse_factor(t, pos, f)?; // right-associative
        let ed = source_to_decimal(&eb, &ea)?;
        if !ed.negative && ed.scale <= 0 {
            let e = dec_to_i64(&ed);
            if (0..=1024).contains(&e) {
                if e == 0 {
                    let (one, oa) = decimal_as_display(&Decimal {
                        negative: false,
                        digits: vec![1],
                        scale: 0,
                    });
                    return Ok((one, oa));
                }
                let mut acc = base.clone();
                let mut acc_attr = battr;
                for _ in 1..e {
                    let (r, ra) = wide_op(Op::Multiply, &acc, &acc_attr, &base, &battr)?;
                    acc = r;
                    acc_attr = ra;
                }
                return Ok((acc, acc_attr));
            }
        }
        let (rb, ra) = crate::intrinsic::cob_intr_pow(&base, &battr, &eb, &ea);
        return Ok((rb, ra));
    }
    Ok((base, battr))
}

/// `primary := '(' expr ')' | '-' primary | operand`.
fn parse_primary(
    t: &[String],
    pos: &mut usize,
    f: &HashMap<String, Field>,
) -> Result<(Vec<u8>, FieldAttr), RunError> {
    match t.get(*pos).map(|s| s.as_str()) {
        Some("(") => {
            *pos += 1;
            let v = parse_expr(t, pos, f)?;
            if t.get(*pos).map(|s| s.as_str()) != Some(")") {
                return Err(RunError::Unsupported("missing ')' in COMPUTE".into()));
            }
            *pos += 1;
            Ok(v)
        }
        Some("-") => {
            *pos += 1;
            let (b, ba) = parse_primary(t, pos, f)?;
            // unary minus: 0 - b.
            let (zero, za) = decimal_as_display(&Decimal {
                negative: false,
                digits: vec![0],
                scale: 0,
            });
            wide_op(Op::Subtract, &zero, &za, &b, &ba)
        }
        Some("+") => {
            *pos += 1;
            parse_primary(t, pos, f)
        }
        Some(_) => {
            let w = t[*pos].clone();
            *pos += 1;
            operand_value(&Tok::Word(w), f)
        }
        None => Err(RunError::Unsupported(
            "unexpected end of COMPUTE expr".into(),
        )),
    }
}

/// `DISPLAY op [op ...]` -- concatenate each operand's display bytes, then a newline.
/// The bytes a literal word contributes to DISPLAY: a numeric literal (incl. E-notation), or a
/// binary/hexadecimal literal (`B'0101'`, `BX'EC'`, `X'EC'`, `H'...'`). Fails closed on a word that
/// is neither a field nor a literal.
fn display_literal_bytes(w: &str) -> Result<Vec<u8>, RunError> {
    // cobc DISPLAYs a numeric literal with its written sign and a canonical point: leading integer
    // zeros are dropped (`.123` for 0.123), the fraction keeps its digits, E-notation is folded.
    let dec = parse_num_literal(w)?;
    let mut out = Vec::new();
    if dec.negative {
        out.push(b'-');
    } else if w.starts_with('+') {
        out.push(b'+');
    }
    let scale = dec.scale.max(0) as usize;
    let digits = &dec.digits;
    let int_len = digits.len().saturating_sub(scale);
    let int_part = &digits[..int_len.min(digits.len())];
    let frac_part = &digits[int_len.min(digits.len())..];
    let int_str: String = int_part.iter().map(|d| (b'0' + d) as char).collect();
    let int_trim = int_str.trim_start_matches('0').to_string();
    let frac_str: String = frac_part.iter().map(|d| (b'0' + d) as char).collect();
    out.extend_from_slice(int_trim.as_bytes());
    if !frac_str.is_empty() {
        out.push(b'.');
        out.extend_from_slice(frac_str.as_bytes());
    }
    if out.is_empty() || (out.len() == 1 && (out[0] == b'-' || out[0] == b'+')) {
        out.push(b'0');
    }
    Ok(out)
}

/// `DISPLAY B'0101'` / `BX'EC'`: the lexer yields `Word("B")` + `Str("0101")`; the operand scanner
/// pairs a literal-prefix word with the following string token. Returns `Some(bytes)` when `w` is a
/// literal prefix (B/BX/X/H/N/NX/HB) and `quote` is the following token; `None` when `w` is not a
/// prefix (the caller then treats it as a field or numeric literal).
fn binary_literal_bytes(w: &str, quote: Option<&Tok>) -> Result<Option<Vec<u8>>, RunError> {
    let up = w.to_ascii_uppercase();
    let q = match quote {
        Some(Tok::Str(s)) => s.clone(),
        _ => return Ok(None),
    };
    // cobc DISPLAYs a binary/hex literal as its DECIMAL value (B'0101' -> "5", BX'EC' -> "236").
    // A plain B literal is a BINARY NUMBER (bits folded directly, not byte-aligned); BX/X/H are
    // hexadecimal numbers (nibbles folded).
    let mut value: u64 = 0;
    match up.as_str() {
        "B" => {
            for &c in &q {
                value = (value << 1)
                    | match c {
                        b'0' => 0,
                        b'1' => 1,
                        _ => {
                            return Err(RunError::Unsupported(format!(
                                "invalid binary digit in {w} literal"
                            )))
                        }
                    };
            }
        }
        "BX" | "X" | "H" => {
            for &c in &q {
                let v = (c as char).to_digit(16).ok_or_else(|| {
                    RunError::Unsupported(format!("invalid hex digit in {w} literal"))
                })? as u64;
                value = (value << 4) | v;
            }
        }
        // N/NX/HB national/bit forms are NOT auto-detected here (a field named N must resolve
        // normally); an actual N'...' literal in the source fails later at the numeric-literal path.
        _ => return Ok(None),
    }
    Ok(Some(value.to_string().into_bytes()))
}

fn exec_display(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    // resolve any FUNCTION reference into a temp field, then display through the ordinary field paths.
    let stmt = resolve_functions(stmt, fields)?;
    let fields = &*fields;
    let mut operands: Vec<(Vec<u8>, FieldAttr)> = Vec::new();
    // `DISPLAY ... UPON PRINTER` (a built-in device mnemonic -- cobc accepts it even when SPECIAL-NAMES
    // does not declare it) is routed to the print redirect when active; UPON CONSOLE/SYSOUT and the
    // default stay on stdout.
    let mut upon_printer = false;
    let mut upon_dev: Option<String> = None;
    let mut i = 0usize;
    while i < stmt.len() {
        let t = &stmt[i];
        match t {
            Tok::Str(s) => operands.push((s.clone(), alnum_attr())),
            Tok::AllLiteral(s) => operands.push((s.clone(), alnum_attr())),
            Tok::Word(w) => {
                if w == "UPON" {
                    if let Some(Tok::Word(dev)) = stmt.get(i + 1) {
                        upon_printer = dev == "PRINTER";
                        upon_dev = Some(dev.clone());
                        i += 1;
                    }
                    i += 1;
                    continue;
                }
                if w == "WITH" || w == "NO" || w == "ADVANCING" {
                    // DISPLAY ... WITH NO ADVANCING handled below (no newline) -- mark it.
                    i += 1;
                    continue;
                }
                // a figurative constant in DISPLAY is a single character (cobc displays it length 1).
                if let Some(fig) = figurative_kind(w) {
                    operands.push((vec![fig_byte(fig)], alnum_attr()));
                    i += 1;
                    continue;
                }
                // A DECLARED FIELD always wins (a field named N or X followed by a string must not be
                // mistaken for an N'/X' literal prefix -- the ROUNDED suites' `DISPLAY M " " N ...`).
                if let Some(f) = read_field(fields, w)? {
                    let bytes = if w == "RETURN-CODE" {
                        display_return_code(&f)
                    } else {
                        display_bytes(&f, ctx.decimal_comma)
                    };
                    operands.push((bytes, alnum_attr()));
                    i += 1;
                    continue;
                }
                // binary/hexadecimal literal prefix + quoted digits: `DISPLAY B'0101'`
                if let Some(bytes) = binary_literal_bytes(w, stmt.get(i + 1))? {
                    operands.push((bytes, alnum_attr()));
                    i += 2;
                    continue;
                }
                // A numeric / floating literal operand (`DISPLAY 123`, `DISPLAY +1.23E0`): render it.
                operands.push((display_literal_bytes(w)?, alnum_attr()));
            }
            Tok::Dot => {}
        }
        i += 1;
    }
    // DISPLAY ... UPON ENVIRONMENT-NAME / ENVIRONMENT-VALUE set the env-name register / a per-run env
    // override; they produce NO stdout (cobc routes them to the runtime environment, not the terminal).
    if let Some(dev) = upon_dev.as_deref() {
        if dev == "ENVIRONMENT-NAME" || dev == "ENVIRONMENT-VALUE" {
            let val: Vec<u8> = operands
                .iter()
                .flat_map(|(b, _)| b.iter().copied())
                .collect();
            if dev == "ENVIRONMENT-NAME" {
                ENV_NAME_REG.with(|r| {
                    *r.borrow_mut() = String::from_utf8_lossy(&val).trim_end().to_string()
                });
            } else if dev == "ARGUMENT-NUMBER" {
                let n = String::from_utf8_lossy(&val)
                    .trim()
                    .parse::<usize>()
                    .unwrap_or(0);
                ARG_NUMBER_REG.with(|r| *r.borrow_mut() = n);
            } else {
                let name = ENV_NAME_REG.with(|r| r.borrow().clone());
                ENV_OVERRIDE.with(|m| m.borrow_mut().insert(name, val));
            }
            return Ok(());
        }
    }
    let no_adv = stmt
        .iter()
        .any(|t| matches!(t, Tok::Word(w) if w=="ADVANCING"));
    let refs: Vec<(&[u8], &FieldAttr)> = operands.iter().map(|(b, a)| (b.as_slice(), a)).collect();
    if upon_printer && ctx.print_redirect {
        let mut p = ctx.printer.borrow_mut();
        cob_display(!no_adv, &refs, &DisplaySettings::default(), &mut p);
    } else {
        cob_display(!no_adv, &refs, &DisplaySettings::default(), out);
    }
    Ok(())
}

/// The bytes a field contributes to DISPLAY: numeric DISPLAY fields are shown via the runtime's
/// display formatting; alphanumeric + edited fields are shown as their stored bytes.
fn display_bytes(f: &Field, decimal_comma: bool) -> Vec<u8> {
    match &f.storage {
        // BLANK WHEN ZERO leaves the data spaces when the value is zero; DISPLAY emits the raw bytes
        // (cob_display_common would re-normalize spaces back to "0000").
        Storage::Numeric(attr) if attr.blank_when_zero() && f.bytes.iter().all(|&b| b == b' ') => {
            f.bytes.clone()
        }
        Storage::Numeric(attr) => {
            let mut o = Vec::new();
            // DISPLAY of a numeric DISPLAY item inserts the module decimal point (comma under
            // DECIMAL-POINT IS COMMA), matching cob_display_common's pretty-display path.
            let settings = DisplaySettings {
                decimal_point: if decimal_comma { b',' } else { b'.' },
                ..DisplaySettings::default()
            };
            crate::termio::cob_display_common(&f.bytes, attr, &settings, &mut o);
            o
        }
        Storage::Alpha(_) | Storage::Edited(..) => f.bytes.clone(),
        Storage::Group { .. } => f.bytes.clone(), // a group displays as its concatenated record image
        Storage::Condition { .. } => Vec::new(),  // a condition-name has no displayable value
    }
}

/// `MOVE src TO d1 [d2 ...]`.
fn exec_move(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    decimal_comma: bool,
) -> Result<(), RunError> {
    // resolve a FUNCTION source into a temp field, then MOVE through the ordinary field paths.
    let stmt = resolve_functions(stmt, fields)?;
    let stmt = &stmt[..];
    // MOVE CORRESPONDING matches elementary leaves between two groups BY NAME. The flat field model keys
    // every leaf by its bare name, so two groups with like-named children collide -- there is no way to
    // tell `A OF G1` from `A OF G2`. Faithful CORRESPONDING needs qualified-name (OF/IN) support first, so
    // fail closed rather than move a leaf onto itself. (Front-end sub-form gap; see COBOL-PARITY.md.)
    if matches!(stmt.first(), Some(Tok::Word(w)) if w == "CORRESPONDING" || w == "CORR") {
        // MOVE CORRESPONDING g1 TO g2: move each elementary leaf of g1 to the like-named leaf of g2.
        let to = stmt
            .iter()
            .position(|t| matches!(t, Tok::Word(w) if w == "TO"))
            .ok_or_else(|| RunError::Unsupported("MOVE CORRESPONDING without TO".into()))?;
        let src = match stmt.get(1) {
            Some(Tok::Word(w)) => w.clone(),
            _ => {
                return Err(RunError::Unsupported(
                    "MOVE CORRESPONDING: missing source group".into(),
                ))
            }
        };
        let dst = match stmt.get(to + 1) {
            Some(Tok::Word(w)) => w.clone(),
            _ => {
                return Err(RunError::Unsupported(
                    "MOVE CORRESPONDING: missing target group".into(),
                ))
            }
        };
        for (sk, dk) in corr_pairs(fields, &src, &dst)? {
            let mv = vec![Tok::Word(sk), Tok::Word("TO".to_string()), Tok::Word(dk)];
            exec_move(&mv, fields, decimal_comma)?;
        }
        return Ok(());
    }
    // split at TO.
    let to = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w=="TO"))
        .ok_or_else(|| RunError::Unsupported("MOVE without TO".into()))?;
    let src_tok = stmt
        .first()
        .ok_or_else(|| RunError::Unsupported("MOVE without source".into()))?;
    let dests: Vec<String> = stmt[to + 1..]
        .iter()
        .filter_map(|t| {
            if let Tok::Word(w) = t {
                Some(w.clone())
            } else {
                None
            }
        })
        .collect();
    // `MOVE ALL <literal>` / `MOVE ALL <figurative>` -- repeat the unit to fill EACH receiver's width.
    if matches!(src_tok, Tok::Word(w) if w == "ALL") {
        match stmt.get(1) {
            Some(Tok::Str(s)) if !s.is_empty() => {
                let unit = s.clone();
                for d in &dests {
                    write_field(fields, d, |f| {
                        f.bytes = unit.iter().copied().cycle().take(f.bytes.len()).collect();
                        Ok(())
                    })?;
                }
                return Ok(());
            }
            Some(Tok::Word(u)) if figurative_kind(u).is_some() => {
                let fig = figurative_kind(u).unwrap();
                for d in &dests {
                    write_field(fields, d, |f| fill_figurative(f, fig, decimal_comma))?;
                }
                return Ok(());
            }
            _ => {
                return Err(RunError::Unsupported(
                    "MOVE ALL: expected a non-empty literal or figurative".into(),
                ))
            }
        }
    }
    // A figurative-constant source (SPACES/ZEROS/HIGH-VALUE/...) fills EACH receiver to its own width.
    if let Tok::Word(w) = src_tok {
        if let Some(fig) = figurative_kind(w) {
            for d in &dests {
                write_field(fields, d, |f| fill_figurative(f, fig, decimal_comma))?;
            }
            return Ok(());
        }
    }
    // resolve the source value as (bytes, attr) once.
    let (sbytes, sattr) = operand_value(src_tok, fields)?;
    for d in dests {
        write_field(fields, &d, |f| move_into(f, &sbytes, &sattr, decimal_comma))?;
    }
    Ok(())
}

/// `SET cond-name [cond-name ...] TO TRUE` -- the write counterpart of the LEVEL-88 predicate
/// (`GNURUST.12 SET ... TO TRUE`): construct the parent's bytes so the condition becomes true by MOVEing
/// the condition's first `VALUE` (or a `THRU` range's lower bound) into the parent. Only `TO TRUE` is in
/// the subset; `TO FALSE`, index/pointer SET, and `UP/DOWN BY` fail closed.
fn exec_set(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    decimal_comma: bool,
    switches: &SwitchEnv,
) -> Result<(), RunError> {
    // form: SET idx [idx ...] UP|DOWN BY n  (index arithmetic).
    if let Some(ud) = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "UP" || w == "DOWN"))
    {
        let up = matches!(stmt.get(ud), Some(Tok::Word(w)) if w == "UP");
        if !matches!(stmt.get(ud + 1), Some(Tok::Word(w)) if w == "BY") {
            return Err(RunError::Unsupported(
                "SET ... UP/DOWN must be followed by BY".into(),
            ));
        }
        let amount = match stmt.get(ud + 2) {
            Some(Tok::Word(w)) => resolve_int(w, fields)
                .ok_or_else(|| RunError::Unsupported(format!("SET ... BY {w}: not an integer")))?,
            _ => return Err(RunError::Unsupported("SET ... BY: missing amount".into())),
        };
        for name in stmt[..ud].iter().filter_map(|t| {
            if let Tok::Word(w) = t {
                Some(w.clone())
            } else {
                None
            }
        }) {
            let cur = resolve_int(&name, fields).ok_or_else(|| {
                RunError::Unsupported(format!("SET {name} UP/DOWN BY: not a numeric index"))
            })?;
            let nv = if up { cur + amount } else { cur - amount };
            let mv = vec![
                Tok::Word(nv.to_string()),
                Tok::Word("TO".to_string()),
                Tok::Word(name),
            ];
            exec_move(&mv, fields, decimal_comma)?;
        }
        return Ok(());
    }
    // Every valid SET form has either `TO` or `UP|DOWN BY`; a SET with neither is a cobc syntax error
    // ("unexpected ..."), so refusing it is faithful validation, not a feature gap.
    let to = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "TO"))
        .ok_or_else(|| {
            RunError::Unsupported(
                "SET: expected `TO` or `UP|DOWN BY` (cobc rejects a SET with neither)".into(),
            )
        })?;
    let targets: Vec<String> = stmt[..to]
        .iter()
        .filter_map(|t| match t {
            Tok::Word(w) => Some(w.clone()),
            _ => None,
        })
        .collect();
    if targets.is_empty() {
        return Err(RunError::Unsupported("SET: no target before TO".into()));
    }
    // form: SET <switch-mnemonic> [...] TO ON|OFF -- toggle a SPECIAL-NAMES UPSI switch at runtime (the
    // condition-name predicates then read the new state).
    if let Some(Tok::Word(v)) = stmt.get(to + 1) {
        if (v == "ON" || v == "OFF") && targets.iter().all(|t| switches.mnemonics.contains_key(t)) {
            let on = v == "ON";
            let mut st = switches.states.borrow_mut();
            for t in &targets {
                if let Some(&idx) = switches.mnemonics.get(t) {
                    if idx < st.len() {
                        st[idx] = on;
                    }
                }
            }
            return Ok(());
        }
    }
    // form: SET ptr TO ADDRESS OF field -- record the pointer's target for FUNCTION CONTENT-OF/-LENGTH.
    if matches!(stmt.get(to + 1), Some(Tok::Word(w)) if w == "ADDRESS")
        && matches!(stmt.get(to + 2), Some(Tok::Word(w)) if w == "OF")
    {
        if let Some(Tok::Word(target)) = stmt.get(to + 3) {
            for name in &targets {
                POINTER_TARGETS.with(|m| m.borrow_mut().insert(name.clone(), target.clone()));
            }
        }
        return Ok(());
    }
    // form: SET cond-name [...] TO FALSE  (store the 88's `WHEN SET TO FALSE` value into the parent).
    if matches!(stmt.get(to + 1), Some(Tok::Word(w)) if w == "FALSE") {
        for name in &targets {
            let (parent, fw) = match fields.get(name) {
                Some(Field {
                    storage:
                        Storage::Condition {
                            parent,
                            false_value: Some(fw),
                            ..
                        },
                    ..
                }) => (parent.clone(), fw.clone()),
                Some(Field {
                    storage:
                        Storage::Condition {
                            false_value: None, ..
                        },
                    ..
                }) => {
                    return Err(RunError::Unsupported(format!(
                        "SET {name} TO FALSE: the 88 has no `WHEN SET TO FALSE` value"
                    )))
                }
                Some(_) => {
                    return Err(RunError::Unsupported(format!(
                        "SET {name} TO FALSE: not an 88 condition-name"
                    )))
                }
                None => return Err(RunError::UndefinedName(name.clone())),
            };
            let src = match fw.strip_prefix('\u{1}') {
                Some(rest) => Tok::Str(rest.as_bytes().to_vec()),
                None => Tok::Word(fw),
            };
            let mv = vec![src, Tok::Word("TO".to_string()), Tok::Word(parent)];
            exec_move(&mv, fields, decimal_comma)?;
        }
        return Ok(());
    }
    // form: SET idx [idx ...] TO value  (set an index/numeric item to a literal or another item's value).
    if !matches!(stmt.get(to + 1), Some(Tok::Word(w)) if w == "TRUE") {
        let src = stmt
            .get(to + 1)
            .cloned()
            .ok_or_else(|| RunError::Unsupported("SET ... TO: missing value".into()))?;
        for name in &targets {
            match fields.get(name) {
                Some(Field {
                    storage: Storage::Numeric(_),
                    ..
                }) => {
                    let mv = vec![
                        src.clone(),
                        Tok::Word("TO".to_string()),
                        Tok::Word(name.clone()),
                    ];
                    exec_move(&mv, fields, decimal_comma)?;
                }
                Some(Field {
                    storage: Storage::Condition { .. },
                    ..
                }) => {
                    return Err(RunError::Unsupported(format!(
                        "SET {name} TO <value>: an 88 condition-name is only `SET ... TO TRUE`"
                    )))
                }
                Some(_) => {
                    return Err(RunError::Unsupported(format!(
                        "SET {name} TO <value>: target is not a numeric/index item"
                    )))
                }
                None => return Err(RunError::UndefinedName(name.clone())),
            }
        }
        return Ok(());
    }
    // form: SET cond-name [cond-name ...] TO TRUE  (LEVEL-88 construction).
    for name in targets {
        let (parent, setword) = match fields.get(&name) {
            Some(Field {
                storage: Storage::Condition { parent, values, .. },
                ..
            }) => {
                let v = values.first().ok_or_else(|| {
                    RunError::Unsupported(format!("88 {name} has no VALUE to SET"))
                })?;
                let w = match v {
                    CondVal::Single(s) => s.clone(),
                    CondVal::Range(lo, _) => lo.clone(),
                };
                (parent.clone(), w)
            }
            Some(_) => {
                return Err(RunError::Unsupported(format!(
                    "SET {name} TO TRUE: not an 88 condition-name"
                )))
            }
            None => return Err(RunError::UndefinedName(name)),
        };
        // decode the stored condition word back into a source token (the `\u{1}` prefix marks a string
        // literal; otherwise it is a numeric/word literal), then MOVE it into the parent.
        let src = match setword.strip_prefix('\u{1}') {
            Some(rest) => Tok::Str(rest.as_bytes().to_vec()),
            None => Tok::Word(setword),
        };
        let mv = vec![src, Tok::Word("TO".to_string()), Tok::Word(parent)];
        exec_move(&mv, fields, decimal_comma)?;
    }
    Ok(())
}

/// The data category an `INITIALIZE ... REPLACING` clause targets. At runtime PIC A and PIC X share one
/// storage type, so ALPHABETIC vs ALPHANUMERIC is resolved via `ALPHABETIC_FIELDS` (the compile-time PIC).
#[derive(Clone, Copy, PartialEq, Eq)]
enum InitCat {
    Numeric,
    Alphanumeric,
    Alphabetic,
    NumericEdited,
    AlphanumericEdited,
}

/// `INITIALIZE item [item ...] [REPLACING cat [DATA] BY val ...]`. Without REPLACING each leaf is reset to
/// its category default (numeric -> ZERO, alphanumeric/edited -> SPACES; VALUE is deliberately NOT used,
/// per the standard). With REPLACING, ONLY the leaves whose category is named are set to that category's
/// value (`cat` in {NUMERIC, ALPHANUMERIC, ALPHABETIC, NUMERIC-EDITED, ALPHANUMERIC-EDITED}); leaves of an
/// unnamed category are left UNCHANGED. `WITH`/`THEN`/`TO VALUE` and OCCURS-table targets fail closed.
fn exec_initialize(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    decimal_comma: bool,
) -> Result<(), RunError> {
    // `INITIALIZE items [WITH FILLER] ALL TO VALUE` -- restore each leaf to its VALUE clause (a leaf with no
    // VALUE is left unchanged). Detected before the head parser, which otherwise rejects ALL/TO/WITH/FILLER.
    if let Some(tp) = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "TO"))
    {
        if matches!(stmt.get(tp + 1), Some(Tok::Word(w)) if w == "VALUE") {
            return exec_initialize_to_value(stmt, tp, fields, decimal_comma);
        }
        // `INITIALIZE items [ALL] TO DEFAULT [THEN] [REPLACING ...]` -- TO DEFAULT is the standard
        // default behaviour (each leaf to its category default), identical to a bare INITIALIZE. Reduce
        // it to the canonical `items [REPLACING ...]` and reuse the default path. (WITH FILLER, which
        // would also init FILLER leaves, stays out of subset and falls through to the guard below.)
        if matches!(stmt.get(tp + 1), Some(Tok::Word(w)) if w == "DEFAULT")
            && !stmt
                .iter()
                .any(|t| matches!(t, Tok::Word(w) if w == "FILLER"))
        {
            let mod_start = stmt
                .iter()
                .position(|t| matches!(t, Tok::Word(w) if w == "ALL" || w == "TO"))
                .unwrap_or(tp);
            let mut rebuilt: Vec<Tok> = stmt[..mod_start].to_vec();
            if let Some(rp) = stmt
                .iter()
                .position(|t| matches!(t, Tok::Word(w) if w == "REPLACING"))
                .filter(|&p| p > tp)
            {
                rebuilt.push(Tok::Word("REPLACING".to_string()));
                rebuilt.extend_from_slice(&stmt[rp + 1..]);
            }
            return exec_initialize(&rebuilt, fields, decimal_comma);
        }
    }
    let repl_pos = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "REPLACING"));
    let head = match repl_pos {
        Some(p) => &stmt[..p],
        None => stmt,
    };
    // `WITH FILLER` includes FILLER leaves (a plain INITIALIZE excludes them, per cobc); a leading `THEN`
    // before REPLACING is a no-op (`items THEN REPLACING` == `items REPLACING`). Strip both from the
    // item-name region. A bare TO/THRU/ALL here (TO VALUE / TO DEFAULT / ALL TO VALUE are handled above) is
    // a placement cobc itself rejects -- faithful validation, not a feature gap.
    // NB: the canonicalize pass rewrites the `FILLER` clause keyword to a unique FILLER data-key
    // (`FILLER\u{1}..`), so match it via `bare_name`, not the literal string.
    let with_filler = head.windows(2).any(|w| {
        matches!(&w[0], Tok::Word(a) if a.as_str() == "WITH")
            && matches!(&w[1], Tok::Word(b) if bare_name(b) == "FILLER")
    });
    let mut names: Vec<String> = Vec::new();
    for t in head {
        match t {
            Tok::Word(w)
                if w.as_str() == "WITH" || w.as_str() == "THEN" || bare_name(w) == "FILLER" => {}
            Tok::Word(w) if matches!(w.as_str(), "TO" | "THRU" | "ALL") => {
                return Err(RunError::Unsupported(format!(
                    "INITIALIZE ... {w}: cobc rejects this clause placement"
                )));
            }
            Tok::Word(w) => names.push(w.clone()),
            _ => {}
        }
    }
    if names.is_empty() {
        return Err(RunError::Unsupported("INITIALIZE: no item named".into()));
    }

    // Both forms flatten the named items to their elementary leaves (OCCURS tables expand to subscripted
    // element leaves). Without REPLACING each leaf gets its category default; with REPLACING only leaves of
    // a named category get that category's value.
    let repl = match repl_pos {
        Some(rp) => Some(parse_initialize_replacing(&stmt[rp + 1..])?),
        None => None,
    };
    for name in &names {
        let mut leaves = Vec::new();
        collect_init_leaves(name, fields, &mut leaves)?;
        for leaf in leaves {
            // A plain INITIALIZE leaves FILLER untouched; only `WITH FILLER` initializes it (cobc).
            if !with_filler && bare_name(&leaf) == "FILLER" {
                continue;
            }
            let cat = init_field_category(&leaf, fields);
            let src = match &repl {
                Some(pairs) => match cat.and_then(|c| pairs.iter().find(|(cc, _)| *cc == c)) {
                    Some((_, val)) => val.clone(),
                    None => continue, // category not named -> leaf left unchanged
                },
                None => match cat {
                    Some(InitCat::Numeric) => Tok::Word("0".to_string()),
                    Some(_) => Tok::Str(vec![b' ']), // alphanumeric / alphabetic / edited -> spaces
                    None => continue,                // 88 / group -> skip
                },
            };
            let mv = vec![src, Tok::Word("TO".to_string()), Tok::Word(leaf.clone())];
            exec_move(&mv, fields, decimal_comma)?;
        }
    }
    Ok(())
}

/// `INITIALIZE items [WITH FILLER] ALL TO VALUE` (`tp` is the `TO` position): each elementary leaf that
/// declared a `VALUE` is restored to that VALUE image (captured in `FIELD_VALUES`); a leaf with no VALUE is
/// left UNCHANGED -- matching cobc. The subset is `ALL TO VALUE` only; `category TO VALUE`, `TO DEFAULT`,
/// a trailing `THEN`/`REPLACING`, and OCCURS-table targets fail closed.
fn exec_initialize_to_value(
    stmt: &[Tok],
    tp: usize,
    fields: &mut HashMap<String, Field>,
    decimal_comma: bool,
) -> Result<(), RunError> {
    // Trailing `[THEN] REPLACING cat BY val ...` after TO VALUE: TO VALUE sets each leaf that HAS a VALUE to
    // its VALUE; REPLACING then sets each leaf WITHOUT a VALUE whose category is named to that value (a leaf
    // with neither is left unchanged). Any other trailing clause (e.g. TO DEFAULT) is out of subset.
    let repl_pos = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "REPLACING"))
        .filter(|&p| p > tp);
    let mut to_default = false;
    let repl = match repl_pos {
        Some(rp) => Some(parse_initialize_replacing(&stmt[rp + 1..])?),
        None => {
            // Trailing region after `TO VALUE` (THEN is a no-op connector): nothing, or `[THEN] TO DEFAULT`
            // -- TO VALUE restores the valued leaves and TO DEFAULT then defaults the no-VALUE leaves. Any
            // other trailing is a placement cobc itself rejects (faithful validation).
            let rest: Vec<&Tok> = stmt[(tp + 2).min(stmt.len())..]
                .iter()
                .filter(|t| !matches!(t, Tok::Word(w) if w == "THEN"))
                .collect();
            match rest.as_slice() {
                [] => {}
                [Tok::Word(a), Tok::Word(b)] if a == "TO" && b == "DEFAULT" => to_default = true,
                _ => return Err(RunError::Unsupported(
                    "INITIALIZE ... TO VALUE: cobc rejects this trailing clause (only `[THEN] REPLACING ...` or `[THEN] TO DEFAULT`)".into())),
            }
            None
        }
    };
    // The modifier before TO is `ALL` or a category (NUMERIC/ALPHANUMERIC/...). cobc 3.2 IGNORES the
    // category for TO VALUE -- every leaf with a VALUE is restored regardless -- so all forms are equivalent.
    // A bare `TO VALUE` with no ALL/category is a cobc syntax error ("unexpected VALUE, expecting DEFAULT"),
    // so refusing it is faithful validation, not a feature gap.
    let modifier = matches!(stmt.get(tp.wrapping_sub(1)),
        Some(Tok::Word(w)) if w == "ALL" || init_cat_from_kw(w).is_some());
    if !modifier {
        return Err(RunError::Unsupported("INITIALIZE ... TO VALUE requires ALL or a category before VALUE (cobc rejects a bare `TO VALUE`)".into()));
    }
    // Item names run up to the modifier region (`WITH FILLER` / `ALL` / a category keyword).
    let mod_start = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "WITH" || w == "ALL" || init_cat_from_kw(w).is_some()))
        .unwrap_or(tp);
    let names: Vec<String> = stmt[..mod_start]
        .iter()
        .filter_map(|t| match t {
            Tok::Word(w) => Some(w.clone()),
            _ => None,
        })
        .collect();
    if names.is_empty() {
        return Err(RunError::Unsupported(
            "INITIALIZE ... TO VALUE: no item named".into(),
        ));
    }
    for name in &names {
        let mut leaves = Vec::new();
        collect_init_leaves(name, fields, &mut leaves)?;
        for leaf in leaves {
            // For a table cell `E(i)` the VALUE image is captured under the base name `E` (one element); a
            // scalar leaf is captured under its own name.
            let base = split_subscript(&leaf).0;
            if let Some(img) = FIELD_VALUES.with(|m| m.borrow().get(base).cloned()) {
                // has a VALUE -> restore it (TO VALUE wins over REPLACING).
                write_field(fields, &leaf, |f| {
                    let n = img.len().min(f.bytes.len());
                    f.bytes[..n].copy_from_slice(&img[..n]);
                    Ok(())
                })?;
            } else if let Some(pairs) = &repl {
                // no VALUE -> a trailing REPLACING sets it by category; an unnamed category is left unchanged.
                let cat = init_field_category(&leaf, fields);
                if let Some((_, val)) = cat.and_then(|c| pairs.iter().find(|(cc, _)| *cc == c)) {
                    let mv = vec![
                        val.clone(),
                        Tok::Word("TO".to_string()),
                        Tok::Word(leaf.clone()),
                    ];
                    exec_move(&mv, fields, decimal_comma)?;
                }
            } else if to_default && bare_name(&leaf) != "FILLER" {
                // no VALUE + trailing `TO DEFAULT` -> set the leaf to its category default (FILLER excluded).
                let src = match init_field_category(&leaf, fields) {
                    Some(InitCat::Numeric) => Tok::Word("0".to_string()),
                    Some(_) => Tok::Str(vec![b' ']),
                    None => continue,
                };
                let mv = vec![src, Tok::Word("TO".to_string()), Tok::Word(leaf.clone())];
                exec_move(&mv, fields, decimal_comma)?;
            }
            // no VALUE and no matching REPLACING/TO DEFAULT -> left unchanged.
        }
    }
    Ok(())
}

/// Parse the `cat [DATA] BY val [cat [DATA] BY val ...]` tail of INITIALIZE REPLACING into (category, value).
fn parse_initialize_replacing(toks: &[Tok]) -> Result<Vec<(InitCat, Tok)>, RunError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let cat = match toks.get(i) {
            Some(Tok::Word(w)) => init_cat_from_kw(w).ok_or_else(|| RunError::Unsupported(format!(
                "INITIALIZE REPLACING: unrecognized category `{w}` (expected NUMERIC/ALPHANUMERIC/ALPHABETIC/NUMERIC-EDITED/ALPHANUMERIC-EDITED)"
            )))?,
            _ => return Err(RunError::Unsupported("INITIALIZE REPLACING: expected a category".into())),
        };
        i += 1;
        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "DATA") {
            i += 1;
        }
        if !matches!(toks.get(i), Some(Tok::Word(w)) if w == "BY") {
            return Err(RunError::Unsupported(
                "INITIALIZE REPLACING: expected BY".into(),
            ));
        }
        i += 1;
        let val = toks.get(i).cloned().ok_or_else(|| {
            RunError::Unsupported("INITIALIZE REPLACING: missing replacement value".into())
        })?;
        out.push((cat, val));
        i += 1;
    }
    if out.is_empty() {
        return Err(RunError::Unsupported(
            "INITIALIZE REPLACING: no category given".into(),
        ));
    }
    Ok(out)
}

fn init_cat_from_kw(w: &str) -> Option<InitCat> {
    Some(match w {
        "NUMERIC" => InitCat::Numeric,
        "ALPHANUMERIC" => InitCat::Alphanumeric,
        "ALPHABETIC" => InitCat::Alphabetic,
        "NUMERIC-EDITED" => InitCat::NumericEdited,
        "ALPHANUMERIC-EDITED" => InitCat::AlphanumericEdited,
        _ => return None,
    })
}

/// The INITIALIZE category of an elementary leaf: numeric, alphabetic (PIC A), alphanumeric (PIC X), or the
/// two edited categories (split by whether the edited PIC carries an X/A). `None` for non-data leaves.
fn init_field_category(name: &str, fields: &HashMap<String, Field>) -> Option<InitCat> {
    let base = split_subscript(name).0;
    match &fields.get(base)?.storage {
        Storage::Numeric(_) => Some(InitCat::Numeric),
        Storage::Alpha(_) => Some(if ALPHABETIC_FIELDS.with(|s| s.borrow().contains(base)) {
            InitCat::Alphabetic
        } else {
            InitCat::Alphanumeric
        }),
        Storage::Edited(pic, ..) => {
            let up = pic.to_ascii_uppercase();
            Some(if up.contains('X') || up.contains('A') {
                InitCat::AlphanumericEdited
            } else {
                InitCat::NumericEdited
            })
        }
        _ => None,
    }
}

/// Flatten an INITIALIZE target into its elementary leaf names (recursing groups). OCCURS tables and
/// group-OCCURS child views are out of the REPLACING subset and fail closed (no silent partial init).
/// Push `base(s1,..,sk)` for every subscript combination of a multi-dimension leaf's `dims`
/// (`(occurs, stride)` outermost-first) -- used to expand a nested table for INITIALIZE.
fn enumerate_combos(base: &str, dims: &[(usize, usize)], out: &mut Vec<String>) {
    let occs: Vec<usize> = dims.iter().map(|&(o, _)| o).collect();
    let total: usize = occs.iter().product::<usize>().max(1);
    for combo in 0..total {
        let mut rem = combo;
        let mut subs = vec![0usize; dims.len()];
        for d in (0..dims.len()).rev() {
            subs[d] = rem % occs[d] + 1;
            rem /= occs[d];
        }
        let s = subs
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");
        out.push(format!("{base}({s})"));
    }
}

fn collect_init_leaves(
    name: &str,
    fields: &HashMap<String, Field>,
    out: &mut Vec<String>,
) -> Result<(), RunError> {
    let (base, sub) = split_subscript(name);
    // An already-subscripted leaf (produced by OCCURS expansion below) is a single leaf.
    if sub.is_some() {
        out.push(name.to_string());
        return Ok(());
    }
    // A multi-dimension leaf expands to one subscripted leaf per (i,j[,k]) combination.
    if let Some((_, _, _, dims)) = nested_leaf_lookup(base) {
        enumerate_combos(base, &dims, out);
        return Ok(());
    }
    // A nested-table BASE group (the outer group-OCCURS that owns the interleaved buffer): enumerate ALL its
    // multi-dimension leaves across their dims. (Its intermediate sub-groups own no field, so we cannot
    // recurse the children list -- the leaves come straight from NESTED_LEAF, in deterministic name order.)
    let mut nleaves: Vec<(String, Vec<(usize, usize)>)> = NESTED_LEAF.with(|m| {
        m.borrow()
            .iter()
            .filter(|(_, (b, _, _, _))| b == base)
            .map(|(n, (_, _, _, dims))| (n.clone(), dims.clone()))
            .collect()
    });
    if !nleaves.is_empty() {
        nleaves.sort();
        for (leaf, dims) in nleaves {
            enumerate_combos(&leaf, &dims, out);
        }
        return Ok(());
    }
    let f = fields
        .get(base)
        .ok_or_else(|| RunError::UndefinedName(base.to_string()))?;
    // A FLAT OCCURS table expands to one subscripted leaf per element: an elementary table is `base(i)`; a
    // single-level group-OCCURS of elementary children is each `child(i)`. A table with nested-leaf or
    // sub-group children is NOT flat -- it falls through to the general recursion (each nested leaf is
    // enumerated via the multi-dimension branch above; each sub-group recurses).
    if f.occurs > 1 {
        let flat = match &f.storage {
            Storage::Group { children } => children.iter().all(|c| {
                nested_leaf_lookup(c).is_none()
                    && matches!(
                        fields.get(c).map(|x| &x.storage),
                        Some(Storage::Numeric(_) | Storage::Alpha(_) | Storage::Edited(..))
                    )
            }),
            Storage::Numeric(_) | Storage::Alpha(_) | Storage::Edited(..) => true,
            _ => false,
        };
        if flat {
            match &f.storage {
                Storage::Group { children } => {
                    let kids = children.clone();
                    for i in 1..=f.occurs {
                        for c in &kids {
                            out.push(format!("{c}({i})"));
                        }
                    }
                }
                _ => {
                    for i in 1..=f.occurs {
                        out.push(format!("{base}({i})"));
                    }
                }
            }
            return Ok(());
        }
        // not flat: fall through to the general recursion (a nested group-OCCURS).
    }
    if f.occurs <= 1 && group_child_lookup(base).is_some() {
        // cobc COMPILE-rejects a bare table-element name here ("'K' requires one subscript"), so refusing it
        // is faithful validation, not a feature gap.
        return Err(RunError::Unsupported(format!(
            "INITIALIZE over OCCURS child `{base}`: a table element requires a subscript (cobc rejects the bare name)"
        )));
    }
    match &f.storage {
        Storage::Group { children } => {
            let kids = children.clone();
            for c in kids {
                collect_init_leaves(&c, fields, out)?;
            }
        }
        Storage::Condition { .. } => {}
        _ => out.push(base.to_string()),
    }
    Ok(())
}

/// True when a PICTURE is ALPHABETIC (only `A` position symbols, no `X`); used to retain the compile-time
/// category that PIC A and PIC X share at runtime.
fn pic_is_alphabetic(pic: &str) -> bool {
    let up = pic.to_ascii_uppercase();
    up.contains('A') && !up.contains('X')
}

/// An `INSPECT` comparand operand -> its bytes: a string literal, the figuratives `SPACE`/`ZERO` (a single
/// character), or an identifier's current bytes. (Other figuratives/forms fail closed.)
fn inspect_operand(t: Option<&Tok>, fields: &HashMap<String, Field>) -> Result<Vec<u8>, RunError> {
    match t {
        Some(Tok::Str(s)) => Ok(s.clone()),
        Some(Tok::Word(w)) => {
            // A figurative constant is a single character here (SPACE / ZERO / HIGH-VALUE / LOW-VALUE /
            // QUOTE -> 0x20 / 0x30 / 0xFF / 0x00 / 0x22); cobc treats it as a 1-byte comparand/replacement.
            if let Some(fig) = figurative_kind(w) {
                return Ok(vec![fig_byte(fig)]);
            }
            // Not a literal, figurative, or known field -> an undeclared data name (cobc rejects it too).
            match read_field(fields, w)? {
                Some(f) => Ok(f.bytes.clone()),
                None => Err(RunError::UndefinedName(w.clone())),
            }
        }
        _ => Err(RunError::Unsupported("INSPECT: missing operand".into())),
    }
}

/// Parse a trailing `INSPECT` region clause into `(kind, delim)` -- `0`=whole, `1`=`BEFORE INITIAL d`,
/// `2`=`AFTER INITIAL d` -- returning the delimiter bytes owned so the caller can build a `Region`.
fn inspect_region(
    rest: &[Tok],
    fields: &HashMap<String, Field>,
) -> Result<(u8, Vec<u8>), RunError> {
    match rest.first() {
        None => Ok((0, Vec::new())),
        Some(Tok::Word(w)) if w == "BEFORE" || w == "AFTER" => {
            let kind = if w == "AFTER" { 2 } else { 1 };
            let mut i = 1;
            if matches!(rest.get(i), Some(Tok::Word(x)) if x == "INITIAL") {
                i += 1;
            }
            Ok((kind, inspect_operand(rest.get(i), fields)?))
        }
        Some(t) => Err(RunError::Unsupported(format!(
            "INSPECT region clause near {t:?}"
        ))),
    }
}

/// Upstream `validate_inspect` (04614ac7a): a REPLACING/CONVERTING operand pair whose operands are
/// both non-const must be equal in size; cobc rejects a mismatch at compile time ("operands
/// incompatible" / note "operands differ in size") and accepts identical operands (warning; the
/// operation degrades to a natural no-op). The candidate mirrors the accept/reject semantics at
/// check time (the wording differs; that is a diagnostic-shape matter).
fn validate_inspect_operands(
    stmt: &[Tok],
    fields: &HashMap<String, Field>,
) -> Result<(), RunError> {
    let mut i = 1usize;
    while i < stmt.len() {
        match stmt.get(i) {
            Some(Tok::Word(w)) if w == "REPLACING" => {
                i += 1;
                let mode = match stmt.get(i) {
                    Some(Tok::Word(m)) => m.clone(),
                    _ => return Ok(()), // malformed; exec_inspect reports it
                };
                match mode.as_str() {
                    "CHARACTERS" => {
                        if matches!(stmt.get(i + 1), Some(Tok::Word(b)) if b == "BY") {
                            i += 2; // BY y
                        } else {
                            return Ok(());
                        }
                    }
                    "ALL" | "LEADING" | "FIRST" => {
                        if matches!(stmt.get(i + 2), Some(Tok::Word(b)) if b == "BY") {
                            inspect_pair_check(
                                stmt.get(i + 1),
                                stmt.get(i + 3),
                                fields,
                                "REPLACING",
                            )?;
                            i += 4;
                        } else {
                            return Ok(());
                        }
                    }
                    _ => return Ok(()),
                }
            }
            Some(Tok::Word(w)) if w == "CONVERTING" => {
                if matches!(stmt.get(i + 2), Some(Tok::Word(b)) if b == "TO") {
                    inspect_pair_check(stmt.get(i + 1), stmt.get(i + 3), fields, "CONVERTING")?;
                    i += 4;
                } else {
                    return Ok(());
                }
            }
            _ => i += 1,
        }
    }
    Ok(())
}

/// One REPLACING/CONVERTING operand pair. Upstream size rules: a FROM figurative constant is
/// 1 byte; a TO figurative constant skips the comparison (its runtime size is the replaced item's);
/// identical field operands are accepted; otherwise non-const sizes must match or the compiler
/// rejects the statement.
fn inspect_pair_check(
    x: Option<&Tok>,
    y: Option<&Tok>,
    fields: &HashMap<String, Field>,
    clause: &str,
) -> Result<(), RunError> {
    let from_size = |t: Option<&Tok>| -> Result<Option<(usize, Option<String>)>, RunError> {
        match t {
            Some(Tok::Str(s)) => Ok(Some((s.len(), None))),
            Some(Tok::Word(w)) => {
                if figurative_kind(w).is_some() {
                    Ok(Some((1, None))) // const: a 1-byte comparand per cobc
                } else {
                    match read_field(fields, w)? {
                        Some(f) => Ok(Some((f.bytes.len(), Some(w.clone())))),
                        None => Err(RunError::UndefinedName(w.clone())),
                    }
                }
            }
            _ => Ok(None),
        }
    };
    // A figurative TO operand is const-like: its runtime size is the replaced item's, so the
    // comparison is skipped (upstream `tag_y != CB_TAG_CONST` guard).
    let to_size = |t: Option<&Tok>| -> Result<Option<(usize, Option<String>)>, RunError> {
        match t {
            Some(Tok::Str(s)) => Ok(Some((s.len(), None))),
            Some(Tok::Word(w)) => {
                if figurative_kind(w).is_some() {
                    Ok(None)
                } else {
                    match read_field(fields, w)? {
                        Some(f) => Ok(Some((f.bytes.len(), Some(w.clone())))),
                        None => Err(RunError::UndefinedName(w.clone())),
                    }
                }
            }
            _ => Ok(None),
        }
    };
    let (sx, fx) = match from_size(x)? {
        Some(v) => v,
        None => return Ok(()),
    };
    let (sy, fy) = match to_size(y)? {
        Some(v) => v,
        None => return Ok(()),
    };
    if fx.is_some() && fx == fy {
        return Ok(()); // identical operands: upstream warns; the operation degrades to a no-op
    }
    if sx != sy {
        return Err(RunError::Unsupported(format!(
            "INSPECT {clause} operands incompatible (operands differ in size)"
        )));
    }
    Ok(())
}

/// `INSPECT target {TALLYING counter FOR <ALL|LEADING> lit | FOR CHARACTERS [region] | REPLACING
/// <ALL|LEADING|FIRST> x BY y [region] | CONVERTING from TO to [region]}` -- the byte effects of the
/// sealed `GNURUST.INSPECT.1` court. A single clause is in the subset; multi-clause/`ALL`-counter-chains
/// and figurative ranges fail closed.
fn exec_inspect(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    decimal_comma: bool,
) -> Result<(), RunError> {
    use crate::inspect::{
        inspect_converting, inspect_replacing, inspect_tallying, Region, ReplaceMode, TallyMode,
    };
    let target = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("INSPECT: missing target".into())),
    };
    let target_bytes = read_field(fields, &target)?
        .ok_or_else(|| RunError::UndefinedName(target.clone()))?
        .bytes;
    // Multi-clause `INSPECT id TALLYING ... REPLACING ...`: the standard applies the TALLYING phrase then
    // the REPLACING phrase as two operations on the ORIGINAL value. Split at REPLACING and run each (the
    // tally pass leaves the field unchanged, so the replace pass still sees the original bytes).
    if matches!(stmt.get(1), Some(Tok::Word(w)) if w == "TALLYING") {
        if let Some(rp) = stmt
            .iter()
            .position(|t| matches!(t, Tok::Word(w) if w == "REPLACING"))
        {
            exec_inspect(&stmt[..rp], fields, decimal_comma)?;
            let mut rest = vec![Tok::Word(target.clone())];
            rest.extend_from_slice(&stmt[rp..]);
            return exec_inspect(&rest, fields, decimal_comma);
        }
    }
    match stmt.get(1) {
        Some(Tok::Word(w)) if w == "TALLYING" => {
            let counter = match stmt.get(2) {
                Some(Tok::Word(w)) => w.clone(),
                _ => {
                    return Err(RunError::Unsupported(
                        "INSPECT TALLYING: missing counter".into(),
                    ))
                }
            };
            if !matches!(stmt.get(3), Some(Tok::Word(w)) if w == "FOR") {
                return Err(RunError::Unsupported(
                    "INSPECT TALLYING: expected FOR".into(),
                ));
            }
            let modekw = match stmt.get(4) {
                Some(Tok::Word(w)) => w.clone(),
                _ => {
                    return Err(RunError::Unsupported(
                        "INSPECT TALLYING: missing FOR mode".into(),
                    ))
                }
            };
            let (item, rstart) = match modekw.as_str() {
                "CHARACTERS" => (Vec::new(), 5),
                "ALL" | "LEADING" => (inspect_operand(stmt.get(5), fields)?, 6),
                other => return Err(RunError::Unsupported(format!("INSPECT TALLYING FOR: unrecognized mode `{other}` (expected ALL/LEADING/CHARACTERS)"))),
            };
            let (rk, d) = inspect_region(&stmt[rstart.min(stmt.len())..], fields)?;
            let region = match rk {
                1 => Region::Before(&d),
                2 => Region::After(&d),
                _ => Region::All,
            };
            let mode = match modekw.as_str() {
                "CHARACTERS" => TallyMode::Characters,
                "ALL" => TallyMode::All(&item),
                _ => TallyMode::Leading(&item),
            };
            let count = inspect_tallying(&target_bytes, mode, region) as i64;
            let nv = resolve_int(&counter, fields).unwrap_or(0) + count;
            let mv = vec![
                Tok::Word(nv.to_string()),
                Tok::Word("TO".to_string()),
                Tok::Word(counter),
            ];
            exec_move(&mv, fields, decimal_comma)
        }
        Some(Tok::Word(w)) if w == "REPLACING" => {
            let modekw = match stmt.get(2) {
                Some(Tok::Word(w)) => w.clone(),
                _ => {
                    return Err(RunError::Unsupported(
                        "INSPECT REPLACING: missing mode".into(),
                    ))
                }
            };
            // CHARACTERS BY y [region] has NO search operand; ALL/LEADING/FIRST take `x BY y`.
            let (x, y, ystart) = if modekw == "CHARACTERS" {
                if !matches!(stmt.get(3), Some(Tok::Word(w)) if w == "BY") {
                    return Err(RunError::Unsupported(
                        "INSPECT REPLACING CHARACTERS: expected BY".into(),
                    ));
                }
                (Vec::new(), inspect_operand(stmt.get(4), fields)?, 5)
            } else if matches!(modekw.as_str(), "ALL" | "LEADING" | "FIRST") {
                let x = inspect_operand(stmt.get(3), fields)?;
                if !matches!(stmt.get(4), Some(Tok::Word(w)) if w == "BY") {
                    return Err(RunError::Unsupported(
                        "INSPECT REPLACING: expected BY".into(),
                    ));
                }
                (x, inspect_operand(stmt.get(5), fields)?, 6)
            } else {
                return Err(RunError::Unsupported(format!("INSPECT REPLACING: unrecognized mode `{modekw}` (expected CHARACTERS/ALL/LEADING/FIRST)")));
            };
            let (rk, d) = inspect_region(&stmt[ystart.min(stmt.len())..], fields)?;
            let region = match rk {
                1 => Region::Before(&d),
                2 => Region::After(&d),
                _ => Region::All,
            };
            let mode = match modekw.as_str() {
                "CHARACTERS" => ReplaceMode::Characters(&y),
                "ALL" => ReplaceMode::All(&x, &y),
                "LEADING" => ReplaceMode::Leading(&x, &y),
                _ => ReplaceMode::First(&x, &y),
            };
            let newb = inspect_replacing(&target_bytes, mode, region);
            write_field(fields, &target, |f| {
                if f.bytes.len() == newb.len() {
                    f.bytes = newb;
                    Ok(())
                } else {
                    Err(RunError::Runtime(
                        "INSPECT REPLACING changed field length".into(),
                    ))
                }
            })
        }
        Some(Tok::Word(w)) if w == "CONVERTING" => {
            let from = inspect_operand(stmt.get(2), fields)?;
            if !matches!(stmt.get(3), Some(Tok::Word(w)) if w == "TO") {
                return Err(RunError::Unsupported(
                    "INSPECT CONVERTING: expected TO".into(),
                ));
            }
            let to = inspect_operand(stmt.get(4), fields)?;
            let (rk, d) = inspect_region(&stmt[5.min(stmt.len())..], fields)?;
            let region = match rk {
                1 => Region::Before(&d),
                2 => Region::After(&d),
                _ => Region::All,
            };
            let newb = inspect_converting(&target_bytes, &from, &to, region);
            write_field(fields, &target, |f| {
                if f.bytes.len() == newb.len() {
                    f.bytes = newb;
                    Ok(())
                } else {
                    Err(RunError::Runtime(
                        "INSPECT CONVERTING changed field length".into(),
                    ))
                }
            })
        }
        other => Err(RunError::Unsupported(format!(
            "INSPECT: unrecognized clause {other:?} (expected TALLYING/REPLACING/CONVERTING)"
        ))),
    }
}

/// `EXHIBIT [CHANGED] [NAMED] name [name ...]` -- the OS/VS debug display, space-joined on one line.
/// cobc 3.2 does NOT implement the `CHANGED` (display-only-if-changed) suppression (`-Wpending`): it runs
/// as plain EXHIBIT, so we ignore the suppression too. The only observable effect of the keywords is the
/// item format: `NAME = <value>` UNLESS `CHANGED` is given WITHOUT `NAMED`, where just `<value>` is shown.
fn exec_exhibit(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let mut i = 0;
    let (mut named, mut changed) = (false, false);
    while let Some(Tok::Word(w)) = stmt.get(i) {
        match w.as_str() {
            "CHANGED" => {
                changed = true;
                i += 1;
            }
            "NAMED" => {
                named = true;
                i += 1;
            }
            _ => break,
        }
    }
    // cobc prints `NAME = value` for plain/NAMED EXHIBIT; only `CHANGED` without `NAMED` drops the name.
    let show_name = named || !changed;
    let names: Vec<String> = stmt[i..]
        .iter()
        .filter_map(|t| {
            if let Tok::Word(w) = t {
                Some(w.clone())
            } else {
                None
            }
        })
        .collect();
    let mut line = Vec::new();
    for (j, name) in names.iter().enumerate() {
        if j > 0 {
            line.push(b' ');
        }
        if show_name {
            line.extend_from_slice(name.as_bytes());
            line.extend_from_slice(b" = ");
        }
        let f = read_field(fields, name)?.ok_or_else(|| RunError::UndefinedName(name.clone()))?;
        line.extend_from_slice(&display_bytes(&f, ctx.decimal_comma));
    }
    line.push(b'\n');
    out.extend_from_slice(&line);
    Ok(())
}

/// `ALTER para TO [PROCEED TO] target [para2 TO ...]` -- retarget the `GO TO` in each named (alterable)
/// paragraph: record the paragraph's GO-token index -> the new target, consulted when that GO TO runs.
fn exec_alter(stmt: &[Tok]) -> Result<(), RunError> {
    let proc = CUR_PROC.with(|c| c.borrow().clone());
    let words: Vec<String> = stmt
        .iter()
        .filter_map(|t| {
            if let Tok::Word(w) = t {
                Some(w.clone())
            } else {
                None
            }
        })
        .collect();
    let mut i = 0;
    while i < words.len() {
        let para = words[i].clone();
        i += 1;
        while i < words.len() && (words[i] == "TO" || words[i] == "PROCEED") {
            i += 1;
        }
        if i >= words.len() {
            break;
        }
        let target = words[i].clone();
        i += 1;
        if let Some((start, end)) = para_range(&para, &para) {
            for gi in start..end.min(proc.len()) {
                if matches!(proc.get(gi), Some(Tok::Word(w)) if w == "GO") {
                    ALTERED.with(|c| {
                        c.borrow_mut().insert(gi, target.clone());
                    });
                    break;
                }
            }
        }
    }
    Ok(())
}

/// `ALLOCATE {id | n CHARACTERS} [INITIALIZED] [RETURNING ptr]` -- obtain BASED storage. With `INITIALIZED`
/// the based item is set to its category defaults (deterministic); the returned pointer address is a
/// non-claim (not displayed). Raw `n CHARACTERS` allocation has no observable item.
fn exec_allocate(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    decimal_comma: bool,
) -> Result<(), RunError> {
    let initialized = stmt
        .iter()
        .any(|t| matches!(t, Tok::Word(w) if w == "INITIALIZED"));
    let is_chars = stmt
        .iter()
        .any(|t| matches!(t, Tok::Word(w) if w == "CHARACTERS" || w == "CHARACTER"));
    if !is_chars && initialized {
        if let Some(Tok::Word(id)) = stmt.first() {
            exec_initialize(&[Tok::Word(id.clone())], fields, decimal_comma)?;
        }
    }
    Ok(())
}

/// `EXAMINE id TALLYING {ALL|LEADING|UNTIL FIRST} lit [REPLACING BY lit2]` / `EXAMINE id REPLACING
/// {ALL|LEADING|FIRST} lit BY lit2` -- the COBOL-68 precursor of INSPECT (an OS/VS dialect verb). TALLYING
/// sets the `TALLY` register; reuses the sealed INSPECT TALLYING/REPLACING courts.
fn exec_examine(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    decimal_comma: bool,
) -> Result<(), RunError> {
    use crate::inspect::{inspect_replacing, inspect_tallying, Region, ReplaceMode, TallyMode};
    let target = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("EXAMINE: missing field".into())),
    };
    let tbytes = read_field(fields, &target)?
        .map(|f| f.bytes)
        .unwrap_or_default();
    let pos_of = |kw: &str| {
        stmt.iter()
            .position(|t| matches!(t, Tok::Word(w) if w == kw))
    };
    let write_target =
        |fields: &mut HashMap<String, Field>, newb: Vec<u8>| -> Result<(), RunError> {
            write_field(fields, &target, |f| {
                if f.bytes.len() == newb.len() {
                    f.bytes = newb;
                    Ok(())
                } else {
                    Err(RunError::Runtime("EXAMINE changed field length".into()))
                }
            })
        };
    if let Some(tp) = pos_of("TALLYING") {
        let mut i = tp + 1;
        let modekw = match stmt.get(i) {
            Some(Tok::Word(w)) => w.clone(),
            _ => return Err(RunError::Unsupported("EXAMINE TALLYING mode".into())),
        };
        i += 1;
        if modekw == "UNTIL" && matches!(stmt.get(i), Some(Tok::Word(w)) if w == "FIRST") {
            i += 1;
        }
        let lit = inspect_operand(stmt.get(i), fields)?;
        let tmode = match modekw.as_str() {
            "ALL" => TallyMode::All(&lit),
            "LEADING" => TallyMode::Leading(&lit),
            "UNTIL" => TallyMode::Characters,
            other => {
                return Err(RunError::Unsupported(format!(
                "EXAMINE TALLYING: unrecognized mode `{other}` (expected ALL/LEADING/UNTIL FIRST)"
            )))
            }
        };
        let region = if modekw == "UNTIL" {
            Region::Before(&lit)
        } else {
            Region::All
        };
        let count = inspect_tallying(&tbytes, tmode, region) as i64;
        let mv = vec![
            Tok::Word(count.to_string()),
            Tok::Word("TO".to_string()),
            Tok::Word("TALLY".to_string()),
        ];
        exec_move(&mv, fields, decimal_comma)?;
        if let Some(rp) = stmt[tp..]
            .iter()
            .position(|t| matches!(t, Tok::Word(w) if w == "REPLACING"))
        {
            let mut j = tp + rp + 1;
            if matches!(stmt.get(j), Some(Tok::Word(w)) if w == "BY") {
                j += 1;
            }
            let lit2 = inspect_operand(stmt.get(j), fields)?;
            // The REPLACING mode mirrors the TALLYING mode; UNTIL FIRST replaces the chars before the
            // delimiter (the same span just tallied) via the CHARACTERS mode over the BEFORE region.
            let (rmode, rregion) = match modekw.as_str() {
                "ALL" => (ReplaceMode::All(&lit, &lit2), Region::All),
                "LEADING" => (ReplaceMode::Leading(&lit, &lit2), Region::All),
                "UNTIL" => (ReplaceMode::Characters(&lit2), Region::Before(&lit)),
                other => {
                    return Err(RunError::Unsupported(format!(
                        "EXAMINE TALLYING {other} REPLACING: unrecognized mode"
                    )))
                }
            };
            let newb = inspect_replacing(&tbytes, rmode, rregion);
            write_target(fields, newb)?;
        }
        return Ok(());
    }
    if let Some(rp) = pos_of("REPLACING") {
        let mut i = rp + 1;
        let modekw = match stmt.get(i) {
            Some(Tok::Word(w)) => w.clone(),
            _ => return Err(RunError::Unsupported("EXAMINE REPLACING mode".into())),
        };
        i += 1;
        if modekw == "UNTIL" && matches!(stmt.get(i), Some(Tok::Word(w)) if w == "FIRST") {
            i += 1;
        }
        let lit = inspect_operand(stmt.get(i), fields)?;
        i += 1;
        if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "BY") {
            i += 1;
        }
        let lit2 = inspect_operand(stmt.get(i), fields)?;
        let (rmode, rregion) = match modekw.as_str() {
            "ALL" => (ReplaceMode::All(&lit, &lit2), Region::All),
            "LEADING" => (ReplaceMode::Leading(&lit, &lit2), Region::All),
            "FIRST" => (ReplaceMode::First(&lit, &lit2), Region::All),
            "UNTIL" => (ReplaceMode::Characters(&lit2), Region::Before(&lit)),
            other => return Err(RunError::Unsupported(format!("EXAMINE REPLACING: unrecognized mode `{other}` (expected ALL/LEADING/FIRST/UNTIL FIRST)"))),
        };
        let newb = inspect_replacing(&tbytes, rmode, rregion);
        write_target(fields, newb)?;
        return Ok(());
    }
    Err(RunError::Unsupported(
        "EXAMINE: expected TALLYING or REPLACING".into(),
    ))
}

/// `TRANSFORM target FROM from TO to` -- the legacy form of `INSPECT target CONVERTING from TO to` (a
/// per-byte translation), reusing the sealed CONVERTING court.
fn exec_transform(stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
    use crate::inspect::{inspect_converting, Region};
    let target = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("TRANSFORM: missing target".into())),
    };
    let fp = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "FROM"))
        .ok_or_else(|| RunError::Unsupported("TRANSFORM without FROM".into()))?;
    let from = inspect_operand(stmt.get(fp + 1), fields)?;
    let tp = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "TO"))
        .ok_or_else(|| RunError::Unsupported("TRANSFORM without TO".into()))?;
    let to = inspect_operand(stmt.get(tp + 1), fields)?;
    let tb = read_field(fields, &target)?
        .map(|f| f.bytes)
        .unwrap_or_default();
    let newb = inspect_converting(&tb, &from, &to, Region::All);
    write_field(fields, &target, |f| {
        if f.bytes.len() == newb.len() {
            f.bytes = newb;
            Ok(())
        } else {
            Err(RunError::Runtime("TRANSFORM changed field length".into()))
        }
    })
}

/// `STRING <src [DELIMITED BY SIZE|lit]> ... INTO target [WITH POINTER p]` -- concatenate the sources into
/// the target at the 1-based pointer, preserving the unwritten tail (`GNURUST.STRING.UNSTRING.1`). The
/// `ON OVERFLOW` handler form is outside the subset (fails closed).
fn exec_string(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    decimal_comma: bool,
) -> Result<bool, RunError> {
    use crate::string_ops::{string_into, StringSource};
    let into = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "INTO"))
        .ok_or_else(|| RunError::Unsupported("STRING without INTO".into()))?;
    let target = match stmt.get(into + 1) {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("STRING: missing target".into())),
    };
    // optional WITH POINTER p (after the target)
    let mut pointer_name: Option<String> = None;
    let mut pointer = 1usize;
    if let Some(rel) = stmt[into + 2..]
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "POINTER"))
    {
        if let Some(Tok::Word(pn)) = stmt.get(into + 2 + rel + 1) {
            pointer = resolve_int(pn, fields).unwrap_or(1).max(1) as usize;
            pointer_name = Some(pn.clone());
        }
    }
    // parse sources (operands [DELIMITED BY {SIZE|lit}]); a DELIMITED BY applies to the preceding run.
    let mut pending: Vec<Vec<u8>> = Vec::new();
    let mut srcs: Vec<(Vec<u8>, Option<Vec<u8>>)> = Vec::new();
    let mut i = 0;
    while i < into {
        match &stmt[i] {
            Tok::Word(w) if w == "DELIMITED" => {
                i += 1;
                if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "BY") {
                    i += 1;
                }
                let delim = match stmt.get(i) {
                    Some(Tok::Word(w)) if w == "SIZE" => None,
                    other => Some(inspect_operand(other, fields)?),
                };
                i += 1;
                for op in pending.drain(..) {
                    srcs.push((op, delim.clone()));
                }
            }
            t => {
                pending.push(inspect_operand(Some(t), fields)?);
                i += 1;
            }
        }
    }
    for op in pending.drain(..) {
        srcs.push((op, None));
    }
    let ss: Vec<StringSource> = srcs
        .iter()
        .map(|(b, d)| match d {
            Some(d) => StringSource::Delimited(b, d),
            None => StringSource::Size(b),
        })
        .collect();
    let prefill = read_field(fields, &target)?
        .ok_or_else(|| RunError::UndefinedName(target.clone()))?
        .bytes;
    let res = string_into(&prefill, &ss, pointer);
    let overflow = res.overflow;
    let newb = res.target;
    write_field(fields, &target, |f| {
        if f.bytes.len() == newb.len() {
            f.bytes = newb;
            Ok(())
        } else {
            Err(RunError::Runtime("STRING changed target length".into()))
        }
    })?;
    if let Some(pn) = pointer_name {
        let mv = vec![
            Tok::Word(res.pointer.to_string()),
            Tok::Word("TO".to_string()),
            Tok::Word(pn),
        ];
        exec_move(&mv, fields, decimal_comma)?;
    }
    Ok(overflow) // `true` when the sources overran the target -> the ON OVERFLOW handler runs
}

/// `UNSTRING source [DELIMITED BY d] INTO f1 f2 ...` -- split the source by the delimiter (or by each
/// receiver's width when absent) into the alphanumeric receiving fields (`GNURUST.STRING.UNSTRING.1`).
/// `DELIMITER IN` / `COUNT IN` / `TALLYING IN` / `WITH POINTER` are supported; returns `true` when the
/// OVERFLOW condition holds (source characters remain after every receiver is filled), for the caller's
/// `ON OVERFLOW` / `NOT ON OVERFLOW` dispatch. `DELIMITED BY [ALL] d1 [OR [ALL] d2]...` multi-delimiter is
/// supported (earliest match splits; `ALL` collapses repeats; `DELIMITER IN` captures the matched one).
fn exec_unstring(stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<bool, RunError> {
    use crate::string_ops::unstring_multi;
    let into = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "INTO"))
        .ok_or_else(|| RunError::Unsupported("UNSTRING without INTO".into()))?;
    let source = inspect_operand(stmt.first(), fields)?;
    // optional `DELIMITED BY [ALL] d1 [OR [ALL] d2]...` between the source and INTO -> (delimiter, all) list.
    let mut delims: Vec<(Vec<u8>, bool)> = Vec::new();
    if let Some(dp) = stmt[..into]
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "DELIMITED"))
    {
        let mut j = dp + 1;
        if matches!(stmt.get(j), Some(Tok::Word(w)) if w == "BY") {
            j += 1;
        }
        loop {
            let all = matches!(stmt.get(j), Some(Tok::Word(w)) if w == "ALL");
            if all {
                j += 1;
            }
            if matches!(stmt.get(j), Some(Tok::Word(w)) if w == "SIZE") {
                j += 1; // DELIMITED BY SIZE: no delimiter for this alternative
            } else {
                delims.push((inspect_operand(stmt.get(j), fields)?, all));
                j += 1;
            }
            if matches!(stmt.get(j), Some(Tok::Word(w)) if w == "OR") {
                j += 1;
                continue;
            }
            break;
        }
    }
    // `[WITH] POINTER p` (a 1-based scan cursor, read in and written back) sits after the receivers.
    let ptr_pos = stmt[into + 1..]
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "POINTER"))
        .map(|p| into + 1 + p);
    let ptr_field: Option<String> = ptr_pos.and_then(|p| match stmt.get(p + 1) {
        Some(Tok::Word(w)) => Some(w.clone()),
        _ => None,
    });
    // The clause is written `[WITH] POINTER p`; the receiver list must stop before the optional `WITH`.
    let ptr_clause = ptr_pos.map(|p| {
        if matches!(stmt.get(p - 1), Some(Tok::Word(w)) if w == "WITH") {
            p - 1
        } else {
            p
        }
    });
    // The receiver list runs from INTO to TALLYING / POINTER (or end). Each receiver is `name [DELIMITER IN
    // d] [COUNT IN c]`; an optional trailing `TALLYING IN t` counts the filled fields (added to t).
    let tally_pos = stmt[into + 1..]
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "TALLYING"))
        .map(|p| into + 1 + p);
    let seg_end = tally_pos
        .unwrap_or(stmt.len())
        .min(ptr_clause.unwrap_or(stmt.len()));
    let seg = &stmt[into + 1..seg_end];
    let mut recvs: Vec<(String, Option<String>, Option<String>)> = Vec::new();
    let mut i = 0;
    while i < seg.len() {
        let Some(Tok::Word(name)) = seg.get(i) else {
            i += 1;
            continue;
        };
        let name = name.clone();
        i += 1;
        let (mut din, mut cin) = (None, None);
        loop {
            // `IN` is optional after DELIMITER / COUNT (cobc accepts `DELIMITER d` / `COUNT c` too).
            let in_at = |k: usize| matches!(seg.get(k), Some(Tok::Word(w)) if w == "IN");
            match seg.get(i) {
                Some(Tok::Word(w)) if w == "DELIMITER" => {
                    let k = if in_at(i + 1) { i + 2 } else { i + 1 };
                    din = Some(match seg.get(k) {
                        Some(Tok::Word(d)) => d.clone(),
                        _ => {
                            return Err(RunError::Unsupported(
                                "UNSTRING DELIMITER IN: missing field".into(),
                            ))
                        }
                    });
                    i = k + 1;
                }
                Some(Tok::Word(w)) if w == "COUNT" => {
                    let k = if in_at(i + 1) { i + 2 } else { i + 1 };
                    cin = Some(match seg.get(k) {
                        Some(Tok::Word(c)) => c.clone(),
                        _ => {
                            return Err(RunError::Unsupported(
                                "UNSTRING COUNT IN: missing field".into(),
                            ))
                        }
                    });
                    i = k + 1;
                }
                _ => break,
            }
        }
        recvs.push((name, din, cin));
    }
    if recvs.is_empty() {
        return Err(RunError::Unsupported("UNSTRING: no receiving field".into()));
    }
    let tally_field = match tally_pos {
        Some(tp) => {
            let mut j = tp + 1;
            if matches!(stmt.get(j), Some(Tok::Word(w)) if w == "IN") {
                j += 1;
            }
            Some(match stmt.get(j) {
                Some(Tok::Word(n)) => n.clone(),
                _ => {
                    return Err(RunError::Unsupported(
                        "UNSTRING TALLYING IN: missing field".into(),
                    ))
                }
            })
        }
        None => None,
    };
    let mut sizes = Vec::with_capacity(recvs.len());
    let mut numeric = Vec::with_capacity(recvs.len());
    for (n, _, _) in &recvs {
        let f = read_field(fields, n)?.ok_or_else(|| RunError::UndefinedName(n.clone()))?;
        // Alphanumeric receivers take the raw delimited bytes; DISPLAY-numeric (incl. scaled V) and
        // numeric-edited receivers take the delimited substring by MOVE (the sealed alnum->numeric/edited
        // conversion). For these the field's byte length IS its character/digit width, so the substring
        // sizing is faithful. Binary/packed (COMP*) receivers fail closed: their PHYSICAL byte length is
        // narrower than the digit width, so the delimited-segment truncation here would diverge from cobc.
        // (is_numeric_move, split_width). The split width is the receiver's CHARACTER/DIGIT count: for
        // alphanumeric/DISPLAY/edited that is its byte length, but for binary/packed (COMP*) the physical
        // byte length is narrower than the digit width, so the digit width drives the delimited-segment
        // truncation; the segment is then stored via MOVE (the sealed alnum->binary/packed conversion).
        let (is_num, width) = match &f.storage {
            Storage::Alpha(_) => (false, f.bytes.len()),
            // A GROUP receiver is treated as alphanumeric over its full byte image: cobc stores the
            // delimited segment left-justified, space-padded to the group's byte length -- identical to a
            // PIC X of the same size. Sized by the group's byte length and written raw (the else branch).
            Storage::Group { .. } => (false, f.bytes.len()),
            Storage::Numeric(a) if a.field_type == COB_TYPE_NUMERIC_DISPLAY => (true, f.bytes.len()),
            Storage::Edited(..) => (true, f.bytes.len()),
            Storage::Numeric(a) => (true, (a.digits as usize).max(1)),
            // Every UNSTRING-able receiver kind is now handled (alphanumeric, DISPLAY/binary/packed
            // numeric, numeric-edited, group). Anything left here is a target cobc itself rejects at
            // compile (e.g. an 88 condition-name or an index) -- so refusing it is faithful validation.
            _ => return Err(RunError::Unsupported(format!(
                "UNSTRING into `{n}`: target must be an elementary or group data item (cobc rejects other targets)"
            ))),
        };
        sizes.push(width);
        numeric.push(is_num);
    }
    // The scan begins at the POINTER's current value (default 1); the final pointer is written back after.
    let ptr_start = match &ptr_field {
        Some(f) => resolve_int(f, fields)
            .map(|v| v.max(1) as usize)
            .unwrap_or(1),
        None => 1,
    };
    let res = unstring_multi(&source, &delims, &sizes, ptr_start);
    if let Some(f) = &ptr_field {
        let mv = vec![
            Tok::Word(res.pointer.to_string()),
            Tok::Word("TO".to_string()),
            Tok::Word(f.clone()),
        ];
        exec_move(&mv, fields, false)?;
    }
    for (((n, din, cin), fld), is_num) in recvs.iter().zip(res.fields.iter()).zip(numeric.iter()) {
        if *is_num {
            // A DISPLAY-numeric receiver takes the delimited substring (its `count` chars) by MOVE, which
            // applies the alphanumeric->numeric conversion (right-justify, zero-fill) -- e.g. "12" -> 012.
            let sub = fld
                .data
                .get(..fld.count.min(fld.data.len()))
                .unwrap_or(&[])
                .to_vec();
            let mv = vec![
                Tok::Str(sub),
                Tok::Word("TO".to_string()),
                Tok::Word(n.clone()),
            ];
            exec_move(&mv, fields, false)?;
        } else {
            let data = fld.data.clone();
            write_field(fields, n, |f| {
                f.bytes = data;
                Ok(())
            })?;
        }
        if let Some(d) = din {
            let dl = fld.delimiter.clone();
            write_field(fields, d, |f| {
                let mut b = vec![b' '; f.bytes.len()];
                let k = dl.len().min(b.len());
                b[..k].copy_from_slice(&dl[..k]);
                f.bytes = b;
                Ok(())
            })?;
        }
        if let Some(c) = cin {
            let mv = vec![
                Tok::Word(fld.count.to_string()),
                Tok::Word("TO".to_string()),
                Tok::Word(c.clone()),
            ];
            exec_move(&mv, fields, false)?;
        }
    }
    if let Some(t) = tally_field {
        let nv = resolve_int(&t, fields).unwrap_or(0) + res.tally as i64;
        let mv = vec![
            Tok::Word(nv.to_string()),
            Tok::Word("TO".to_string()),
            Tok::Word(t),
        ];
        exec_move(&mv, fields, false)?;
    }
    Ok(res.overflow)
}

/// Day-of-year (1..366) for a Gregorian `(year, month, day)`.
fn day_of_year(y: i32, m: i32, d: i32) -> i32 {
    let mdays = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let mut s = 0;
    for (i, &md) in mdays.iter().enumerate().take((m - 1).max(0) as usize) {
        s += md;
        if i == 1 && leap {
            s += 1;
        }
    }
    s + d
}

/// COBOL `DAY-OF-WEEK` (1 = Monday .. 7 = Sunday) for a Gregorian date (Sakamoto's algorithm).
fn day_of_week(y: i32, m: i32, d: i32) -> i32 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { y - 1 } else { y };
    let w = (yy + yy / 4 - yy / 100 + yy / 400 + t[(m - 1).max(0) as usize] + d) % 7; // 0 = Sunday
    if w == 0 {
        7
    } else {
        w
    }
}

/// Civil (Gregorian) `(year, month, day)` from a count of days since 1970-01-01 (day 0). Howard
/// Hinnant's branch-free `civil_from_days` algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The interpreter's **compile step**: the compile date/time as `(year, month, day, hour, min, sec)`,
/// taken from a pinned `SOURCE_DATE_EPOCH` exactly as the admitted cobc derives WHEN-COMPILED /
/// MODULE-DATE / MODULE-TIME (libcob `cob_set_date_from_epoch`: `tm_mday = epoch/86400`, 1-based from
/// 1970-01, then normalized -- so day 1 is 1970-01-01 and the displayed fields are TZ-independent). cobc
/// honours `SOURCE_DATE_EPOCH` (the reproducible-builds standard); without the pin the live compile clock
/// is a non-claim and these intrinsics fail closed.
fn compile_tm() -> Result<(i64, u32, u32, u32, u32, u32), RunError> {
    let raw = std::env::var("SOURCE_DATE_EPOCH").map_err(|_| {
        RunError::Unsupported(
            "FUNCTION WHEN-COMPILED / MODULE-DATE / MODULE-TIME / MODULE-FORMATTED-DATE requires a pinned SOURCE_DATE_EPOCH (the live compile clock is a non-claim)".into(),
        )
    })?;
    let digits: String = raw
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let epoch: i64 = digits
        .parse()
        .map_err(|_| RunError::Unsupported("SOURCE_DATE_EPOCH is not a number".into()))?;
    if epoch > 253_402_300_799 {
        return Err(RunError::Unsupported(
            "SOURCE_DATE_EPOCH exceeds the year-9999 ceiling".into(),
        ));
    }
    let days = epoch / 86400;
    let sod = epoch % 86400;
    // Upstream 946f3e638: the epoch conversion was off by one day (the old C stuffed tm_mday =
    // days + 1 into mktime); the fix uses the straight localtime conversion, i.e. day 0 is
    // 1970-01-01 -- the same civil-calendar arithmetic cob_set_date_from_epoch uses. (The STABLE
    // 3.2 oracle still has the bug: SOURCE_DATE_EPOCH=1000000000 -> MODULE-DATE 20010908.)
    let (y, mon, d) = civil_from_days(days);
    Ok((
        y,
        mon,
        d,
        (sod / 3600) as u32,
        ((sod / 60) % 60) as u32,
        (sod % 60) as u32,
    ))
}

/// `ACCEPT identifier FROM {DATE [YYYYMMDD] | DAY [YYYYDDD] | TIME | DAY-OF-WEEK}` -- the system date/time
/// registers. To stay oracle-deterministic the front-end reads the **pinned** `COB_CURRENT_DATE` (the same
/// override `libcob` honors); with it unset (a live wall clock) ACCEPT fails closed rather than guess. The
/// terminal-input form (`ACCEPT id` with no FROM) is also outside the subset.
fn exec_accept(stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
    let target = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("ACCEPT: missing receiver".into())),
    };
    if !matches!(stmt.get(1), Some(Tok::Word(w)) if w == "FROM") {
        return Err(RunError::Unsupported("ACCEPT FROM terminal/console: interactive input is a runtime non-claim (no deterministic oracle); the wired sources are DATE/DAY/TIME/DAY-OF-WEEK/ENVIRONMENT".into()));
    }
    let src = match stmt.get(2) {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("ACCEPT FROM: missing source".into())),
    };
    // `ACCEPT id FROM ENVIRONMENT "name"` (or a name field): read the environment variable (deterministic
    // under the pinned harness env) and MOVE its value into the receiver; an unset variable yields spaces.
    if src == "ENVIRONMENT" || src == "ENVIRONMENT-VALUE" {
        // ENVIRONMENT-VALUE reads the variable named by the env-name register (set via DISPLAY UPON
        // ENVIRONMENT-NAME); ENVIRONMENT "name" reads the named variable. A per-run override (DISPLAY UPON
        // ENVIRONMENT-VALUE) wins over the real process env; an unset variable yields spaces.
        let name = if src == "ENVIRONMENT-VALUE" {
            ENV_NAME_REG.with(|r| r.borrow().clone())
        } else {
            match stmt.get(3) {
                Some(Tok::Str(s)) => String::from_utf8_lossy(s).to_string(),
                Some(Tok::Word(w)) => read_field(fields, w)?
                    .map(|f| String::from_utf8_lossy(&f.bytes).trim_end().to_string())
                    .unwrap_or_else(|| w.clone()),
                _ => {
                    return Err(RunError::Unsupported(
                        "ACCEPT FROM ENVIRONMENT: missing variable name".into(),
                    ))
                }
            }
        };
        let val = ENV_OVERRIDE
            .with(|m| m.borrow().get(&name).cloned())
            .unwrap_or_else(|| std::env::var(&name).unwrap_or_default().into_bytes());
        let mv = vec![
            Tok::Str(val),
            Tok::Word("TO".to_string()),
            Tok::Word(target),
        ];
        return exec_move(&mv, fields, false);
    }
    // COMMAND-LINE sources (the run boundary set the command line via set_command_line):
    //   ACCEPT x FROM COMMAND-LINE            -> the whole argument string (args joined by spaces)
    //   ACCEPT x FROM ARGUMENT-VALUE          -> the argument named by the ARGUMENT-NUMBER register
    //   ACCEPT x FROM ARGUMENT-NUMBER         -> the count of arguments
    // (cobcrun passes everything after the module name; the oracle behaves identically for a
    // pinned argv, so this is deterministic under the suite.)
    if src == "COMMAND-LINE" {
        let joined = COMMAND_LINE.with(|c| c.borrow().join(" "));
        let mv = vec![
            Tok::Str(joined.into_bytes()),
            Tok::Word("TO".to_string()),
            Tok::Word(target),
        ];
        return exec_move(&mv, fields, false);
    }
    if src == "ARGUMENT-VALUE" {
        let n = ARG_NUMBER_REG.with(|r| r.borrow().clone());
        let val = COMMAND_LINE.with(|c| c.borrow().get(n).cloned().unwrap_or_default());
        let mv = vec![
            Tok::Str(val.into_bytes()),
            Tok::Word("TO".to_string()),
            Tok::Word(target),
        ];
        return exec_move(&mv, fields, false);
    }
    if src == "ARGUMENT-NUMBER" {
        let n = COMMAND_LINE.with(|c| c.borrow().len());
        let mv = vec![
            Tok::Str(n.to_string().into_bytes()),
            Tok::Word("TO".to_string()),
            Tok::Word(target),
        ];
        return exec_move(&mv, fields, false);
    }
    let long_year = matches!(stmt.get(3), Some(Tok::Word(w)) if w == "YYYYMMDD" || w == "YYYYDDD");
    let raw = std::env::var("COB_CURRENT_DATE").map_err(|_| RunError::Unsupported(
        "ACCEPT FROM DATE/TIME requires a pinned COB_CURRENT_DATE (the live clock is a non-claim)".into()))?;
    let cd = crate::common_signal::check_current_date(raw.as_bytes());
    if cd.invalid || cd.year < 0 || cd.month < 0 || cd.day < 0 {
        return Err(RunError::Runtime(
            "COB_CURRENT_DATE did not parse to a full date".into(),
        ));
    }
    let (y, m, d) = (cd.year, cd.month, cd.day);
    let digits = match src.as_str() {
        "DATE" if long_year => format!("{:04}{:02}{:02}", y, m, d),
        "DATE" => format!("{:02}{:02}{:02}", y % 100, m, d),
        "DAY" if long_year => format!("{:04}{:03}", y, day_of_year(y, m, d)),
        "DAY" => format!("{:02}{:03}", y % 100, day_of_year(y, m, d)),
        "DAY-OF-WEEK" => format!("{}", day_of_week(y, m, d)),
        "TIME" => {
            let (hh, mi, ss) = (cd.hour.max(0), cd.minute.max(0), cd.second.max(0));
            let cs = if cd.nanosecond >= 0 { cd.nanosecond / 10_000_000 } else { 0 };
            format!("{:02}{:02}{:02}{:02}", hh, mi, ss, cs)
        }
        other => return Err(RunError::Unsupported(format!("ACCEPT FROM {other}: a runtime non-claim (terminal/command-line input has no deterministic oracle); wired sources are DATE/DAY/TIME/DAY-OF-WEEK/ENVIRONMENT"))),
    };
    let s = digits.into_bytes();
    let n = s.len();
    write_field(fields, &target, |f| match &f.storage {
        Storage::Numeric(_) | Storage::Alpha(_) if f.bytes.len() == n => {
            f.bytes = s;
            Ok(())
        }
        _ => Err(RunError::Unsupported(format!(
            "ACCEPT FROM {src}: receiver must be a {n}-digit numeric/alphanumeric item"
        ))),
    })
}

/// The in-memory file-store key for a COBOL file name: its ASSIGN target (so two SELECTs on the same
/// physical name share storage), falling back to the name.
fn fkey(ctx: &Ctx, name: &str) -> String {
    ctx.file_defs
        .get(name)
        .map(|d| d.assign.clone())
        .filter(|a| !a.is_empty())
        .unwrap_or_else(|| name.to_string())
}

/// Resolve a file's `ASSIGN` target to a real path on disk, mirroring cobc: an environment variable named
/// exactly like the assign target overrides the path (cobc's `dd_`/env mapping), otherwise the target is a
/// filename looked up in the current directory and then each `COB_FILE_PATH` entry. Returns the first that
/// exists. Used to let `OPEN INPUT` read a pre-existing real data file (the in-memory store is the default).
fn resolve_disk_file(assign: &str) -> Option<std::path::PathBuf> {
    if assign.is_empty() {
        return None;
    }
    let candidate = std::env::var(assign).unwrap_or_else(|_| assign.to_string());
    let direct = std::path::Path::new(&candidate);
    if direct.is_file() {
        return Some(direct.to_path_buf());
    }
    if let Ok(fp) = std::env::var("COB_FILE_PATH") {
        for dir in fp.split([':', ';']).filter(|d| !d.is_empty()) {
            let c = std::path::Path::new(dir).join(&candidate);
            if c.is_file() {
                return Some(c);
            }
        }
    }
    None
}

/// Load a pre-existing real data file into logical records for `OPEN INPUT`/`I-O`: LINE SEQUENTIAL splits on
/// newlines (CRLF tolerated), record-sequential / relative / indexed splits into fixed record-length chunks;
/// each record is padded/truncated to the FD record width. Returns `None` when no such file exists on disk
/// (the caller then keeps the deterministic in-memory behaviour -- a "file not found" status). READ-ONLY: it
/// never writes the host filesystem, and the sweep's self-contained fixtures (no external files) are
/// unaffected.
fn load_file_from_disk(def: &FileDef, fields: &HashMap<String, Field>) -> Option<Vec<Vec<u8>>> {
    let path = resolve_disk_file(&def.assign)?;
    let data = std::fs::read(path).ok()?;
    if def.varying_dep.is_some() {
        // GnuCOBOL variable-length record-sequential on-disk format: each record is preceded by a 4-byte
        // header whose first two bytes are the record length (u16, big-endian); the record data follows at
        // its true length and is kept UNPADDED here so READ can publish the DEPENDING ON length faithfully.
        let mut records = Vec::new();
        let mut p = 0usize;
        while p + 4 <= data.len() {
            let len = u16::from_be_bytes([data[p], data[p + 1]]) as usize;
            p += 4;
            if len == 0 || p + len > data.len() {
                break;
            }
            records.push(data[p..p + len].to_vec());
            p += len;
        }
        return Some(records);
    }
    // The FD record width: use read_field (reconstructs a GROUP record from its leaves) rather than the
    // group field's own `bytes` (which is empty for a group whose children hold the storage) -- so a
    // group-structured FD record (e.g. `01 R. 05 A PIC X(13). 05 FILLER PIC X(67).`) loads correctly.
    let reclen = read_field(fields, def.primary_record())
        .ok()
        .flatten()
        .map(|f| f.bytes.len())
        .filter(|&n| n > 0)?;
    let fit = |bytes: &[u8]| -> Vec<u8> {
        let mut r = bytes.to_vec();
        r.resize(reclen, b' '); // pad short / truncate long to the fixed record width
        r
    };
    let records = match def.org {
        FileOrg::LineSequential => {
            let mut v: Vec<Vec<u8>> = data
                .split(|&b| b == b'\n')
                .map(|line| fit(line.strip_suffix(b"\r").unwrap_or(line)))
                .collect();
            if data.last() == Some(&b'\n') {
                v.pop(); // a trailing newline yields a spurious empty final record
            }
            v
        }
        // ORGANIZATION INDEXED: the on-disk file is a Berkeley DB B-tree (the same format cobc/libcob use).
        // Parse it with the pure-Rust gnucobol-rs-bdb-format reader and load each record (the value of each
        // key->record pair); exec_read's INDEXED branch then presents them in RECORD KEY order (READ NEXT)
        // or by key (random READ). The keys are intrinsic to the records, so only the values are stored.
        FileOrg::Indexed => {
            let db = gnucobol_rs_bdb_format::BdbFile::parse(&data).ok()?;
            db.records()
                .ok()?
                .into_iter()
                .map(|(_k, v)| fit(&v))
                .collect()
        }
        _ => data.chunks(reclen.max(1)).map(fit).collect(),
    };
    Some(records)
}

/// Set a file's `FILE STATUS` field (if declared) to a 2-character code (`"00"` ok, `"10"` end-of-file).
fn set_file_status(fields: &mut HashMap<String, Field>, def: &FileDef, code: &str) {
    // FUNCTION EXCEPTION-FILE reflects the LAST I/O operation (regardless of a FILE STATUS clause).
    set_file_exception(code, &def.name);
    if let Some(s) = &def.status {
        let mv = vec![
            Tok::Str(code.as_bytes().to_vec()),
            Tok::Word("TO".to_string()),
            Tok::Word(s.clone()),
        ];
        let _ = exec_move(&mv, fields, false);
    }
}

/// `OPEN {INPUT|OUTPUT|EXTEND|I-O} file [file ...]` -- set each file's open mode (OUTPUT truncates the
/// logical file). The subset is a single mode keyword per statement.
fn exec_open(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let mode = match stmt.first() {
        Some(Tok::Word(w)) => match w.as_str() {
            "INPUT" => 1u8,
            "OUTPUT" => 2,
            "EXTEND" => 3,
            "I-O" => 4,
            other => {
                return Err(RunError::Unsupported(format!(
                    "OPEN: unrecognized mode `{other}` (expected INPUT/OUTPUT/EXTEND/I-O)"
                )))
            }
        },
        _ => return Err(RunError::Unsupported("OPEN: missing mode".into())),
    };
    for name in stmt[1..].iter().filter_map(|t| {
        if let Tok::Word(w) = t {
            Some(w.clone())
        } else {
            None
        }
    }) {
        let def = ctx
            .file_defs
            .get(&name)
            .ok_or_else(|| RunError::Unsupported(format!("OPEN: `{name}` is not a declared file")))?
            .clone();
        // Upstream state guards (62b39805c / cob_open): a LOCKED file rejects any re-OPEN with
        // status 38 (CLOSED WITH LOCK); an already-open file reports 41 (ALREADY OPEN).
        if let Some(st) = ctx.files.borrow().get(&fkey(ctx, &name)) {
            if st.mode == 5 {
                set_file_status(fields, &def, "38");
                continue;
            }
            if st.mode != 0 {
                set_file_status(fields, &def, "41");
                continue;
            }
        }
        // A VARIABLE-length FD record area is LOW-VALUES (NUL) until written -- cobc does not space-init it,
        // and READ ... INTO later moves the PHYSICAL buffer (its tail past the record included). Reset the
        // record's storage leaf to NUL at OPEN so a freshly-read short record carries cobc's NUL tail.
        if def.varying_dep.is_some() {
            let leaf = match fields.get(def.primary_record()).map(|f| f.storage.clone()) {
                Some(Storage::Group { children }) => children
                    .iter()
                    .find(|c| !c.starts_with('\u{3}'))
                    .cloned()
                    .unwrap_or_else(|| def.primary_record().to_string()),
                _ => def.primary_record().to_string(),
            };
            if let Some(lf) = fields.get_mut(&leaf) {
                lf.bytes.iter_mut().for_each(|b| *b = 0);
            }
        }
        // OPEN INPUT / I-O on a file that was never created (never OPEN OUTPUT'd / written) is "file not
        // found": status "35", then any DECLARATIVES USE handler for the file runs.
        let exists = ctx.files.borrow().contains_key(&fkey(ctx, &name));
        if (mode == 1 || mode == 4) && !exists {
            // Not in the in-memory store: load a pre-existing REAL file from disk if one exists (read-only,
            // resolved from the ASSIGN target / COB_FILE_PATH). Absent -> the deterministic "file not found".
            if let Some(records) = load_file_from_disk(&def, fields) {
                let mut files = ctx.files.borrow_mut();
                let st = files.entry(fkey(ctx, &name)).or_default();
                st.records = records;
                st.read_pos = 0;
                st.mode = mode;
                drop(files);
                set_file_status(fields, &def, "00");
                continue;
            }
            set_file_status(fields, &def, "35");
            run_use_handler(&name, fields, out, ctx)?;
            continue;
        }
        {
            let mut files = ctx.files.borrow_mut();
            let st = files.entry(fkey(ctx, &name)).or_default();
            if mode == 2 {
                st.records.clear();
            }
            st.read_pos = 0;
            st.mode = mode;
        }
        set_file_status(fields, &def, "00");
    }
    Ok(())
}

/// Run the DECLARATIVES `USE ... ON file` handler paragraph (if any) after a file error on `file`.
fn run_use_handler(
    file: &str,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    let range = USE_PROCS.with(|c| c.borrow().get(file).cloned());
    if let Some((start, end)) = range {
        let proc = CUR_PROC.with(|c| c.borrow().clone());
        return run_range(&proc, start, end, fields, out, ctx);
    }
    Ok(false)
}

/// `CLOSE file [file ...]` -- mark each file closed (its logical records persist so a later OPEN INPUT can
/// re-read them within the same run).
fn exec_close(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let with_lock = stmt
        .iter()
        .any(|t| matches!(t, Tok::Word(w) if w == "LOCK"));
    for name in stmt.iter().filter_map(|t| {
        if let Tok::Word(w) = t {
            if w == "WITH" || w == "LOCK" {
                None
            } else {
                Some(w.clone())
            }
        } else {
            None
        }
    }) {
        let def = ctx.file_defs.get(&name).ok_or_else(|| {
            RunError::Unsupported(format!("CLOSE: `{name}` is not a declared file"))
        })?;
        match ctx.files.borrow_mut().get_mut(&fkey(ctx, &name)) {
            Some(st) if st.mode == 0 || st.mode == 5 => {
                // Upstream 62b39805c (cob_close): CLOSED and LOCKED files report 42 (NOT OPEN);
                // the backend is never called on a locked file (the old abend path).
                set_file_status(fields, def, "42");
            }
            Some(st) => {
                // GnuCOBOL emits a final `\n` at close when the last write was AFTER ADVANCING
                // (`flag_needs_nl`) -- mirror it so the store holds the oracle's disk bytes.
                if st.pending_nl && matches!(def.org, FileOrg::Sequential | FileOrg::LineSequential)
                {
                    st.records.push(vec![b'\n']);
                    st.pending_nl = false;
                }
                st.mode = if with_lock { 5 } else { 0 };
                set_file_status(fields, def, "00");
            }
            None => {
                // Never opened in this run: closing a closed file is "42" per upstream.
                set_file_status(fields, def, "42");
            }
        }
    }
    Ok(())
}

/// `WRITE record [FROM id]` -- append the record's current bytes to its file (LINE SEQUENTIAL trims trailing
/// spaces, matching the oracle). The operand is the FD record name.
fn exec_write(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let rec = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("WRITE: missing record".into())),
    };
    // optional FROM id: MOVE id into the record first.
    if let Some(fp) = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "FROM"))
    {
        if let Some(src) = stmt.get(fp + 1) {
            let mv = vec![
                src.clone(),
                Tok::Word("TO".to_string()),
                Tok::Word(rec.clone()),
            ];
            exec_move(&mv, fields, ctx.decimal_comma)?;
        }
    }
    // Line control: `[BEFORE|AFTER] ADVANCING n [LINE|LINES]` -- the CCVS85 print-file shape. The
    // oracle's model (admitted libcob fileio.c): AFTER n writes n x LF BEFORE the record (n = 0 writes
    // CR); BEFORE n writes n x LF AFTER the record; a final LF is emitted at CLOSE when an AFTER left
    // the line pending (`flag_needs_nl`). The advancing bytes are FILE data (they land in the store
    // beside the record), never the record area itself. `ADVANCING PAGE` (form feed) fails closed.
    let mut line_control: Option<(bool, usize)> = None; // (is_after, count)
    for (i, t) in stmt.iter().enumerate().skip(1) {
        if let Tok::Word(w) = t {
            if (w == "AFTER" || w == "BEFORE")
                && matches!(stmt.get(i + 1), Some(Tok::Word(x)) if x == "ADVANCING")
            {
                if matches!(stmt.get(i + 2), Some(Tok::Word(x)) if x == "PAGE") {
                    return Err(RunError::Unsupported("WRITE ... ADVANCING PAGE: form-feed page control is a fail-closed boundary".into()));
                }
                let is_after = w == "AFTER";
                // count: a literal / identifier up to LINE|LINES (bare ADVANCING = 1).
                let cnt = stmt
                    .get(i + 2)
                    .filter(|t| !matches!(t, Tok::Word(x) if x == "LINE" || x == "LINES"));
                let n = match cnt {
                    Some(Tok::Word(c)) => resolve_int(c, fields).ok_or_else(|| {
                        RunError::Unsupported(format!(
                            "WRITE ... ADVANCING: `{c}` is not an integer"
                        ))
                    })?,
                    _ => 1,
                };
                if n < 0 {
                    return Err(RunError::Runtime(
                        "WRITE ... ADVANCING: negative count".into(),
                    ));
                }
                line_control = Some((is_after, n as usize));
                break;
            }
        }
    }
    let def = {
        let fname = ctx
            .record_files
            .get(&rec)
            .ok_or_else(|| RunError::Unsupported(format!("WRITE `{rec}`: not an FD record")))?;
        ctx.file_defs
            .get(fname)
            .ok_or_else(|| RunError::Unsupported(format!("WRITE `{rec}`: not an FD record")))?
            .clone()
    };
    let mut bytes = read_field(fields, &rec)?
        .map(|f| f.bytes)
        .unwrap_or_default();
    if def.org == FileOrg::LineSequential {
        while bytes.last() == Some(&b' ') {
            bytes.pop();
        }
    }
    if def.org == FileOrg::Relative {
        // place the record at the 1-based RELATIVE KEY position (empty slots = absent records).
        let pos = relative_key_value(&def, fields)?;
        let mut files = ctx.files.borrow_mut();
        let st = files.entry(def.assign.clone()).or_default();
        if st.records.len() < pos {
            st.records.resize(pos, Vec::new());
        }
        let occupied = !st.records[pos - 1].is_empty();
        if !occupied {
            st.records[pos - 1] = bytes;
        }
        drop(files);
        set_file_status(fields, &def, if occupied { "22" } else { "00" });
        return Ok(());
    }
    if let Some((is_after, n)) = line_control {
        if !matches!(def.org, FileOrg::Sequential | FileOrg::LineSequential) {
            return Err(RunError::Unsupported("WRITE ... ADVANCING is valid only on SEQUENTIAL / LINE SEQUENTIAL files (a RELATIVE/INDEXED advancing write is a fail-closed boundary)".into()));
        }
        let mut chunk = Vec::with_capacity(n + bytes.len());
        if is_after {
            // AFTER n: n x LF BEFORE the record; n = 0 writes CR (the oracle's cob_file_write_opt).
            for _ in 0..n {
                chunk.push(b'\n');
            }
            chunk.extend_from_slice(&bytes);
        } else {
            // BEFORE n: n x LF AFTER the record (the record then ends the line -- no pending newline).
            chunk.extend_from_slice(&bytes);
            for _ in 0..n {
                chunk.push(b'\n');
            }
        }
        bytes = chunk;
    }
    {
        let mut files = ctx.files.borrow_mut();
        let st = files.entry(def.assign.clone()).or_default();
        st.records.push(bytes);
        if let Some((is_after, _)) = line_control {
            if is_after {
                st.pending_nl = true;
            }
        }
    }
    set_file_status(fields, &def, "00");
    Ok(())
}

/// The current 1-based value of a RELATIVE file's RELATIVE KEY field (>= 1 required).
fn relative_key_value(def: &FileDef, fields: &HashMap<String, Field>) -> Result<usize, RunError> {
    let key = def.rel_key.as_ref().ok_or_else(|| {
        RunError::Unsupported(format!("RELATIVE file `{}` has no RELATIVE KEY", def.name))
    })?;
    let v = resolve_int(key, fields)
        .ok_or_else(|| RunError::Unsupported(format!("RELATIVE KEY `{key}` is not an integer")))?;
    if v < 1 {
        return Err(RunError::Runtime(format!(
            "RELATIVE KEY `{key}` = {v} (< 1)"
        )));
    }
    Ok(v as usize)
}

/// `REWRITE record [FROM id]` -- replace the record last READ (under OPEN I-O) with the record buffer's
/// current bytes. With no current record (no prior READ) it fails with status `"43"`.
fn exec_rewrite(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let rec = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("REWRITE: missing record".into())),
    };
    if let Some(fp) = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "FROM"))
    {
        if let Some(src) = stmt.get(fp + 1) {
            let mv = vec![
                src.clone(),
                Tok::Word("TO".to_string()),
                Tok::Word(rec.clone()),
            ];
            exec_move(&mv, fields, ctx.decimal_comma)?;
        }
    }
    let def = {
        let fname = ctx
            .record_files
            .get(&rec)
            .ok_or_else(|| RunError::Unsupported(format!("REWRITE `{rec}`: not an FD record")))?;
        ctx.file_defs
            .get(fname)
            .ok_or_else(|| RunError::Unsupported(format!("REWRITE `{rec}`: not an FD record")))?
            .clone()
    };
    let mut bytes = read_field(fields, &rec)?
        .map(|f| f.bytes)
        .unwrap_or_default();
    if def.org == FileOrg::LineSequential {
        while bytes.last() == Some(&b' ') {
            bytes.pop();
        }
    }
    let no_current = {
        let mut files = ctx.files.borrow_mut();
        match files.get_mut(&def.assign) {
            Some(st) if st.read_pos >= 1 && st.read_pos <= st.records.len() => {
                let idx = st.read_pos - 1;
                st.records[idx] = bytes;
                false
            }
            _ => true,
        }
    };
    set_file_status(fields, &def, if no_current { "43" } else { "00" });
    Ok(())
}

/// `UNLOCK file [file ...]` -- release record locks. The front-end's in-memory model holds no locks, so this
/// is a faithful no-op (status `"00"`), matching libcob on a non-locked sequential file.
fn exec_unlock(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    for name in stmt.iter().filter_map(|t| {
        if let Tok::Word(w) = t {
            Some(w.clone())
        } else {
            None
        }
    }) {
        let def = ctx.file_defs.get(&name).ok_or_else(|| {
            RunError::Unsupported(format!("UNLOCK: `{name}` is not a declared file"))
        })?;
        // Upstream cob_unlock (current head): a CLOSED file reports 42; a LOCKED file falls through
        // to cob_file_unlock (which skips the work on LOCKED) and reports 00.
        let mode = ctx
            .files
            .borrow()
            .get(&fkey(ctx, &name))
            .map(|st| st.mode)
            .unwrap_or(0);
        let status = match mode {
            1 | 2 | 3 | 4 => "00",
            5 => "00", // locked: unlock is accepted (work skipped), status 00
            _ => "42",
        };
        set_file_status(fields, def, status);
    }
    Ok(())
}

/// `SORT sd-file ON {ASCENDING|DESCENDING} KEY key USING in... GIVING out...` (and `MERGE`, same shape) --
/// read every record from the USING files, order them, and write them to the GIVING files. The subset is a
/// single **whole-record** KEY (sub-field keys need group items); INPUT/OUTPUT PROCEDURE (which drive
/// RELEASE/RETURN) is supported for in-memory records. The operand must be an `SD` (sort-description) file
/// -- an FD is the upstream "must be an SD filename" compile error (23f850352/277a07c2e).
fn exec_sort(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let sf = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("SORT: missing sort file".into())),
    };
    let sd_def = ctx
        .file_defs
        .get(&sf)
        .ok_or_else(|| RunError::Unsupported(format!("SORT: `{sf}` is not a declared file")))?
        .clone();
    // Upstream sort_merge_body (23f850352/277a07c2e): the SORT/MERGE operand must be an `SD` file
    // description -- an FD (or any other organization) is a compile-time "must be an SD filename" error.
    if sd_def.org != FileOrg::Sort {
        return Err(RunError::Unsupported(format!(
            "SORT/MERGE: `{sf}` must be an SD filename"
        )));
    }
    let reclen = read_field(fields, sd_def.primary_record())?
        .map(|f| f.bytes.len())
        .unwrap_or(0);
    let kw = |w: &str| {
        matches!(
            w,
            "ON" | "KEY"
                | "ASCENDING"
                | "DESCENDING"
                | "USING"
                | "GIVING"
                | "INPUT"
                | "OUTPUT"
                | "PROCEDURE"
                | "IS"
                | "THRU"
                | "THROUGH"
        )
    };
    // Each KEY records the ASCENDING/DESCENDING direction in effect when it was named (a SORT may mix
    // directions: `ASCENDING KEY a DESCENDING KEY b`). The keys compare in declared order.
    let mut cur_desc = false;
    let mut keys: Vec<(String, bool)> = vec![];
    let (mut using, mut giving): (Vec<String>, Vec<String>) = (vec![], vec![]);
    let (mut in_proc, mut out_proc): (Option<(String, String)>, Option<(String, String)>) =
        (None, None);
    let word = |i: usize| {
        stmt.get(i).and_then(|t| {
            if let Tok::Word(w) = t {
                Some(w.clone())
            } else {
                None
            }
        })
    };
    let mut i = 1;
    while i < stmt.len() {
        match word(i).as_deref() {
            Some("ON") | Some("KEY") => i += 1,
            Some("ASCENDING") => {
                cur_desc = false;
                i += 1;
            }
            Some("DESCENDING") => {
                cur_desc = true;
                i += 1;
            }
            Some("USING") => {
                i += 1;
                while let Some(w) = word(i) {
                    if kw(&w) {
                        break;
                    }
                    using.push(w);
                    i += 1;
                }
            }
            Some("GIVING") => {
                i += 1;
                while let Some(w) = word(i) {
                    if kw(&w) {
                        break;
                    }
                    giving.push(w);
                    i += 1;
                }
            }
            Some("INPUT") | Some("OUTPUT") => {
                let is_in = word(i).as_deref() == Some("INPUT");
                i += 1;
                if word(i).as_deref() == Some("PROCEDURE") {
                    i += 1;
                }
                if word(i).as_deref() == Some("IS") {
                    i += 1;
                }
                let p1 = word(i).unwrap_or_default();
                i += 1;
                let mut p2 = p1.clone();
                if matches!(word(i).as_deref(), Some("THRU") | Some("THROUGH")) {
                    i += 1;
                    if let Some(w) = word(i) {
                        p2 = w;
                        i += 1;
                    }
                }
                if is_in {
                    in_proc = Some((p1, p2));
                } else {
                    out_proc = Some((p1, p2));
                }
            }
            Some(w) => {
                keys.push((w.to_string(), cur_desc));
                i += 1;
            }
            None => i += 1,
        }
    }
    if keys.is_empty() {
        return Err(RunError::Unsupported("SORT/MERGE: no KEY given".into()));
    }
    // (offset, length, descending) for each key, in declared (major-to-minor) order.
    let mut spans: Vec<(usize, usize, bool)> = Vec::with_capacity(keys.len());
    for (k, desc) in &keys {
        let (off, len) =
            sort_key_span(sd_def.primary_record(), k, reclen, fields).ok_or_else(|| {
                RunError::Unsupported(format!(
                    "SORT/MERGE KEY `{k}` is not a field of the sort record"
                ))
            })?;
        spans.push((off, len, *desc));
    }
    // the current body's tokens, for running INPUT/OUTPUT PROCEDURE ranges.
    let proc = CUR_PROC.with(|c| c.borrow().clone());
    // ---- gather phase: INPUT PROCEDURE (RELEASE records into the sort file) or USING files ----
    let mut recs: Vec<Vec<u8>> = Vec::new();
    if let Some((p1, p2)) = &in_proc {
        ctx.files
            .borrow_mut()
            .entry(fkey(ctx, &sf))
            .or_default()
            .records
            .clear();
        let (start, end) = para_range(p1, p2).ok_or_else(|| {
            RunError::Unsupported(format!("SORT INPUT PROCEDURE: unknown paragraph `{p1}`"))
        })?;
        run_range_goto(&proc, start, end, fields, out, ctx)?;
        recs = ctx
            .files
            .borrow()
            .get(&fkey(ctx, &sf))
            .map(|st| st.records.clone())
            .unwrap_or_default();
    } else if !using.is_empty() {
        let mut files = ctx.files.borrow_mut();
        for f in &using {
            let key = fkey(ctx, f);
            // Upstream implicitly opens the USING files for input (no explicit OPEN needed); mirror that
            // by loading a pre-existing REAL file from disk on first use, and fail closed (status 35 --
            // "file does not exist") when the file is neither in the store nor on disk, exactly like
            // `OPEN INPUT` does. Never silently sort an empty input that the oracle would reject.
            if !files.contains_key(&key) {
                if let Some(def) = ctx.file_defs.get(f) {
                    if let Some(records) = load_file_from_disk(def, fields) {
                        files.entry(key.clone()).or_default().records = records;
                    } else {
                        drop(files);
                        return Err(RunError::Runtime(format!(
                            "SORT/MERGE USING: file does not exist (status = 35) for file `{f}`"
                        )));
                    }
                } else {
                    drop(files);
                    return Err(RunError::Unsupported(format!(
                        "SORT/MERGE USING: `{f}` is not a declared file"
                    )));
                }
            }
            if let Some(st) = files.get(&key) {
                for r in &st.records {
                    if r.is_empty() {
                        continue;
                    }
                    recs.push(r.clone());
                }
            }
        }
    } else {
        return Err(RunError::Unsupported(
            "SORT/MERGE requires USING or INPUT PROCEDURE".into(),
        ));
    }
    for r in recs.iter_mut() {
        r.resize(reclen, b' ');
    }
    recs.sort_by(|a, b| {
        for &(off, len, desc) in &spans {
            let (sa, ea) = (off.min(a.len()), (off + len).min(a.len()));
            let (sb, eb) = (off.min(b.len()), (off + len).min(b.len()));
            let ord = a[sa..ea].cmp(&b[sb..eb]);
            let ord = if desc { ord.reverse() } else { ord };
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    });
    // ---- distribute phase: OUTPUT PROCEDURE (RETURN records) or GIVING files ----
    if let Some((p3, p4)) = &out_proc {
        {
            let mut files = ctx.files.borrow_mut();
            let st = files.entry(fkey(ctx, &sf)).or_default();
            st.records = recs;
            st.read_pos = 0;
            st.mode = 1;
        }
        set_file_status(fields, &sd_def, "00");
        let (start, end) = para_range(p3, p4).ok_or_else(|| {
            RunError::Unsupported(format!("SORT OUTPUT PROCEDURE: unknown paragraph `{p3}`"))
        })?;
        run_range_goto(&proc, start, end, fields, out, ctx)?;
    } else if !giving.is_empty() {
        let mut files = ctx.files.borrow_mut();
        for f in &giving {
            let gorg = ctx
                .file_defs
                .get(f)
                .map(|d| d.org)
                .unwrap_or(FileOrg::Sequential);
            let st = files.entry(fkey(ctx, f)).or_default();
            st.records = recs
                .iter()
                .map(|r| {
                    let mut b = r.clone();
                    if gorg == FileOrg::LineSequential {
                        while b.last() == Some(&b' ') {
                            b.pop();
                        }
                    }
                    b
                })
                .collect();
            st.read_pos = 0;
            st.mode = 0;
        }
        set_file_status(fields, &sd_def, "00");
    } else {
        return Err(RunError::Unsupported(
            "SORT/MERGE requires GIVING or OUTPUT PROCEDURE".into(),
        ));
    }
    Ok(())
}

/// Format a decimal as a JSON/XML number: sign (if negative non-zero), integer part with leading zeros
/// stripped, and the scale's fractional digits kept (e.g. `12.50`), matching GnuCOBOL `JSON/XML GENERATE`.
fn num_to_json(dec: &Decimal) -> String {
    let scale = dec.scale.max(0) as usize;
    let total = dec.digits.len();
    let intlen = total.saturating_sub(scale);
    let int_str: String = dec.digits[..intlen]
        .iter()
        .map(|d| (b'0' + d) as char)
        .collect();
    let int_t = int_str.trim_start_matches('0');
    let int_final = if int_t.is_empty() { "0" } else { int_t };
    let frac: String = dec.digits[intlen..]
        .iter()
        .map(|d| (b'0' + d) as char)
        .collect();
    let is_zero = int_final == "0" && frac.chars().all(|c| c == '0');
    let sign = if dec.negative && !is_zero { "-" } else { "" };
    if scale > 0 {
        format!("{sign}{int_final}.{frac}")
    } else {
        format!("{sign}{int_final}")
    }
}

/// A field's alphanumeric value with trailing spaces trimmed, with the `escape` map applied per byte.
fn trimmed_escaped(bytes: &[u8], escape: impl Fn(u8) -> Option<&'static str>) -> String {
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b' ' {
        end -= 1;
    }
    // An all-spaces (non-empty) alphanumeric keeps ONE space, not the empty string -- cobc renders
    // `{"A":" "}` / `<A> </A>`, never `{"A":""}` / `<A></A>`.
    if end == 0 && !bytes.is_empty() {
        end = 1;
    }
    let mut s = String::new();
    for &b in &bytes[..end] {
        match escape(b) {
            Some(e) => s.push_str(e),
            None => s.push(b as char),
        }
    }
    s
}

/// The JSON value of a field: `{...}` for a group, a trimmed number for numeric, a quoted escaped string
/// otherwise -- recursively over the group tree (GnuCOBOL `JSON GENERATE`).
fn json_value(
    name: &str,
    fields: &HashMap<String, Field>,
    rename: &HashMap<String, String>,
    suppress: &std::collections::HashSet<String>,
) -> String {
    json_value_occ(name, None, fields, rename, suppress)
}

/// JSON value of a field. `occ` is the element index to read when inside a group-OCCURS table (cobc 3.2
/// renders only the FIRST occurrence -- `{"ROW":{<element-1 children>}}` -- a `-Wpending` behaviour).
fn json_value_occ(
    name: &str,
    occ: Option<usize>,
    fields: &HashMap<String, Field>,
    rename: &HashMap<String, String>,
    suppress: &std::collections::HashSet<String>,
) -> String {
    let read_occ = |n: &str| match occ {
        Some(i) => read_field(fields, &format!("{n}({i})"))
            .ok()
            .flatten()
            .map(|f| f.bytes)
            .unwrap_or_default(),
        None => ml_first_elem(read_field(fields, n).ok().flatten().as_ref()),
    };
    match fields.get(name).map(|f| (f.storage.clone(), f.occurs)) {
        Some((Storage::Group { children }, n)) => {
            // A group-OCCURS renders element 1's children (occ=1); a plain group propagates the context.
            let child_occ = if n > 1 && occ.is_none() { Some(1) } else { occ };
            let parts: Vec<String> = children
                .iter()
                .filter(|c| !suppress.contains(*c))
                .map(|c| {
                    let key = rename.get(c).map(String::as_str).unwrap_or(c);
                    format!(
                        "\"{}\":{}",
                        key,
                        json_value_occ(c, child_occ, fields, rename, suppress)
                    )
                })
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        Some((Storage::Numeric(attr), _)) => {
            let bytes = match occ {
                Some(_) => read_occ(name),
                None => ml_first_elem(fields.get(name)),
            };
            source_to_decimal(&bytes, &attr)
                .map(|d| num_to_json(&d))
                .unwrap_or_else(|_| "0".into())
        }
        _ => {
            let bytes = read_occ(name);
            let esc = |b: u8| match b {
                b'"' => Some("\\\""),
                b'\\' => Some("\\\\"),
                0x08 => Some("\\b"),
                0x0c => Some("\\f"),
                b'\n' => Some("\\n"),
                b'\r' => Some("\\r"),
                b'\t' => Some("\\t"),
                _ => None,
            };
            format!("\"{}\"", trimmed_escaped(&bytes, esc))
        }
    }
}

/// The first-element bytes of a field for JSON/XML GENERATE: an elementary OCCURS table emits only its FIRST
/// occurrence (cobc 3.2 behaviour), a scalar its whole value.
fn ml_first_elem(f: Option<&Field>) -> Vec<u8> {
    match f {
        Some(f) => {
            let occ = f.occurs.max(1);
            let es = f.bytes.len() / occ;
            f.bytes
                .get(..es)
                .map(|s| s.to_vec())
                .unwrap_or_else(|| f.bytes.clone())
        }
        None => Vec::new(),
    }
}

/// True if the source contains a group-OCCURS the ML renderer cannot reduce to element 1: a group-OCCURS
/// whose child is itself a group or another OCCURS (nested / multi-dimension). A FLAT group-OCCURS of
/// elementary scalars is supported (the renderers read its element-1 children), so it is NOT complex.
fn ml_group_occurs_complex(name: &str, fields: &HashMap<String, Field>) -> bool {
    match fields.get(name).map(|f| (f.storage.clone(), f.occurs)) {
        Some((Storage::Group { children }, occ)) => {
            if occ > 1 {
                children.iter().any(|c| {
                    match fields.get(c).map(|f| (f.storage.clone(), f.occurs)) {
                        Some((Storage::Group { .. }, _)) => true, // sub-group child
                        Some((_, o)) => o > 1,                    // nested OCCURS child (multi-dim)
                        None => true,
                    }
                })
            } else {
                children.iter().any(|c| ml_group_occurs_complex(c, fields))
            }
        }
        _ => false,
    }
}

/// `XML GENERATE ... SUPPRESS id WHEN {ZERO | SPACE | LOW-VALUE | HIGH-VALUE}` -- true when `id`'s value
/// matches the figurative, so the element is omitted. (JSON GENERATE does not allow SUPPRESS WHEN -- cobc
/// rejects it at compile time; the caller fails that closed as validation.)
fn ml_suppress_when(name: &str, fig: &str, fields: &HashMap<String, Field>) -> bool {
    let Some(f) = read_field(fields, name).ok().flatten() else {
        return false;
    };
    let b = &f.bytes;
    match fig {
        "ZERO" | "ZEROS" | "ZEROES" => match &f.storage {
            Storage::Numeric(a) => source_to_decimal(b, a)
                .map(|d| dec_is_zero(&d))
                .unwrap_or(false),
            _ => !b.is_empty() && b.iter().all(|&c| c == b'0' || c == b' '),
        },
        "SPACE" | "SPACES" => !b.is_empty() && b.iter().all(|&c| c == b' '),
        "LOW-VALUE" | "LOW-VALUES" => !b.is_empty() && b.iter().all(|&c| c == 0x00),
        "HIGH-VALUE" | "HIGH-VALUES" => !b.is_empty() && b.iter().all(|&c| c == 0xFF),
        _ => false,
    }
}

/// The XML element of a field: `<name>...</name>` with children nested, numeric/alnum content (XML-escaped),
/// recursively over the group tree (GnuCOBOL `XML GENERATE`, no declaration).
fn xml_value(
    name: &str,
    fields: &HashMap<String, Field>,
    rename: &HashMap<String, String>,
    suppress: &std::collections::HashSet<String>,
) -> String {
    xml_value_occ(name, None, fields, rename, suppress)
}

/// XML element of a field. `occ` is the element index inside a group-OCCURS table (cobc renders element 1).
fn xml_value_occ(
    name: &str,
    occ: Option<usize>,
    fields: &HashMap<String, Field>,
    rename: &HashMap<String, String>,
    suppress: &std::collections::HashSet<String>,
) -> String {
    let read_occ = |n: &str| match occ {
        Some(i) => read_field(fields, &format!("{n}({i})"))
            .ok()
            .flatten()
            .map(|f| f.bytes)
            .unwrap_or_default(),
        None => ml_first_elem(read_field(fields, n).ok().flatten().as_ref()),
    };
    let inner = match fields.get(name).map(|f| (f.storage.clone(), f.occurs)) {
        Some((Storage::Group { children }, n)) => {
            let child_occ = if n > 1 && occ.is_none() { Some(1) } else { occ };
            children
                .iter()
                .filter(|c| !suppress.contains(*c))
                .map(|c| xml_value_occ(c, child_occ, fields, rename, suppress))
                .collect::<String>()
        }
        Some((Storage::Numeric(attr), _)) => {
            let bytes = match occ {
                Some(_) => read_occ(name),
                None => ml_first_elem(fields.get(name)),
            };
            source_to_decimal(&bytes, &attr)
                .map(|d| num_to_json(&d))
                .unwrap_or_else(|_| "0".into())
        }
        _ => {
            let bytes = read_occ(name);
            let esc = |b: u8| match b {
                b'&' => Some("&amp;"),
                b'<' => Some("&lt;"),
                b'>' => Some("&gt;"),
                b'"' => Some("&quot;"),
                _ => None,
            };
            trimmed_escaped(&bytes, esc)
        }
    };
    let tag = rename.get(name).map(String::as_str).unwrap_or(name);
    format!("<{tag}>{inner}</{tag}>")
}

/// `{JSON|XML} GENERATE dest FROM source [COUNT IN c]` -- serialize the source group into `dest`. `NAME`/
/// `SUPPRESS`/`ON EXCEPTION` are out of subset.
fn exec_ml_generate(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
    xml: bool,
) -> Result<(), RunError> {
    // ON EXCEPTION / NOT ON EXCEPTION are parsed + dispatched by the caller (this `stmt` is the core form);
    // SUPPRESS WHEN <cond> stays out of subset (the rest of NAME/SUPPRESS is wired).
    let dest = match stmt.get(1) {
        Some(Tok::Word(w)) => w.clone(),
        _ => {
            return Err(RunError::Unsupported(
                "JSON/XML GENERATE: missing destination".into(),
            ))
        }
    };
    let fp = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "FROM"))
        .ok_or_else(|| RunError::Unsupported("JSON/XML GENERATE without FROM".into()))?;
    let source = match stmt.get(fp + 1) {
        Some(Tok::Word(w)) => w.clone(),
        _ => {
            return Err(RunError::Unsupported(
                "JSON/XML GENERATE: missing source".into(),
            ))
        }
    };
    // Clause boundaries (NAME / SUPPRESS / COUNT) after the source.
    let pos_of = |kw: &str| {
        stmt.iter()
            .position(|t| matches!(t, Tok::Word(w) if w == kw))
    };
    let (name_p, sup_p, cnt_p) = (pos_of("NAME"), pos_of("SUPPRESS"), pos_of("COUNT"));
    let clause_end = |start: usize| {
        [name_p, sup_p, cnt_p]
            .into_iter()
            .flatten()
            .filter(|&p| p > start)
            .min()
            .unwrap_or(stmt.len())
    };
    // NAME data-name IS "key" [data-name IS "key"]... -> a key-rename map.
    let mut rename: HashMap<String, String> = HashMap::new();
    if let Some(np) = name_p {
        let mut i = np + 1;
        let end = clause_end(np);
        while i < end {
            if let Some(Tok::Word(f)) = stmt.get(i) {
                let mut j = i + 1;
                if matches!(stmt.get(j), Some(Tok::Word(w)) if w == "IS") {
                    j += 1;
                }
                match stmt.get(j) {
                    Some(Tok::Str(s)) => {
                        rename.insert(f.clone(), String::from_utf8_lossy(s).to_string());
                        i = j + 1;
                    }
                    _ => {
                        return Err(RunError::Unsupported(
                            "JSON/XML GENERATE NAME: expected `data-name IS \"key\"`".into(),
                        ))
                    }
                }
            } else {
                i += 1;
            }
        }
    }
    // SUPPRESS data-name [WHEN fig] [data-name ...] -> a set of omitted fields. A bare data-name is always
    // suppressed; `id WHEN {ZERO|SPACE|LOW-VALUE|HIGH-VALUE}` suppresses it only when its value matches (XML
    // only -- cobc rejects SUPPRESS WHEN for JSON GENERATE, so we fail that closed as validation).
    let mut suppress: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Some(sp) = sup_p {
        let seg: Vec<&Tok> = stmt[sp + 1..clause_end(sp)].iter().collect();
        let mut i = 0;
        while i < seg.len() {
            if let Tok::Word(id) = seg[i] {
                i += 1;
                if matches!(seg.get(i), Some(Tok::Word(w)) if *w == "WHEN") {
                    if !xml {
                        return Err(RunError::Unsupported("JSON GENERATE SUPPRESS WHEN: cobc rejects WHEN on a JSON SUPPRESS (a compile error)".into()));
                    }
                    let fig = match seg.get(i + 1) {
                        Some(Tok::Word(w)) => w.clone(),
                        _ => return Err(RunError::Unsupported("XML GENERATE SUPPRESS ... WHEN: expected a figurative (ZERO/SPACE/LOW-VALUE/HIGH-VALUE)".into())),
                    };
                    i += 2;
                    if ml_suppress_when(id, &fig, fields) {
                        suppress.insert(id.clone());
                    }
                } else {
                    suppress.insert(id.clone());
                }
            } else {
                i += 1;
            }
        }
    }
    // cobc 3.2 renders only element 1 of an OCCURS (a `-Wpending` behaviour): a flat group-OCCURS of
    // elementary children -> `{"ROW":{<element-1 children>}}` (now handled via the occ-threaded renderers).
    // A nested / multi-dimension group-OCCURS (a sub-group or further-OCCURS child) would need deeper
    // element-1 addressing the renderer does not do -- still fail closed.
    if ml_group_occurs_complex(&source, fields) {
        return Err(RunError::Unsupported(format!("JSON/XML GENERATE: source `{source}` has a nested or multi-dimension group-OCCURS (cobc handles only a flat group-OCCURS here; deeper nesting is -Wpending)")));
    }
    let outer = rename.get(&source).map(String::as_str).unwrap_or(&source);
    let text = if xml {
        xml_value(&source, fields, &rename, &suppress)
    } else {
        format!(
            "{{\"{}\":{}}}",
            outer,
            json_value(&source, fields, &rename, &suppress)
        )
    };
    let bytes = text.into_bytes();
    let mv = vec![
        Tok::Str(bytes.clone()),
        Tok::Word("TO".to_string()),
        Tok::Word(dest),
    ];
    exec_move(&mv, fields, ctx.decimal_comma)?;
    // optional COUNT IN counter -> the generated length.
    if let Some(cp) = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "COUNT"))
    {
        let mut i = cp + 1;
        if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "IN") {
            i += 1;
        }
        if let Some(Tok::Word(c)) = stmt.get(i) {
            let mv = vec![
                Tok::Word(bytes.len().to_string()),
                Tok::Word("TO".to_string()),
                Tok::Word(c.clone()),
            ];
            exec_move(&mv, fields, ctx.decimal_comma)?;
        }
    }
    Ok(())
}

/// `JSON PARSE` / `XML PARSE` are a faithful no-op: GnuCOBOL 3.2 compiles them with
/// `warning: JSON/XML PARSE is not implemented [-Wpending]` and they do nothing at run time, so the
/// oracle-faithful front-end accepts the statement and leaves the destination unchanged.
fn exec_ml_parse_noop() -> Result<(), RunError> {
    Ok(())
}

/// The displayed bytes of a report element: a `PIC` field holding its SOURCE value (or VALUE literal).
fn format_relem(
    el: &RElem,
    fields: &HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<Vec<u8>, RunError> {
    let mut temp = make_field(
        &el.pic,
        el.value.as_ref(),
        ctx.currency,
        ctx.decimal_comma,
        ctx.dialect,
        Usage::Display,
        (false, false),
        0,
    )?;
    if let Some(src) = &el.source {
        let (sb, sa) = operand_value(&Tok::Word(src.clone()), fields)?;
        move_into(&mut temp, &sb, &sa, ctx.decimal_comma)?;
    }
    Ok(display_bytes(&temp, ctx.decimal_comma))
}

/// Render one report element to its display bytes: a SUM element shows the accumulated total, else the
/// SOURCE field value or the literal VALUE (via [`format_relem`]).
/// Render one report element to display bytes: a SUM element shows the accumulated total for its CONTROL
/// FOOTING's control level; else the SOURCE field value or the literal VALUE (via [`format_relem`]).
fn render_relem(
    el: &RElem,
    fields: &HashMap<String, Field>,
    ctx: &Ctx,
    run: &ReportRun,
    control: &str,
) -> Result<Vec<u8>, RunError> {
    if let Some(s) = &el.sum {
        let dec = run
            .sums
            .get(&(control.to_string(), s.clone()))
            .cloned()
            .unwrap_or(Decimal {
                negative: false,
                digits: vec![0],
                scale: 0,
            });
        let mut temp = make_field(
            &el.pic,
            None,
            ctx.currency,
            ctx.decimal_comma,
            ctx.dialect,
            Usage::Display,
            (false, false),
            0,
        )?;
        let (b, a) = decimal_as_display(&dec);
        move_into(&mut temp, &b, &a, ctx.decimal_comma)?;
        return Ok(display_bytes(&temp, ctx.decimal_comma));
    }
    format_relem(el, fields, ctx)
}

/// The last line a body group (DETAIL / CONTROL FOOTING/HEADING) may occupy before a page break: RD FOOTING,
/// else PAGE LIMIT, else no limit.
fn report_body_limit(def: &ReportDef) -> usize {
    if def.footing > 0 {
        def.footing
    } else if def.page_limit > 0 {
        def.page_limit
    } else {
        usize::MAX
    }
}

/// A fresh per-report run state: current line below HEADING, an empty (or PAGE-LIMIT-sized) page, all SUM
/// accumulators zeroed.
fn fresh_report_run(def: &ReportDef) -> ReportRun {
    let mut sums: HashMap<(String, String), Decimal> = HashMap::new();
    for g in &def.groups {
        if let GType::ControlFooting(c) = &g.gtype {
            for l in &g.lines {
                for e in &l.elems {
                    if let Some(s) = &e.sum {
                        sums.insert(
                            (c.clone(), s.clone()),
                            Decimal {
                                negative: false,
                                digits: vec![0],
                                scale: 0,
                            },
                        );
                    }
                }
            }
        }
    }
    ReportRun {
        line: def.heading.saturating_sub(1),
        page: Vec::new(),
        rh_done: false,
        page_first: true,
        sums,
        ctrl_prev: HashMap::new(),
    }
}

/// Flush the current page buffer to the report's file: padded to PAGE LIMIT (`pad`, the final page at
/// TERMINATE) or to the high-water line (a mid-report page break); trailing-trimmed for LINE SEQUENTIAL.
fn flush_page(run: &ReportRun, def: &ReportDef, ctx: &Ctx, pad: bool) {
    let org = ctx
        .file_defs
        .get(&def.file)
        .map(|d| d.org)
        .unwrap_or(FileOrg::LineSequential);
    let limit = if pad && def.page_limit > 0 {
        def.page_limit.max(run.page.len())
    } else {
        run.page.len()
    };
    let mut files = ctx.files.borrow_mut();
    let st = files.entry(fkey(ctx, &def.file)).or_default();
    for n in 0..limit {
        let mut buf = run.page.get(n).cloned().unwrap_or_default();
        if org == FileOrg::LineSequential {
            while buf.last() == Some(&b' ') {
                buf.pop();
            }
        }
        st.records.push(buf);
    }
}

/// Advance to a new page: flush the current page (high-water, no pad), reset the buffer, emit PAGE HEADING.
fn page_advance(
    run: &mut ReportRun,
    def: &ReportDef,
    fields: &HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    flush_page(run, def, ctx, false);
    run.page = Vec::new();
    run.line = def.heading.saturating_sub(1);
    run.page_first = true;
    if let Some(g) = def.groups.iter().find(|g| g.gtype == GType::PageHeading) {
        place_group(run, def, g, fields, ctx)?;
    }
    Ok(())
}

/// Place one report group's lines into the page buffer at their LINE positions, handling page-break overflow
/// (a body line past FOOTING/PAGE LIMIT advances the page) and the FIRST DETAIL bump for the first body line.
fn place_group(
    run: &mut ReportRun,
    def: &ReportDef,
    group: &RGroup,
    fields: &HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let control = match &group.gtype {
        GType::ControlFooting(c) | GType::ControlHeading(c) => c.clone(),
        _ => String::new(),
    };
    let is_body = !matches!(
        group.gtype,
        GType::ReportHeading | GType::PageHeading | GType::PageFooting
    );
    for rl in &group.lines {
        let mut pos = match rl.spec {
            LineSpec::Abs(n) => n,
            LineSpec::Plus(k) => run.line + k,
        };
        if is_body && run.page_first && pos < def.first_detail {
            pos = def.first_detail;
        }
        if is_body && def.page_limit > 0 && pos > report_body_limit(def) {
            page_advance(run, def, fields, ctx)?;
            pos = match rl.spec {
                LineSpec::Abs(n) => n,
                LineSpec::Plus(k) => run.line + k,
            };
            if pos < def.first_detail {
                pos = def.first_detail;
            }
        }
        let mut buf: Vec<u8> = Vec::new();
        for el in &rl.elems {
            let val = render_relem(el, fields, ctx, run, &control)?;
            let s = el.column.saturating_sub(1);
            let e = s + val.len();
            if buf.len() < e {
                buf.resize(e, b' ');
            }
            buf[s..e].copy_from_slice(&val);
        }
        if pos >= 1 {
            if run.page.len() < pos {
                run.page.resize(pos, Vec::new());
            }
            run.page[pos - 1] = buf;
        }
        run.line = pos;
        if is_body {
            run.page_first = false;
        }
    }
    Ok(())
}

/// Find the report whose groups contain a DETAIL group named `gname`.
fn report_of_detail<'a>(ctx: &'a Ctx, gname: &str) -> Option<(&'a String, &'a ReportDef)> {
    ctx.reports.iter().find(|(_, rd)| {
        rd.groups
            .iter()
            .any(|g| g.name.as_deref() == Some(gname) && g.gtype == GType::Detail)
    })
}

/// The current bytes of a control data-name (for control-break detection).
fn control_value(name: &str, fields: &HashMap<String, Field>) -> Vec<u8> {
    read_field(fields, name)
        .ok()
        .flatten()
        .map(|f| f.bytes)
        .unwrap_or_default()
}

/// Add every CONTROL FOOTING's SUM source into its (control, source) running total.
fn accumulate_sums(run: &mut ReportRun, def: &ReportDef, fields: &HashMap<String, Field>) {
    for g in &def.groups {
        let GType::ControlFooting(c) = &g.gtype else {
            continue;
        };
        for l in &g.lines {
            for e in &l.elems {
                let Some(src) = &e.sum else { continue };
                let Ok((b, a)) = operand_value(&Tok::Word(src.clone()), fields) else {
                    continue;
                };
                let key = (c.clone(), src.clone());
                let cur = run.sums.get(&key).cloned().unwrap_or(Decimal {
                    negative: false,
                    digits: vec![0],
                    scale: 0,
                });
                let (cb, ca) = decimal_as_display(&cur);
                if let Ok((rb, ra)) = wide_op(Op::Add, &cb, &ca, &b, &a) {
                    if let Ok(nd) = source_to_decimal(&rb, &ra) {
                        run.sums.insert(key, nd);
                    }
                }
            }
        }
    }
}

/// `INITIATE report` -- begin a report: fresh page buffer, current line, SUM accumulators.
fn exec_initiate(stmt: &[Tok], ctx: &Ctx) -> Result<(), RunError> {
    let rname = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Ok(()),
    };
    let Some(def) = ctx.reports.get(&rname) else {
        return Ok(());
    };
    let run = fresh_report_run(def);
    REPORT_STATE.with(|m| m.borrow_mut().insert(rname, run));
    Ok(())
}

/// `GENERATE detail-group` -- on first GENERATE emit REPORT/PAGE HEADING + the opening CONTROL HEADINGs; on a
/// control break emit the changed CONTROL FOOTINGs (minor->major, with subtotals) then CONTROL HEADINGs
/// (major->minor); accumulate SUMs; then place the DETAIL (with page-break overflow handling).
fn exec_generate(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let gname = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => {
            return Err(RunError::Unsupported(
                "GENERATE: missing report group".into(),
            ))
        }
    };
    let (rname, def) = match report_of_detail(ctx, &gname) {
        Some((n, d)) => (n.clone(), d.clone()),
        None => return Err(RunError::Unsupported(format!("GENERATE: `{gname}` is not a report group -- cobc rejects GENERATE of a non-report item (\"data item is not part of a report\")"))),
    };
    let mut run = REPORT_STATE
        .with(|m| m.borrow_mut().remove(&rname))
        .unwrap_or_else(|| fresh_report_run(&def));
    let data_controls: Vec<String> = def
        .controls
        .iter()
        .filter(|c| c.as_str() != "FINAL")
        .cloned()
        .collect();
    let find = |pred: GType| def.groups.iter().find(move |g| g.gtype == pred);
    if !run.rh_done {
        if let Some(g) = def.groups.iter().find(|g| g.gtype == GType::ReportHeading) {
            place_group(&mut run, &def, g, fields, ctx)?;
        }
        if let Some(g) = def.groups.iter().find(|g| g.gtype == GType::PageHeading) {
            place_group(&mut run, &def, g, fields, ctx)?;
        }
        run.rh_done = true;
        if let Some(g) = find(GType::ControlHeading("FINAL".into())) {
            place_group(&mut run, &def, g, fields, ctx)?;
        }
        for c in &data_controls {
            if let Some(g) = def
                .groups
                .iter()
                .find(|g| g.gtype == GType::ControlHeading(c.clone()))
            {
                place_group(&mut run, &def, g, fields, ctx)?;
            }
        }
        for c in &def.controls {
            run.ctrl_prev.insert(c.clone(), control_value(c, fields));
        }
    } else {
        let mut bl: Option<usize> = None;
        for (idx, c) in data_controls.iter().enumerate() {
            if control_value(c, fields)
                != *run.ctrl_prev.get(c).cloned().get_or_insert_with(Vec::new)
            {
                bl = Some(idx);
                break;
            }
        }
        if let Some(bl) = bl {
            for c in data_controls[bl..].iter().rev() {
                if let Some(g) = def
                    .groups
                    .iter()
                    .find(|g| g.gtype == GType::ControlFooting(c.clone()))
                {
                    place_group(&mut run, &def, g, fields, ctx)?;
                }
                let keys: Vec<(String, String)> =
                    run.sums.keys().filter(|(cc, _)| cc == c).cloned().collect();
                for k in keys {
                    run.sums.insert(
                        k,
                        Decimal {
                            negative: false,
                            digits: vec![0],
                            scale: 0,
                        },
                    );
                }
            }
            for c in data_controls[bl..].iter() {
                if let Some(g) = def
                    .groups
                    .iter()
                    .find(|g| g.gtype == GType::ControlHeading(c.clone()))
                {
                    place_group(&mut run, &def, g, fields, ctx)?;
                }
            }
            for c in &data_controls {
                run.ctrl_prev.insert(c.clone(), control_value(c, fields));
            }
        }
    }
    accumulate_sums(&mut run, &def, fields);
    if let Some(g) = def
        .groups
        .iter()
        .find(|g| g.name.as_deref() == Some(gname.as_str()) && g.gtype == GType::Detail)
    {
        place_group(&mut run, &def, g, fields, ctx)?;
    }
    REPORT_STATE.with(|m| m.borrow_mut().insert(rname, run));
    Ok(())
}

/// `TERMINATE report` -- emit the CONTROL FOOTINGs (data minor->major, then FINAL) and REPORT FOOTING, then
/// flush the final page (padded to PAGE LIMIT).
fn exec_terminate(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let rname = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Ok(()),
    };
    let Some(def) = ctx.reports.get(&rname).cloned() else {
        return Ok(());
    };
    let Some(mut run) = REPORT_STATE.with(|m| m.borrow_mut().remove(&rname)) else {
        return Ok(());
    };
    let data_controls: Vec<String> = def
        .controls
        .iter()
        .filter(|c| c.as_str() != "FINAL")
        .cloned()
        .collect();
    for c in data_controls.iter().rev() {
        if let Some(g) = def
            .groups
            .iter()
            .find(|g| g.gtype == GType::ControlFooting(c.clone()))
        {
            place_group(&mut run, &def, g, fields, ctx)?;
        }
    }
    if let Some(g) = def
        .groups
        .iter()
        .find(|g| g.gtype == GType::ControlFooting("FINAL".into()))
    {
        place_group(&mut run, &def, g, fields, ctx)?;
    }
    if let Some(g) = def.groups.iter().find(|g| g.gtype == GType::ReportFooting) {
        place_group(&mut run, &def, g, fields, ctx)?;
    }
    flush_page(&run, &def, ctx, true);
    Ok(())
}

/// `RELEASE record [FROM id]` -- write a record to its sort file during a SORT INPUT PROCEDURE (the records
/// accumulate, then SORT orders them).
fn exec_release(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let rec = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("RELEASE: missing record".into())),
    };
    if let Some(fp) = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "FROM"))
    {
        if let Some(src) = stmt.get(fp + 1) {
            let mv = vec![
                src.clone(),
                Tok::Word("TO".to_string()),
                Tok::Word(rec.clone()),
            ];
            exec_move(&mv, fields, ctx.decimal_comma)?;
        }
    }
    let def = {
        let fname = ctx.record_files.get(&rec).ok_or_else(|| {
            RunError::Unsupported(format!("RELEASE `{rec}`: not an SD/FD record"))
        })?;
        ctx.file_defs
            .get(fname)
            .ok_or_else(|| RunError::Unsupported(format!("RELEASE `{rec}`: not an SD/FD record")))?
            .clone()
    };
    // Upstream (277a07c2e): RELEASE is only allowed on a record description of the SD (sort) file --
    // releasing an FD record is rejected ("RELEASE not allowed on this record item").
    if def.org != FileOrg::Sort {
        return Err(RunError::Unsupported(format!(
            "RELEASE `{rec}`: record is not a sort-file record"
        )));
    }
    let bytes = read_field(fields, &rec)?
        .map(|f| f.bytes)
        .unwrap_or_default();
    ctx.files
        .borrow_mut()
        .entry(def.assign.clone())
        .or_default()
        .records
        .push(bytes);
    Ok(())
}

/// `RETURN sort-file [RECORD] [INTO id] [AT END imperative] [END-RETURN]` -- read the next ordered record
/// from a sort file during a SORT OUTPUT PROCEDURE; at end-of-set run the AT END imperative.
fn exec_return(
    toks: &[Tok],
    pos: &mut usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    exec: bool,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    let file = match toks.get(*pos) {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("RETURN: missing sort file".into())),
    };
    *pos += 1;
    // Upstream typeck.c cb_emit_return (277a07c2e): the RETURN target must be an `SD` file description
    // (an FD, or any undeclared/record name, is a compile-time error -- also checked by `-fsyntax-only`).
    let def = ctx
        .file_defs
        .get(&file)
        .ok_or_else(|| RunError::Unsupported(format!("RETURN: `{file}` is not a declared file")))?
        .clone();
    if def.org != FileOrg::Sort {
        return Err(RunError::Unsupported(format!(
            "RETURN: `{file}` must be an SD filename"
        )));
    }
    while matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "RECORD") {
        *pos += 1;
    }
    let mut into: Option<String> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "INTO") {
        *pos += 1;
        if let Some(Tok::Word(w)) = toks.get(*pos) {
            into = Some(w.clone());
            *pos += 1;
        }
    }
    let mut at_end: Option<usize> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "AT") {
        *pos += 1;
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END") {
            *pos += 1;
        }
        at_end = Some(*pos);
        let mut scan = *pos;
        let _ = run_block(toks, &mut scan, fields, out, false, ctx)?;
        *pos = scan;
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-RETURN") {
        *pos += 1;
    }
    if !exec {
        return Ok(false);
    }
    let reclen = read_field(fields, def.primary_record())?
        .map(|f| f.bytes.len())
        .unwrap_or(0);
    let next = {
        let files = ctx.files.borrow();
        files
            .get(&fkey(ctx, &file))
            .and_then(|st| st.records.get(st.read_pos).cloned())
    };
    match next {
        Some(mut bytes) => {
            if let Some(st) = ctx.files.borrow_mut().get_mut(&fkey(ctx, &file)) {
                st.read_pos += 1;
            }
            bytes.resize(reclen, b' ');
            write_field(fields, def.primary_record(), |f| {
                f.bytes = bytes;
                Ok(())
            })?;
            if let Some(id) = into {
                let mv = vec![
                    Tok::Word(def.primary_record().to_string()),
                    Tok::Word("TO".to_string()),
                    Tok::Word(id),
                ];
                exec_move(&mv, fields, ctx.decimal_comma)?;
            }
            Ok(false)
        }
        None => {
            if let Some(s) = at_end {
                let mut p = s;
                return run_block(toks, &mut p, fields, out, true, ctx);
            }
            Ok(false)
        }
    }
}

/// The byte offset + length of a SORT/MERGE key within the sort record: the whole record when the key
/// names the record, otherwise the key leaf's position within the SD group.
fn sort_key_span(
    record: &str,
    key: &str,
    reclen: usize,
    fields: &HashMap<String, Field>,
) -> Option<(usize, usize)> {
    if key == record {
        return Some((0, reclen));
    }
    if let Some(Field {
        storage: Storage::Group { children },
        ..
    }) = fields.get(record)
    {
        let mut off = 0;
        for c in children {
            let len = fields.get(c).map(|f| f.bytes.len()).unwrap_or(0);
            if c == key {
                return Some((off, len));
            }
            off += len;
        }
    }
    let kl = fields.get(key).map(|f| f.bytes.len()).unwrap_or(0);
    if kl == reclen {
        Some((0, reclen))
    } else {
        None
    }
}

/// (offset, length) of the RECORD KEY within an INDEXED file's record.
fn indexed_key_span(
    def: &FileDef,
    reclen: usize,
    fields: &HashMap<String, Field>,
) -> Result<(usize, usize), RunError> {
    let key = def.record_key.as_ref().ok_or_else(|| {
        RunError::Unsupported(format!("INDEXED file `{}` has no RECORD KEY", def.name))
    })?;
    sort_key_span(def.primary_record(), key, reclen, fields).ok_or_else(|| {
        RunError::Unsupported(format!(
            "INDEXED RECORD KEY `{key}` is not a field of the record"
        ))
    })
}

/// The RECORD KEY bytes of a stored record.
fn rec_key_bytes(r: &[u8], koff: usize, klen: usize) -> &[u8] {
    let s = koff.min(r.len());
    let e = (koff + klen).min(r.len());
    &r[s..e]
}

/// Indices of non-empty records in ascending RECORD KEY order (stable on ties).
fn indexed_order(records: &[Vec<u8>], koff: usize, klen: usize) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..records.len())
        .filter(|&i| !records[i].is_empty())
        .collect();
    idx.sort_by(|&a, &b| {
        rec_key_bytes(&records[a], koff, klen).cmp(rec_key_bytes(&records[b], koff, klen))
    });
    idx
}

/// `DELETE file [RECORD]` -- remove the record at the current key: the RELATIVE record at the current
/// RELATIVE KEY, or the INDEXED record whose RECORD KEY equals the key field. Status `"23"` if no such
/// record. DELETE on a sequential file is invalid (out of subset).
fn exec_delete(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let file = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("DELETE: missing file".into())),
    };
    let def = ctx
        .file_defs
        .get(&file)
        .ok_or_else(|| RunError::Unsupported(format!("DELETE: `{file}` is not a declared file")))?
        .clone();
    let deleted = match def.org {
        FileOrg::Relative => {
            let pos = relative_key_value(&def, fields)?;
            let mut files = ctx.files.borrow_mut();
            match files.get_mut(&fkey(ctx, &file)) {
                Some(st) if pos <= st.records.len() && !st.records[pos - 1].is_empty() => {
                    st.records[pos - 1] = Vec::new();
                    true
                }
                _ => false,
            }
        }
        FileOrg::Indexed => {
            let reclen = read_field(fields, def.primary_record())?
                .map(|f| f.bytes.len())
                .unwrap_or(0);
            let (koff, klen) = indexed_key_span(&def, reclen, fields)?;
            let want = read_field(fields, def.record_key.as_ref().unwrap())?
                .map(|f| f.bytes)
                .unwrap_or_default();
            let mut files = ctx.files.borrow_mut();
            match files.get_mut(&fkey(ctx, &file)) {
                Some(st) => {
                    match st.records.iter().position(|r| {
                        !r.is_empty() && rec_key_bytes(r, koff, klen) == want.as_slice()
                    }) {
                        Some(p) => {
                            st.records[p] = Vec::new();
                            true
                        }
                        None => false,
                    }
                }
                None => false,
            }
        }
        _ => {
            return Err(RunError::Unsupported(
                "DELETE requires a RELATIVE or INDEXED file (invalid on SEQUENTIAL)".into(),
            ))
        }
    };
    set_file_status(fields, &def, if deleted { "00" } else { "23" });
    Ok(())
}

/// A START handler block (`INVALID KEY` / `NOT INVALID KEY`) or the relation head ends at END-START, an
/// outer scope terminator, a period, or a following `NOT INVALID` clause.
fn at_start_block_end(toks: &[Tok], p: usize) -> bool {
    match toks.get(p) {
        None | Some(Tok::Dot) => true,
        Some(Tok::Word(w)) if w == "END-START" || SCOPE_ENDERS.contains(&w.as_str()) => true,
        Some(Tok::Word(w))
            if w == "NOT" && matches!(toks.get(p + 1), Some(Tok::Word(x)) if x == "INVALID") =>
        {
            true
        }
        _ => false,
    }
}

/// `START file [KEY [IS] {= | > | >= | < | <= | NOT < | NOT >} key-field] [INVALID KEY imp] [NOT INVALID
/// KEY imp] [END-START]` -- position a RELATIVE/INDEXED file so the next sequential READ returns the first
/// record whose key satisfies the relation (default `=` on the current key). Status `"23"` if none qualifies;
/// the INVALID KEY / NOT INVALID KEY imperatives run on miss / success.
fn exec_start(
    toks: &[Tok],
    pos: &mut usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    exec: bool,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    let file = match toks.get(*pos) {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("START: missing file".into())),
    };
    *pos += 1;
    // The relation head runs until END-START / INVALID KEY / NOT INVALID KEY / a scope boundary.
    let head_start = *pos;
    while *pos < toks.len()
        && !at_start_block_end(toks, *pos)
        && !matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "INVALID")
    {
        *pos += 1;
    }
    let head: Vec<Tok> = toks[head_start..*pos].to_vec();
    let mut invalid_blk: Option<Vec<Tok>> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "INVALID") {
        *pos += 1;
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "KEY") {
            *pos += 1;
        }
        let s = *pos;
        while *pos < toks.len() && !at_start_block_end(toks, *pos) {
            *pos += 1;
        }
        invalid_blk = Some(toks[s..*pos].to_vec());
    }
    let mut not_invalid_blk: Option<Vec<Tok>> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "NOT") {
        *pos += 1;
        for kw in ["INVALID", "KEY"] {
            if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == kw) {
                *pos += 1;
            }
        }
        let s = *pos;
        while *pos < toks.len() && !at_start_block_end(toks, *pos) {
            *pos += 1;
        }
        not_invalid_blk = Some(toks[s..*pos].to_vec());
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-START") {
        *pos += 1;
    }
    if !exec {
        return Ok(false);
    }
    let def = ctx
        .file_defs
        .get(&file)
        .ok_or_else(|| RunError::Unsupported(format!("START: `{file}` is not a declared file")))?
        .clone();
    if !matches!(def.org, FileOrg::Relative | FileOrg::Indexed) {
        return Err(RunError::Unsupported(
            "START requires a RELATIVE or INDEXED file (invalid on SEQUENTIAL)".into(),
        ));
    }
    // Parse the relation (default `=`) and the optional named key field from the head.
    let mut rel = "=".to_string();
    let mut key_field: Option<String> = None;
    if let Some(kp) = head
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "KEY"))
    {
        let mut i = kp + 1;
        if matches!(head.get(i), Some(Tok::Word(w)) if w == "IS") {
            i += 1;
        }
        rel = match head.get(i).and_then(|t| {
            if let Tok::Word(w) = t {
                Some(w.as_str())
            } else {
                None
            }
        }) {
            Some("=") | Some("EQUAL") => {
                i += 1;
                if matches!(head.get(i), Some(Tok::Word(w)) if w == "TO") {
                    i += 1;
                }
                "=".into()
            }
            Some(">=") => {
                i += 1;
                ">=".into()
            }
            Some("<=") => {
                i += 1;
                "<=".into()
            }
            Some(">") | Some("GREATER") => {
                i += 1;
                if matches!(head.get(i), Some(Tok::Word(w)) if w == "THAN") {
                    i += 1;
                }
                ">".into()
            }
            Some("<") | Some("LESS") => {
                i += 1;
                if matches!(head.get(i), Some(Tok::Word(w)) if w == "THAN") {
                    i += 1;
                }
                "<".into()
            }
            Some("NOT") => {
                i += 1;
                let r = match head.get(i).and_then(|t| if let Tok::Word(w) = t { Some(w.as_str()) } else { None }) {
                    Some("<") | Some("LESS") => ">=",
                    Some(">") | Some("GREATER") => "<=",
                    // `NOT =` / `NOT EQUAL` is not a legal START relation -- cobc rejects it at compile time
                    // ("NOT EQUAL condition not allowed on START statement"), so refusing it is faithful.
                    _ => return Err(RunError::Unsupported("START KEY: NOT EQUAL relation is not allowed on START (only NOT < / NOT >)".into())),
                };
                i += 1;
                if matches!(head.get(i), Some(Tok::Word(w)) if w == "THAN") {
                    i += 1;
                }
                r.into()
            }
            other => {
                return Err(RunError::Unsupported(format!(
                    "START KEY: unrecognized relation {other:?} (expected = > < >= <= NOT< NOT>)"
                )))
            }
        };
        if let Some(Tok::Word(field)) = head.get(i) {
            key_field = Some(field.clone());
        }
    }
    // foundpos is the `read_pos` value the next sequential READ should resume from: a record index for
    // RELATIVE, an index into the ascending-key order for INDEXED.
    let foundpos = match def.org {
        FileOrg::Relative => {
            let keyval = match &key_field {
                Some(f) => resolve_int(f, fields)
                    .map(|v| v.max(0) as usize)
                    .unwrap_or(relative_key_value(&def, fields)?),
                None => relative_key_value(&def, fields)?,
            };
            let files = ctx.files.borrow();
            let mut fp = None;
            if let Some(st) = files.get(&fkey(ctx, &file)) {
                for n in 1..=st.records.len() {
                    if st.records[n - 1].is_empty() {
                        continue;
                    }
                    let ok = match rel.as_str() {
                        "=" => n == keyval,
                        ">" => n > keyval,
                        ">=" => n >= keyval,
                        "<" => n < keyval,
                        "<=" => n <= keyval,
                        _ => false,
                    };
                    if ok {
                        fp = Some(n - 1);
                        break;
                    }
                }
            }
            fp
        }
        FileOrg::Indexed => {
            let reclen = read_field(fields, def.primary_record())?
                .map(|f| f.bytes.len())
                .unwrap_or(0);
            let (koff, klen) = indexed_key_span(&def, reclen, fields)?;
            let kf = key_field
                .or_else(|| def.record_key.clone())
                .ok_or_else(|| {
                    RunError::Unsupported(format!("INDEXED file `{}` has no RECORD KEY", def.name))
                })?;
            let want = read_field(fields, &kf)?
                .map(|f| f.bytes)
                .unwrap_or_default();
            let files = ctx.files.borrow();
            let mut fp = None;
            if let Some(st) = files.get(&fkey(ctx, &file)) {
                let order = indexed_order(&st.records, koff, klen);
                for (oi, &ri) in order.iter().enumerate() {
                    let rk = rec_key_bytes(&st.records[ri], koff, klen);
                    let ok = match rel.as_str() {
                        "=" => rk == want.as_slice(),
                        ">" => rk > want.as_slice(),
                        ">=" => rk >= want.as_slice(),
                        "<" => rk < want.as_slice(),
                        "<=" => rk <= want.as_slice(),
                        _ => false,
                    };
                    if ok {
                        fp = Some(oi);
                        break;
                    }
                }
            }
            fp
        }
        _ => None,
    };
    match foundpos {
        Some(p) => {
            if let Some(st) = ctx.files.borrow_mut().get_mut(&fkey(ctx, &file)) {
                st.read_pos = p;
            }
            set_file_status(fields, &def, "00");
            if let Some(b) = &not_invalid_blk {
                return run_handler(b, fields, out, ctx);
            }
        }
        None => {
            set_file_status(fields, &def, "23");
            if let Some(b) = &invalid_blk {
                return run_handler(b, fields, out, ctx);
            }
        }
    }
    Ok(false)
}

/// `READ file [NEXT] [RECORD] [INTO id] [AT END imperative] [END-READ]` -- read the next sequential record
/// into the FD record (and optionally MOVE it INTO `id`); at end-of-file set status `"10"` and run the AT
/// END imperative. `NOT AT END` / keyed reads are out of subset.
fn exec_read(
    toks: &[Tok],
    pos: &mut usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    exec: bool,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    let file = match toks.get(*pos) {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("READ: missing file".into())),
    };
    *pos += 1;
    let mut had_next = false;
    while matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "NEXT" || w == "RECORD") {
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "NEXT") {
            had_next = true;
        }
        *pos += 1;
    }
    let mut into: Option<String> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "INTO") {
        *pos += 1;
        if let Some(Tok::Word(w)) = toks.get(*pos) {
            into = Some(w.clone());
            *pos += 1;
        }
    }
    // Optional handlers: `AT END` / `INVALID KEY` run on a miss/EOF; `NOT AT END` / `NOT INVALID KEY` run
    // on a successful read. Each is collected as its own token block (bounded by `at_read_terminator`).
    let w_eq = |p: usize, k: &str| matches!(toks.get(p), Some(Tok::Word(w)) if w == k);
    let mut at_end: Option<Vec<Tok>> = None;
    let mut not_end: Option<Vec<Tok>> = None;
    loop {
        if w_eq(*pos, "AT") || w_eq(*pos, "INVALID") {
            *pos += 1;
            if w_eq(*pos, "END") || w_eq(*pos, "KEY") {
                *pos += 1;
            }
            let start = *pos;
            let other = |q: usize| {
                matches!(toks.get(q), Some(Tok::Word(w)) if w == "NOT")
                    && matches!(toks.get(q + 1), Some(Tok::Word(x)) if x == "AT" || x == "INVALID")
            };
            *pos = scan_handler_block(toks, start, |w: &str| w == "END-READ", other);
            at_end = Some(toks[start..*pos].to_vec());
        } else if w_eq(*pos, "NOT") {
            *pos += 1;
            for kw in ["AT", "END", "INVALID", "KEY"] {
                if w_eq(*pos, kw) {
                    *pos += 1;
                }
            }
            let start = *pos;
            let other = |q: usize| {
                matches!(toks.get(q), Some(Tok::Word(w)) if w == "NOT")
                    && matches!(toks.get(q + 1), Some(Tok::Word(x)) if x == "AT" || x == "INVALID")
            };
            *pos = scan_handler_block(toks, start, |w: &str| w == "END-READ", other);
            not_end = Some(toks[start..*pos].to_vec());
        } else {
            break;
        }
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-READ") {
        *pos += 1;
    }
    if !exec {
        return Ok(false);
    }
    let def = ctx
        .file_defs
        .get(&file)
        .ok_or_else(|| RunError::Unsupported(format!("READ: `{file}` is not a declared file")))?
        .clone();
    // Upstream cob_read_next (current head): a READ requires the file open INPUT or I-O; any other
    // state (closed, locked, output) reports status 47 (INPUT DENIED) and runs no AT END handler.
    // (62b39805c's 30-status guards are indexed_close-internal; the sequential READ path is 47.)
    if let Some(st) = ctx.files.borrow().get(&fkey(ctx, &file)) {
        if st.mode != 1 && st.mode != 4 {
            set_file_status(fields, &def, "47");
            return Ok(false);
        }
    }
    let reclen = read_field(fields, def.primary_record())?
        .map(|f| f.bytes.len())
        .unwrap_or(0);
    let loaded: Option<Vec<u8>> = match def.org {
        // RELATIVE random read: by the RELATIVE KEY (no position advance).
        FileOrg::Relative if !had_next => {
            let pos = relative_key_value(&def, fields)?;
            let files = ctx.files.borrow();
            files
                .get(&fkey(ctx, &file))
                .and_then(|st| st.records.get(pos - 1))
                .filter(|r| !r.is_empty())
                .cloned()
        }
        // RELATIVE sequential read: the next non-empty slot, setting the RELATIVE KEY to its number.
        FileOrg::Relative => {
            let mut found = None;
            if let Some(st) = ctx.files.borrow_mut().get_mut(&fkey(ctx, &file)) {
                while st.read_pos < st.records.len() {
                    let p = st.read_pos;
                    st.read_pos += 1;
                    if !st.records[p].is_empty() {
                        found = Some((p + 1, st.records[p].clone()));
                        break;
                    }
                }
            }
            match found {
                Some((relnum, bytes)) => {
                    if let Some(key) = &def.rel_key {
                        let mv = vec![
                            Tok::Word(relnum.to_string()),
                            Tok::Word("TO".to_string()),
                            Tok::Word(key.clone()),
                        ];
                        exec_move(&mv, fields, ctx.decimal_comma)?;
                    }
                    Some(bytes)
                }
                None => None,
            }
        }
        // INDEXED random read: by the RECORD KEY field value (no position advance). Only when ACCESS is
        // RANDOM/DYNAMIC -- under ACCESS SEQUENTIAL a plain READ is a sequential next-in-key-order read.
        FileOrg::Indexed if !had_next && def.access_random => {
            let (koff, klen) = indexed_key_span(&def, reclen, fields)?;
            let want = read_field(fields, def.record_key.as_ref().unwrap())?
                .map(|f| f.bytes)
                .unwrap_or_default();
            let files = ctx.files.borrow();
            files.get(&fkey(ctx, &file)).and_then(|st| {
                st.records
                    .iter()
                    .find(|r| !r.is_empty() && rec_key_bytes(r, koff, klen) == want.as_slice())
                    .cloned()
            })
        }
        // INDEXED sequential read: the next record in ascending RECORD KEY order.
        FileOrg::Indexed => {
            let (koff, klen) = indexed_key_span(&def, reclen, fields)?;
            let mut found = None;
            if let Some(st) = ctx.files.borrow_mut().get_mut(&fkey(ctx, &file)) {
                let order = indexed_order(&st.records, koff, klen);
                if st.read_pos < order.len() {
                    found = Some(st.records[order[st.read_pos]].clone());
                    st.read_pos += 1;
                }
            }
            found
        }
        // sequential / line-sequential: the next record.
        _ => {
            let bytes = {
                let files = ctx.files.borrow();
                files
                    .get(&fkey(ctx, &file))
                    .and_then(|st| st.records.get(st.read_pos).cloned())
            };
            if bytes.is_some() {
                if let Some(st) = ctx.files.borrow_mut().get_mut(&fkey(ctx, &file)) {
                    st.read_pos += 1;
                }
            }
            bytes
        }
    };
    match loaded {
        Some(mut bytes) => {
            if let Some(dep) = &def.varying_dep {
                // variable-length record: publish the DEPENDING ON length FIRST (so the FD record's OCCURS
                // DEPENDING describes the live n-byte record area), then store the record image into it.
                let n = bytes.len();
                let mv = vec![
                    Tok::Word(n.to_string()),
                    Tok::Word("TO".to_string()),
                    Tok::Word(dep.clone()),
                ];
                exec_move(&mv, fields, ctx.decimal_comma)?;
                // Store the record image into the FD record's elementary OCCURS DEPENDING leaf, overwriting
                // its first n bytes IN PLACE (the leaf is built at MAX; set_field_image would shrink the
                // physical buffer to the live size, so a later longer record would read truncated). A record
                // read then truncates the MAX buffer to the live DEPENDING length = exactly these n bytes.
                let leaf = match fields.get(def.primary_record()).map(|f| f.storage.clone()) {
                    Some(Storage::Group { children }) => children
                        .iter()
                        .find(|c| !c.starts_with('\u{3}'))
                        .cloned()
                        .unwrap_or_else(|| def.primary_record().to_string()),
                    _ => def.primary_record().to_string(),
                };
                if let Some(lf) = fields.get_mut(&leaf) {
                    if lf.bytes.len() < n {
                        lf.bytes.resize(n, b' ');
                    }
                    lf.bytes[..n].copy_from_slice(&bytes); // overwrite the front; the tail (LOW-VALUES from OPEN) persists
                }
                // READ ... INTO moves the PHYSICAL record area (its low-values tail past the just-read record
                // included), NOT the logical recsize -- cobc copies the whole record buffer, so for a freshly
                // read variable record the receiver carries cobc's NUL tail. Capped/space-padded to the
                // receiver width (a longer receiver). Falls back to the raw record bytes if the leaf is absent.
                if let Some(id) = &into {
                    let area = fields
                        .get(&leaf)
                        .map(|f| f.bytes.clone())
                        .unwrap_or_else(|| bytes.clone());
                    let tlen = read_field(fields, id)?
                        .map(|f| f.bytes.len())
                        .filter(|&n| n > 0)
                        .unwrap_or(area.len());
                    let mut v = area;
                    v.resize(tlen, b' ');
                    set_field_image(fields, id, &v)?;
                }
            } else if def.org == FileOrg::LineSequential {
                // Line-sequential READ pads a short line with spaces (libcob lineseq_read memsets).
                bytes.resize(reclen, b' ');
                write_field(fields, def.primary_record(), |f| {
                    f.bytes = bytes;
                    Ok(())
                })?;
                if let Some(id) = &into {
                    let mv = vec![
                        Tok::Word(def.primary_record().to_string()),
                        Tok::Word("TO".to_string()),
                        Tok::Word(id.clone()),
                    ];
                    exec_move(&mv, fields, ctx.decimal_comma)?;
                }
            } else {
                // Record-sequential / relative / indexed READ: the record area's tail beyond the read
                // bytes is left AS-IS (GnuCOBOL cob_seq_read: "we leave the data not read as-is"). With
                // several alternative FD records (or differing record lengths) the previously-read bytes
                // stay visible through the shared record-area views -- verified against the oracle.
                let mut area = bytes;
                if area.len() < reclen {
                    let prev = read_field(fields, def.primary_record())?
                        .map(|f| f.bytes)
                        .unwrap_or_default();
                    let tail = prev
                        .get(area.len()..)
                        .map(|s| s.to_vec())
                        .unwrap_or_default();
                    area.extend_from_slice(&tail);
                    area.truncate(reclen);
                }
                write_field(fields, def.primary_record(), |f| {
                    f.bytes = area;
                    Ok(())
                })?;
                if let Some(id) = &into {
                    let mv = vec![
                        Tok::Word(def.primary_record().to_string()),
                        Tok::Word("TO".to_string()),
                        Tok::Word(id.clone()),
                    ];
                    exec_move(&mv, fields, ctx.decimal_comma)?;
                }
            }
            set_file_status(fields, &def, "00");
            if let Some(b) = &not_end {
                return run_handler(b, fields, out, ctx);
            }
            Ok(false)
        }
        None => {
            // a RANDOM-read miss is "23" (record not found); a SEQUENTIAL next-end is "10" (EOF). A read is
            // random only for RELATIVE plain READ, or INDEXED plain READ under ACCESS RANDOM/DYNAMIC -- an
            // INDEXED ACCESS-SEQUENTIAL plain READ runs off the end as EOF, not as a not-found miss.
            let random_read = (matches!(def.org, FileOrg::Relative) && !had_next)
                || (matches!(def.org, FileOrg::Indexed) && !had_next && def.access_random);
            let code = if random_read { "23" } else { "10" };
            set_file_status(fields, &def, code);
            if let Some(b) = &at_end {
                return run_handler(b, fields, out, ctx);
            }
            Ok(false)
        }
    }
}

/// Resolve a single operand token to `(bytes, attr)` (identifier -> its stored numeric/alnum form;
/// string literal -> alnum; numeric literal -> zoned display).
fn operand_value(
    t: &Tok,
    fields: &HashMap<String, Field>,
) -> Result<(Vec<u8>, FieldAttr), RunError> {
    match t {
        Tok::Str(s) => Ok((s.clone(), alnum_attr())),
        Tok::AllLiteral(s) => Ok((s.clone(), alnum_attr())),
        Tok::Word(w) => {
            if let Some(f) = read_field(fields, w)? {
                match &f.storage {
                    Storage::Numeric(a) => Ok((f.bytes.clone(), *a)),
                    Storage::Alpha(a) => Ok((f.bytes.clone(), *a)),
                    Storage::Edited(..) => Ok((f.bytes.clone(), alnum_attr())),
                    // a group is an alphanumeric value of its concatenated leaves (read_field filled bytes).
                    Storage::Group { .. } => Ok((f.bytes.clone(), alnum_attr())),
                    Storage::Condition { .. } => Err(RunError::Unsupported(
                        "88 condition-name is not a value operand".into(),
                    )),
                }
            } else {
                // a numeric literal operand.
                let dec = parse_num_literal(w)?;
                Ok(decimal_as_display(&dec))
            }
        }
        Tok::Dot => Err(RunError::Unsupported("unexpected '.'".into())),
    }
}

/// The base-10 digits of `n` (most-significant first); `0` -> `[0]`.
fn digits_of(mut n: u128) -> Vec<u8> {
    if n == 0 {
        return vec![0];
    }
    let mut d = Vec::new();
    while n > 0 {
        d.push((n % 10) as u8);
        n /= 10;
    }
    d.reverse();
    d
}

/// The integer value of a [`Decimal`] (its fractional digits dropped), as `i64`.
fn dec_to_i64(d: &Decimal) -> i64 {
    let intlen = d.digits.len().saturating_sub(d.scale.max(0) as usize);
    let mut v: i64 = 0;
    for &dig in &d.digits[..intlen] {
        v = v * 10 + dig as i64;
    }
    if d.negative {
        -v
    } else {
        v
    }
}

/// Dispatch a parsed `FUNCTION name(args)` to the ported `cob_intr_*` runtime, returning the result
/// `(bytes, attr)`. Each helper's libcob-faithful result is reproduced exactly; `LENGTH`/`BYTE-LENGTH`
/// reproduce cobc's *compile-time* constant fold (a minimal-width integer like `10`, not the 9-digit
/// binary the runtime returns) because cobc never calls libcob for them. Names outside the wired subset
/// fail closed.
fn eval_intrinsic(
    name: &str,
    args: &[(Vec<u8>, FieldAttr)],
) -> Result<(Vec<u8>, FieldAttr), RunError> {
    use crate::intrinsic as ix;
    let a0 = || {
        args.first()
            .ok_or_else(|| RunError::Unsupported(format!("FUNCTION {name}: missing argument")))
    };
    let pair =
        |i: usize, j: usize| -> Result<(&(Vec<u8>, FieldAttr), &(Vec<u8>, FieldAttr)), RunError> {
            match (args.get(i), args.get(j)) {
                (Some(x), Some(y)) => Ok((x, y)),
                _ => Err(RunError::Unsupported(format!(
                    "FUNCTION {name}: needs two arguments"
                ))),
            }
        };
    let list = || -> Result<Vec<(&[u8], &FieldAttr)>, RunError> {
        if args.is_empty() {
            return Err(RunError::Unsupported(format!(
                "FUNCTION {name}: needs at least one argument"
            )));
        }
        Ok(args.iter().map(|(b, at)| (b.as_slice(), at)).collect())
    };
    let r = match name {
        // cobc folds LENGTH/BYTE-LENGTH of a fixed item to an integer literal at compile time.
        "LENGTH" | "BYTE-LENGTH" => {
            let n = a0()?.0.len() as u128;
            decimal_as_display(&Decimal { negative: false, digits: digits_of(n), scale: 0 })
        }
        "UPPER-CASE" => ix::cob_intr_upper_case(0, 0, &a0()?.0),
        "LOWER-CASE" => ix::cob_intr_lower_case(0, 0, &a0()?.0),
        "REVERSE" => ix::cob_intr_reverse(0, 0, &a0()?.0),
        "TRIM" => {
            let a = a0()?;
            ix::cob_intr_trim(0, 0, &a.0, &a.1, 0)
        }
        "NUMVAL" => ix::cob_intr_numval(&a0()?.0),
        "NUMVAL-C" => ix::cob_intr_numval_c(&a0()?.0),
        "INTEGER" => {
            let a = a0()?;
            ix::cob_intr_integer(&a.0, &a.1)
        }
        "INTEGER-PART" => {
            let a = a0()?;
            ix::cob_intr_integer_part(&a.0, &a.1)
        }
        "FRACTION-PART" => {
            let a = a0()?;
            ix::cob_intr_fraction_part(&a.0, &a.1)
        }
        "ABS" | "ABSOLUTE-VALUE" => {
            let a = a0()?;
            ix::cob_intr_abs(&a.0, &a.1)
        }
        "FACTORIAL" => {
            let a = a0()?;
            ix::cob_intr_factorial(&a.0, &a.1)
        }
        "SIGN" => {
            let a = a0()?;
            ix::cob_intr_sign(&a.0, &a.1)
        }
        "ORD" => ix::cob_intr_ord(&a0()?.0, current_collation().as_ref()),
        "CHAR" => {
            let a = a0()?;
            let (f, arg_err) = ix::cob_intr_char(&a.0, &a.1, current_collation().as_ref());
            if arg_err {
                set_exception("EC-ARGUMENT-FUNCTION");
            }
            f
        }
        "HEX-OF" => ix::cob_intr_hex_of(&a0()?.0),
        "HEX-TO-CHAR" => ix::cob_intr_hex_to_char(&a0()?.0),
        "MOD" => {
            let (x, y) = pair(0, 1)?;
            ix::cob_intr_mod(&x.0, &x.1, &y.0, &y.1)
        }
        "REM" => {
            let (x, y) = pair(0, 1)?;
            ix::cob_intr_rem(&x.0, &x.1, &y.0, &y.1)
        }
        "MAX" => ix::cob_intr_max(&list()?),
        "MIN" => ix::cob_intr_min(&list()?),
        "SUM" => ix::cob_intr_sum(&list()?),
        "MEAN" => ix::cob_intr_mean(&list()?),
        "MEDIAN" => ix::cob_intr_median(&list()?),
        "RANGE" => ix::cob_intr_range(&list()?),
        "MIDRANGE" => ix::cob_intr_midrange(&list()?),
        "ORD-MAX" => ix::cob_intr_ord_max(&list()?),
        "ORD-MIN" => ix::cob_intr_ord_min(&list()?),
        // --- transcendental + roots (2048-bit Mpf) ---
        "SQRT" => { let a = a0()?; ix::cob_intr_sqrt(&a.0, &a.1) }
        "EXP" => { let a = a0()?; ix::cob_intr_exp(&a.0, &a.1) }
        "EXP10" => { let a = a0()?; ix::cob_intr_exp10(&a.0, &a.1) }
        "LOG" => { let a = a0()?; ix::cob_intr_log(&a.0, &a.1) }
        "LOG10" => { let a = a0()?; ix::cob_intr_log10(&a.0, &a.1) }
        "SIN" => { let a = a0()?; ix::cob_intr_sin(&a.0, &a.1) }
        "COS" => { let a = a0()?; ix::cob_intr_cos(&a.0, &a.1) }
        "TAN" => { let a = a0()?; ix::cob_intr_tan(&a.0, &a.1) }
        "ASIN" => { let a = a0()?; ix::cob_intr_asin(&a.0, &a.1) }
        "ACOS" => { let a = a0()?; ix::cob_intr_acos(&a.0, &a.1) }
        "ATAN" => { let a = a0()?; ix::cob_intr_atan(&a.0, &a.1) }
        "PI" => ix::cob_intr_pi(),
        "E" => ix::cob_intr_e(),
        // --- statistical + financial ---
        "VARIANCE" => ix::cob_intr_variance(&list()?),
        "STANDARD-DEVIATION" => ix::cob_intr_standard_deviation(&list()?),
        "ANNUITY" => {
            let (x, y) = pair(0, 1)?;
            ix::cob_intr_annuity(&x.0, &x.1, &y.0, &y.1)
        }
        "PRESENT-VALUE" => {
            let rate = a0()?;
            let flows: Vec<(&[u8], &FieldAttr)> = args[1..].iter().map(|(b, at)| (b.as_slice(), at)).collect();
            if flows.is_empty() {
                return Err(RunError::Unsupported("FUNCTION PRESENT-VALUE: needs a rate and at least one flow".into()));
            }
            ix::cob_intr_present_value(&rate.0, &rate.1, &flows)
        }
        // --- date/time integer conversions (deterministic) ---
        "INTEGER-OF-DATE" => { let a = a0()?; ix::cob_intr_integer_of_date(&a.0, &a.1) }
        "INTEGER-OF-DAY" => { let a = a0()?; ix::cob_intr_integer_of_day(&a.0, &a.1) }
        "DATE-OF-INTEGER" => { let a = a0()?; ix::cob_intr_date_of_integer(&a.0, &a.1) }
        "DAY-OF-INTEGER" => { let a = a0()?; ix::cob_intr_day_of_integer(&a.0, &a.1) }
        "TEST-DATE-YYYYMMDD" => { let a = a0()?; ix::cob_intr_test_date_yyyymmdd(&a.0, &a.1) }
        "TEST-DAY-YYYYDDD" => { let a = a0()?; ix::cob_intr_test_day_yyyyddd(&a.0, &a.1) }
        // --- NUMVAL validators + variants ---
        "TEST-NUMVAL" => ix::cob_intr_test_numval(&a0()?.0),
        "TEST-NUMVAL-C" => ix::cob_intr_test_numval_c(&a0()?.0, None),
        "TEST-NUMVAL-F" => ix::cob_intr_test_numval_f(&a0()?.0),
        "NUMVAL-F" => ix::cob_intr_numval_f(&a0()?.0, b'.'),
        // --- bit/char + algebraic bounds + lengths ---
        "BIT-OF" => ix::cob_intr_bit_of(&a0()?.0),
        "BIT-TO-CHAR" => ix::cob_intr_bit_to_char(&a0()?.0),
        "STORED-CHAR-LENGTH" => ix::cob_intr_stored_char_length(&a0()?.0),
        "LOWEST-ALGEBRAIC" => { let a = a0()?; ix::cob_intr_lowest_algebraic(a.0.len(), &a.1) }
        "HIGHEST-ALGEBRAIC" => { let a = a0()?; ix::cob_intr_highest_algebraic(a.0.len(), &a.1) }
        "CONCATENATE" => {
            let parts: Vec<&[u8]> = args.iter().map(|(b, _)| b.as_slice()).collect();
            if parts.is_empty() {
                return Err(RunError::Unsupported("FUNCTION CONCATENATE: needs at least one argument".into()));
            }
            ix::cob_intr_concatenate(0, 0, &parts)
        }
        // --- SUBSTITUTE(subject, from1, to1, from2, to2, ...) ---
        "SUBSTITUTE" | "SUBSTITUTE-CASE" => {
            if args.len() < 3 || args.len() % 2 == 0 {
                return Err(RunError::Unsupported(format!(
                    "FUNCTION {name}: needs a subject and from/to pairs"
                )));
            }
            let original = args[0].0.as_slice();
            let pairs: Vec<(&[u8], &[u8])> = args[1..]
                .chunks(2)
                .map(|c| (c[0].0.as_slice(), c[1].0.as_slice()))
                .collect();
            if name == "SUBSTITUTE-CASE" {
                ix::cob_intr_substitute_case(0, 0, original, &pairs)
            } else {
                ix::cob_intr_substitute(0, 0, original, &pairs)
            }
        }
        // --- formatted date/time conversions (deterministic) ---
        "FORMATTED-DATE" => {
            let (fmt, d) = pair(0, 1)?;
            ix::cob_intr_formatted_date(0, 0, &fmt.0, &d.0, &d.1)
        }
        "INTEGER-OF-FORMATTED-DATE" => {
            let (fmt, d) = pair(0, 1)?;
            ix::cob_intr_integer_of_formatted_date(&fmt.0, &d.0)
        }
        "TEST-FORMATTED-DATETIME" => {
            let (fmt, dt) = pair(0, 1)?;
            ix::cob_intr_test_formatted_datetime(&fmt.0, &dt.0)
        }
        "SECONDS-FROM-FORMATTED-TIME" => {
            let (fmt, t) = pair(0, 1)?;
            ix::cob_intr_seconds_from_formatted_time(&fmt.0, &t.0)
        }
        "COMBINED-DATETIME" => {
            let (d, t) = pair(0, 1)?;
            ix::cob_intr_combined_datetime(&d.0, &d.1, &t.0, &t.1)
        }
        "FORMATTED-TIME" => {
            let (fmt, t) = pair(0, 1)?;
            ix::cob_intr_formatted_time(0, 0, &fmt.0, &t.0, &t.1, None, false)
        }
        "FORMATTED-DATETIME" => {
            let fmt = a0()?;
            let (d, t) = pair(1, 2)?;
            ix::cob_intr_formatted_datetime(0, 0, &fmt.0, &d.0, &d.1, &t.0, &t.1, None, false)
        }
        // CURRENCY-SYMBOL is the only locale separator GnuCOBOL 3.2 exposes as a user FUNCTION; the
        // NUM-/MON- helpers exist in libcob but cobc rejects them as unknown functions, so they stay
        // unwired (a program using them does not compile under the oracle -- nothing to match).
        "CURRENCY-SYMBOL" => ix::cob_intr_currency_symbol(),
        // SECONDS-PAST-MIDNIGHT reads the LIVE wall clock exactly as libcob (it ignores COB_CURRENT_DATE);
        // the value is the current time-of-day in seconds. Deterministic-vs-oracle only when both run in
        // the same wall-clock second (the sweep runs cobc + cobrun back-to-back under a pinned TZ).
        "SECONDS-PAST-MIDNIGHT" => ix::cob_intr_seconds_past_midnight(),
        // LOCALE conversions are deterministic under a fixed (pinned) locale, which the sweep enforces
        // (LC_ALL=C); the runtime ignores the optional locale name and uses the active locale.
        "LOCALE-DATE" => {
            let a = a0()?;
            ix::cob_intr_locale_date(0, 0, &a.0, &a.1, None)
        }
        "LOCALE-TIME" => {
            let a = a0()?;
            ix::cob_intr_locale_time(0, 0, &a.0, &a.1, None)
        }
        "LOCALE-COMPARE" => {
            let (x, y) = pair(0, 1)?;
            ix::cob_intr_locale_compare(&x.0, &y.0, None)
        }
        // MODULE-ID / MODULE-CALLER-ID are deterministic: the running PROGRAM-ID and its caller's (read
        // from the program stack the interpreter maintains across CALLs).
        "MODULE-ID" => ix::cob_intr_module_id(current_program_id().as_bytes()),
        "MODULE-CALLER-ID" => {
            let caller = caller_program_id();
            ix::cob_intr_module_caller_id(caller.as_deref().map(str::as_bytes))
        }
        // MODULE-SOURCE is the source-file path the host is running (cobc embeds the name it was given).
        "MODULE-SOURCE" => {
            let src = SOURCE_FILE.with(|s| s.borrow().clone());
            ix::cob_intr_module_source(src.as_bytes())
        }
        // EXCEPTION-STATUS: the last raised arithmetic condition (EC-SIZE-*), sticky, from the register
        // the front-end maintains as arithmetic SIZE ERRORs occur.
        "EXCEPTION-STATUS" => exception_status_field(),
        // EXCEPTION-STATEMENT is spaces unless `>>TURN EC-ALL CHECKING` is on (outside the sealed subset),
        // so the default-dialect value is the empty (spaces) form.
        "EXCEPTION-STATEMENT" => ix::cob_intr_exception_statement(None),
        // EXCEPTION-LOCATION: "<prog>; ; 0" once an exception has been raised, else a single space.
        "EXCEPTION-LOCATION" => exception_location_field(),
        // EXCEPTION-FILE: the last I/O "<status><SELECT>", or "00" before any I/O.
        "EXCEPTION-FILE" => exception_file_field(),
        // The compile-stamp intrinsics: deterministic under a pinned SOURCE_DATE_EPOCH (the reproducible-
        // builds standard cobc honours), via the interpreter's compile step.
        "MODULE-DATE" => {
            let (y, mo, d, ..) = compile_tm()?;
            ix::cob_intr_module_date(y as u32 * 10000 + mo * 100 + d)
        }
        "MODULE-TIME" => {
            let (_, _, _, h, mi, s) = compile_tm()?;
            ix::cob_intr_module_time(h * 10000 + mi * 100 + s)
        }
        "WHEN-COMPILED" => {
            let (y, mo, d, h, mi, s) = compile_tm()?;
            // 21-char YYYYMMDDHHMMSS + hundredths(00) + offset(00000) -- zero under SOURCE_DATE_EPOCH.
            let bytes = format!("{y:04}{mo:02}{d:02}{h:02}{mi:02}{s:02}0000000").into_bytes();
            ix::cob_intr_when_compiled(0, 0, &bytes, &alnum_attr())
        }
        "MODULE-FORMATTED-DATE" => {
            let (y, mo, d, h, mi, s) = compile_tm()?;
            const MON: [&str; 12] =
                ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            let bytes = format!("{} {d:02} {y:04} {h:02}:{mi:02}:{s:02}", MON[(mo - 1) as usize]).into_bytes();
            ix::cob_intr_module_formatted_date(&bytes)
        }
        // CURRENT-DATE honours the pinned COB_CURRENT_DATE (same override cobc's libcob reads), so the
        // result is oracle-deterministic; the live clock is a non-claim (fail closed without the pin).
        "CURRENT-DATE" => {
            let raw = std::env::var("COB_CURRENT_DATE").map_err(|_| {
                RunError::Unsupported(
                    "FUNCTION CURRENT-DATE requires a pinned COB_CURRENT_DATE (the live clock is a non-claim)".into(),
                )
            })?;
            ix::cob_intr_current_date_cfg(0, 0, Some(raw.as_bytes()))
        }
        // FORMATTED-CURRENT-DATE routes through the same env-aware clock (cob_get_current_datetime reads
        // COB_CURRENT_DATE), so it is oracle-deterministic under the pin.
        "FORMATTED-CURRENT-DATE" => ix::cob_intr_formatted_current_date(0, 0, &a0()?.0),
        // year-window conversions: the windowing pivot is the current year (taken from the pinned
        // COB_CURRENT_DATE); the optional second argument is the max-year offset (default 50).
        "YEAR-TO-YYYY" | "DATE-TO-YYYYMMDD" | "DAY-TO-YYYYDDD" => {
            let cur_year = {
                let raw = std::env::var("COB_CURRENT_DATE").map_err(|_| {
                    RunError::Unsupported(format!("FUNCTION {name} requires a pinned COB_CURRENT_DATE"))
                })?;
                raw.get(0..4)
                    .and_then(|s| s.parse::<i32>().ok())
                    .ok_or_else(|| RunError::Unsupported(format!("FUNCTION {name}: COB_CURRENT_DATE has no year")))?
            };
            let arg_i32 = |i: usize| -> Result<i32, RunError> {
                let a = args
                    .get(i)
                    .ok_or_else(|| RunError::Unsupported(format!("FUNCTION {name}: missing argument")))?;
                Ok(dec_to_i64(&source_to_decimal(&a.0, &a.1)?) as i32)
            };
            let value = arg_i32(0)?;
            let interval = if args.len() > 1 { arg_i32(1)? } else { 50 };
            match name {
                "YEAR-TO-YYYY" => ix::cob_intr_year_to_yyyy(value, interval, cur_year),
                "DATE-TO-YYYYMMDD" => ix::cob_intr_date_to_yyyymmdd(value, interval, cur_year),
                _ => ix::cob_intr_day_to_yyyyddd(value, interval, cur_year),
            }
        }
        other => {
            return Err(RunError::Unsupported(format!(
                "FUNCTION {other}: cobc 3.2 does not implement it (a compile-reject) or it is a live-clock/locale/GMP-PRNG non-claim"
            )))
        }
    };
    Ok(r)
}

/// Evaluate the `FUNCTION name(args)` reference beginning at `toks[start]` (which is the `FUNCTION`
/// keyword). The lexer glues parens into words (`MAX(A`, `C)`, `UPPER-CASE(X)`), so the call is
/// reconstructed by tracking paren depth across tokens; string-literal arguments are carried through a
/// `\u{1}N` placeholder. Returns the result `(bytes, attr)` and the index just past the call.
fn eval_function_call(
    toks: &[Tok],
    start: usize,
    fields: &HashMap<String, Field>,
) -> Result<((Vec<u8>, FieldAttr), usize), RunError> {
    let mut raw = String::new();
    let mut strs: Vec<Vec<u8>> = Vec::new();
    let mut k = start + 1;
    let (mut depth, mut seen) = (0i32, false);
    while k < toks.len() {
        match &toks[k] {
            Tok::Word(w) => {
                raw.push(' ');
                raw.push_str(w);
                for c in w.bytes() {
                    if c == b'(' {
                        depth += 1;
                        seen = true;
                    } else if c == b')' {
                        depth -= 1;
                    }
                }
            }
            Tok::Str(s) => {
                raw.push(' ');
                raw.push('\u{1}');
                raw.push_str(&strs.len().to_string());
                strs.push(s.clone());
            }
            Tok::AllLiteral(s) => {
                // VALUE ALL literal used as an operand: its bytes behave like a string literal.
                raw.push(' ');
                raw.push('\u{1}');
                raw.push_str(&strs.len().to_string());
                strs.push(s.clone());
            }
            Tok::Dot => break,
        }
        k += 1;
        if seen && depth <= 0 {
            break;
        }
        // a no-argument function (e.g. PI, E): the name token carries no '(' and the next token does
        // not open one.
        if !seen && !matches!(toks.get(k), Some(Tok::Word(w)) if w.starts_with('(')) {
            break;
        }
    }
    let raw = raw.trim();
    let (name, arg_strs): (String, Vec<String>) = match raw.find('(') {
        Some(o) => {
            let close = raw.rfind(')').unwrap_or(raw.len());
            // Split the argument list at TOP-LEVEL spaces/commas only, so a subscripted or reference-modified
            // argument keeps its own parens intact -- `MAX(A(1) A(2))`, `NUMVAL(D(1:5))`, `UPPER-CASE(S(1:4))`.
            // (A naive split + paren-trim would mangle `A(1)` into `A(1`.)
            let inner = &raw[o + 1..close];
            let mut args: Vec<String> = Vec::new();
            let mut cur = String::new();
            let mut depth = 0i32;
            for ch in inner.chars() {
                match ch {
                    '(' => {
                        depth += 1;
                        cur.push(ch);
                    }
                    ')' => {
                        depth -= 1;
                        cur.push(ch);
                    }
                    ' ' | ',' if depth == 0 => {
                        if !cur.is_empty() {
                            args.push(std::mem::take(&mut cur));
                        }
                    }
                    _ => cur.push(ch),
                }
            }
            if !cur.is_empty() {
                args.push(cur);
            }
            (raw[..o].trim().to_ascii_uppercase(), args)
        }
        None => (raw.trim().to_ascii_uppercase(), Vec::new()),
    };
    // FUNCTION TRIM(x [LEADING | TRAILING]) -- the optional direction keyword is a MODIFIER, not an argument
    // (0 = both ends, 1 = leading, 2 = trailing). Handle it here so the keyword isn't evaluated as a value.
    if name == "TRIM" {
        let mut items = arg_strs.clone();
        let dir = match items.last().map(|s| s.to_ascii_uppercase()).as_deref() {
            Some("LEADING") => {
                items.pop();
                1
            }
            Some("TRAILING") => {
                items.pop();
                2
            }
            _ => 0,
        };
        let a = items
            .first()
            .ok_or_else(|| RunError::Unsupported("FUNCTION TRIM: missing argument".into()))?;
        let arg = if let Some(idx) = a.strip_prefix('\u{1}') {
            let i: usize = idx.parse().unwrap_or(0);
            (strs.get(i).cloned().unwrap_or_default(), alnum_attr())
        } else {
            operand_value(&Tok::Word(a.clone()), fields)?
        };
        return Ok((
            crate::intrinsic::cob_intr_trim(0, 0, &arg.0, &arg.1, dir),
            k,
        ));
    }
    let mut args: Vec<(Vec<u8>, FieldAttr)> = Vec::new();
    for a in &arg_strs {
        if let Some(idx) = a.strip_prefix('\u{1}') {
            let i: usize = idx.parse().unwrap_or(0);
            args.push((strs.get(i).cloned().unwrap_or_default(), alnum_attr()));
        } else {
            args.push(operand_value(&Tok::Word(a.clone()), fields)?);
        }
    }
    // cobc's LENGTH/BYTE-LENGTH of a reference-modified item is the BASE item's length -- the refmod is
    // ignored (`FUNCTION LENGTH(S(2:4))` == LENGTH OF S). Resolve the base and fold/compute its length.
    if matches!(name.as_str(), "LENGTH" | "BYTE-LENGTH") {
        if let Some((base, _, _)) = arg_strs.first().and_then(|a| parse_refmod(a)) {
            let n = read_field(fields, base)?
                .map(|f| f.bytes.len())
                .unwrap_or(0);
            return if is_variable_length(base, fields) {
                Ok((crate::intrinsic::cob_intr_length(n), k))
            } else {
                Ok((
                    decimal_as_display(&Decimal {
                        negative: false,
                        digits: digits_of(n as u128),
                        scale: 0,
                    }),
                    k,
                ))
            };
        }
    }
    // LENGTH/BYTE-LENGTH of a VARIABLE-length item (one with an OCCURS DEPENDING ON in its subtree) is a
    // RUNTIME call in cobc, not the compile-time constant fold -- so it displays as the 9-digit cob_intr
    // form, not a minimal integer.
    if matches!(name.as_str(), "LENGTH" | "BYTE-LENGTH")
        && arg_strs
            .first()
            .is_some_and(|a| is_variable_length(a, fields))
    {
        let n = args.first().map(|(b, _)| b.len()).unwrap_or(0);
        return Ok((crate::intrinsic::cob_intr_length(n), k));
    }
    // CONTENT-OF(ptr [, len]) / CONTENT-LENGTH(ptr): dereference a USAGE POINTER (set via SET ptr TO
    // ADDRESS OF field) to its target's bytes / length.
    if matches!(name.as_str(), "CONTENT-OF" | "CONTENT-LENGTH") {
        let target = arg_strs
            .first()
            .and_then(|p| POINTER_TARGETS.with(|m| m.borrow().get(p).cloned()));
        let target = target.ok_or_else(|| {
            RunError::Unsupported(format!(
                "FUNCTION {name}: the pointer has no SET ... TO ADDRESS OF target"
            ))
        })?;
        if name == "CONTENT-LENGTH" {
            return Ok((
                crate::intrinsic::cob_intr_length(field_len(&target, fields)),
                k,
            ));
        }
        let bytes = read_field(fields, &target)
            .ok()
            .flatten()
            .map(|f| f.bytes)
            .unwrap_or_default();
        let bytes = match args.get(1) {
            Some(lenarg) => {
                let n = dec_to_i64(&source_to_decimal(&lenarg.0, &lenarg.1)?).max(0) as usize;
                bytes.into_iter().take(n).collect()
            }
            None => bytes,
        };
        return Ok(((bytes, alnum_attr()), k));
    }
    Ok((eval_intrinsic(&name, &args)?, k))
}

/// Replace every `FUNCTION name(args)` reference in `toks` with a freshly-evaluated temporary field,
/// returning the rewritten token stream. The temp is inserted into `fields` under a sentinel name no
/// COBOL identifier can collide with, so every downstream path (operand_value / display_bytes /
/// wide_op / move_into) resolves the function result through the ordinary field machinery -- which keeps
/// binary, scaled and signed intrinsic results byte-faithful for free. The no-FUNCTION fast path avoids
/// any allocation churn.
fn resolve_functions(
    toks: &[Tok],
    fields: &mut HashMap<String, Field>,
) -> Result<Vec<Tok>, RunError> {
    let is_len = |t: Option<&Tok>| matches!(t, Some(Tok::Word(w)) if w.eq_ignore_ascii_case("LENGTH") || w.eq_ignore_ascii_case("BYTE-LENGTH"));
    let has_len_of = toks.windows(2).any(|w| {
        is_len(Some(&w[0])) && matches!(&w[1], Tok::Word(b) if b.eq_ignore_ascii_case("OF"))
    });
    if !has_len_of
        && !toks
            .iter()
            .any(|t| matches!(t, Tok::Word(w) if w.eq_ignore_ascii_case("FUNCTION")))
    {
        return Ok(toks.to_vec());
    }
    let mut out = Vec::new();
    let mut i = 0;
    let mut n = 0usize;
    while i < toks.len() {
        // `LENGTH OF id` / `BYTE-LENGTH OF id` -- the special-register form (no FUNCTION keyword), usable as a
        // numeric operand in ADD/COMPUTE/etc. Route it through the same intrinsic evaluator as FUNCTION LENGTH.
        if is_len(Some(&toks[i]))
            && matches!(toks.get(i + 1), Some(Tok::Word(w)) if w.eq_ignore_ascii_case("OF"))
            && matches!(toks.get(i + 2), Some(Tok::Word(_)))
        {
            let fname = if let Tok::Word(w) = &toks[i] {
                w.clone()
            } else {
                unreachable!()
            };
            let refw = if let Tok::Word(w) = &toks[i + 2] {
                w.clone()
            } else {
                unreachable!()
            };
            let synth = vec![
                Tok::Word("FUNCTION".into()),
                Tok::Word(fname),
                Tok::Word(format!("({refw})")),
                Tok::Dot,
            ];
            let ((bytes, attr), _next) = eval_function_call(&synth, 0, fields)?;
            let name = format!("\u{2}FN{n}");
            n += 1;
            let storage = if attr.field_type >= 0x20 {
                Storage::Alpha(attr)
            } else {
                Storage::Numeric(attr)
            };
            fields.insert(
                name.clone(),
                Field {
                    storage,
                    bytes,
                    occurs: 0,
                    redefines: None,
                },
            );
            out.push(Tok::Word(name));
            i += 3;
            continue;
        }
        if matches!(&toks[i], Tok::Word(w) if w.eq_ignore_ascii_case("FUNCTION")) {
            let ((bytes, attr), next) = eval_function_call(toks, i, fields)?;
            // \u{2} prefix (not \u{1}, which eval_cond reserves for string-literal markers).
            let name = format!("\u{2}FN{n}");
            n += 1;
            let storage = if attr.field_type >= 0x20 {
                Storage::Alpha(attr)
            } else {
                Storage::Numeric(attr)
            };
            fields.insert(
                name.clone(),
                Field {
                    storage,
                    bytes,
                    occurs: 0,
                    redefines: None,
                },
            );
            out.push(Tok::Word(name));
            i = next;
        } else {
            out.push(toks[i].clone());
            i += 1;
        }
    }
    Ok(out)
}

/// Split a possibly-subscripted reference `NAME(sub)` into `(NAME, Some(sub))`; a bare `NAME` -> `(NAME,
/// None)`. A reference-modification `NAME(a:b)` (contains `:`) is NOT a subscript here -> `(NAME, None)`.
fn split_subscript(w: &str) -> (&str, Option<&str>) {
    if let Some(open) = w.find('(') {
        if w.ends_with(')') {
            let inner = &w[open + 1..w.len() - 1];
            if !inner.contains(':') {
                return (&w[..open], Some(inner));
            }
        }
    }
    (w, None)
}

/// Parse a reference-modification reference `base(start:len)` / `base(start:)`. The refmod is the LAST
/// parenthesized group and is the one containing `:`; `base` is everything before it and may itself carry a
/// subscript (`T(i)(s:l)`). Returns `(base, start_expr, Some(len_expr) | None-for-to-end)`.
fn parse_refmod(w: &str) -> Option<(&str, &str, Option<&str>)> {
    if !w.ends_with(')') {
        return None;
    }
    let open = w.rfind('(')?;
    let inner = &w[open + 1..w.len() - 1];
    let colon = inner.find(':')?;
    let start = inner[..colon].trim();
    let len = inner[colon + 1..].trim();
    if start.is_empty() {
        return None;
    }
    Some((
        &w[..open],
        start,
        if len.is_empty() { None } else { Some(len) },
    ))
}

thread_local! {
    /// Whether `EC-BOUND-SUBSCRIPT` checking is ENABLED for this run. Default OFF -- matching cobc, whose
    /// default emits NO subscript check (an out-of-range read is undefined, reading adjacent storage).
    /// `>>TURN EC-BOUND-SUBSCRIPT CHECKING ON` (or `EC-ALL`) turns it on. Set per run in
    /// run_program_redirected; read in [`table_element`] / [`write_field`].
    static EC_BOUND_SUBSCRIPT_ON: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Scan for a program-level `>>TURN EC-BOUND-SUBSCRIPT CHECKING ON` / `>>TURN EC-ALL CHECKING ON` directive
/// (a later matching `... CHECKING OFF` turns it back off). cobc's default is OFF (no emitted check).
fn parse_ec_bound_check(src_upper: &str) -> bool {
    let mut on = false;
    for line in src_upper.lines() {
        let l = line.trim_start();
        if l.starts_with(">>TURN") && (l.contains("EC-BOUND-SUBSCRIPT") || l.contains("EC-ALL")) {
            if l.contains("CHECKING ON") {
                on = true;
            } else if l.contains("CHECKING OFF") {
                on = false;
            }
        }
    }
    on
}

/// A category-default element (`'0'` numeric, space alphanumeric) of `elem` bytes -- the safe value the
/// port returns for a suppressed out-of-range subscript read (the C reads adjacent storage; that is UB).
fn default_element(storage: &Storage, elem: usize) -> Field {
    let fill = match storage {
        Storage::Numeric(_) => b'0',
        Storage::Alpha(_)
        | Storage::Edited(..)
        | Storage::Condition { .. }
        | Storage::Group { .. } => b' ',
    };
    Field {
        storage: storage.clone(),
        bytes: vec![fill; elem],
        occurs: 1,
        redefines: None,
    }
}

/// The element Field of an `OCCURS` table at 1-based subscript `idx` (a transient single-element Field).
/// Out of range fails closed: cobc's default (no `>>TURN ... CHECKING ON`) OOB read is UNDEFINED (it reads
/// adjacent storage); under #![forbid(unsafe_code)] the port cannot read past the table, so it errors
/// rather than fabricate a byte -- and under an active bound check this IS the EC-BOUND-SUBSCRIPT raise.
fn table_element(f: &Field, idx: usize, name: &str) -> Result<Field, RunError> {
    let occ = f.occurs.max(1);
    let elem = f.bytes.len() / occ;
    if idx < 1 || idx > occ {
        // EC-BOUND-SUBSCRIPT: when the check is enabled (>>TURN ... CHECKING ON) an OOB subscript raises
        // (the libcob abort); when OFF (the cobc default) the check is SUPPRESSED -- cobc reads adjacent
        // storage (UB), the safe port returns a category-default element and continues.
        if EC_BOUND_SUBSCRIPT_ON.with(|c| c.get()) {
            return Err(RunError::Runtime(format!(
                "subscript of '{name}' out of bounds: {idx} (maximum: {occ})"
            )));
        }
        return Ok(default_element(&f.storage, elem));
    }
    let start = (idx - 1) * elem;
    Ok(Field {
        storage: f.storage.clone(),
        bytes: f.bytes[start..start + elem].to_vec(),
        occurs: 1,
        redefines: None,
    })
}

/// The live authoritative buffer for `name` -- normally `own` (its own bytes), but for a descendant store of
/// a REDEFINES group (registered in [`REDEF_VIEW`]) the redefined target's current image sliced at the
/// stored offset, padded/truncated to `own`'s length. So a group-OCCURS table that REDEFINES a VALUE-bearing
/// group reads the live values, not its own (empty) buffer.
fn redef_buffer(fields: &HashMap<String, Field>, name: &str, own: &[u8]) -> Vec<u8> {
    if let Some((target, off)) = REDEF_VIEW.with(|m| m.borrow().get(name).cloned()) {
        let img = read_field(fields, &target)
            .ok()
            .flatten()
            .map(|t| t.bytes)
            .unwrap_or_default();
        let mut b = img.get(off..).map(|s| s.to_vec()).unwrap_or_default();
        b.resize(own.len(), b' ');
        return b;
    }
    own.to_vec()
}

/// Write `slice` at byte `local` within `name`'s authoritative storage. For a descendant store of a
/// REDEFINES group (registered in [`REDEF_VIEW`]) this lands in the redefined target's shared image at the
/// recorded offset and returns `Ok(true)`; otherwise `Ok(false)` and the caller performs its own write.
fn redef_write(
    fields: &mut HashMap<String, Field>,
    name: &str,
    local: usize,
    slice: &[u8],
) -> Result<bool, RunError> {
    let Some((target, off)) = REDEF_VIEW.with(|m| m.borrow().get(name).cloned()) else {
        return Ok(false);
    };
    let mut img = read_field(fields, &target)
        .ok()
        .flatten()
        .map(|t| t.bytes)
        .unwrap_or_default();
    let at = off + local;
    let n = slice.len().min(img.len().saturating_sub(at));
    if n > 0 {
        img[at..at + n].copy_from_slice(&slice[..n]);
        set_field_image(fields, &target, &img)?;
    }
    Ok(true)
}

/// The bytes a field's storage operates on -- its own, or, for a `REDEFINES` alias, the target field's
/// bytes viewed at this field's size (the first `size` bytes; REDEFINES width <= target width). A single
/// alias hop (the common 01-level case).
fn aliased(fields: &HashMap<String, Field>, f: &Field) -> Field {
    match &f.redefines {
        Some(target) => {
            let size = f.bytes.len();
            // Read the target's live IMAGE (not its raw `bytes`): an elementary target's image is its
            // bytes, but a GROUP target's bytes are empty -- its image is the concatenated/interleaved
            // leaves (so REDEFINES over a group, incl. a group-OCCURS interleaved buffer, sees real data).
            let mut bytes = read_field(fields, target)
                .ok()
                .flatten()
                .map(|t| t.bytes)
                .unwrap_or_default();
            bytes.resize(size, b' ');
            bytes.truncate(size);
            Field {
                storage: f.storage.clone(),
                bytes,
                occurs: f.occurs,
                redefines: None,
            }
        }
        None => f.clone(),
    }
}

/// Resolve a (possibly subscripted) field reference word to an owned Field for READING. Returns `Ok(None)`
/// when `word` names no field (e.g. it is a numeric literal). The subscript may itself be a field (`E(I)`);
/// a `REDEFINES` field reads its target's storage (so an alias sees the other field's current bytes).
fn read_field(fields: &HashMap<String, Field>, word: &str) -> Result<Option<Field>, RunError> {
    // Reference modification `base(start:len)` / `base(start:)` -- an alphanumeric SUBSTRING of the base item
    // (which may itself carry a subscript, e.g. T(i)(s:l)). Always category alphanumeric, 1-based start.
    if let Some((base, start_s, len_s)) = parse_refmod(word) {
        let Some(basef) = read_field(fields, base)? else {
            return Ok(None);
        };
        let total = basef.bytes.len();
        let start = resolve_int(start_s, fields).ok_or_else(|| {
            RunError::Unsupported(format!(
                "reference-modification start '{start_s}' is not an integer"
            ))
        })?;
        let len = match len_s {
            Some(l) => resolve_int(l, fields).ok_or_else(|| {
                RunError::Unsupported(format!(
                    "reference-modification length '{l}' is not an integer"
                ))
            })?,
            None => total as i64 - start + 1, // `(start:)` runs to the end of the item
        };
        let s = (start - 1).clamp(0, total as i64) as usize;
        let e = (s as i64 + len.max(0)).clamp(0, total as i64) as usize;
        return Ok(Some(Field {
            storage: Storage::Alpha(alnum_attr()),
            bytes: basef.bytes[s..e].to_vec(),
            occurs: 1,
            redefines: None,
        }));
    }
    let (base, sub) = split_subscript(word);
    // MULTI-DIMENSION leaf: `C(i,j)` -- a strided cell of the base group-OCCURS buffer, addressed by dims.
    if let Some((basef, offset, size, dims)) = nested_leaf_lookup(base) {
        let cstore = fields
            .get(base)
            .map(|f| f.storage.clone())
            .unwrap_or(Storage::Alpha(alnum_attr()));
        let subs = sub.map(subscripts).unwrap_or_default();
        return match nested_addr(offset, &dims, &subs, fields)? {
            Some(off) => {
                let pf = fields
                    .get(&basef)
                    .ok_or_else(|| RunError::UndefinedName(basef.clone()))?;
                Ok(Some(Field {
                    storage: cstore,
                    bytes: pf.bytes[off..off + size].to_vec(),
                    occurs: 1,
                    redefines: None,
                }))
            }
            None => Ok(Some(default_element(&cstore, size))), // suppressed OOB
        };
    }
    // group-OCCURS CHILD: `EK(i)` is a single strided slice into the PARENT's interleaved buffer.
    if let Some((parent, coff, csz)) = group_child_lookup(base) {
        let cstore = fields
            .get(base)
            .map(|f| f.storage.clone())
            .unwrap_or(Storage::Alpha(alnum_attr()));
        let (stride, occ) = group_occurs_lookup(&parent).unwrap_or((csz, 1));
        let idx = match sub {
            Some(s) => resolve_int(s, fields).ok_or_else(|| {
                RunError::Unsupported(format!("subscript '{s}' is not an integer"))
            })? as usize,
            None => {
                return Err(RunError::Unsupported(format!(
                    "group-OCCURS child `{base}` must be subscripted"
                )))
            }
        };
        if idx < 1 || idx > occ {
            if EC_BOUND_SUBSCRIPT_ON.with(|c| c.get()) {
                return Err(RunError::Runtime(format!(
                    "subscript of '{base}' out of bounds: {idx} (maximum: {occ})"
                )));
            }
            return Ok(Some(default_element(&cstore, csz)));
        }
        let own = fields
            .get(&parent)
            .ok_or_else(|| RunError::UndefinedName(parent.clone()))?
            .bytes
            .clone();
        let buf = redef_buffer(fields, &parent, &own); // alias the redefined target, if this group-OCCURS redefines one
        let start = (idx - 1) * stride + coff;
        return Ok(Some(Field {
            storage: cstore,
            bytes: buf[start..start + csz].to_vec(),
            occurs: 1,
            redefines: None,
        }));
    }
    let Some(f) = fields.get(base) else {
        return Ok(None);
    };
    if let Storage::Group { children } = &f.storage {
        // group-OCCURS TABLE: bytes are the live interleaved buffer; `ENT(i)` via the unchanged table_element.
        if let Some((_stride, occ)) = group_occurs_lookup(base) {
            let buf = redef_buffer(fields, base, &f.bytes); // alias the redefined target, if this table redefines one
            let tbl = Field {
                storage: Storage::Group {
                    children: children.clone(),
                },
                bytes: buf,
                occurs: occ,
                redefines: None,
            };
            return match sub {
                // whole interleaved image (REDEFINES X(n) read / group DISPLAY/MOVE). OCCURS DEPENDING ON:
                // the LIVE image is counter*stride (built at MAX); subscripted ENT(i) still uses MAX above.
                None => {
                    if let Some((counter, st)) = odo_lookup(base) {
                        let n = resolve_int(&counter, fields).unwrap_or(0).max(0) as usize;
                        let live = (n * st).min(tbl.bytes.len());
                        return Ok(Some(Field {
                            storage: tbl.storage,
                            bytes: tbl.bytes[..live].to_vec(),
                            occurs: n,
                            redefines: None,
                        }));
                    }
                    Ok(Some(tbl))
                }
                Some(s) => {
                    let idx = resolve_int(s, fields).ok_or_else(|| {
                        RunError::Unsupported(format!("subscript '{s}' is not an integer"))
                    })?;
                    Ok(Some(table_element(&tbl, idx as usize, base)?))
                }
            };
        }
        // A non-OCCURS group reads as the concatenation of its leaves' current bytes (its live record image).
        debug_assert!(
            group_occurs_lookup(base).is_none(),
            "group_bytes reached for group-OCCURS `{base}`"
        );
        let bytes = group_bytes(children, fields);
        return Ok(Some(Field {
            storage: Storage::Group {
                children: children.clone(),
            },
            bytes,
            occurs: 1,
            redefines: None,
        }));
    }
    let mut f = aliased(fields, f);
    if REDEF_VIEW.with(|m| m.borrow().contains_key(base)) {
        f.bytes = redef_buffer(fields, base, &f.bytes); // elementary leaf of a REDEFINES group aliases the target
    }
    // OCCURS DEPENDING ON: the live image is only `counter * elem` bytes (the field is built at MAX).
    if let Some((counter, elem)) = odo_lookup(base) {
        let n = resolve_int(&counter, fields).unwrap_or(0).max(0) as usize;
        let cur = (n * elem).min(f.bytes.len());
        f.bytes.truncate(cur);
        f.occurs = n;
    }
    match sub {
        None => Ok(Some(f)),
        Some(s) => {
            let idx = resolve_int(s, fields).ok_or_else(|| {
                RunError::Unsupported(format!("subscript '{s}' is not an integer"))
            })?;
            Ok(Some(table_element(&f, idx as usize, base)?))
        }
    }
}

/// Apply `apply` to a (possibly subscripted) field reference for WRITING: a bare `NAME` mutates the field;
/// `NAME(i)` extracts the element, applies, and writes the element bytes back into the table.
/// Overwrite the first `img.len()` bytes of `name`'s live image with `img`, preserving the tail -- the
/// group-aware byte store used by REDEFINES write-through. Handles a group-OCCURS interleaved buffer, a
/// plain group (distributes to leaves via put_group_bytes), and an elementary/aliased field. Non-generic
/// (so it can be called from the generic `write_field` without recursive monomorphization).
fn set_field_image(
    fields: &mut HashMap<String, Field>,
    name: &str,
    img: &[u8],
) -> Result<(), RunError> {
    // A descendant store of a REDEFINES group: splice the image into the redefined target at the offset.
    if let Some((target, off)) = REDEF_VIEW.with(|m| m.borrow().get(name).cloned()) {
        let mut t = read_field(fields, &target)
            .ok()
            .flatten()
            .map(|x| x.bytes)
            .unwrap_or_default();
        let n = img.len().min(t.len().saturating_sub(off));
        if n > 0 {
            t[off..off + n].copy_from_slice(&img[..n]);
            set_field_image(fields, &target, &t)?;
        }
        return Ok(());
    }
    if group_occurs_lookup(name).is_some() {
        let f = fields
            .get_mut(name)
            .ok_or_else(|| RunError::UndefinedName(name.to_string()))?;
        let n = img.len().min(f.bytes.len());
        f.bytes[..n].copy_from_slice(&img[..n]);
        return Ok(());
    }
    match fields.get(name).map(|f| f.storage.clone()) {
        Some(Storage::Group { children }) => {
            let mut cur = group_bytes(&children, fields);
            let n = img.len().min(cur.len());
            cur[..n].copy_from_slice(&img[..n]);
            put_group_bytes(&children, cur, fields);
            Ok(())
        }
        Some(_) => {
            let f = fields
                .get_mut(name)
                .ok_or_else(|| RunError::UndefinedName(name.to_string()))?;
            let n = img.len().min(f.bytes.len());
            f.bytes[..n].copy_from_slice(&img[..n]);
            Ok(())
        }
        None => Err(RunError::UndefinedName(name.to_string())),
    }
}

fn write_field(
    fields: &mut HashMap<String, Field>,
    word: &str,
    apply: impl FnOnce(&mut Field) -> Result<(), RunError>,
) -> Result<(), RunError> {
    // Reference-modification RECEIVER `base(start:len)`: shape an alphanumeric temp over the substring, apply
    // (the move/inspect stores left-justified, space-padded/truncated to len), splice the result back into
    // the base bytes, then write the whole base back THROUGH write_field (so a subscripted/group base works).
    if let Some((base, start_s, len_s)) = parse_refmod(word) {
        let basef =
            read_field(fields, base)?.ok_or_else(|| RunError::UndefinedName(base.to_string()))?;
        let total = basef.bytes.len();
        let start = resolve_int(start_s, fields).ok_or_else(|| {
            RunError::Unsupported(format!(
                "reference-modification start '{start_s}' is not an integer"
            ))
        })?;
        let len = match len_s {
            Some(l) => resolve_int(l, fields).ok_or_else(|| {
                RunError::Unsupported(format!(
                    "reference-modification length '{l}' is not an integer"
                ))
            })?,
            None => total as i64 - start + 1,
        };
        let s = (start - 1).clamp(0, total as i64) as usize;
        let e = (s as i64 + len.max(0)).clamp(0, total as i64) as usize;
        let mut tmp = Field {
            storage: Storage::Alpha(alnum_attr()),
            bytes: basef.bytes[s..e].to_vec(),
            occurs: 1,
            redefines: None,
        };
        apply(&mut tmp)?;
        let mut newbytes = basef.bytes.clone();
        let n = tmp.bytes.len().min(e - s);
        newbytes[s..s + n].copy_from_slice(&tmp.bytes[..n]);
        // write the spliced base image back via the non-generic helper (a generic write_field recursion would
        // blow the monomorphization limit). Covers a plain or group base; a subscripted base fails closed.
        return set_field_image(fields, base, &newbytes);
    }
    let (base, sub) = split_subscript(word);
    // MULTI-DIMENSION leaf write-back: `C(i,j)` shapes a temp over the strided cell, applies, copies back.
    if let Some((basef, offset, size, dims)) = nested_leaf_lookup(base) {
        let cstore = fields
            .get(base)
            .map(|f| f.storage.clone())
            .unwrap_or(Storage::Alpha(alnum_attr()));
        let subs = sub.map(subscripts).unwrap_or_default();
        let off = match nested_addr(offset, &dims, &subs, fields)? {
            Some(o) => o,
            None => return Ok(()), // suppressed OOB write -> no-op (cobc writes adjacent storage; UB)
        };
        let pf = fields
            .get(&basef)
            .ok_or_else(|| RunError::UndefinedName(basef.clone()))?;
        let mut tmp = Field {
            storage: cstore,
            bytes: pf.bytes[off..off + size].to_vec(),
            occurs: 1,
            redefines: None,
        };
        apply(&mut tmp)?;
        let pf = fields.get_mut(&basef).expect("base present");
        let n = tmp.bytes.len().min(size);
        pf.bytes[off..off + n].copy_from_slice(&tmp.bytes[..n]);
        return Ok(());
    }
    // group-OCCURS CHILD write-back: `EK(i)` shapes a temp over the child's strided slice of the parent
    // buffer, applies, and copies the result back into the parent buffer.
    if let Some((parent, coff, csz)) = group_child_lookup(base) {
        let Some(s) = sub else {
            return Err(RunError::Unsupported(format!(
                "group-OCCURS child `{base}` must be subscripted"
            )));
        };
        let idx = resolve_int(s, fields)
            .ok_or_else(|| RunError::Unsupported(format!("subscript '{s}' is not an integer")))?
            as usize;
        let (stride, occ) = group_occurs_lookup(&parent).unwrap_or((csz, 1));
        let cstore = fields
            .get(base)
            .map(|f| f.storage.clone())
            .unwrap_or(Storage::Alpha(alnum_attr()));
        if idx < 1 || idx > occ {
            if EC_BOUND_SUBSCRIPT_ON.with(|c| c.get()) {
                return Err(RunError::Runtime(format!(
                    "subscript of '{base}' out of bounds: {idx} (maximum: {occ})"
                )));
            }
            return Ok(());
        }
        let start = (idx - 1) * stride + coff;
        let own = fields
            .get(&parent)
            .ok_or_else(|| RunError::UndefinedName(parent.clone()))?
            .bytes
            .clone();
        let cur = redef_buffer(fields, &parent, &own); // current element bytes (aliased target, if redefining)
        let mut tmp = Field {
            storage: cstore,
            bytes: cur[start..start + csz].to_vec(),
            occurs: 1,
            redefines: None,
        };
        apply(&mut tmp)?;
        let n = tmp.bytes.len().min(csz);
        if !redef_write(fields, &parent, start, &tmp.bytes[..n])? {
            let pf = fields.get_mut(&parent).expect("parent present");
            pf.bytes[start..start + n].copy_from_slice(&tmp.bytes[..n]);
        }
        return Ok(());
    }
    // group-OCCURS TABLE write (whole image, or `ENT(i)` element), BEFORE the group-distribute branch.
    if let Some((stride, occ)) = group_occurs_lookup(base) {
        match sub {
            None => {
                let own = fields.get(base).expect("present").bytes.clone();
                let cur = redef_buffer(fields, base, &own);
                let mut tmp = Field {
                    storage: Storage::Alpha(alnum_attr()),
                    bytes: cur,
                    occurs: occ,
                    redefines: None,
                };
                apply(&mut tmp)?;
                let n = tmp.bytes.len().min(own.len());
                if !redef_write(fields, base, 0, &tmp.bytes[..n])? {
                    let f = fields.get_mut(base).expect("present");
                    f.bytes[..n].copy_from_slice(&tmp.bytes[..n]);
                }
                return Ok(());
            }
            Some(s) => {
                let idx = resolve_int(s, fields).ok_or_else(|| {
                    RunError::Unsupported(format!("subscript '{s}' is not an integer"))
                })? as usize;
                if idx < 1 || idx > occ {
                    if EC_BOUND_SUBSCRIPT_ON.with(|c| c.get()) {
                        return Err(RunError::Runtime(format!(
                            "subscript of '{base}' out of bounds: {idx} (maximum: {occ})"
                        )));
                    }
                    return Ok(());
                }
                // The whole group element is moved with alphanumeric (byte-copy) semantics -- shape an
                // Alpha temp over the element bytes so move_into does a raw left-justified store.
                let own = fields.get(base).expect("present").bytes.clone();
                let cur = redef_buffer(fields, base, &own);
                let start = (idx - 1) * stride;
                let mut tmp = Field {
                    storage: Storage::Alpha(alnum_attr()),
                    bytes: cur[start..start + stride].to_vec(),
                    occurs: 1,
                    redefines: None,
                };
                apply(&mut tmp)?;
                let n = tmp.bytes.len().min(stride);
                if !redef_write(fields, base, start, &tmp.bytes[..n])? {
                    let f = fields.get_mut(base).expect("present");
                    f.bytes[start..start + n].copy_from_slice(&tmp.bytes[..n]);
                }
                return Ok(());
            }
        }
    }
    // A group write distributes the result across its leaves: shape a temp alphanumeric field over the
    // group's current concatenation, apply, then split the bytes back into the leaves by length.
    if sub.is_none() {
        if let Some(Storage::Group { children }) = fields.get(base).map(|f| f.storage.clone()) {
            debug_assert!(
                group_occurs_lookup(base).is_none(),
                "group_bytes write reached for group-OCCURS `{base}`"
            );
            let concat = group_bytes(&children, fields);
            let mut tmp = Field {
                storage: Storage::Alpha(alnum_attr()),
                bytes: concat,
                occurs: 1,
                redefines: None,
            };
            apply(&mut tmp)?;
            put_group_bytes(&children, tmp.bytes, fields);
            return Ok(());
        }
    }
    // A REDEFINES field writes THROUGH its alias into the target's storage: shape a temp with this field's
    // storage over the target's bytes, apply, and copy the result back into the target.
    if sub.is_none() {
        if let Some(target) = fields.get(base).and_then(|f| f.redefines.clone()) {
            // A REDEFINES field writes THROUGH its alias into the target's storage. Shape a temp with this
            // field's storage over the target's live IMAGE (elementary bytes, or a group's concatenated /
            // interleaved leaves), apply, then write the result back THROUGH the target via write_field --
            // which is group-aware (distributes to leaves, or overwrites a group-OCCURS interleaved buffer).
            let f = fields.get(base).expect("base present");
            let storage = f.storage.clone();
            let size = f.bytes.len();
            let occ = f.occurs;
            let mut bytes = read_field(fields, &target)
                .ok()
                .flatten()
                .map(|t| t.bytes)
                .unwrap_or_default();
            bytes.resize(size, b' ');
            bytes.truncate(size);
            let mut tmp = Field {
                storage,
                bytes,
                occurs: occ,
                redefines: None,
            };
            apply(&mut tmp)?;
            set_field_image(fields, &target, &tmp.bytes)?; // the alias covers the target's first `size` bytes
            return Ok(());
        }
    }
    // A descendant elementary leaf (incl. an elementary OCCURS leaf) of a REDEFINES group writes THROUGH to
    // the redefined target's shared storage at the recorded offset (the read path already aliases it).
    if REDEF_VIEW.with(|m| m.borrow().contains_key(base)) {
        let f0 = fields
            .get(base)
            .ok_or_else(|| RunError::UndefinedName(base.to_string()))?;
        let (storage, occ_field, own) = (f0.storage.clone(), f0.occurs, f0.bytes.clone());
        let occ = occ_field.max(1);
        let elem = own.len() / occ;
        let cur = redef_buffer(fields, base, &own);
        match sub {
            None => {
                let mut tmp = Field {
                    storage,
                    bytes: cur,
                    occurs: occ_field,
                    redefines: None,
                };
                apply(&mut tmp)?;
                redef_write(fields, base, 0, &tmp.bytes)?;
            }
            Some(s) => {
                let idx = resolve_int(s, fields).ok_or_else(|| {
                    RunError::Unsupported(format!("subscript '{s}' is not an integer"))
                })? as usize;
                if idx < 1 || idx > occ {
                    if EC_BOUND_SUBSCRIPT_ON.with(|c| c.get()) {
                        return Err(RunError::Runtime(format!(
                            "subscript of '{base}' out of bounds: {idx} (maximum: {occ})"
                        )));
                    }
                    return Ok(());
                }
                let start = (idx - 1) * elem;
                let mut tmp = Field {
                    storage,
                    bytes: cur[start..start + elem].to_vec(),
                    occurs: 1,
                    redefines: None,
                };
                apply(&mut tmp)?;
                redef_write(fields, base, start, &tmp.bytes)?;
            }
        }
        return Ok(());
    }
    match sub {
        None => {
            let f = fields
                .get_mut(base)
                .ok_or_else(|| RunError::UndefinedName(base.to_string()))?;
            apply(f)
        }
        Some(s) => {
            let idx = resolve_int(s, fields).ok_or_else(|| {
                RunError::Unsupported(format!("subscript '{s}' is not an integer"))
            })? as usize;
            let f = fields
                .get(base)
                .ok_or_else(|| RunError::UndefinedName(base.to_string()))?;
            let occ = f.occurs.max(1);
            let elem = f.bytes.len() / occ;
            if idx < 1 || idx > occ {
                // EC-BOUND-SUBSCRIPT: ON -> raise; OFF (default) -> suppressed, the OOB write is a no-op
                // (cobc writes into adjacent storage, UB; the safe port does nothing).
                if EC_BOUND_SUBSCRIPT_ON.with(|c| c.get()) {
                    return Err(RunError::Runtime(format!(
                        "subscript of '{base}' out of bounds: {idx} (maximum: {occ})"
                    )));
                }
                return Ok(());
            }
            let mut tmp = table_element(f, idx, base)?;
            apply(&mut tmp)?;
            let f = fields.get_mut(base).expect("base field present");
            let start = (idx - 1) * elem;
            f.bytes[start..start + elem].copy_from_slice(&tmp.bytes);
            Ok(())
        }
    }
}

/// MOVE a source `(bytes, attr)` into a field via the right runtime path (edited vs cob_move).
fn move_into(
    f: &mut Field,
    sbytes: &[u8],
    sattr: &FieldAttr,
    decimal_comma: bool,
) -> Result<(), RunError> {
    match &f.storage {
        Storage::Edited(pic, currency, decimal_comma, blank_zero) => {
            // numeric/alnum source into a numeric-edited receiver: decode the source to a decimal,
            // then encode per the edited PIC (the move.c numeric->edited path).
            let pic = pic.clone();
            let cur = *currency;
            let dc = *decimal_comma;
            let blank = *blank_zero;
            let dec = decode_numeric_source(sbytes, sattr)?;
            // BLANK WHEN ZERO: a zero value blanks the whole edited field.
            f.bytes = if blank && dec_is_zero(&dec) {
                vec![b' '; f.bytes.len()]
            } else {
                encode_edited_cfg(&pic, &dec, cur, dc)
                    .map_err(|e| RunError::Runtime(format!("{e:?}")))?
            };
            Ok(())
        }
        // JUSTIFIED RIGHT alphanumeric receiver: right-align the source (left-truncating).
        Storage::Alpha(attr) if attr.justified() => {
            f.bytes = alnum_justified_or_left(sbytes, f.bytes.len(), true);
            Ok(())
        }
        Storage::Numeric(attr) | Storage::Alpha(attr) => {
            let attr = *attr;
            // BLANK WHEN ZERO on a numeric receiver: a zero value blanks the field.
            if attr.blank_when_zero()
                && source_to_decimal(sbytes, sattr)
                    .map(|d| dec_is_zero(&d))
                    .unwrap_or(false)
            {
                f.bytes = vec![b' '; f.bytes.len()];
                return Ok(());
            }
            let mut dst = f.bytes.clone();
            // cob_move_cfg honors DECIMAL-POINT IS COMMA on the alphanumeric->numeric leaf (move.c reads
            // dec_pt/num_sep from the module): MOVE "12,34" under comma stores 12.34, not 1234.
            cob_move_cfg(sbytes, sattr, &mut dst, &attr, decimal_comma)
                .map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            f.bytes = dst;
            Ok(())
        }
        Storage::Group { .. } => Err(RunError::Unsupported(
            "a group MOVE is distributed across its leaves by write_field".into(),
        )),
        Storage::Condition { .. } => Err(RunError::Unsupported(
            "cannot MOVE into an 88 condition-name".into(),
        )),
    }
}

/// Decode a numeric DISPLAY (or alnum-of-digits) source `(bytes, attr)` to a [`Decimal`].
/// Decode any numeric source to a [`Decimal`]: DISPLAY / alphanumeric digit strings go straight through
/// [`source_to_decimal`]; a binary (COMP/COMP-5) / packed (COMP-3) / float source -- which that function
/// cannot read -- is first converted to a signed DISPLAY intermediate via the sealed `cob_move`.
fn decode_numeric_source(bytes: &[u8], attr: &FieldAttr) -> Result<Decimal, RunError> {
    use crate::attr::{COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_PACKED};
    let needs_convert = matches!(
        attr.field_type,
        COB_TYPE_NUMERIC_BINARY | COB_TYPE_NUMERIC_PACKED
    ) || matches!(attr.field_type, 0x13 | 0x14 | 0x15); // COMP-1 / COMP-2 / extended float
    if !needs_convert {
        return source_to_decimal(bytes, attr);
    }
    let tattr = lit_num_attr(attr.digits.max(1), attr.scale.max(0), true);
    let mut buff = vec![b'0'; tattr.digits as usize];
    crate::move_ops::cob_move(bytes, attr, &mut buff, &tattr)
        .map_err(|e| RunError::Runtime(format!("{e:?}")))?;
    source_to_decimal(&buff, &tattr)
}

fn source_to_decimal(bytes: &[u8], attr: &FieldAttr) -> Result<Decimal, RunError> {
    let mut digits = Vec::new();
    let mut negative = false;
    for (idx, &b) in bytes.iter().enumerate() {
        let last = idx + 1 == bytes.len();
        if b.is_ascii_digit() {
            digits.push(b - b'0');
        } else if (0x70..=0x79).contains(&b) {
            // trailing overpunch negative zone.
            digits.push(b - 0x70);
            if last {
                negative = true;
            }
        } else if (0x40..=0x49).contains(&b) || (0x50..=0x59).contains(&b) {
            // positive overpunch / EBCDIC-ish zones -> treat low nibble as the digit.
            digits.push(b & 0x0f);
        } else if b == b' ' {
            // skip spaces (alnum padding)
        } else {
            digits.push(b & 0x0f);
        }
    }
    if digits.is_empty() {
        digits.push(0);
    }
    let scale = attr.scale.max(0);
    Ok(Decimal {
        negative,
        digits,
        scale,
    })
}

/// `ADD/SUBTRACT/MULTIPLY/DIVIDE ...` -- the `TO`/`FROM`/`BY`/`INTO`/`GIVING` forms over numeric
/// receivers, dispatched onto the sealed arithmetic primitives.
/// `ADD`/`SUBTRACT`/`MULTIPLY`/`DIVIDE` -- returns `true` if a SIZE ERROR (e.g. DIVIDE by zero) occurred,
/// leaving the receiver UNCHANGED. The caller dispatches the `ON SIZE ERROR` handler.
fn exec_arith(
    verb: &str,
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    has_handler: bool,
) -> Result<bool, RunError> {
    match exec_arith_inner(verb, stmt, fields, has_handler) {
        Ok(size_err) => Ok(size_err),
        Err(RunError::SizeError) => {
            set_exception("EC-SIZE-ZERO-DIVIDE");
            Ok(true)
        }
        Err(e) => Err(e),
    }
}

fn exec_arith_inner(
    verb: &str,
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    has_handler: bool,
) -> Result<bool, RunError> {
    // Resolve any FUNCTION reference or `LENGTH OF id` register in the operands into a temp field first, so
    // `ADD LENGTH OF X TO Y` / `COMPUTE`-free arithmetic over intrinsics works (no-op if none present).
    let stmt = resolve_functions(stmt, fields)?;
    let stmt = &stmt[..];
    // ADD/SUBTRACT CORRESPONDING pairs elementary leaves between two groups BY NAME: `ADD CORR g1 TO g2`
    // does `g2.leaf += g1.leaf` for each like-named NUMERIC pair; `SUBTRACT CORR g1 FROM g2` subtracts.
    // (MULTIPLY/DIVIDE have no CORR form.) Only numeric leaves participate; others are skipped.
    if matches!(stmt.first(), Some(Tok::Word(w)) if w == "CORRESPONDING" || w == "CORR") {
        let conn = if verb == "SUBTRACT" { "FROM" } else { "TO" };
        if verb != "ADD" && verb != "SUBTRACT" {
            return Err(RunError::Unsupported(format!(
                "{verb} CORRESPONDING is not a valid form"
            )));
        }
        let cp = stmt
            .iter()
            .position(|t| matches!(t, Tok::Word(w) if w == conn))
            .ok_or_else(|| RunError::Unsupported(format!("{verb} CORRESPONDING without {conn}")))?;
        let src = match stmt.get(1) {
            Some(Tok::Word(w)) => w.clone(),
            _ => {
                return Err(RunError::Unsupported(format!(
                    "{verb} CORRESPONDING: missing source group"
                )))
            }
        };
        let dst = match stmt.get(cp + 1) {
            Some(Tok::Word(w)) => w.clone(),
            _ => {
                return Err(RunError::Unsupported(format!(
                    "{verb} CORRESPONDING: missing target group"
                )))
            }
        };
        let mut any_se = false;
        for (sk, dk) in corr_pairs(fields, &src, &dst)? {
            let both_numeric = matches!(
                fields.get(&sk).map(|f| &f.storage),
                Some(Storage::Numeric(_))
            ) && matches!(
                fields.get(&dk).map(|f| &f.storage),
                Some(Storage::Numeric(_))
            );
            if !both_numeric {
                continue; // CORR arithmetic applies only to numeric leaves
            }
            let pair = vec![Tok::Word(sk), Tok::Word(conn.to_string()), Tok::Word(dk)];
            any_se |= exec_arith_inner(verb, &pair, fields, has_handler)?;
        }
        return Ok(any_se);
    }
    // find a GIVING receiver if present.
    let giving = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w=="GIVING"));
    let kw = match verb {
        "ADD" => "TO",
        "SUBTRACT" => "FROM",
        "MULTIPLY" => "BY",
        "DIVIDE" => {
            if stmt.iter().any(|t| matches!(t, Tok::Word(w) if w=="INTO")) {
                "INTO"
            } else {
                "BY"
            }
        }
        _ => unreachable!(),
    };
    let kw_at = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w==kw));

    // the source operand list is everything before kw (or before GIVING for the keyword-less forms).
    let end_src = kw_at.or(giving).unwrap_or(stmt.len());
    let sources: Vec<&Tok> = stmt[..end_src]
        .iter()
        .filter(|t| matches!(t, Tok::Str(_)) || matches!(t, Tok::Word(w) if !is_kw(w)))
        .collect();
    if sources.is_empty() {
        return Err(RunError::Unsupported(format!("{verb}: no source operands")));
    }

    // Fold the source operands into a single decimal-bearing (bytes, attr) accumulator. Operands are
    // normalized so binary (COMP/COMP-5/COMP-X) sources participate (cob_arith's decode is DISPLAY+PACKED).
    let (mut acc, mut acc_attr) = {
        let (b, a) = operand_value(sources[0], fields)?;
        to_arith_operand(&b, &a)?
    };
    for s in &sources[1..] {
        let (b, a) = operand_value(s, fields)?;
        // Widen-THEN-add (wide_op), not add-then-widen: cob_arith truncates the result to acc_attr's width,
        // so folding into a narrow acc_attr (e.g. two 2-digit literals) dropped the carry digit (60+60 ->
        // "20", then the SIZE ERROR was judged on the wrong value). wide_op widens acc to 18 digits first.
        let (r, ra) = wide_op(Op::Add, &acc, &acc_attr, &b, &a)?;
        acc = r;
        acc_attr = ra;
    }

    // DIVIDE ... GIVING q REMAINDER r -- wire the sealed `cob_divide_remainder` primitive
    // (GNURUST.REMAINDER.1): quotient truncated toward zero to q's scale, r = dividend - (that quotient *
    // divisor). ON SIZE ERROR / NOT ON SIZE ERROR handlers are honored (divide-by-zero + receiver overflow).
    if verb == "DIVIDE"
        && stmt
            .iter()
            .any(|t| matches!(t, Tok::Word(w) if w == "REMAINDER"))
    {
        return exec_divide_remainder(
            stmt,
            kw,
            giving,
            kw_at,
            &acc,
            &acc_attr,
            fields,
            has_handler,
        );
    }

    // Collect every receiver. A GIVING phrase stores one computed result into EACH named receiver
    // (`ADD a b GIVING c d` -> c = d = a+b). The in-place TO/FROM/BY/INTO forms instead update each
    // receiver by ITS OWN current value (`ADD 1 TO Y Z` -> Y+1 and Z+1; `MULTIPLY 3 BY Y Z` -> Y*3, Z*3).
    let giving_names: Vec<String> = match giving {
        Some(gp) => stmt[gp + 1..]
            .iter()
            .filter_map(|t| match t {
                Tok::Word(w) if !is_kw(w) => Some(w.clone()),
                _ => None,
            })
            .collect(),
        None => vec![],
    };
    let target_words: Vec<String> = match kw_at {
        Some(kp) => {
            let end = giving.unwrap_or(stmt.len());
            stmt[kp + 1..end]
                .iter()
                .filter_map(|t| match t {
                    Tok::Word(w) if !is_kw(w) => Some(w.clone()),
                    _ => None,
                })
                .collect()
        }
        None => vec![],
    };

    // (receiver, target-for-computation). GIVING: one value (computed from the single in-place operand,
    // if any) into every receiver. In-place: each receiver is also its own computation target.
    let receivers: Vec<(String, Option<String>)> = if !giving_names.is_empty() {
        let t = target_words.first().cloned();
        giving_names
            .iter()
            .map(|g| (g.clone(), t.clone()))
            .collect()
    } else {
        target_words
            .iter()
            .map(|r| (r.clone(), Some(r.clone())))
            .collect()
    };
    if receivers.is_empty() {
        return Err(RunError::Unsupported(format!("{verb}: no receiver")));
    }

    // A `ROUNDED [MODE [IS] <mode>]` phrase rounds each result to its receiver's scale before the store
    // (default mode: NEAREST-AWAY-FROM-ZERO); otherwise the store truncates.
    let round_mode = round_mode_of(stmt);
    let mut any_size_err = false;
    for (recv_name, tgt) in &receivers {
        // The result is a WIDE numeric (bytes, attr); the per-receiver store truncates/edits it into the
        // receiver's exact format. libcob pattern: arithmetic is exact, the store is the rounding point.
        let (rb, ra) = arith_compute(verb, kw, &acc, &acc_attr, tgt.as_deref(), fields)?;
        let f = fields
            .get_mut(recv_name)
            .ok_or_else(|| RunError::UndefinedName(recv_name.clone()))?;
        let se = if let Some(mode) = round_mode {
            let dec = source_to_decimal(&rb, &ra)?;
            let (rdec, prohibited) = round_decimal_mode(&dec, receiver_scale(f), mode);
            if prohibited {
                true // MODE PROHIBITED + dropped non-zero digit -> size error, receiver unchanged
            } else {
                let (nb, na) = decimal_as_display(&rdec);
                store_arith_result(f, &nb, &na, has_handler, false)?
            }
        } else {
            store_arith_result(f, &rb, &ra, has_handler, false)?
        };
        any_size_err |= se;
    }
    Ok(any_size_err)
}

/// Compute the WIDE numeric result for one receiver of an arithmetic statement, given an optional
/// in-place `target` operand. `ADD a... GIVING c` has no target (result = sum); the in-place / `TO t
/// GIVING` forms supply the receiver (or the single TO/BY/INTO operand) as `target`.
fn arith_compute(
    verb: &str,
    kw: &str,
    acc: &[u8],
    acc_attr: &FieldAttr,
    target: Option<&str>,
    fields: &HashMap<String, Field>,
) -> Result<(Vec<u8>, FieldAttr), RunError> {
    Ok(match (verb, target) {
        // ADD a... TO t [GIVING c]:  result = sum(a...) + t
        ("ADD", Some(t)) => {
            let (tb, ta) = operand_value(&Tok::Word(t.to_string()), fields)?;
            wide_op(Op::Add, acc, acc_attr, &tb, &ta)?
        }
        // ADD a... GIVING c:  result = sum(a...)
        ("ADD", None) => (acc.to_vec(), *acc_attr),
        // SUBTRACT a... FROM t [GIVING c]:  result = t - sum(a...)
        ("SUBTRACT", Some(t)) => {
            let (tb, ta) = operand_value(&Tok::Word(t.to_string()), fields)?;
            wide_op(Op::Subtract, &tb, &ta, acc, acc_attr)?
        }
        // MULTIPLY a BY t [GIVING c]:  result = a * t
        ("MULTIPLY", Some(t)) => {
            let (tb, ta) = operand_value(&Tok::Word(t.to_string()), fields)?;
            wide_op(Op::Multiply, acc, acc_attr, &tb, &ta)?
        }
        // DIVIDE a INTO t [GIVING c]: result = t / a ;  DIVIDE a BY t [GIVING c]: result = a / t
        ("DIVIDE", Some(t)) => {
            let (tb, ta) = operand_value(&Tok::Word(t.to_string()), fields)?;
            // Normalize a binary (COMP/COMP-5/COMP-X) operand to zoned DISPLAY -- cob_divide's decoder
            // handles only DISPLAY+PACKED (a raw binary operand would be InvalidAttr).
            let (tb, ta) = to_arith_operand(&tb, &ta)?;
            let (num, na, den, da) = if kw == "INTO" {
                (tb, ta, acc.to_vec(), *acc_attr)
            } else {
                (acc.to_vec(), *acc_attr, tb, ta)
            };
            let wide = lit_num_attr(36, 18, true); // generous quotient scale; the store truncates.
            let q =
                cob_divide(&num, &na, &den, &da, &wide, Round::Truncate).map_err(map_arith_err)?;
            (q, wide)
        }
        _ => {
            return Err(RunError::Unsupported(format!(
                "{verb} form (target/giving)"
            )))
        }
    })
}

/// `DIVIDE a {INTO|BY} b GIVING q REMAINDER r [ROUNDED]` -- wires the sealed `cob_divide_remainder`
/// (GNURUST.REMAINDER.1). `acc` is the single source operand `a`; `b` is the operand after INTO/BY. The
/// remainder uses the UN-rounded quotient (truncated toward zero to q's scale); a `ROUNDED` phrase rounds
/// only the quotient STORE. Quotient/remainder receivers must be numeric. `ON SIZE ERROR` / `NOT ON SIZE
/// ERROR` are honored: a zero divisor (whole statement) or a receiver that loses high-order digits raises
/// the size error; an overflowing receiver is left UNCHANGED with a handler, truncate-stored without one.
fn exec_divide_remainder(
    stmt: &[Tok],
    kw: &str,
    giving: Option<usize>,
    kw_at: Option<usize>,
    acc: &[u8],
    acc_attr: &FieldAttr,
    fields: &mut HashMap<String, Field>,
    has_handler: bool,
) -> Result<bool, RunError> {
    let gp = giving
        .ok_or_else(|| RunError::Unsupported("DIVIDE ... REMAINDER requires GIVING".into()))?;
    let kp = kw_at
        .ok_or_else(|| RunError::Unsupported("DIVIDE ... REMAINDER: missing INTO/BY".into()))?;
    // q = first data-name after GIVING (before REMAINDER), r = the data-name after REMAINDER.
    let names: Vec<String> = stmt[gp + 1..]
        .iter()
        .filter_map(|t| match t {
            Tok::Word(w) if !is_kw(w) => Some(w.clone()),
            _ => None,
        })
        .collect();
    if names.len() != 2 {
        return Err(RunError::Unsupported(
            "DIVIDE ... REMAINDER: GIVING needs exactly one quotient + one remainder receiver"
                .into(),
        ));
    }
    let (qn, rn) = (names[0].clone(), names[1].clone());
    // The dividend/divisor operand `b` sits between INTO/BY and GIVING.
    let b_tok = stmt[kp + 1..gp]
        .iter()
        .find(|t| matches!(t, Tok::Str(_)) || matches!(t, Tok::Word(w) if !is_kw(w)))
        .ok_or_else(|| {
            RunError::Unsupported("DIVIDE ... REMAINDER: missing dividend/divisor operand".into())
        })?;
    let (b_bytes, b_attr) = {
        let (x, y) = operand_value(b_tok, fields)?;
        to_arith_operand(&x, &y)?
    };
    // INTO: dividend = b, divisor = a;  BY: dividend = a, divisor = b.
    let (lhs, la, rhs, ra): (&[u8], &FieldAttr, &[u8], &FieldAttr) = if kw == "INTO" {
        (&b_bytes, &b_attr, acc, acc_attr)
    } else {
        (acc, acc_attr, &b_bytes, &b_attr)
    };
    let q_attr = numeric_receiver_attr(fields, &qn)?;
    let r_attr = numeric_receiver_attr(fields, &rn)?;
    let (qb, rb) =
        cob_divide_remainder(lhs, la, rhs, ra, &q_attr, &r_attr).map_err(map_arith_err)?;

    // ON SIZE ERROR: a receiver that would lose high-order integer digits raises EC-SIZE-OVERFLOW. The
    // stored bytes stay the sealed primitive's (`qb`/`rb`); we re-derive the WIDE quotient (truncated to
    // the quotient's scale at full integer precision) and the exact remainder ONLY to test each receiver's
    // capacity. With a handler an overflowing receiver is left UNCHANGED; without one it truncate-stores
    // (both match cobc). Divide-by-zero already propagated above as a SizeError (caught by the wrapper).
    let qwide_attr = lit_num_attr(18 + q_attr.scale.max(0) as u16, q_attr.scale.max(0), true);
    let q_wide =
        cob_divide(lhs, la, rhs, ra, &qwide_attr, Round::Truncate).map_err(map_arith_err)?;
    let (prod, pa) = wide_op(Op::Multiply, &q_wide, &qwide_attr, rhs, ra)?;
    let (r_wide, rwa) = wide_op(Op::Subtract, lhs, la, &prod, &pa)?;
    let mut size_err = false;

    // Remainder receiver: store the sealed `rb`, but leave it unchanged on overflow when a handler is present.
    {
        let rf = fields
            .get(&rn)
            .ok_or_else(|| RunError::UndefinedName(rn.clone()))?;
        let overflow =
            receiver_int_digits(rf).map_or(false, |cap| arith_overflows(&r_wide, &rwa, cap));
        if overflow {
            set_exception("EC-SIZE-OVERFLOW");
            size_err = true;
        }
        if !overflow || !has_handler {
            fields
                .get_mut(&rn)
                .ok_or_else(|| RunError::UndefinedName(rn.clone()))?
                .bytes = rb;
        }
    }

    // Quotient receiver: ROUNDED rounds only the quotient STORE (the remainder used the un-rounded quotient).
    // The overflow test uses the value that will actually be stored (rounded when ROUNDED is present).
    let rounded = stmt
        .iter()
        .any(|t| matches!(t, Tok::Word(w) if w == "ROUNDED"));
    let rounded_q: Option<(Vec<u8>, FieldAttr)> = if rounded {
        let wide = lit_num_attr(36, 18, true);
        let wide_q = cob_divide(lhs, la, rhs, ra, &wide, Round::Truncate).map_err(map_arith_err)?;
        let dec = source_to_decimal(&wide_q, &wide)?;
        Some(decimal_as_display(&round_decimal(&dec, q_attr.scale)))
    } else {
        None
    };
    {
        // Overflow is measured against the to-be-stored value: the rounded image, else the full-precision quotient.
        let (chk_b, chk_a) = rounded_q
            .as_ref()
            .map_or((q_wide.as_slice(), &qwide_attr), |(b, a)| (b.as_slice(), a));
        let qf = fields
            .get(&qn)
            .ok_or_else(|| RunError::UndefinedName(qn.clone()))?;
        let overflow =
            receiver_int_digits(qf).map_or(false, |cap| arith_overflows(chk_b, chk_a, cap));
        if overflow {
            set_exception("EC-SIZE-OVERFLOW");
            size_err = true;
        }
        if !overflow || !has_handler {
            let qf = fields
                .get_mut(&qn)
                .ok_or_else(|| RunError::UndefinedName(qn.clone()))?;
            match &rounded_q {
                Some((nb, na)) => move_into(qf, nb, na, false)?,
                None => qf.bytes = qb,
            }
        }
    }
    Ok(size_err)
}

/// A field's numeric `FieldAttr` for use as a `DIVIDE ... REMAINDER` receiver; edited/group/alpha
/// receivers are out of the sealed remainder subset and fail closed.
fn numeric_receiver_attr(
    fields: &HashMap<String, Field>,
    name: &str,
) -> Result<FieldAttr, RunError> {
    match fields.get(name) {
        Some(Field {
            storage: Storage::Numeric(a),
            ..
        }) => Ok(*a),
        Some(_) => Err(RunError::Unsupported(format!(
            "DIVIDE ... REMAINDER: receiver `{name}` must be a numeric (non-edited) item"
        ))),
        None => Err(RunError::UndefinedName(name.to_string())),
    }
}

/// Compute `op(a, b)` exactly into a wide numeric DISPLAY `(bytes, attr)` -- 18 integer digits plus a
/// scale generous enough to be exact for add/subtract (max operand scale) and multiply (sum of
/// scales); the receiver store is the truncation point.
fn wide_op(
    op: Op,
    a: &[u8],
    aa: &FieldAttr,
    b: &[u8],
    ba: &FieldAttr,
) -> Result<(Vec<u8>, FieldAttr), RunError> {
    let scale = match op {
        Op::Multiply => (aa.scale.max(0) + ba.scale.max(0)).max(0),
        _ => aa.scale.max(ba.scale).max(0),
    };
    let wide = lit_num_attr(18 + scale as u16, scale, true);
    let wsize = wide.digits as usize;
    let mut a_wide = vec![b'0'; wsize.max(1)];
    cob_move(a, aa, &mut a_wide, &wide).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
    // The `b` operand goes straight to cob_arith, whose decode handles only DISPLAY + PACKED; normalize a
    // binary (COMP/COMP-5/COMP-X) `b` to zoned DISPLAY first (the `a` side was already widened by cob_move).
    let (bn, ban) = to_arith_operand(b, ba)?;
    let r = cob_arith(op, &a_wide, &wide, &bn, &ban, Round::Truncate)
        .map_err(|e| RunError::Runtime(format!("{e:?}")))?;
    Ok((r, wide))
}

/// Normalize a numeric operand to a form `cob_arith`'s decoder accepts. DISPLAY and PACKED
/// (COMP-3/COMP-6) pass through unchanged; binary (COMP/COMP-5/COMP-X) is converted to zoned DISPLAY via
/// the sealed `cob_move`. This keeps binary-USAGE arithmetic entirely on the oracle-sealed conversion +
/// arithmetic paths without extending the runtime decode.
fn to_arith_operand(bytes: &[u8], attr: &FieldAttr) -> Result<(Vec<u8>, FieldAttr), RunError> {
    use crate::attr::COB_TYPE_NUMERIC_PACKED;
    if matches!(
        attr.field_type,
        COB_TYPE_NUMERIC_DISPLAY | COB_TYPE_NUMERIC_PACKED
    ) {
        return Ok((bytes.to_vec(), *attr));
    }
    let disp = lit_num_attr(attr.digits.max(1), attr.scale.max(0), true);
    let mut out = vec![b'0'; disp.digits as usize];
    cob_move(bytes, attr, &mut out, &disp).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
    Ok((out, disp))
}

/// Is `w` an arithmetic keyword (not an operand name)?
fn is_kw(w: &str) -> bool {
    matches!(
        w,
        "TO" | "FROM"
            | "BY"
            | "INTO"
            | "GIVING"
            | "ROUNDED"
            | "REMAINDER"
            | "MODE"
            | "TRUNCATION"
            | "NEAREST-AWAY-FROM-ZERO"
            | "AWAY-FROM-ZERO"
            | "NEAREST-TOWARD-ZERO"
            | "NEAREST-EVEN"
            | "TOWARD-GREATER"
            | "TOWARD-LESSER"
            | "PROHIBITED"
    )
}

/// Parse the `ROUNDED [MODE [IS] <mode-name>]` phrase from an arithmetic statement's tokens. Returns
/// `None` when there is no `ROUNDED` (the store truncates), `Some(mode)` otherwise (plain `ROUNDED` = the
/// default NEAREST-AWAY-FROM-ZERO).
fn round_mode_of(stmt: &[Tok]) -> Option<Round> {
    let pos = stmt
        .iter()
        .position(|t| matches!(t, Tok::Word(w) if w == "ROUNDED"))?;
    let mut i = pos + 1;
    if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "MODE") {
        i += 1;
        if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "IS") {
            i += 1;
        }
        if let Some(Tok::Word(name)) = stmt.get(i) {
            return Some(match name.as_str() {
                "TRUNCATION" => Round::Truncate,
                "AWAY-FROM-ZERO" => Round::AwayFromZero,
                "NEAREST-TOWARD-ZERO" => Round::NearTowardZero,
                "NEAREST-EVEN" => Round::NearEven,
                "TOWARD-GREATER" => Round::TowardGreater,
                "TOWARD-LESSER" => Round::TowardLesser,
                "PROHIBITED" => Round::Prohibited,
                _ => Round::NearAwayFromZero, // NEAREST-AWAY-FROM-ZERO + any unrecognized
            });
        }
    }
    Some(Round::NearAwayFromZero)
}

/// Round `dec` to `target_scale` fractional digits under a COBOL rounding mode. Returns the rounded value
/// and whether `MODE PROHIBITED` was violated (a non-zero digit was dropped) -- the caller raises a size
/// error and leaves the receiver unchanged.
fn round_decimal_mode(dec: &Decimal, target_scale: i16, mode: Round) -> (Decimal, bool) {
    let ts = target_scale.max(0);
    if dec.scale <= ts {
        return (dec.clone(), false);
    }
    let drop = (dec.scale - ts) as usize;
    let keep = dec.digits.len().saturating_sub(drop);
    let first = dec.digits[keep];
    let rest_nonzero = dec.digits[keep + 1..].iter().any(|&d| d != 0);
    let any_nonzero = first != 0 || rest_nonzero;
    let mut kept: Vec<u8> = dec.digits[..keep].to_vec();
    let last_kept = kept.last().copied().unwrap_or(0);
    let round_up = match mode {
        Round::Truncate | Round::Prohibited => false,
        Round::NearAwayFromZero => first >= 5,
        Round::AwayFromZero => any_nonzero,
        Round::NearTowardZero => first > 5 || (first == 5 && rest_nonzero),
        Round::NearEven => first > 5 || (first == 5 && (rest_nonzero || last_kept % 2 == 1)),
        Round::TowardGreater => any_nonzero && !dec.negative,
        Round::TowardLesser => any_nonzero && dec.negative,
    };
    let prohibited_violation = matches!(mode, Round::Prohibited) && any_nonzero;
    if round_up {
        kept = inc_magnitude(kept);
    }
    if kept.is_empty() {
        kept.push(0);
    }
    (
        Decimal {
            negative: dec.negative,
            digits: kept,
            scale: ts,
        },
        prohibited_violation,
    )
}

/// Find the index of a contiguous keyword sequence (e.g. `["PROCEDURE","DIVISION"]`).
/// `SPECIAL-NAMES. CURRENCY SIGN IS "x".` -> the currency symbol byte (default `b'$'`). Scans the tokens
/// before the data division for `CURRENCY [SIGN] [IS] "<sym>"`. Only the first byte of the literal is the
/// symbol (GnuCOBOL allows a 1-char currency sign for editing).
fn parse_currency_sign(toks: &[Tok], before: usize) -> u8 {
    let mut i = 0;
    while i < before {
        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "CURRENCY") {
            let mut k = i + 1;
            if matches!(toks.get(k), Some(Tok::Word(w)) if w == "SIGN") {
                k += 1;
            }
            if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS") {
                k += 1;
            }
            if let Some(Tok::Str(s)) = toks.get(k) {
                if let Some(&b) = s.first() {
                    return b;
                }
            }
        }
        i += 1;
    }
    b'$'
}

/// `SPECIAL-NAMES. DECIMAL-POINT IS COMMA.` -> true if the program swaps the roles of `.` and `,` (`,`
/// becomes the decimal point, `.` the grouping separator). Scans the tokens before the data division for
/// `DECIMAL-POINT [IS] COMMA` (the lexer keeps the hyphenated `DECIMAL-POINT` as one word).
fn parse_decimal_comma(toks: &[Tok], before: usize) -> bool {
    let mut i = 0;
    while i < before {
        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "DECIMAL-POINT") {
            let mut k = i + 1;
            if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS") {
                k += 1;
            }
            if matches!(toks.get(k), Some(Tok::Word(w)) if w == "COMMA") {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// True when a word is shaped exactly like a signed decimal-comma numeric literal (`[+-]?digits,digits`),
/// e.g. `1234,56` or `-12,5`. Used under DECIMAL-POINT IS COMMA to rewrite such literals to the internal
/// `.`-decimal form. PICTURE strings (which contain letters/`9`/`Z`/`(`) and field names never match, so
/// the rewrite cannot corrupt them.
fn is_comma_decimal_literal(w: &str) -> bool {
    let body = w.trim_start_matches(['+', '-']);
    let mut parts = body.splitn(3, ',');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(int_p), Some(frac_p), None) => {
            !int_p.is_empty()
                && !frac_p.is_empty()
                && int_p.bytes().all(|b| b.is_ascii_digit())
                && frac_p.bytes().all(|b| b.is_ascii_digit())
        }
        _ => false,
    }
}

fn find_seq(toks: &[Tok], seq: &[&str]) -> Option<usize> {
    if seq.is_empty() {
        return None;
    }
    'outer: for i in 0..toks.len() {
        for (j, s) in seq.iter().enumerate() {
            match toks.get(i + j) {
                Some(Tok::Word(w)) if w == s => {}
                _ => continue 'outer,
            }
        }
        return Some(i);
    }
    None
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.FRONTEND.1, GNURUST.FILEIO.MULTI-RECORD-FD.1
    /// The lexer is total: tokenizing any short ASCII byte sequence never panics (the front-end's
    /// parse entry must fail closed, never crash, on garbage). Bounded to a few bytes for tractability.
    #[kani::proof]
    #[kani::unwind(6)]
    fn lex_never_panics() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let c: u8 = kani::any();
        kani::assume(a.is_ascii() && b.is_ascii() && c.is_ascii());
        let s = [a, b, c];
        if let Ok(text) = core::str::from_utf8(&s) {
            let _ = lex(text); // must return without panicking
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<u8> {
        run_program(src).expect("run")
    }

    #[test]
    fn display_numeric_literals_match_cobc_format() {
        // GnuCOBOL 3.2 DISPLAY of numeric literals (run_fundamental.at:25): written sign preserved,
        // leading integer zeros dropped (`.123`), E-notation folded, binary/hex shown as decimal.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. p.\n       PROCEDURE DIVISION.\n           DISPLAY 123.\n           DISPLAY +123.\n           DISPLAY -123.\n           DISPLAY 12.3.\n           DISPLAY 1.23E0.\n           DISPLAY 12.3E-2.\n           DISPLAY B'0101'.\n           DISPLAY BX'EC'.\n           STOP RUN.\n";
        let out = String::from_utf8(run(src)).unwrap();
        assert_eq!(out, "123\n+123\n-123\n12.3\n1.23\n.123\n5\n236\n");
    }

    #[test]
    fn cancel_of_active_program_is_fatal_libcob_error() {
        // run_fundamental.at:2277 -- CANCELing the currently-executing (non-INITIAL) program is a
        // fatal runtime error carrying the source line, exit 1.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. prog.\n       PROCEDURE DIVISION.\n           CANCEL \"notthere\".\n           CANCEL \"prog\".\n           DISPLAY \"NG\" NO ADVANCING END-DISPLAY.\n           STOP RUN.\n";
        set_source_file("prog.cob");
        let err = run_program(src).unwrap_err();
        set_source_file("");
        let msg = err.to_string();
        assert!(
            msg.contains("attempt to CANCEL active program"),
            "got: {msg}"
        );
    }

    #[test]
    fn command_line_accept_reads_host_args() {
        // cobcrun module arg1 arg2 -> ACCEPT FROM COMMAND-LINE / ARGUMENT-VALUE / ARGUMENT-NUMBER.
        set_command_line(&["job123".to_string()]);
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. p.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 CLI PIC X(20).\n       PROCEDURE DIVISION.\n           ACCEPT CLI FROM COMMAND-LINE\n           DISPLAY CLI WITH NO ADVANCING END-DISPLAY.\n           STOP RUN.\n";
        let out = String::from_utf8(run(src)).unwrap();
        set_command_line(&[]);
        // PIC X(20) receiver: the value is stored left-justified and DISPLAYed padded to 20.
        assert_eq!(out, "job123              ");
    }

    #[test]
    fn line_leading_star_slash_comment_is_dropped() {
        // Fixed-format column-7 comment (`*`/`/` as the first non-blank char of the line), not just column 1.
        // These real-world programs carry `      **** ... ****` banner lines; they must be dropped, not lexed.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n      **** banner level 77 verb ****\n      / page eject\n       PROCEDURE DIVISION.\n           DISPLAY \"OK\".\n           STOP RUN.\n";
        assert_eq!(run(src), b"OK\n");
    }

    #[test]
    fn open_input_reads_real_disk_file() {
        use std::io::Write;
        // OPEN INPUT of a pre-existing real file on disk (resolved via an env-named ASSIGN target) loads it
        // into fixed records and READ returns them -- the safe real-file read (no host writes).
        let path = std::env::temp_dir().join(format!("gcrs_fileread_{}.dat", std::process::id()));
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"ABC       DEF       ")
            .unwrap(); // 2 x 10-byte recs
        std::env::set_var("GCRSTESTFILE", &path);
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO GCRSTESTFILE ORGANIZATION SEQUENTIAL FILE STATUS IS ST.\n       DATA DIVISION. FILE SECTION.\n       FD F. 01 R PIC X(10).\n       WORKING-STORAGE SECTION. 01 ST PIC XX. 01 E PIC X VALUE \"N\".\n       PROCEDURE DIVISION.\n           OPEN INPUT F.\n           PERFORM UNTIL E = \"Y\"\n              READ F AT END MOVE \"Y\" TO E NOT AT END DISPLAY R(1:3) END-READ\n           END-PERFORM.\n           CLOSE F. STOP RUN.\n";
        let out = run_program(src).expect("run");
        std::env::remove_var("GCRSTESTFILE");
        std::fs::remove_file(&path).ok();
        assert_eq!(
            out,
            b"ABC\nDEF\n",
            "got {:?}",
            String::from_utf8_lossy(&out)
        );
    }

    #[test]
    fn inline_star_gt_comment_with_apostrophe_is_stripped() {
        use crate::dialect::Dialect;
        // A free-format `*>` inline comment (after indentation) containing an apostrophe must be stripped
        // before quote tokenization -- otherwise the "'" opens a spurious string that swallows the rest of
        // the program (cobc strips it; cobrun must too). Regression for the p37 corpus discovery.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"OK\".  *> here's the caller's note: don't break\n           STOP RUN.\n";
        let (out, _rc) = run_program_dialect_with_rc(src, Dialect::DEFAULT).unwrap();
        assert_eq!(out, b"OK\n");
    }

    #[test]
    fn turn_ec_bound_subscript_is_honored() {
        use crate::dialect::Dialect;
        let prog = |head: &str| {
            format!(
                "{head}       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 E PIC 9 OCCURS 3.\n       01 I PIC 9 VALUE 5.\n       PROCEDURE DIVISION.\n           MOVE 9 TO E(I).\n           DISPLAY \"OK\".\n           STOP RUN.\n"
            )
        };
        // Default (no >>TURN): EC-BOUND-SUBSCRIPT is OFF -> the out-of-range MOVE is suppressed and the
        // program continues to completion (cobc's default reads/writes adjacent storage and continues).
        let (out, _rc) = run_program_dialect_with_rc(&prog(""), Dialect::DEFAULT).unwrap();
        assert_eq!(out, b"OK\n");
        // >>TURN EC-BOUND-SUBSCRIPT CHECKING ON: the SAME out-of-range subscript now RAISES (honored).
        let on = run_program_dialect_with_rc(
            &prog(">>TURN EC-BOUND-SUBSCRIPT CHECKING ON\n"),
            Dialect::DEFAULT,
        );
        assert!(
            on.is_err(),
            "EC-BOUND-SUBSCRIPT ON must raise on an out-of-range subscript"
        );
        // EC-ALL CHECKING ON also enables it.
        let all =
            run_program_dialect_with_rc(&prog(">>TURN EC-ALL CHECKING ON\n"), Dialect::DEFAULT);
        assert!(all.is_err());
    }

    #[test]
    fn level_88_condition_names() {
        use crate::dialect::Dialect;
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 G PIC 99.\n          88 PASS VALUE 50 THRU 100.\n       01 F PIC X.\n          88 YES VALUE \"Y\" \"y\".\n       PROCEDURE DIVISION.\n           MOVE 75 TO G.\n           IF PASS DISPLAY \"P\" ELSE DISPLAY \"F\" END-IF.\n           MOVE 10 TO G.\n           IF PASS DISPLAY \"P\" ELSE DISPLAY \"F\" END-IF.\n           MOVE \"y\" TO F.\n           IF YES DISPLAY \"Y\" ELSE DISPLAY \"N\" END-IF.\n           STOP RUN.\n";
        let (out, _rc) = run_program_dialect_with_rc(src, Dialect::DEFAULT).unwrap();
        assert_eq!(out, b"P\nF\nY\n");
    }

    #[test]
    fn evaluate_subject_match_and_true() {
        use crate::dialect::Dialect;
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 N PIC 99 VALUE 7.\n       PROCEDURE DIVISION.\n           EVALUATE N\n               WHEN 1 DISPLAY \"ONE\"\n               WHEN 5 THRU 9 DISPLAY \"RANGE\"\n               WHEN OTHER DISPLAY \"OTHER\"\n           END-EVALUATE.\n           EVALUATE TRUE\n               WHEN N > 50 DISPLAY \"BIG\"\n               WHEN N < 50 DISPLAY \"SMALL\"\n           END-EVALUATE.\n           STOP RUN.\n";
        let (out, _rc) = run_program_dialect_with_rc(src, Dialect::DEFAULT).unwrap();
        // N=7 -> 5 THRU 9 -> RANGE ; TRUE -> N<50 -> SMALL.
        assert_eq!(out, b"RANGE\nSMALL\n");
    }

    #[test]
    fn redefines_aliases_shared_storage_both_ways() {
        use crate::dialect::Dialect;
        // REDEFINES makes A and B share storage: a MOVE into one is seen when the other is read.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 N PIC 9(4) VALUE 1234.\n       01 C REDEFINES N PIC X(4).\n       PROCEDURE DIVISION.\n           DISPLAY C.\n           MOVE \"9876\" TO C.\n           DISPLAY N.\n           MOVE 5050 TO N.\n           DISPLAY C.\n           STOP RUN.\n";
        let (out, _rc) = run_program_dialect_with_rc(src, Dialect::DEFAULT).unwrap();
        // C reads N's "1234" ; MOVE "9876" TO C -> N reads 9876 ; MOVE 5050 TO N -> C reads "5050".
        assert_eq!(out, b"1234\n9876\n5050\n");
    }

    #[test]
    fn occurs_table_subscript_read_write() {
        use crate::dialect::Dialect;
        // 01-level OCCURS table: subscripted MOVE/DISPLAY/IF/ADD with literal and variable subscripts.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 E PIC 99 OCCURS 3 TIMES.\n       01 I PIC 9 VALUE 2.\n       01 S PIC 999.\n       PROCEDURE DIVISION.\n           MOVE 11 TO E(1).\n           MOVE 22 TO E(2).\n           MOVE 33 TO E(3).\n           DISPLAY \"A\" E(1) E(I) E(3).\n           ADD E(1) E(3) GIVING S.\n           DISPLAY \"S\" S.\n           IF E(I) > E(1) DISPLAY \"GT\" ELSE DISPLAY \"LE\" END-IF.\n           STOP RUN.\n";
        let (out, _rc) = run_program_dialect_with_rc(src, Dialect::DEFAULT).unwrap();
        assert_eq!(out, b"A112233\nS044\nGT\n");
    }

    #[test]
    fn occurs_subscript_in_compute_expression() {
        use crate::dialect::Dialect;
        // Subscripts inside a COMPUTE arithmetic expression, including grouping `(E(1) + E(3))`.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 E PIC 99 OCCURS 3 TIMES.\n       01 I PIC 9 VALUE 3.\n       01 S PIC 999.\n       PROCEDURE DIVISION.\n           MOVE 10 TO E(1). MOVE 20 TO E(2). MOVE 30 TO E(3).\n           COMPUTE S = (E(1) + E(3)) * 2 - E(I).\n           DISPLAY S.\n           STOP RUN.\n";
        let (out, _rc) = run_program_dialect_with_rc(src, Dialect::DEFAULT).unwrap();
        // (10 + 30) * 2 - 30 = 50.
        assert_eq!(out, b"050\n");
    }

    #[test]
    fn ebcdic_collating_sequence_orders_alphanumeric() {
        use crate::dialect::Dialect;
        // PROGRAM COLLATING SEQUENCE IS <ebcdic-alphabet>: EBCDIC order is lowercase < uppercase < digits
        // (opposite of ASCII). String-literal case is preserved through the uppercase-outside-quotes pass.
        let prog = |cs: &str| {
            format!(
                "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION.\n       CONFIGURATION SECTION.\n       SPECIAL-NAMES. ALPHABET EB IS EBCDIC.\n       {cs}\n       PROCEDURE DIVISION.\n           IF \"a\" < \"A\" DISPLAY \"lo\" ELSE DISPLAY \"up\" END-IF.\n           IF \"Z\" < \"0\" DISPLAY \"let\" ELSE DISPLAY \"dig\" END-IF.\n           STOP RUN.\n"
            )
        };
        // EBCDIC: a<A (lo), Z<0 (let).
        let (out, _rc) = run_program_dialect_with_rc(
            &prog("OBJECT-COMPUTER. PROGRAM COLLATING SEQUENCE IS EB."),
            Dialect::DEFAULT,
        )
        .unwrap();
        assert_eq!(out, b"lo\nlet\n");
        // No collating sequence -> native ASCII: A<a (up), 0<Z (dig).
        let (out2, _rc2) = run_program_dialect_with_rc(&prog(""), Dialect::DEFAULT).unwrap();
        assert_eq!(out2, b"up\ndig\n");
    }

    #[test]
    fn display_preserves_string_literal_case() {
        use crate::dialect::Dialect;
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"Hello, World\".\n           STOP RUN.\n";
        let (out, _rc) = run_program_dialect_with_rc(src, Dialect::DEFAULT).unwrap();
        assert_eq!(out, b"Hello, World\n");
    }

    #[test]
    fn subprogram_ws_persists_cancel_resets_initial_reinits() {
        use crate::dialect::Dialect;
        // A CALLed contained program's WORKING-STORAGE is static (persists across CALLs); CANCEL drops it
        // (next CALL rebuilds from VALUE); an INITIAL program re-initializes every entry.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. M.\n       PROCEDURE DIVISION.\n           CALL \"S\".\n           CALL \"S\".\n           CANCEL \"S\".\n           CALL \"S\".\n           CALL \"I\".\n           CALL \"I\".\n           STOP RUN.\n       END PROGRAM M.\n       IDENTIFICATION DIVISION.\n       PROGRAM-ID. S.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 C PIC 9 VALUE 0.\n       PROCEDURE DIVISION.\n           ADD 1 TO C. DISPLAY \"C=\" C.\n       END PROGRAM S.\n       IDENTIFICATION DIVISION.\n       PROGRAM-ID. I IS INITIAL.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 N PIC 9 VALUE 0.\n       PROCEDURE DIVISION.\n           ADD 1 TO N. DISPLAY \"N=\" N.\n       END PROGRAM I.\n";
        let (out, _rc) = run_program_dialect_with_rc(src, Dialect::DEFAULT).unwrap();
        // persist 1,2 ; CANCEL -> 1 ; INITIAL re-inits 1,1.
        assert_eq!(out, b"C=1\nC=2\nC=1\nN=1\nN=1\n");
    }

    #[test]
    fn decimal_constant_after_cancel_and_recall_is_clean() {
        use crate::dialect::Dialect;
        // Upstream f67da51cae (bugs:#917) + 303917744 (bugs:#923): the C codegen stored decimal
        // constants globally, so a constant used after CANCEL on a subprogram could read stale/
        // freed state (segfault); the fix moves them to per-program local storage, re-initialized
        // on entry, and only the constants a program actually uses are emitted. The candidate has
        // no persistent decimal-constant cache (literals are materialized per execution and CANCEL
        // drops all module state), so the bug class is inapplicable; this court pins the fixed
        // semantics: a large decimal constant used in a called program yields identical results
        // across CALL / CANCEL / re-CALL.
        let src = "       IDENTIFICATION DIVISION.\n\
                   PROGRAM-ID. M.\n\
                   PROCEDURE DIVISION.\n\
                       CALL \"S\".\n\
                       CANCEL \"S\".\n\
                       CALL \"S\".\n\
                       STOP RUN.\n\
                   END PROGRAM M.\n\
                   IDENTIFICATION DIVISION.\n\
                   PROGRAM-ID. S.\n\
                   DATA DIVISION.\n\
                   WORKING-STORAGE SECTION.\n\
                   01 R PIC 9(18).\n\
                   PROCEDURE DIVISION.\n\
                       COMPUTE R = 123456789012345678 / 2.\n\
                       DISPLAY R.\n\
                   END PROGRAM S.\n";
        let (out, rc) = run_program_dialect_with_rc(src, Dialect::DEFAULT).unwrap();
        assert_eq!(rc, 0);
        assert_eq!(out, b"061728394506172839\n061728394506172839\n");
    }

    #[test]
    fn stop_run_in_callee_unwinds_whole_run() {
        use crate::dialect::Dialect;
        // STOP RUN inside a CALLed contained program halts the WHOLE run: the caller's post-CALL
        // statement must not execute. GOBACK returns to the caller (post-CALL statement runs).
        let prog = |sub_term: &str| {
            format!(
                "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. M.\n       PROCEDURE DIVISION.\n           DISPLAY \"A\".\n           CALL \"S\".\n           DISPLAY \"B\".\n           STOP RUN.\n       END PROGRAM M.\n       IDENTIFICATION DIVISION.\n       PROGRAM-ID. S.\n       PROCEDURE DIVISION.\n           DISPLAY \"S\".\n           {sub_term}\n       END PROGRAM S.\n"
            )
        };
        // STOP RUN in the callee -> "B" never prints.
        let (out, _rc) = run_program_dialect_with_rc(&prog("STOP RUN."), Dialect::DEFAULT).unwrap();
        assert_eq!(out, b"A\nS\n");
        // STOP RUN 9 in the callee -> exit code 9 propagates to the run boundary.
        let (out2, rc2) =
            run_program_dialect_with_rc(&prog("STOP RUN 9."), Dialect::DEFAULT).unwrap();
        assert_eq!(out2, b"A\nS\n");
        assert_eq!(rc2, 9);
        // GOBACK in the callee -> returns to caller, "B" prints.
        let (out3, _rc3) = run_program_dialect_with_rc(&prog("GOBACK."), Dialect::DEFAULT).unwrap();
        assert_eq!(out3, b"A\nS\nB\n");
    }

    #[test]
    fn return_code_flows_to_process_exit() {
        use crate::dialect::Dialect;
        let rc = |body: &str| {
            let src = format!(
                "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n{body}"
            );
            run_program_dialect_with_rc(&src, Dialect::DEFAULT)
                .unwrap()
                .1
        };
        // MOVE n TO RETURN-CODE -> process exit code n (oracle: 5->5, 42->42).
        assert_eq!(
            rc("           MOVE 42 TO RETURN-CODE.\n           STOP RUN."),
            42
        );
        assert_eq!(
            rc("           MOVE 5 TO RETURN-CODE.\n           STOP RUN."),
            5
        );
        // default 0.
        assert_eq!(rc("           DISPLAY \"X\".\n           STOP RUN."), 0);
        // STOP RUN n sets the exit code directly (oracle: STOP RUN 7 -> 7).
        assert_eq!(rc("           STOP RUN 7."), 7);
    }

    #[test]
    fn display_upon_printer_redirect_separates_stream() {
        use crate::dialect::Dialect;
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"A\".\n           DISPLAY \"P\" UPON PRINTER.\n           DISPLAY \"B\".\n           STOP RUN.\n";
        // redirect ON (COB_DISPLAY_PRINT_FILE set): UPON PRINTER -> separate stream, stdout omits it.
        let (out, printer, _rc) = run_program_redirected(src, Dialect::DEFAULT, true).unwrap();
        assert_eq!(out, b"A\nB\n");
        assert_eq!(printer, b"P\n");
        // redirect OFF (default): UPON PRINTER interleaves into stdout (oracle default), printer empty.
        let (out2, printer2, _rc2) = run_program_redirected(src, Dialect::DEFAULT, false).unwrap();
        assert_eq!(out2, b"A\nP\nB\n");
        assert!(printer2.is_empty());
    }

    #[test]
    fn fixed_to_free_strips_seqnum_indicator_and_col73() {
        // cols 1-6 sequence (ignored), col 7 '*' = comment (dropped), code in 8-72, 73+ ignored.
        let fixed = "000100 DISPLAY \"OK\".\n000200* a comment\n000300 STOP RUN.";
        assert_eq!(fixed_to_free(fixed), "DISPLAY \"OK\".\n\nSTOP RUN.\n");
        // a 73+-column tail is dropped: build a line whose code fills cols 8..72 then has junk at 73+.
        let mut line = String::from("000400"); // cols 1-6
        line.push(' '); // col 7
        line.push_str(&"X".repeat(65)); // cols 8..=72 (65 chars)
        line.push_str("JUNK73"); // cols 73+
        assert_eq!(fixed_to_free(&line), format!("{}\n", "X".repeat(65)));
        // a short line (<7 chars) is a blank line.
        assert_eq!(fixed_to_free("0001\n"), "\n");
    }

    #[test]
    fn garbage_fails_closed_never_panics() {
        // The fail-closed contract: arbitrary non-program input returns an Err, not a panic.
        for s in [
            "",
            "garbage tokens here",
            "MOVE",
            "01 X PIC",
            "PROCEDURE DIVISION.",
        ] {
            let _ = run_program(s); // must not panic
        }
    }

    #[test]
    fn add_move_display() {
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 WS-A   PIC 9(5) VALUE 100.\n\
                    01 WS-B   PIC 9(5) VALUE 250.\n\
                    01 WS-RES PIC ZZ,ZZ9.\n\
                    PROCEDURE DIVISION.\n\
                        ADD WS-A TO WS-B.\n\
                        MOVE WS-B TO WS-RES.\n\
                        DISPLAY \"TOTAL=\" WS-RES.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"TOTAL=   350\n");
    }

    #[test]
    fn multiply_giving() {
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 P PIC 9(3) VALUE 12.\n\
                    01 Q PIC 9(3) VALUE 4.\n\
                    01 R PIC ZZ9.\n\
                    PROCEDURE DIVISION.\n\
                        MULTIPLY P BY Q GIVING R.\n\
                        DISPLAY \"P=\" R.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"P= 48\n");
    }

    #[test]
    fn compute_precedence_and_div() {
        // COMPUTE with operator precedence + division intermediate precision.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC 9(6) VALUE 22.\n\
                    01 B PIC 9(6) VALUE 7.\n\
                    01 R PIC 9.9(8).\n\
                    PROCEDURE DIVISION.\n\
                        COMPUTE R = A / B.\n\
                        DISPLAY \"PI=\" R.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"PI=3.14285714\n"); // 22/7 truncated to 8 fractional digits
    }

    #[test]
    fn if_else_and_perform() {
        // IF/ELSE branch selection + PERFORM UNTIL loop (factorial) + alphanumeric compare.
        let fac = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 I PIC 9(4) VALUE 1.\n\
                    01 F PIC 9(8) VALUE 1.\n\
                    01 R PIC ZZZZZZZ9.\n\
                    PROCEDURE DIVISION.\n\
                        PERFORM UNTIL I > 5\n\
                            MULTIPLY I BY F GIVING F\n\
                            ADD 1 TO I\n\
                        END-PERFORM.\n\
                        MOVE F TO R.\n\
                        DISPLAY R.\n\
                        STOP RUN.\n");
        assert_eq!(fac, b"     120\n"); // 5! = 120

        let branch = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC 9 VALUE 3.\n\
                    01 R PIC X(3).\n\
                    PROCEDURE DIVISION.\n\
                        IF A > 5 MOVE \"BIG\" TO R ELSE MOVE \"LOW\" TO R END-IF.\n\
                        DISPLAY R.\n\
                        STOP RUN.\n");
        assert_eq!(branch, b"LOW\n");
    }

    #[test]
    fn compute_rounded_rounds_to_receiver_scale() {
        // ROUNDED (default mode: NEAREST, ties away from zero). 10/3 = 3.33 -> 3; 5/2 = 2.5 -> 3 (tie away).
        let out = run_program(
            "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC 9(4) VALUE 10.\n\
                    01 R PIC 9.\n\
                    PROCEDURE DIVISION.\n\
                        COMPUTE R ROUNDED = A / 3.\n\
                        DISPLAY R.\n\
                        COMPUTE R ROUNDED = 5 / 2.\n\
                        DISPLAY R.\n\
                        STOP RUN.\n",
        )
        .unwrap();
        assert_eq!(out, b"3\n3\n");
    }

    #[test]
    fn string_move_and_display() {
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 WS-N PIC X(5).\n\
                    PROCEDURE DIVISION.\n\
                        MOVE \"HI\" TO WS-N.\n\
                        DISPLAY \"[\" WS-N \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(out, b"[HI   ]\n");
    }

    #[test]
    fn unstring_with_pointer_scans_from_cursor_and_writes_back() {
        // Oracle (cobc 3.2.0): P=4 -> scan starts at the 4th byte of "AA,BBB,CC,DDD"; "BBB" then "CC"
        // are split out and the pointer advances past the last delimiter to 11. Matches `lab` differential.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 SRC PIC X(20) VALUE \"AA,BBB,CC,DDD\".\n\
                    01 R1 PIC X(5).\n\
                    01 R2 PIC X(5).\n\
                    01 P  PIC 99 VALUE 4.\n\
                    PROCEDURE DIVISION.\n\
                        UNSTRING SRC DELIMITED BY \",\" INTO R1 R2 WITH POINTER P.\n\
                        DISPLAY \"R1=[\" R1 \"]\".\n\
                        DISPLAY \"R2=[\" R2 \"]\".\n\
                        DISPLAY \"P=\" P.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"R1=[BBB  ]\nR2=[CC   ]\nP=11\n");
    }

    #[test]
    fn unstring_pointer_then_tallying_clauses_coexist() {
        // Oracle (cobc 3.2.0): `WITH POINTER P TALLYING IN TC` -- P starts at 1, two fields filled, so
        // P advances to 8 (past "AA,BBB,") and TC counts the 2 receivers filled. Clause order POINTER<TALLYING.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 SRC PIC X(20) VALUE \"AA,BBB,CC,DDD\".\n\
                    01 R1 PIC X(5).\n\
                    01 R2 PIC X(5).\n\
                    01 P  PIC 99 VALUE 1.\n\
                    01 TC PIC 99 VALUE 0.\n\
                    PROCEDURE DIVISION.\n\
                        UNSTRING SRC DELIMITED BY \",\" INTO R1 R2 WITH POINTER P TALLYING IN TC.\n\
                        DISPLAY \"R1=[\" R1 \"]\".\n\
                        DISPLAY \"R2=[\" R2 \"]\".\n\
                        DISPLAY \"P=\" P \" TC=\" TC.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"R1=[AA   ]\nR2=[BBB  ]\nP=08 TC=02\n");
    }

    #[test]
    fn divide_remainder_on_size_error_divide_by_zero() {
        // Oracle (cobc 3.2.0): a normal divide takes NOT ON SIZE ERROR; a zero divisor takes ON SIZE ERROR
        // and leaves BOTH the quotient and remainder receivers UNCHANGED (17/5 -> Q=003 R=002, then 17/0).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 N PIC 999 VALUE 17.\n\
                    01 D PIC 999 VALUE 5.\n\
                    01 Q PIC 999 VALUE 111.\n\
                    01 R PIC 999 VALUE 222.\n\
                    PROCEDURE DIVISION.\n\
                        DIVIDE N BY D GIVING Q REMAINDER R\n\
                           ON SIZE ERROR DISPLAY \"SE1\"\n\
                           NOT ON SIZE ERROR DISPLAY \"OK1\"\n\
                        END-DIVIDE.\n\
                        DISPLAY \"Q=\" Q \" R=\" R.\n\
                        MOVE 0 TO D.\n\
                        DIVIDE N BY D GIVING Q REMAINDER R\n\
                           ON SIZE ERROR DISPLAY \"SE2\"\n\
                           NOT ON SIZE ERROR DISPLAY \"OK2\"\n\
                        END-DIVIDE.\n\
                        DISPLAY \"Q=\" Q \" R=\" R.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"OK1\nQ=003 R=002\nSE2\nQ=003 R=002\n");
    }

    #[test]
    fn divide_remainder_on_size_error_per_receiver_overflow() {
        // Oracle (cobc 3.2.0): 999/1 into a 1-digit quotient overflows -> ON SIZE ERROR; the quotient is left
        // UNCHANGED (7) but the remainder (0, which fits) IS stored. With no handler the quotient truncate-stores.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 QS PIC 9 VALUE 7.\n\
                    01 RS PIC 9 VALUE 8.\n\
                    PROCEDURE DIVISION.\n\
                        DIVIDE 999 BY 1 GIVING QS REMAINDER RS\n\
                           ON SIZE ERROR DISPLAY \"SE3\"\n\
                           NOT ON SIZE ERROR DISPLAY \"OK3\"\n\
                        END-DIVIDE.\n\
                        DISPLAY \"QS=\" QS \" RS=\" RS.\n\
                        DIVIDE 888 BY 1 GIVING QS REMAINDER RS.\n\
                        DISPLAY \"QS=\" QS \" RS=\" RS.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"SE3\nQS=7 RS=0\nQS=8 RS=0\n");
    }

    #[test]
    fn inspect_figurative_constant_operands() {
        // Oracle (cobc 3.2.0): figuratives are 1-byte operands in INSPECT. "AB" + LOW-VALUES*3 -> TALLYING
        // ALL LOW-VALUE counts 3; REPLACING ALL LOW-VALUE BY "Z" then ALL "Z" BY QUOTE yields `AB"""`.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 G.\n\
                       05 H PIC X(2).\n\
                       05 L PIC X(3).\n\
                    01 C PIC 99 VALUE 0.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE \"AB\" TO H.\n\
                        MOVE LOW-VALUES TO L.\n\
                        INSPECT G TALLYING C FOR ALL LOW-VALUE.\n\
                        DISPLAY \"C=\" C.\n\
                        INSPECT G REPLACING ALL LOW-VALUE BY \"Z\".\n\
                        INSPECT G REPLACING ALL \"Z\" BY QUOTE.\n\
                        DISPLAY \"G=[\" G \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(out, b"C=03\nG=[AB\"\"\"]\n");
    }

    #[test]
    fn value_all_literal_repeats_to_fill() {
        // Upstream 61479ba0c widened the SCREEN SECTION VALUE clause to ALL-literals; the general
        // `VALUE ALL "lit"` surface is a pre-existing cobc feature. Oracle (cobc 3.2.0):
        // PIC X(5) VALUE ALL "-" -> "-----"; PIC X(5) VALUE ALL "ab" -> "ababa" (repeat,
        // truncate). Residual: VALUE ALL into a NUMERIC field compiles with a warning and stores
        // the raw bytes; the candidate initializes those bytes but its numeric-display path
        // normalizes them, so that warning-only corner is a documented residual.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC X(5) VALUE ALL \"-\".\n\
                    01 B PIC X(5) VALUE ALL \"ab\".\n\
                    01 E PIC ZZZZ9.\n\
                    PROCEDURE DIVISION.\n\
                        DISPLAY \"[\" A \"]\".\n\
                        DISPLAY \"[\" B \"]\".\n\
                        MOVE 42 TO E.\n\
                        DISPLAY \"[\" E \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(out, b"[-----]\n[ababa]\n[   42]\n");
    }

    #[test]
    fn prototype_declaration_is_not_the_main_program() {
        // Upstream 14f0d0908 guards prototype-argument checking (ANY LENGTH items only as BY
        // REFERENCE formal parameters; the C segfault on an error node). The candidate's
        // prototype surface: `PROGRAM-ID. name PROTOTYPE.` is a signature-only declaration --
        // never selected as the run's main program (oracle: cobc compiles a prototype-containing
        // source with the FIRST executable program as entry), and CALL to a prototype is a typed
        // boundary.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. SUB PROTOTYPE.\n\
                    DATA DIVISION.\n\
                    LINKAGE SECTION.\n\
                    01 P.\n\
                    PROCEDURE DIVISION USING BY VALUE P.\n\
                    END PROGRAM SUB.\n\
                    IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. M.\n\
                    PROCEDURE DIVISION.\n\
                        DISPLAY \"M\".\n\
                        STOP RUN.\n");
        assert_eq!(
            out, b"M\n",
            "the executable program, not the prototype, runs"
        );
        // CALL to a prototype-only unit fails closed (signature-only).
        let src2 = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. M.\n       PROCEDURE DIVISION.\n           CALL \"SUB\".\n           STOP RUN.\n       END PROGRAM M.\n       IDENTIFICATION DIVISION.\n       PROGRAM-ID. SUB PROTOTYPE.\n       PROCEDURE DIVISION.\n       END PROGRAM SUB.\n";
        set_source_file("prog.cob");
        let err = run_program(src2).unwrap_err();
        set_source_file("");
        assert!(
            format!("{err:?}").contains("PROTOTYPE"),
            "CALL to a prototype fails closed: {err:?}"
        );
    }

    #[test]
    fn move_to_edited_family_matches_current_upstream() {
        // Upstream move-to-edited family (7c7b55b93, 435454f8d, 44c96d20a, 87c1dd579,
        // 921108ea2, ec5562cfb): the optimized numeric->edited path handles sign normalization,
        // BLANK WHEN ZERO on signed edited fields, the insertion symbols B / 0 / /, and the
        // 2023-standard floating-currency semantics (a floating `$`/`+`/`-` before the decimal
        // point suppresses zero only before it; after the point the float char is a literal).
        // The candidate's edited-encode already implements these fixed semantics; this court pins
        // the oracle-verified expectations (cobc 3.2.0 + current upstream agree on all four
        // shapes; the 3.2-vs-current drift is covered by the oracle-drift report).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 E1 PIC -9.99 BLANK WHEN ZERO.\n\
                    01 E2 PIC 9(2)B9(2).\n\
                    01 E3 PIC 9(2)/9(2).\n\
                    01 E4 PIC 9(2)09(2).\n\
                    01 F1 PIC $,$$$.99.\n\
                    01 F2 PIC $$,$$$.99.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE 0 TO E1.\n\
                        DISPLAY \"[\" E1 \"]\".\n\
                        MOVE 1234 TO E2.\n\
                        DISPLAY \"[\" E2 \"]\".\n\
                        MOVE 1234 TO E3.\n\
                        DISPLAY \"[\" E3 \"]\".\n\
                        MOVE 1234 TO E4.\n\
                        DISPLAY \"[\" E4 \"]\".\n\
                        MOVE 0 TO F1.\n\
                        DISPLAY \"[\" F1 \"]\".\n\
                        MOVE 0 TO F2.\n\
                        DISPLAY \"[\" F2 \"]\".\n\
                        MOVE 1234.56 TO F2.\n\
                        DISPLAY \"[\" F2 \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(
            out,
            b"[     ]\n[12 34]\n[12/34]\n[12034]\n[    $.00]\n[     $.00]\n[$1,234.56]\n"
        );
    }

    #[test]
    fn default_rounded_and_screen_dump_boundaries() {
        // e51b091b9 (bug 934): the default ROUNDED fast path -- the candidate's Round model
        // handles rounding uniformly; the C opt-flag fast-path guard is inapplicable. Pin the
        // observable: a ROUNDED arithmetic store rounds NEAREST-AWAY as the default (oracle:
        // COMPUTE R ROUNDED = 12.46 -> 12.5; COMPUTE S = 12.44 -> 12.4).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 R PIC 9(2)V9.\n\
                    01 S PIC 9(2)V9.\n\
                    PROCEDURE DIVISION.\n\
                        COMPUTE R ROUNDED = 12.46.\n\
                        DISPLAY R.\n\
                        COMPUTE S = 12.44.\n\
                        DISPLAY S.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"12.5\n12.4\n");
        // d33f2ec97 (CBL_GC_SCR_DUMP / CBL_GC_SCR_RESTORE): screen dump/restore is a
        // screen-model boundary; an external CALL to them fails closed (not a contained program).
        let src2 = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           CALL \"CBL_GC_SCR_DUMP\".\n           STOP RUN.\n";
        let err = run_program(src2).unwrap_err();
        assert!(
            format!("{err:?}").contains("external CALL"),
            "CBL screen dump call fails closed: {err:?}"
        );
    }

    #[test]
    fn broken_expressions_fail_closed_cleanly() {
        // Upstream a0937bf49 (bugs:#933 #938 #966) hardened the C expression evaluation against
        // broken expressions (NULL expression-stack guards; const-correctness). The candidate is
        // Rust (no null-deref class) and must reject the same broken inputs fail-closed with a
        // typed diagnostic, terminating promptly (no hang, no panic).
        let cases: &[&str] = &[
            // COMPUTE with a dangling operator: the `)` reaches the numeric-literal parse.
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 X PIC 9.\n       PROCEDURE DIVISION.\n           COMPUTE X = (1 + ).\n           STOP RUN.\n",
            // IF with an unterminated condition.
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           IF ( THEN DISPLAY \"A\".\n           STOP RUN.\n",
            // COMPUTE with a trailing operator.
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 X PIC 9.\n       PROCEDURE DIVISION.\n           COMPUTE X = 1 + .\n           STOP RUN.\n",
        ];
        for src in cases {
            let t0 = std::time::Instant::now();
            let err = run_program(src).unwrap_err();
            assert!(
                t0.elapsed().as_millis() < 1000,
                "broken expression must terminate promptly: {err:?}"
            );
            let _ = err;
        }
    }

    #[test]
    fn sd_sort_file_accept_reject_matrix() {
        // Upstream 277a07c2e + 23f850352 "improve SD syntax checks and error recovery" (syn_file.at
        // "SORT files"): SORT/MERGE work files and RETURN targets must be `SD` (sort-description)
        // files -- an FD operand is cobc's "must be an SD filename" compile error; RELEASE is only
        // allowed on a record of the SD file (cobc: "RELEASE not allowed on this record item"); an SD
        // without a SELECT is "not defined"; and a record name in RETURN is "not a file name". The
        // candidate's typed equivalents fail closed in BOTH the executor and the -fsyntax-only checker.
        // First the accepted shape: an in-memory SORT round trip through INPUT/OUTPUT PROCEDURE
        // (RELEASE gathers, RETURN distributes) runs and yields the sorted records:
        let src = "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    ENVIRONMENT DIVISION.\n\
                    INPUT-OUTPUT SECTION.\n\
                    FILE-CONTROL.\n\
                        SELECT SRT ASSIGN TO \"SRT\".\n\
                    DATA DIVISION.\n\
                    FILE SECTION.\n\
                    SD SRT.\n\
                    01 SREC PIC X(1).\n\
                    PROCEDURE DIVISION.\n\
                        SORT SRT ASCENDING KEY SREC\n\
                             INPUT PROCEDURE INP-PARA THRU INP-EXIT\n\
                             OUTPUT PROCEDURE OUTP-PARA THRU OUTP-EXIT.\n\
                        STOP RUN.\n\
                    INP-PARA.\n\
                        MOVE \"c\" TO SREC.\n\
                        RELEASE SREC.\n\
                        MOVE \"a\" TO SREC.\n\
                        RELEASE SREC.\n\
                        MOVE \"b\" TO SREC.\n\
                        RELEASE SREC.\n\
                    INP-EXIT. EXIT.\n\
                    OUTP-PARA.\n\
                        RETURN SRT AT END DISPLAY \"DONE\".\n\
                        DISPLAY SREC.\n\
                        RETURN SRT AT END DISPLAY \"DONE\".\n\
                        DISPLAY SREC.\n\
                        RETURN SRT AT END DISPLAY \"DONE\".\n\
                        DISPLAY SREC.\n\
                        RETURN SRT AT END DISPLAY \"DONE\".\n\
                    OUTP-EXIT. EXIT.\n";
        assert_eq!(run(src), b"a\nb\nc\nDONE\n");
        // The reject matrix -- every case fails closed with the typed diagnostic in BOTH the executor
        // and the checker (cobc's errors are compile-time; the candidate mirrors the accept/reject):
        let prog = |body: &str| {
            format!(
                "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION.\n       INPUT-OUTPUT SECTION.\n       FILE-CONTROL.\n           SELECT FD1 ASSIGN TO \"FD1\" ORGANIZATION LINE SEQUENTIAL.\n           SELECT SD1 ASSIGN TO \"SD1\".\n       DATA DIVISION.\n       FILE SECTION.\n       FD FD1.\n       01 FDREC PIC X(3).\n       SD SD1.\n       01 SDREC PIC X(3).\n       PROCEDURE DIVISION.\n           {body}\n           STOP RUN.\n"
            )
        };
        let d = crate::dialect::Dialect::DEFAULT;
        let rejects: &[(&str, &str)] = &[
            // SORT on an FD file: cobc "must be an SD filename".
            ("SORT FD1 ASCENDING KEY FDREC.", "must be an SD filename"),
            // RETURN on an FD file: cobc "must be an SD filename".
            ("RETURN FD1 AT END CONTINUE.", "must be an SD filename"),
            // RETURN on a record name (not a file): cobc "'SDREC' is not a file name".
            ("RETURN SDREC AT END CONTINUE.", "not a declared file"),
            // RELEASE of an FD record: cobc "RELEASE not allowed on this record item".
            ("RELEASE FDREC.", "record is not a sort-file record"),
        ];
        for (body, needle) in rejects {
            let e = run_program(&prog(body)).unwrap_err();
            assert!(
                format!("{e:?}").contains(needle),
                "run {body} -> {e:?} (want {needle})"
            );
            let e = check_program(&prog(body), d).unwrap_err();
            assert!(
                format!("{e:?}").contains(needle),
                "check {body} -> {e:?} (want {needle})"
            );
        }
        // An SD without a SELECT is not a declared file (cobc: "'name' is not defined").
        let no_select = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       FILE SECTION.\n       SD NOSEL.\n       01 NREC PIC X.\n       PROCEDURE DIVISION.\n           SORT NOSEL ASCENDING KEY NREC.\n           STOP RUN.\n";
        let e = run_program(no_select).unwrap_err();
        assert!(
            format!("{e:?}").contains("not a declared file"),
            "run -> {e:?}"
        );
        let e = check_program(no_select, d).unwrap_err();
        assert!(
            format!("{e:?}").contains("not a declared file"),
            "check -> {e:?}"
        );
    }

    #[test]
    fn incomplete_code_fails_closed_promptly() {
        // Upstream 7b324f50e "parser cleanup and better handling of incomplete code": missing headers,
        // truncated statements, and malformed SD/FD references must not hang or panic -- they terminate
        // promptly with a typed diagnostic. The candidate's fail-fast checker is the Rust equivalent of
        // cobc's bounded parser recovery (whose broken-SD dummy-file fix addresses bugs:#1151).
        let cases: &[&str] = &[
            // A data-only source with no PROCEDURE DIVISION (upstream's relaxed copybook parse).
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 X PIC 9.\n",
            // A truncated SORT statement (no USING/GIVING, no period).
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION.\n       INPUT-OUTPUT SECTION.\n       FILE-CONTROL.\n           SELECT SRT ASSIGN TO \"SRT\".\n       DATA DIVISION.\n       FILE SECTION.\n       SD SRT.\n       01 SREC PIC X.\n       PROCEDURE DIVISION.\n           SORT SRT ASCENDING KEY SREC\n",
            // SORT of a completely undeclared file -- a broken SD reference (bugs:#1151's shape).
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           SORT GHOST ASCENDING KEY K.\n",
            // An SD declaring no record description, then SORTed.
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION.\n       INPUT-OUTPUT SECTION.\n       FILE-CONTROL.\n           SELECT SRT ASSIGN TO \"SRT\".\n       DATA DIVISION.\n       FILE SECTION.\n       SD SRT.\n       PROCEDURE DIVISION.\n           SORT SRT ASCENDING KEY K.\n",
            // MS-DOS EOF (0x1A) / file-separator (0x1C) control codes in the source (upstream pplex.l
            // consumes them as a single newline): the candidate lexes them as an invalid symbol and
            // must reject the statement promptly instead of looping.
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n\x1a\x1a\x1c           DISPLAY \"OK\".\n           STOP RUN.\n",
        ];
        for src in cases {
            let t0 = std::time::Instant::now();
            let _ = run_program(src);
            assert!(
                t0.elapsed().as_millis() < 1000,
                "incomplete code must terminate promptly (run): {src:?}"
            );
            let t0 = std::time::Instant::now();
            let _ = check_program(src, crate::dialect::Dialect::DEFAULT);
            assert!(
                t0.elapsed().as_millis() < 1000,
                "incomplete code must terminate promptly (check): {src:?}"
            );
        }
        // The truncated SORT must fail closed with a typed SORT diagnostic (not a hang).
        let e = run_program(cases[1]).unwrap_err();
        assert!(format!("{e:?}").contains("SORT"), "got: {e:?}");
    }

    #[test]
    fn dialect_grammar_verification_cluster() {
        // Upstream verification cluster:
        // - 0fa2bf5f5 (MF/ACU portability): the grammar now accepts the ENVIRONMENT DIVISION
        //   sections in either order -- the candidate already does (verified below).
        // - 71ea358aa/41e2e4488 (ALPHABET, esp. NATIONAL): the candidate parses ALPHABET
        //   definitions; the national/alphanumeric literal-type check sits inside the NATIONAL
        //   boundary.
        // - 1104bda61 (MOVE/SET incompatible-data check, numeric receivers only): the candidate's
        //   runtime MOVE matches the oracle for alphanumeric->numeric via a variable source
        //   (zero-fill on non-digits, verified); a literal source is compile-time folded by cobc
        //   into the raw initial image -- a codegen fold the candidate does not model (residual).
        // - 5a8666888 ('U' precedence), 9f1a64c32 (string state structs), 7b09c750f (cursor
        //   line-1), 7ba5f9fcb (WINDOW pointer), 3f7c44b6f (stdin): C-internal reorders/refactors
        //   or native-curses boundaries; the candidate's stdin support is already tested.
        let reversed = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    ENVIRONMENT DIVISION.\n\
                    INPUT-OUTPUT SECTION.\n\
                    FILE-CONTROL.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 X PIC 9 VALUE 1.\n\
                    PROCEDURE DIVISION.\n\
                        DISPLAY X.\n\
                        STOP RUN.\n");
        assert_eq!(reversed, b"1\n", "reversed ENVIRONMENT sections accepted");
        let alpha = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    ENVIRONMENT DIVISION.\n\
                    CONFIGURATION SECTION.\n\
                    SPECIAL-NAMES.\n\
                        ALPHABET A IS \"abc\".\n\
                    DATA DIVISION.\n\
                    PROCEDURE DIVISION.\n\
                        DISPLAY \"OK\".\n\
                        STOP RUN.\n");
        assert_eq!(alpha, b"OK\n", "ALPHABET definition parses");
        let mv = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC X(3) VALUE \"abc\".\n\
                    01 N PIC 9(3).\n\
                    PROCEDURE DIVISION.\n\
                        MOVE A TO N.\n\
                        DISPLAY \"[\" N \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(
            mv, b"[000]\n",
            "alphanumeric->numeric via variable zero-fills (oracle-matched)"
        );
    }

    #[test]
    fn epoch_family_matches_fixed_upstream_semantics() {
        // Upstream 486565722 / 946f3e638 / eb8536cfc: the C's mktime-stuffed epoch conversion was
        // off by one day and SOURCE_DATE_EPOCH could be ignored on later invocations; the fix
        // delegates to set_cob_time_from_localtime and re-reads SOURCE_DATE_EPOCH per compile.
        // The candidate's cob_set_date_from_epoch is pure civil-calendar arithmetic (day 0 =
        // 1970-01-01) and compile_tm reads SOURCE_DATE_EPOCH fresh per run -- both bug classes
        // are inapplicable. This court pins the fixed semantics: the epoch decomposition matches
        // the known-good values and FUNCTION MODULE-DATE is stable across runs under a pinned
        // epoch.
        let t0 = crate::common_cmdline::cob_set_date_from_epoch(b"0").unwrap();
        assert_eq!((t0.year, t0.month, t0.day_of_month), (1970, 1, 1));
        let t1 = crate::common_cmdline::cob_set_date_from_epoch(b"1000000000").unwrap();
        assert_eq!((t1.year, t1.month, t1.day_of_month), (2001, 9, 9));
        // SOURCE_DATE_EPOCH is honoured identically on every invocation (fresh read per run).
        std::env::set_var("SOURCE_DATE_EPOCH", "1000000000");
        let prog = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY FUNCTION MODULE-DATE.\n           STOP RUN.\n";
        let a = run(prog);
        let b = run(prog);
        assert_eq!(
            a, b,
            "SOURCE_DATE_EPOCH yields a stable MODULE-DATE across invocations"
        );
        assert_eq!(a, b"20010909\n");
        std::env::remove_var("SOURCE_DATE_EPOCH");
    }

    #[test]
    fn char_ord_honor_program_collating_sequence() {
        // Upstream 5bb0fbe1b: FUNCTION CHAR / ORD must use the program collating sequence
        // (the pre-fix 3.2 behaviour is noted there). Under ALPHABET EB IS EBCDIC + PROGRAM
        // COLLATING SEQUENCE IS EB:
        // ORD("A") = 0xC1 + 1 = 194 and CHAR(194) = "A" (the inverse). The STABLE 3.2 oracle
        // still prints 66 and the raw byte (pre-fix); the candidate follows current upstream
        // (drift recorded). CHAR outside 1..256, or a weight with no source character, raises
        // EC-ARGUMENT-FUNCTION and returns 0.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    ENVIRONMENT DIVISION.\n\
                    CONFIGURATION SECTION.\n\
                    SPECIAL-NAMES. ALPHABET EB IS EBCDIC.\n\
                    OBJECT-COMPUTER. PROGRAM COLLATING SEQUENCE IS EB.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 O PIC 9(5).\n\
                    01 C PIC X.\n\
                    PROCEDURE DIVISION.\n\
                        COMPUTE O = FUNCTION ORD(\"A\").\n\
                        DISPLAY O.\n\
                        MOVE FUNCTION CHAR(194) TO C.\n\
                        DISPLAY \"[\" C \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(out, b"00194\n[A]\n");
        // without a collating sequence the identity path is unchanged (ORD('A') = 66, CHAR(66) = 'A').
        let out2 = run("       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 O PIC 9(5).\n       01 C PIC X.\n       PROCEDURE DIVISION.\n           COMPUTE O = FUNCTION ORD(\"A\").\n           DISPLAY O.\n           MOVE FUNCTION CHAR(66) TO C.\n           DISPLAY \"[\" C \"]\".\n           STOP RUN.\n");
        assert_eq!(out2, b"00066\n[A]\n");
    }

    #[test]
    fn remaining_boundary_surfaces_fail_closed() {
        // Upstream batch of small surfaces; each candidate boundary is verified to fail closed
        // promptly (no hang, no silent mis-execution):
        // - ac862070c (ACCEPT with TIMEOUT): interactive/screen ACCEPT is a runtime non-claim.
        // - c4eea8102 (ENTRY area-B rule): ENTRY is rejected as a statement (the area rule sits
        //   inside that boundary).
        // - 2c092ca14 (>>SET trailing periods): the candidate's preprocessor drops unknown >>
        //   directives without hanging; the SET-directive period rule is inside the preprocessor
        //   boundary.
        // - 8a7c349d1 (>>IMP INCLUDE), 13963e15a / 39ab4808c (listing/-ftcmd), 50b58f682
        //   (COB_LOAD_GLOBAL), 02964e42e (is_test rename): native-code, listing, DSO-registry and
        //   C-ABI surfaces the candidate does not implement (typed boundaries recorded).
        let t0 = std::time::Instant::now();
        let err = run_program(
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 X PIC 9.\n       PROCEDURE DIVISION.\n           ACCEPT X WITH TIMEOUT 5.\n           STOP RUN.\n",
        )
        .unwrap_err();
        assert!(
            t0.elapsed().as_millis() < 1000,
            "ACCEPT TIMEOUT terminates promptly"
        );
        assert!(format!("{err:?}").contains("non-claim"), "{err:?}");
        let err2 = run_program(
            "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           ENTRY \"ALT\".\n           STOP RUN.\n",
        )
        .unwrap_err();
        assert!(format!("{err2:?}").contains("ENTRY"), "{err2:?}");
        // a >>SET directive is dropped without hanging (the run proceeds).
        let out = run("       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n       >>SET EC-ALL-CHECKING\n           DISPLAY \"OK\".\n           STOP RUN.\n");
        assert_eq!(out, b"OK\n");
    }

    #[test]
    fn high_value_comparison_under_collating_sequence() {
        // Upstream 79c65d0ec (bugs:#948): comparisons with HIGH-VALUE / LOW-VALUE must use the
        // program collating sequence's per-program low/high values, not the hard-coded 0x00/0xFF.
        // Under ALPHABET EB IS EBCDIC + PROGRAM COLLATING SEQUENCE IS EB, X = HIGH-VALUE compares
        // the EBCDIC weights; the candidate resolves the figuratives to raw bytes and compares by
        // weight, which is equivalent (verified against cobc 3.2.0: both print EQ).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    ENVIRONMENT DIVISION.\n\
                    CONFIGURATION SECTION.\n\
                    SPECIAL-NAMES. ALPHABET EB IS EBCDIC.\n\
                    OBJECT-COMPUTER. PROGRAM COLLATING SEQUENCE IS EB.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 X PIC X.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE HIGH-VALUE TO X.\n\
                        IF X = HIGH-VALUE DISPLAY \"EQ\" ELSE DISPLAY \"NE\" END-IF.\n\
                        IF X > LOW-VALUE DISPLAY \"GT\" ELSE DISPLAY \"LE\" END-IF.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"EQ\nGT\n");
        // 9e0d66418 (EXTFH INDEXED warning): the candidate never emits the compiler's
        // ORGANIZATION INDEXED warning (it has no --without-db build warning surface), so there is
        // nothing to suppress; an EXTFH INDEXED file parses and runs under the candidate's
        // indexed model.
        // 87500ead4 (nested elements 'with attributes'): SCREEN SECTION items are a screen-model
        // boundary; the nested-qualification fix sits inside it (recorded).
    }

    #[test]
    fn close_with_lock_state_machine_matches_current_upstream() {
        // Upstream 62b39805c (bugs:#914) + 0b22d4417: CLOSE WITH LOCK puts the file in the LOCKED
        // state; re-OPEN reports 38 (CLOSED WITH LOCK); READ on a non-INPUT/I-O file reports 47
        // (INPUT DENIED, cob_read_next); a second CLOSE of a locked file reports 42 (NOT OPEN).
        // Differential evidence: the STABLE 3.2 oracle reports 00/38/47/00 (pre-fix: the backend
        // close on a locked file ran, reporting 00); the candidate follows current-upstream
        // semantics 00/38/47/42.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    ENVIRONMENT DIVISION.\n\
                    INPUT-OUTPUT SECTION.\n\
                    FILE-CONTROL.\n\
                        SELECT F ASSIGN TO \"tf2\" ORGANIZATION IS SEQUENTIAL\n\
                               FILE STATUS IS WS.\n\
                    DATA DIVISION.\n\
                    FILE SECTION.\n\
                    FD F. 01 R PIC X(4).\n\
                    WORKING-STORAGE SECTION.\n\
                    01 WS PIC XX.\n\
                    PROCEDURE DIVISION.\n\
                        OPEN OUTPUT F.\n\
                        MOVE 'abcd' TO R.\n\
                        WRITE R.\n\
                        CLOSE F WITH LOCK.\n\
                        DISPLAY 'S1=' WS.\n\
                        OPEN INPUT F.\n\
                        DISPLAY 'S2=' WS.\n\
                        READ F.\n\
                        DISPLAY 'S3=' WS.\n\
                        CLOSE F.\n\
                        DISPLAY 'S4=' WS.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"S1=00\nS2=38\nS3=47\nS4=42\n");
    }

    #[test]
    fn inspect_operand_validation_matches_upstream_validate_inspect() {
        // Upstream 04614ac7a (validate_inspect): non-const REPLACING/CONVERTING operand pairs must
        // be equal in size (compile error otherwise); identical field operands are accepted (warning
        // upstream, no-op); a figurative TO operand skips the comparison. Verified against cobc 3.2:
        // 'A' BY 'BB' and SPACES BY 'XY' are compile errors; 'A' BY 'A' and F BY F compile and no-op.
        let prog = |ins: &str| {
            format!(
                "       IDENTIFICATION DIVISION.\n\
                 PROGRAM-ID. T.\n\
                 DATA DIVISION.\n\
                 WORKING-STORAGE SECTION.\n\
                 01 X PIC X(5) VALUE 'ABABA'.\n\
                 01 F PIC X(1) VALUE 'A'.\n\
                 PROCEDURE DIVISION.\n\
                     {ins}\n\
                     STOP RUN.\n"
            )
        };
        let d = crate::dialect::Dialect::DEFAULT;
        // accepted: equal sizes, identical operands, figurative TO
        check_program(&prog("INSPECT X REPLACING ALL 'A' BY 'B'."), d).expect("equal sizes ok");
        check_program(&prog("INSPECT X REPLACING ALL F BY F."), d).expect("identical operands ok");
        check_program(&prog("INSPECT X CONVERTING F TO F."), d).expect("identical converting ok");
        check_program(&prog("INSPECT X REPLACING ALL 'A' BY SPACES."), d)
            .expect("figurative TO ok");
        check_program(&prog("INSPECT X CONVERTING 'ab' TO 'AB'."), d).expect("equal converting ok");
        // rejected: non-const size mismatch (oracle: compile error)
        let e = check_program(&prog("INSPECT X REPLACING ALL 'A' BY 'BB'."), d).unwrap_err();
        assert!(
            format!("{e:?}").contains("REPLACING operands incompatible"),
            "got: {e:?}"
        );
        let e = check_program(&prog("INSPECT X REPLACING ALL SPACES BY 'XY'."), d).unwrap_err();
        assert!(
            format!("{e:?}").contains("REPLACING operands incompatible"),
            "got: {e:?}"
        );
        let e = check_program(&prog("INSPECT X CONVERTING 'ab' TO 'A'."), d).unwrap_err();
        assert!(
            format!("{e:?}").contains("CONVERTING operands incompatible"),
            "got: {e:?}"
        );
    }

    #[test]
    fn initialize_all_to_value_restores_value_clauses() {
        // Oracle (cobc 3.2.0): `INITIALIZE G ALL TO VALUE` sets each leaf WITH a VALUE clause to that VALUE
        // (A->"abc", B->42) and leaves no-VALUE leaves UNCHANGED (N, M stay "ZZ"); WITH FILLER is identical
        // here since the no-VALUE items have nothing to apply.
        let prog = |verb: &str| {
            format!(
                "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 G.\n\
                       05 A PIC X(3) VALUE \"abc\".\n\
                       05 N PIC X(2).\n\
                       05 B PIC 99 VALUE 42.\n\
                       05 M PIC 99.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE \"ZZZZZZZZZ\" TO G.\n\
                        {verb}.\n\
                        DISPLAY \"G=[\" G \"]\".\n\
                        STOP RUN.\n"
            )
        };
        assert_eq!(run(&prog("INITIALIZE G ALL TO VALUE")), b"G=[abcZZ42ZZ]\n");
        assert_eq!(
            run(&prog("INITIALIZE G WITH FILLER ALL TO VALUE")),
            b"G=[abcZZ42ZZ]\n"
        );
    }

    #[test]
    fn unstring_on_overflow_handler() {
        // Oracle (cobc 3.2.0): 5 comma-segments into 2 receivers leaves source chars unexamined -> ON
        // OVERFLOW (OVF1); a fully-consumed source ("X,Y") takes NOT ON OVERFLOW (OK2).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 SRC PIC X(12) VALUE \"A,B,C,D,E\".\n\
                    01 R1 PIC X(3).\n\
                    01 R2 PIC X(3).\n\
                    PROCEDURE DIVISION.\n\
                        UNSTRING SRC DELIMITED BY \",\" INTO R1 R2\n\
                           ON OVERFLOW DISPLAY \"OVF1\"\n\
                           NOT ON OVERFLOW DISPLAY \"OK1\"\n\
                        END-UNSTRING.\n\
                        DISPLAY \"R1=[\" R1 \"] R2=[\" R2 \"]\".\n\
                        MOVE \"X,Y\" TO SRC.\n\
                        UNSTRING SRC DELIMITED BY \",\" INTO R1 R2\n\
                           ON OVERFLOW DISPLAY \"OVF2\"\n\
                           NOT ON OVERFLOW DISPLAY \"OK2\"\n\
                        END-UNSTRING.\n\
                        DISPLAY \"R1=[\" R1 \"] R2=[\" R2 \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(out, b"OVF1\nR1=[A  ] R2=[B  ]\nOK2\nR1=[X  ] R2=[Y  ]\n");
    }

    #[test]
    fn unstring_multi_delimiter_or_and_all() {
        // Oracle (cobc 3.2.0): DELIMITED BY "," OR ";" splits on the earliest of either (DELIMITER IN
        // captures which); DELIMITED BY ALL "," collapses consecutive commas into one delimiter.
        let out = run(
            "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 SRC PIC X(12) VALUE \"A,B;C,D\".\n\
                    01 R1 PIC X(3). 01 R2 PIC X(3). 01 R3 PIC X(3). 01 R4 PIC X(3).\n\
                    01 D1 PIC X. 01 D2 PIC X.\n\
                    01 S2 PIC X(8) VALUE \"A,,,B\".\n\
                    01 Q1 PIC X(3). 01 Q2 PIC X(3).\n\
                    PROCEDURE DIVISION.\n\
                        UNSTRING SRC DELIMITED BY \",\" OR \";\" INTO R1 DELIMITER IN D1 R2 DELIMITER IN D2 R3 R4.\n\
                        DISPLAY \"A=[\" R1 \"][\" D1 \"][\" R2 \"][\" D2 \"][\" R3 \"][\" R4 \"]\".\n\
                        UNSTRING S2 DELIMITED BY ALL \",\" INTO Q1 Q2.\n\
                        DISPLAY \"B=[\" Q1 \"][\" Q2 \"]\".\n\
                        STOP RUN.\n",
        );
        assert_eq!(out, b"A=[A  ][,][B  ][;][C  ][D  ]\nB=[A  ][B  ]\n");
    }

    #[test]
    fn perform_bare_inline_runs_body_once() {
        // Oracle (cobc 3.2.0): a bare inline `PERFORM <body> END-PERFORM` (no TIMES/UNTIL/VARYING) runs the
        // body exactly once -> N incremented to 1.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 N PIC 9 VALUE 0.\n\
                    PROCEDURE DIVISION.\n\
                        PERFORM\n\
                           ADD 1 TO N\n\
                           DISPLAY \"IN \" N\n\
                        END-PERFORM.\n\
                        DISPLAY \"OUT \" N.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"IN 1\nOUT 1\n");
    }

    #[test]
    fn exhibit_changed_runs_as_plain_exhibit() {
        // Oracle (cobc 3.2.0): CHANGED suppression is unimplemented (-Wpending) -> EXHIBIT CHANGED runs as
        // plain EXHIBIT. Item format is `NAME = value` for plain/NAMED; only CHANGED-without-NAMED is value-only.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC 9 VALUE 3.\n\
                    01 B PIC X(2) VALUE \"hi\".\n\
                    PROCEDURE DIVISION.\n\
                        EXHIBIT CHANGED A B.\n\
                        EXHIBIT CHANGED NAMED A B.\n\
                        EXHIBIT A.\n\
                        EXHIBIT NAMED B.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"3 hi\nA = 3 B = hi\nA = 3\nB = hi\n");
    }

    #[test]
    fn move_alphanumeric_literal_to_binary_and_packed() {
        // Oracle (cobc 3.2.0): MOVE of an alphanumeric literal into COMP/COMP-3/COMP-5 receivers goes
        // through the move.c indirect display path -> the digit value is stored (previously yielded 0).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 C1 PIC 99 COMP.\n\
                    01 P2 PIC 9(3)V9 COMP-3.\n\
                    01 B5 PIC 9(4) COMP-5.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE \"12\" TO C1.\n\
                        MOVE \"1234\" TO P2.\n\
                        MOVE \"99\" TO B5.\n\
                        DISPLAY \"C1=\" C1 \" P2=\" P2 \" B5=\" B5.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"C1=12 P2=234.0 B5=00099\n");
    }

    #[test]
    fn unstring_into_edited_and_scaled_receivers() {
        // Oracle (cobc 3.2.0): UNSTRING delimited substrings into a numeric-edited (ZZ9) and a scaled
        // DISPLAY (9V9) receiver -> "12"->" 12", "34"->4.0, "56" raw into XX.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 SRC PIC X(8) VALUE \"12,34,56\".\n\
                    01 E1 PIC ZZ9.\n\
                    01 N1 PIC 9V9.\n\
                    01 A1 PIC XX.\n\
                    PROCEDURE DIVISION.\n\
                        UNSTRING SRC DELIMITED BY \",\" INTO E1 N1 A1.\n\
                        DISPLAY \"E1=[\" E1 \"] N1=\" N1 \" A1=[\" A1 \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(out, b"E1=[ 12] N1=4.0 A1=[56]\n");
    }

    #[test]
    fn move_add_subtract_corresponding() {
        // Oracle (cobc 3.2.0): CORRESPONDING matches like-named elementary leaves between two groups.
        // MOVE: A2<-11, B2<-"abc", D2 untouched (no D in G1). ADD: A2 11+11=22. SUBTRACT: A2 22-11=11.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 G1.\n\
                       05 A PIC 99 VALUE 11.\n\
                       05 B PIC X(3) VALUE \"abc\".\n\
                       05 C PIC 99 VALUE 5.\n\
                    01 G2.\n\
                       05 A PIC 99 VALUE 99.\n\
                       05 B PIC X(3) VALUE \"zzz\".\n\
                       05 D PIC 99 VALUE 7.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE CORRESPONDING G1 TO G2.\n\
                        DISPLAY \"MOVE A2=\" A OF G2 \" B2=\" B OF G2 \" D2=\" D OF G2.\n\
                        ADD CORRESPONDING G1 TO G2.\n\
                        DISPLAY \"ADD A2=\" A OF G2.\n\
                        SUBTRACT CORRESPONDING G1 FROM G2.\n\
                        DISPLAY \"SUB A2=\" A OF G2.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"MOVE A2=11 B2=abc D2=07\nADD A2=22\nSUB A2=11\n");
    }

    #[test]
    fn qualified_names_disambiguate_duplicate_children() {
        // Oracle (cobc 3.2.0): `AMT OF REC-IN` vs `AMT OF REC-OUT` resolve to distinct fields despite the
        // shared child names; reads and writes each address the right one.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 REC-IN.\n\
                       05 AMT PIC 999 VALUE 123.\n\
                       05 NM PIC X(2) VALUE \"in\".\n\
                    01 REC-OUT.\n\
                       05 AMT PIC 999 VALUE 456.\n\
                       05 NM PIC X(2) VALUE \"ot\".\n\
                    PROCEDURE DIVISION.\n\
                        DISPLAY \"IN=\" AMT OF REC-IN \" OUT=\" AMT OF REC-OUT.\n\
                        MOVE AMT OF REC-IN TO AMT OF REC-OUT.\n\
                        MOVE \"QQ\" TO NM IN REC-OUT.\n\
                        DISPLAY \"OUT-AMT=\" AMT OF REC-OUT \" OUT-NM=\" NM OF REC-OUT.\n\
                        DISPLAY \"IN-NM=\" NM OF REC-IN.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"IN=123 OUT=456\nOUT-AMT=123 OUT-NM=QQ\nIN-NM=in\n");
    }

    #[test]
    fn two_dimensional_table_row_major() {
        // Oracle (cobc 3.2.0): outer group-OCCURS ROW + inner elementary-OCCURS CEL -> CEL(i,j) row-major;
        // the whole 01 image is the interleaved buffer.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 T1.\n\
                       05 ROW OCCURS 2.\n\
                          10 CEL PIC 99 OCCURS 3.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE 11 TO CEL(1,1). MOVE 12 TO CEL(1,2). MOVE 13 TO CEL(1,3).\n\
                        MOVE 21 TO CEL(2,1). MOVE 22 TO CEL(2,2). MOVE 23 TO CEL(2,3).\n\
                        DISPLAY \"CEL22=\" CEL(2,2) \" CEL13=\" CEL(1,3).\n\
                        DISPLAY \"T1=[\" T1 \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(out, b"CEL22=22 CEL13=13\nT1=[111213212223]\n");
    }

    #[test]
    fn two_dimensional_matrix_fill_and_sum() {
        // Oracle (cobc 3.2.0): a row-table with a scalar TAG + an inner OCCURS N(i,j); filled by nested
        // PERFORM VARYING + COMPUTE into the 2-D receiver, summed via ADD over both subscripts.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 GRID.\n\
                       05 R OCCURS 3.\n\
                          10 TAG PIC X.\n\
                          10 N PIC 99 OCCURS 2.\n\
                    01 I PIC 9.\n\
                    01 J PIC 9.\n\
                    01 S PIC 999 VALUE 0.\n\
                    PROCEDURE DIVISION.\n\
                        PERFORM VARYING I FROM 1 BY 1 UNTIL I > 3\n\
                           MOVE \"R\" TO TAG(I)\n\
                           PERFORM VARYING J FROM 1 BY 1 UNTIL J > 2\n\
                              COMPUTE N(I, J) = I * 10 + J\n\
                           END-PERFORM\n\
                        END-PERFORM.\n\
                        PERFORM VARYING I FROM 1 BY 1 UNTIL I > 3\n\
                           PERFORM VARYING J FROM 1 BY 1 UNTIL J > 2\n\
                              ADD N(I, J) TO S\n\
                           END-PERFORM\n\
                        END-PERFORM.\n\
                        DISPLAY \"GRID=[\" GRID \"]\".\n\
                        DISPLAY \"N32=\" N(3,2) \" TAG2=\" TAG(2) \" SUM=\" S.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"GRID=[R1112R2122R3132]\nN32=32 TAG2=R SUM=129\n");
    }

    #[test]
    fn group_of_group_table_and_initialize() {
        // Oracle (cobc 3.2.0): a group-OCCURS over a sub-group -> leaves A(i)/B(i) (one subscript reaches a
        // deeper leaf); INITIALIZE zeros the numeric leaf and spaces the alphanumeric.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 GOG.\n\
                       05 GRP OCCURS 2.\n\
                          10 SUB.\n\
                             15 A PIC X(2).\n\
                             15 B PIC 9.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE \"ab\" TO A(1). MOVE 1 TO B(1).\n\
                        MOVE \"cd\" TO A(2). MOVE 2 TO B(2).\n\
                        DISPLAY \"GOG=[\" GOG \"] A2=\" A(2) \" B1=\" B(1).\n\
                        INITIALIZE GOG.\n\
                        DISPLAY \"AFTER=[\" GOG \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(out, b"GOG=[ab1cd2] A2=cd B1=1\nAFTER=[  0  0]\n");
    }

    #[test]
    fn three_dimensional_table() {
        // Oracle (cobc 3.2.0): C(i,j,k) over PL OCCURS 2 / RW OCCURS 2 / C OCCURS 2 (strides 8/4/2).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 CUBE.\n\
                       05 PL OCCURS 2.\n\
                          10 RW OCCURS 2.\n\
                             15 C PIC 99 OCCURS 2.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE 11 TO C(1,1,1). MOVE 88 TO C(2,2,2). MOVE 55 TO C(2,1,2).\n\
                        DISPLAY \"CUBE=[\" CUBE \"] C222=\" C(2,2,2) \" C212=\" C(2,1,2).\n\
                        STOP RUN.\n");
        assert_eq!(out, b"CUBE=[1100000000550088] C222=88 C212=55\n");
    }

    #[test]
    fn occurs_depending_on_group() {
        // Oracle (cobc 3.2.0): OCCURS DEPENDING ON on a group -> the live image (and LENGTH) is counter*elem
        // (built at MAX), while subscripted access still reaches the physical MAX storage.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 CNT PIC 9 VALUE 2.\n\
                    01 TBL.\n\
                       05 ENT OCCURS 1 TO 4 DEPENDING ON CNT.\n\
                          10 K PIC 99.\n\
                          10 V PIC X.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE 11 TO K(1). MOVE \"a\" TO V(1).\n\
                        MOVE 22 TO K(2). MOVE \"b\" TO V(2).\n\
                        DISPLAY \"TBL=[\" TBL \"] LEN=\" FUNCTION LENGTH(TBL).\n\
                        DISPLAY \"K2=\" K(2) \" V1=\" V(1).\n\
                        STOP RUN.\n");
        assert_eq!(out, b"TBL=[11a22b] LEN=000000006\nK2=22 V1=a\n");
    }

    #[test]
    fn redefines_over_group_occurs_read_and_write() {
        // Oracle (cobc 3.2.0): a REDEFINES alias over a group-OCCURS interleaved buffer reads it AND writes
        // through it (MOVE "999999" TO R -> E(1..3) read back 99).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 G.\n\
                       05 E PIC 99 OCCURS 3.\n\
                    01 R REDEFINES G PIC X(6).\n\
                    PROCEDURE DIVISION.\n\
                        MOVE 12 TO E(1). MOVE 34 TO E(2). MOVE 56 TO E(3).\n\
                        DISPLAY \"R=[\" R \"]\".\n\
                        MOVE \"999999\" TO R.\n\
                        DISPLAY \"E1=\" E(1) \" E2=\" E(2) \" E3=\" E(3).\n\
                        STOP RUN.\n");
        assert_eq!(out, b"R=[123456]\nE1=99 E2=99 E3=99\n");
    }

    #[test]
    fn multiple_different_sized_fillers_keep_group_offsets() {
        // Oracle (cobc 3.2.0): a group MOVE into a record with several DIFFERENT-sized FILLERs lands each
        // named child at the right offset (date splits 2021/09/15). Regression for FILLER key collision.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 WS-IN PIC X(19) VALUE '2021 09 15'.\n\
                    01 WS-DATE.\n\
                       05 YYYY PIC 9(04) VALUE ZERO.\n\
                       05 FILLER PIC X(01) VALUE SPACES.\n\
                       05 MM PIC 9(02) VALUE ZERO.\n\
                       05 FILLER PIC X(01) VALUE SPACES.\n\
                       05 DD PIC 9(02) VALUE ZERO.\n\
                       05 FILLER PIC X(09) VALUE SPACES.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE WS-IN TO WS-DATE.\n\
                        DISPLAY YYYY \"/\" MM \"/\" DD.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"2021/09/15\n");
    }

    #[test]
    fn leading_dot_decimal_literal_and_qualified_compute_operand() {
        // Oracle (cobc 3.2.0): `.08` is the numeric literal 0.08 (the leading `.` is not a terminator), and a
        // qualified name resolves both as a COMPUTE target and inside a parenthesised operand.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 SR. 05 SA PIC 9(4)V99 VALUE 200.00.\n\
                    01 DL. 05 SA PIC 9(4)V99 VALUE ZERO.\n\
                    PROCEDURE DIVISION.\n\
                        COMPUTE SA OF DL = (SA OF SR * .08).\n\
                        DISPLAY \"AMT=\" SA OF SR \" TAX=\" SA OF DL.\n\
                        STOP RUN.\n");
        assert_eq!(out, b"AMT=0200.00 TAX=0016.00\n");
    }

    #[test]
    fn compute_slash_continuation_line_is_division_not_comment() {
        // A continuation line beginning (deeply indented) with `/` is the DIVISION operator, not a
        // fixed-format page-eject comment -- the divide must survive (regression for the column-7 indicator
        // bound). (A * 100) / (B - 1) = 1202454651 / 11674329.16 ~= 103.00.
        let out = run(
            "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC S9(14)V99 VALUE 12024546.51.\n\
                    01 B PIC S9(14)V99 VALUE 11674330.16.\n\
                    01 R PIC S9(3)V99 VALUE ZERO COMP-3.\n\
                    PROCEDURE DIVISION.\n\
                        COMPUTE R ROUNDED = (A * 100)\n            / (B - 1.0)\n            END-COMPUTE.\n\
                        DISPLAY \"R=\" R.\n\
                        STOP RUN.\n",
        );
        // (the `/` and END-COMPUTE carry explicit leading spaces so Rust's `\`-continuation, which strips
        // leading whitespace, leaves the `/` indented past column 7 -- a division operator, not a comment.)
        assert_eq!(out, b"R=+103.00\n");
    }

    #[test]
    fn move_corresponding_skips_filler() {
        // Oracle (cobc 3.2.0): MOVE CORR moves like-NAMED leaves only; FILLER never corresponds, so the
        // target's `-` separators survive (a blank source FILLER must NOT overwrite them).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 S.\n\
                       05 YYYY PIC 9(4) VALUE 2021.\n\
                       05 FILLER PIC X VALUE SPACE.\n\
                       05 MM PIC 9(2) VALUE 09.\n\
                    01 D.\n\
                       05 YYYY PIC 9(4) VALUE ZERO.\n\
                       05 FILLER PIC X VALUE '-'.\n\
                       05 MM PIC 9(2) VALUE ZERO.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE CORRESPONDING S TO D.\n\
                        DISPLAY \"D=[\" D \"]\".\n\
                        STOP RUN.\n");
        assert_eq!(out, b"D=[2021-09]\n");
    }

    #[test]
    fn group_occurs_redefines_value_group_read_and_write() {
        // Oracle (cobc 3.2.0): a group-OCCURS table that REDEFINES a VALUE-bearing group reads the entries
        // through the redefinition (902/903), and a write through the table lands in the shared storage (E5).
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 G.\n\
                       05 D.\n\
                          10 E1 PIC X(3) VALUE '901'.\n\
                          10 E2 PIC X(3) VALUE '902'.\n\
                          10 E3 PIC X(3) VALUE '903'.\n\
                    01 R REDEFINES G.\n\
                       05 ENT OCCURS 3.\n\
                          10 V PIC X(3).\n\
                    PROCEDURE DIVISION.\n\
                        DISPLAY \"v2=\" V(2) \" v3=\" V(3).\n\
                        MOVE 'ZZZ' TO V(3).\n\
                        DISPLAY \"E3=\" E3 \" v3=\" V(3).\n\
                        STOP RUN.\n");
        assert_eq!(out, b"v2=902 v3=903\nE3=ZZZ v3=ZZZ\n");
    }

    #[test]
    fn json_generate_name_and_suppress() {
        // Oracle (cobc 3.2.0): NAME renames JSON keys (incl. the outer via the source name); SUPPRESS omits
        // fields. (Field names avoid the reserved word ID.)
        let out = run(
            "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 REC.\n\
                       05 ANUM PIC 99 VALUE 7.\n\
                       05 ANM PIC X(3) VALUE \"abc\".\n\
                       05 SECRET PIC 99 VALUE 42.\n\
                    01 OUT PIC X(120).\n\
                    PROCEDURE DIVISION.\n\
                        JSON GENERATE OUT FROM REC NAME ANUM IS \"id\" ANM IS \"name\".\n\
                        DISPLAY \"N=[\" FUNCTION TRIM(OUT) \"]\".\n\
                        MOVE SPACES TO OUT.\n\
                        JSON GENERATE OUT FROM REC NAME REC IS \"rec\" ANUM IS \"id\" SUPPRESS SECRET ANM.\n\
                        DISPLAY \"C=[\" FUNCTION TRIM(OUT) \"]\".\n\
                        STOP RUN.\n",
        );
        assert_eq!(
            out,
            b"N=[{\"REC\":{\"id\":7,\"name\":\"abc\",\"SECRET\":42}}]\nC=[{\"rec\":{\"id\":7}}]\n"
        );
    }

    #[test]
    fn initialize_to_value_category_and_table() {
        // Oracle (cobc 3.2.0): `category TO VALUE` ignores the category (every valued leaf restored, like
        // ALL TO VALUE); `ALL TO VALUE` over an OCCURS table restores each element to its VALUE.
        let out = run("       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 G.\n\
                       05 A PIC X(3) VALUE \"abc\".\n\
                       05 B PIC 99 VALUE 42.\n\
                       05 C PIC X(2) VALUE \"yz\".\n\
                    01 TB.\n\
                       05 E PIC 99 VALUE 7 OCCURS 3.\n\
                    PROCEDURE DIVISION.\n\
                        MOVE \"ZZZZZZ\" TO G.\n\
                        INITIALIZE G NUMERIC TO VALUE.\n\
                        DISPLAY \"NUM=[\" G \"]\".\n\
                        MOVE 1 TO E(1). MOVE 2 TO E(2). MOVE 3 TO E(3).\n\
                        INITIALIZE TB ALL TO VALUE.\n\
                        DISPLAY \"TB=\" E(1) E(2) E(3).\n\
                        STOP RUN.\n");
        assert_eq!(out, b"NUM=[abc42yz]\nTB=070707\n");
    }
    // ---- multiple 01-level records beneath one FD (GNURUST.FILEIO.MULTI-RECORD-FD.1) ----
    // The front-end models FD record descriptions as ALTERNATIVE views of ONE shared record area
    // (GnuCOBOL union semantics, verified against the pinned oracle): a MOVE into one record is visible
    // through every other, and WRITE of the NAMED record emits the shared bytes at that record's length.

    #[test]
    fn fd_two_records_both_writeable_in_source_order() {
        // Two alternative 01 records under one FD; WRITE of either emits its own bytes, in source order.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO MRFILE ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD F.\n       01 FIRST-REC PIC X(5).\n       01 SECOND-REC PIC X(5).\n       WORKING-STORAGE SECTION. 01 E PIC X VALUE \"N\".\n       PROCEDURE DIVISION.\n           OPEN OUTPUT F.\n           MOVE \"FIRST\" TO FIRST-REC.\n           WRITE FIRST-REC.\n           MOVE \"OTHER\" TO SECOND-REC.\n           WRITE SECOND-REC.\n           CLOSE F.\n           OPEN INPUT F.\n           PERFORM UNTIL E = \"Y\"\n              READ F AT END MOVE \"Y\" TO E NOT AT END DISPLAY FIRST-REC END-READ\n           END-PERFORM.\n           CLOSE F.\n           STOP RUN.\n";
        assert_eq!(run(src), b"FIRST\nOTHER\n");
    }

    #[test]
    fn fd_records_share_one_record_area() {
        // The FD record descriptions are alternative views of ONE record area: a MOVE into one record is
        // visible through every other, and WRITE of a record emits the shared bytes (oracle-verified:
        // MOVE "11111" TO A then WRITE B writes "11111").
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO MRFILE ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD F.\n       01 FIRST-REC PIC X(5).\n       01 SECOND-REC PIC X(5).\n       WORKING-STORAGE SECTION. 01 E PIC X VALUE \"N\".\n       PROCEDURE DIVISION.\n           OPEN OUTPUT F.\n           MOVE \"11111\" TO FIRST-REC.\n           WRITE SECOND-REC.\n           MOVE \"22222\" TO SECOND-REC.\n           WRITE FIRST-REC.\n           CLOSE F.\n           OPEN INPUT F.\n           PERFORM UNTIL E = \"Y\"\n              READ F AT END MOVE \"Y\" TO E NOT AT END DISPLAY FIRST-REC END-READ\n           END-PERFORM.\n           CLOSE F.\n           STOP RUN.\n";
        assert_eq!(run(src), b"11111\n22222\n");
    }

    #[test]
    fn fd_three_records_preserve_source_order_and_lengths() {
        // Three records with DIFFERENT lengths: each WRITE emits the NAMED record's own length, in source
        // order; a read of a shorter view over a longer record shows the shared area's first bytes.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO MRFILE ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD F.\n       01 S PIC X(3).\n       01 L PIC X(6).\n       01 M PIC X(4).\n       WORKING-STORAGE SECTION. 01 E PIC X VALUE \"N\".\n       PROCEDURE DIVISION.\n           OPEN OUTPUT F.\n           MOVE \"ABC\" TO S.\n           WRITE S.\n           MOVE \"123456\" TO L.\n           WRITE L.\n           MOVE \"WXYZ\" TO M.\n           WRITE M.\n           CLOSE F.\n           OPEN INPUT F.\n           PERFORM UNTIL E = \"Y\"\n              READ F AT END MOVE \"Y\" TO E NOT AT END DISPLAY \"[\" S \"]\" END-READ\n           END-PERFORM.\n           CLOSE F.\n           STOP RUN.\n";
        // read1 = "ABC" (S) -> "[ABC]"; read2 = "123456" (L) -> S shows "123"; read3 = "WXYZ" (M) -> S shows "WXY"
        assert_eq!(run(src), b"[ABC]\n[123]\n[WXY]\n");
    }

    #[test]
    fn fd_group_records_share_area_and_write_independently() {
        // Group records beneath one FD: MOVE into one group is visible through the other (shared area),
        // and WRITE of each emits the NAMED record's own layout length.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO MRFILE ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD F.\n       01 DETAIL-REC.\n          05 ITEM-CODE PIC 9(3).\n          05 ITEM-TEXT PIC X(7).\n       01 TOTAL-REC.\n          05 ITEM-LABEL PIC X(6).\n          05 ITEM-AMOUNT PIC 9(5).\n       WORKING-STORAGE SECTION. 01 E PIC X VALUE \"N\".\n       PROCEDURE DIVISION.\n           OPEN OUTPUT F.\n           MOVE \"123ABCDEFG\" TO DETAIL-REC.\n           WRITE DETAIL-REC.\n           MOVE \"TOTAL00042\" TO TOTAL-REC.\n           WRITE TOTAL-REC.\n           CLOSE F.\n           OPEN INPUT F.\n           PERFORM UNTIL E = \"Y\"\n              READ F AT END MOVE \"Y\" TO E NOT AT END DISPLAY \"[\" DETAIL-REC \"]\" END-READ\n           END-PERFORM.\n           CLOSE F.\n           STOP RUN.\n";
        assert_eq!(run(src), b"[123ABCDEFG]\n[TOTAL00042]\n");
    }

    #[test]
    fn fd_two_files_records_do_not_cross_associate() {
        // Two FDs with structurally identical records: each WRITE resolves to the CORRECT owning file.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT FA ASSIGN TO MRA ORGANIZATION SEQUENTIAL.\n           SELECT FB ASSIGN TO MRB ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD FA.\n       01 REC-A PIC X(5).\n       01 DUP-REC PIC X(5).\n       FD FB.\n       01 REC-B PIC X(5).\n       01 DUP-REC PIC X(5).\n       WORKING-STORAGE SECTION. 01 E PIC X VALUE \"N\".\n       PROCEDURE DIVISION.\n           OPEN OUTPUT FA FB.\n           MOVE \"AAAAA\" TO DUP-REC.\n           WRITE DUP-REC.\n           MOVE \"BBBBB\" TO REC-B.\n           WRITE REC-B.\n           CLOSE FA FB.\n           OPEN INPUT FB.\n           PERFORM UNTIL E = \"Y\"\n              READ FB AT END MOVE \"Y\" TO E NOT AT END DISPLAY REC-B END-READ\n           END-PERFORM.\n           CLOSE FB.\n           STOP RUN.\n";
        // The FIRST DUP-REC declaration owns the name (WRITE DUP-REC -> FA); REC-B -> FB.
        assert_eq!(run(src), b"BBBBB\n");
    }

    #[test]
    fn fd_write_working_storage_01_fails_closed() {
        // A WORKING-STORAGE 01 is NOT an FD record: WRITE must fail closed, never silently redirect.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO MRFILE ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD F.\n       01 A PIC X(5).\n       WORKING-STORAGE SECTION.\n       01 NOT-A-RECORD PIC X(5).\n       PROCEDURE DIVISION.\n           OPEN OUTPUT F.\n           MOVE \"HELLO\" TO NOT-A-RECORD.\n           WRITE NOT-A-RECORD.\n           CLOSE F.\n           STOP RUN.\n";
        let err = run_program(src).unwrap_err().to_string();
        assert!(err.contains("not an FD record"), "got: {err}");
    }

    #[test]
    fn fd_write_unknown_record_fails_closed() {
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO MRFILE ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD F.\n       01 A PIC X(5).\n       PROCEDURE DIVISION.\n           OPEN OUTPUT F.\n           WRITE NO-SUCH-RECORD.\n           CLOSE F.\n           STOP RUN.\n";
        let err = run_program(src).unwrap_err().to_string();
        assert!(err.contains("not an FD record"), "got: {err}");
    }

    #[test]
    fn fd_rewrite_via_second_record() {
        // REWRITE resolves any FD record of the file and replaces the last READ record with its bytes.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO MRFILE ORGANIZATION SEQUENTIAL FILE STATUS IS ST.\n       DATA DIVISION. FILE SECTION.\n       FD F.\n       01 FIRST-REC PIC X(5).\n       01 SECOND-REC PIC X(5).\n       WORKING-STORAGE SECTION. 01 ST PIC XX. 01 E PIC X VALUE \"N\".\n       PROCEDURE DIVISION.\n           OPEN OUTPUT F.\n           MOVE \"AAAAA\" TO FIRST-REC.\n           WRITE FIRST-REC.\n           CLOSE F.\n           OPEN I-O F.\n           READ F.\n           MOVE \"ZZZZZ\" TO SECOND-REC.\n           REWRITE SECOND-REC.\n           CLOSE F.\n           OPEN INPUT F.\n           PERFORM UNTIL E = \"Y\"\n              READ F AT END MOVE \"Y\" TO E NOT AT END DISPLAY FIRST-REC END-READ\n           END-PERFORM.\n           CLOSE F.\n           STOP RUN.\n";
        assert_eq!(run(src), b"ZZZZZ\n");
    }

    #[test]
    fn fd_advancing_on_second_record_is_accepted() {
        // The CCVS85 report shape: WRITE of the SECOND record with AFTER ADVANCING must parse and run.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO MRFILE ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD F RECORD CONTAINS 120 CHARACTERS.\n       01 PRINT-REC PIC X(120).\n       01 DUMMY-RECORD PIC X(120).\n       WORKING-STORAGE SECTION. 01 H PIC X(120) VALUE \"HELLO\".\n       PROCEDURE DIVISION.\n           MOVE H TO DUMMY-RECORD.\n           OPEN OUTPUT F.\n           WRITE DUMMY-RECORD AFTER ADVANCING 1 LINES.\n           CLOSE F.\n           STOP RUN.\n";
        assert_eq!(run(src), b"");
    }

    #[test]
    fn fd_read_into_shared_area_visible_through_all_records() {
        // A READ fills the shared record area: every record description sees the read bytes (union).
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO MRFILE ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD F.\n       01 FIRST-REC PIC X(5).\n       01 SECOND-REC PIC X(5).\n       WORKING-STORAGE SECTION. 01 E PIC X VALUE \"N\".\n       PROCEDURE DIVISION.\n           OPEN OUTPUT F.\n           MOVE \"ABCDE\" TO FIRST-REC.\n           WRITE FIRST-REC.\n           MOVE \"VWXYZ\" TO SECOND-REC.\n           WRITE SECOND-REC.\n           CLOSE F.\n           OPEN INPUT F.\n           PERFORM UNTIL E = \"Y\"\n              READ F AT END MOVE \"Y\" TO E NOT AT END DISPLAY FIRST-REC \";\" SECOND-REC END-READ\n           END-PERFORM.\n           CLOSE F.\n           STOP RUN.\n";
        assert_eq!(run(src), b"ABCDE;ABCDE\nVWXYZ;VWXYZ\n");
    }

    #[test]
    fn perform_thru_range_goto_last_paragraph_returns_after_perform() {
        // The CCVS85 report idiom: `PERFORM X THRU X-EXIT` whose body conditionally `GO TO X-EXIT`
        // (jumping to the LAST paragraph of the performed range). Control must stay INSIDE the performed
        // range and return to the statement AFTER the PERFORM -- NOT resume linear execution at X-EXIT
        // (which re-ran the following section forever; the front-end previously propagated the jump to
        // the body level, an unbounded loop that only the 1e7-jump guard caught). Oracle-verified.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 CORRECT-A PIC X(5) VALUE SPACES.\n       01 N PIC 9 VALUE 0.\n       PROCEDURE DIVISION.\n           PERFORM BAIL-OUT THRU BAIL-OUT-EX.\n           ADD 1 TO N.\n           DISPLAY \"AFTER \" N.\n           STOP RUN.\n       BAIL-OUT.\n           IF CORRECT-A EQUAL TO SPACE GO TO BAIL-OUT-EX.\n           DISPLAY \"BAIL WRITE\".\n       BAIL-OUT-EX.\n           EXIT.\n";
        assert_eq!(run(src), b"AFTER 1\n");
    }

    #[test]
    fn perform_thru_range_internal_goto_stays_in_range() {
        // A GO TO to a MIDDLE paragraph of the performed range also stays inside the range; the range
        // completes and the PERFORM returns once (no unbounded body-level jump loop).
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 FLAG PIC X VALUE \"Y\".\n       01 N PIC 9 VALUE 0.\n       PROCEDURE DIVISION.\n           PERFORM P1 THRU P3.\n           DISPLAY \"N=\".\n           DISPLAY N.\n           STOP RUN.\n       P1.\n           ADD 1 TO N.\n           IF FLAG = \"Y\" GO TO P3.\n           ADD 1 TO N.\n       P2.\n           ADD 1 TO N.\n       P3.\n           EXIT.\n";
        assert_eq!(run(src), b"N=\n1\n");
    }

    #[test]
    fn perform_inside_size_error_handler_runs_against_the_body() {
        // The CCVS85 idiom `ON SIZE ERROR PERFORM PASS GO TO ...`: a PERFORM dispatched from an
        // imperative HANDLER runs against the CURRENT program body (CUR_PROC) -- the handler's copied
        // token block has no paragraph labels, so running it there mis-indexed and spun forever.
        // The size error fires (99999 + 1 overflows 9(5)); the handler must run PASS and GO TO the
        // write paragraph, and the run must finish (no 1e7-jump / timeout loop).
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 N-13 PIC 9(5) VALUE 99999.\n       01 N-10 PIC S9(5) VALUE -1.\n       01 PASS-CNT PIC 9 VALUE 0.\n       PROCEDURE DIVISION.\n           SUBTRACT N-10 FROM N-13 ON SIZE ERROR\n               PERFORM PASS\n               GO TO SUB-WRITE.\n           DISPLAY \"FAILED\".\n           STOP RUN.\n       SUB-WRITE.\n           DISPLAY \"DONE \" PASS-CNT.\n           STOP RUN.\n       PASS.\n           ADD 1 TO PASS-CNT.\n";
        assert_eq!(run(src), b"DONE 1\n");
    }

    #[test]
    fn fixed_to_free_merges_literal_continuation() {
        // A col-7 `-` continuation of a NONNUMERIC literal joins the previous line FLUSH, the quote
        // at column 12 being the continuation marker (not part of the value). The CCVS85 corpus splits
        // VALUE literals this way (e.g. the HYPHEN-LINE / column-header records); the merged value
        // must be the oracle's 24 + 41 = 65 asterisks, one display line.
        let src = "000100 IDENTIFICATION DIVISION.\n000200 PROGRAM-ID. CT.\n000300 DATA DIVISION.\n000400 WORKING-STORAGE SECTION.\n000500 01  H.\n000600     02 FILLER PIC X(65) VALUE IS \"************************\n000700-    \"*****************************************\".\n000800 PROCEDURE DIVISION.\n000900     DISPLAY H.\n001000     STOP RUN.\n";
        let conv = fixed_to_free(src);
        assert!(
            conv.contains(
                "VALUE IS \"*****************************************************************\"."
            ),
            "merged literal: {conv:?}"
        );
        // run_program parses FREE format; fixed_to_free is the cobrun-side conversion under test.
        assert_eq!(
            run(&conv),
            vec![b'*'; 65]
                .into_iter()
                .chain([b'\n'])
                .collect::<Vec<u8>>()
        );
    }

    #[test]
    fn write_after_advancing_emits_oracle_line_control_bytes() {
        // WRITE ... AFTER ADVANCING n writes n x LF before the record; CLOSE appends a final LF
        // (GnuCOBOL flag_needs_nl). The dumped file store must equal the oracle's bytes
        // (verified byte-identical against the built oracle for the CCVS85 report shape).
        let dir = std::env::temp_dir().join(format!("gcrs_adv_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        set_file_dump_dir(dir.clone());
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       ENVIRONMENT DIVISION. INPUT-OUTPUT SECTION. FILE-CONTROL.\n           SELECT F ASSIGN TO OUTFILE ORGANIZATION SEQUENTIAL.\n       DATA DIVISION. FILE SECTION.\n       FD F RECORD CONTAINS 120 CHARACTERS.\n       01 PRINT-REC PIC X(120).\n       01 DUMMY-RECORD PIC X(120).\n       WORKING-STORAGE SECTION. 01 H PIC X(120) VALUE \"HELLO\".\n       PROCEDURE DIVISION.\n           MOVE H TO DUMMY-RECORD.\n           OPEN OUTPUT F.\n           WRITE DUMMY-RECORD AFTER ADVANCING 1 LINES.\n           WRITE DUMMY-RECORD AFTER ADVANCING 2 LINES.\n           CLOSE F.\n           STOP RUN.\n";
        run(src);
        let bytes = std::fs::read(dir.join("OUTFILE")).expect("dump written");
        let _ = std::fs::remove_dir_all(&dir);
        // 1 LF + 120 + 2 LF + 120 + final LF at close = 244 (the sweep-pinned oracle size).
        assert_eq!(bytes.len(), 244, "oracle line-control size");
        assert_eq!(&bytes[..1], b"\n");
        assert_eq!(&bytes[121..123], b"\n\n");
        assert_eq!(bytes.last(), Some(&b'\n'));
    }
}

#[cfg(test)]
mod probe_phase_tests {
    use super::*;

    const GOOD: &str = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 W PIC X(6) VALUE \"ABCDEF\".\n       PROCEDURE DIVISION.\n           DISPLAY W(2:3).\n           STOP RUN.\n";
    const BAD_SYNTAX: &str = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           BOGUS-STATEMENT X.\n           STOP RUN.\n";

    #[test]
    fn probe_all_phases_ok_on_good_source() {
        let probes = probe_phases(GOOD, crate::dialect::Dialect::DEFAULT, true);
        let phases: Vec<&str> = probes.iter().map(|p| p.phase.as_str()).collect();
        assert_eq!(
            phases,
            vec![
                "preprocess",
                "lex",
                "parse",
                "resolution",
                "layout",
                "check",
                "prepare",
                "execute"
            ]
        );
        assert!(probes.iter().all(|p| p.ok), "{probes:?}");
        // probe agreement with the canonical entry points
        assert!(check_program(GOOD, crate::dialect::Dialect::DEFAULT).is_ok());
        assert_eq!(
            run_program_dialect_with_rc(GOOD, crate::dialect::Dialect::DEFAULT)
                .unwrap()
                .1,
            0
        );
    }

    #[test]
    fn probe_stops_at_first_static_failure() {
        let probes = probe_phases(BAD_SYNTAX, crate::dialect::Dialect::DEFAULT, false);
        // first failure is a check-phase diagnostic (the unknown statement verb), never a guess
        let first_bad = probes.iter().find(|p| !p.ok).expect("a failure");
        assert_eq!(first_bad.phase, "check");
        assert!(!first_bad.diagnostic.is_empty());
        assert_eq!(probes.len(), 6); // five ok probes (preprocess..layout) + the failing check
        assert_eq!(probes[0].phase, "preprocess");
        // agreement: check_program also rejects it
        assert!(check_program(BAD_SYNTAX, crate::dialect::Dialect::DEFAULT).is_err());
    }

    #[test]
    fn probe_missing_procedure_division_is_parse_failure() {
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n";
        let probes = probe_phases(src, crate::dialect::Dialect::DEFAULT, false);
        let first_bad = probes.iter().find(|p| !p.ok).expect("a failure");
        assert_eq!(first_bad.phase, "parse");
        assert!(first_bad.diagnostic.contains("no PROCEDURE DIVISION"));
    }

    #[test]
    fn probe_run_failure_is_execute_phase() {
        // statically fine; runtime fatal (CANCEL of the active program, exit 1)
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. prog.\n       PROCEDURE DIVISION.\n           CANCEL \"prog\".\n           DISPLAY \"NG\" NO ADVANCING END-DISPLAY.\n           STOP RUN.\n";
        set_source_file("prog.cob");
        let probes = probe_phases(src, crate::dialect::Dialect::DEFAULT, true);
        let execute = probes.last().expect("execute probe");
        assert_eq!(execute.phase, "execute");
        assert!(!execute.ok, "{execute:?}");
        assert!(execute.diagnostic.contains("CANCEL active program"));
    }

    #[test]
    fn prepared_program_run_equals_fresh_run() {
        // a program with arithmetic, files-free output, and a loop, run once prepared and
        // once fresh; the outputs and exit codes must agree byte-for-byte.
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n       01 I PIC 9(2).\n       01 S PIC 9(4).\n       01 SE PIC Z(3)9.\n       PROCEDURE DIVISION.\n           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 20\n               ADD I TO S\n           END-PERFORM\n           MOVE S TO SE\n           DISPLAY SE\n           STOP RUN.\n";
        let prepared = prepare_program(src, crate::dialect::Dialect::DEFAULT).expect("prepare");
        // the prepared program's probes are all ok
        assert!(
            prepared.probes.iter().all(|p| p.ok),
            "{:?}",
            prepared.probes
        );
        let (out1, _printer1, rc1) = prepared.run(false).expect("prepared run");
        let (out2, rc2) =
            run_program_dialect_with_rc(src, crate::dialect::Dialect::DEFAULT).expect("fresh run");
        assert_eq!(out1, out2);
        assert_eq!(rc1, rc2);
        // repeated prepared runs are byte-identical (no parse between them)
        let (out3, _printer3, _rc3) = prepared.run(false).expect("repeat run");
        assert_eq!(out1, out3);
        // identity is deterministic
        assert_eq!(prepared.source_hash.len(), 64);
        assert_eq!(prepared.expanded_hash.len(), 64);
        assert_eq!(prepared.compat, "prepared-v1");
    }

    #[test]
    fn prepare_rejects_bad_program() {
        let src = "       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           BOGUS-STATEMENT X.\n           STOP RUN.\n";
        let e = prepare_program(src, crate::dialect::Dialect::DEFAULT)
            .err()
            .expect("prepare must reject");
        assert!(!e.to_string().is_empty());
    }
}
