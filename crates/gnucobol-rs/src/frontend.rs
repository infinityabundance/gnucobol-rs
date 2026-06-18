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

use crate::arith::{cob_arith, cob_divide, ArithError, Op, Round};
use crate::attr::{FieldAttr, COB_TYPE_NUMERIC_DISPLAY};
use crate::edited::{edited_size, encode_edited_cfg};
use crate::move_ops::{cob_move, cob_move_cfg};
use crate::pic::{build_field, Usage};
use crate::termio::{cob_display, DisplaySettings};
use crate::value::Decimal;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

/// The COBOL statement verbs the front-end actually EXECUTES (not merely recognizes as a boundary).
/// The generated parity tracker (`xtask cobol-parity`) reads this to report front-end coverage, so it
/// stays honest as the subset grows. Keep this in sync with the dispatch in `exec_stmt` + `run_block`.
pub const WIRED_STATEMENTS: &[&str] = &[
    "DISPLAY", "MOVE", "SET", "INITIALIZE", "INSPECT", "STRING", "UNSTRING", "ACCEPT", "ADD", "SUBTRACT",
    "MULTIPLY", "DIVIDE", "COMPUTE", "IF", "PERFORM", "STOP", "CONTINUE", "GOTO", "GOBACK", "EXIT", "CALL",
    "CANCEL", "EVALUATE", "SEARCH", "OPEN", "CLOSE", "READ", "WRITE", "REWRITE", "DELETE", "START",
    "UNLOCK", "COMMIT", "ROLLBACK", "SORT", "MERGE", "RELEASE", "RETURN", "JSON", "XML", "TRANSFORM", "RAISE",
    "VALIDATE", "DESTROY", "READY", "RESET", "EXHIBIT", "ALTER", "GENERATE", "INITIATE", "TERMINATE",
    "SUPPRESS", "EXAMINE",
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
    /// An arithmetic SIZE ERROR condition (EC-SIZE-*): a divide-by-zero (or, in future, a result too
    /// large for the receiver). The receiver is left UNCHANGED and the statement's `ON SIZE ERROR`
    /// handler (if any) runs; with no handler, execution continues silently. Caught by `run_block` /
    /// the `exec_arith` / `exec_compute` wrappers -- it never propagates out as a fatal error.
    SizeError,
}

impl core::fmt::Display for RunError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RunError::Unsupported(s) => write!(f, "unsupported: {s}"),
            RunError::UndefinedName(s) => write!(f, "undefined data name: {s}"),
            RunError::Runtime(s) => write!(f, "runtime error: {s}"),
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
    Edited(String, u8, bool),
    /// An `88`-level condition-name: true when its `parent` field's value equals any of `values` (a single
    /// value or a `lo THRU hi` range). Carries no storage of its own.
    Condition { parent: String, values: Vec<CondVal> },
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
    FieldAttr { field_type: 0x21, digits: 0, scale: 0, flags: 0 }
}

/// Build a numeric `USAGE DISPLAY` attr for an integer/decimal literal of `digits` digits and
/// `scale` fractional digits (sign per `signed`).
fn lit_num_attr(digits: u16, scale: i16, signed: bool) -> FieldAttr {
    let flags = if signed { crate::attr::COB_FLAG_HAVE_SIGN } else { 0 };
    FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits, scale, flags }
}

// ---------------------------------------------------------------------------------------------
// Lexer: COBOL source -> a flat token stream. Strings are kept as single tokens; '.' that ends a
// sentence is emitted as its own "." token (a period glued to a word, like "RUN.", is split).
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Word(String),
    Str(Vec<u8>),
    Dot,
}

fn lex(src: &str) -> Vec<Tok> {
    let mut toks = Vec::new();
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
        if c == b'*' && (i == 0 || bytes[i - 1] == b'\n') {
            // a full-line comment (col-1 '*' in free form, or a comment line); skip to EOL.
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_whitespace() {
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
            continue;
        }
        if c == b'.' {
            toks.push(Tok::Dot);
            i += 1;
            continue;
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
    }
    toks
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
            '*' | '/' => out.push('\n'),              // comment / page-eject: drop the line
            _ => {
                let end = chars.len().min(72);        // columns 8..=72 (0-indexed 7..72); 73+ ignored
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
            // an unrecognized >> directive: pass it through (only if currently including).
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
    let p: Vec<&str> = c.split_whitespace().collect();
    if p.len() >= 3 && p[0] == "NOT" && p[2] == "DEFINED" {
        return !defines.contains_key(p[1]);
    }
    if p.len() >= 2 && p[1] == "DEFINED" {
        return defines.contains_key(p[0]);
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
    EC_BOUND_SUBSCRIPT_ON.with(|c| c.set(parse_ec_bound_check(&up)));
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
    let file_defs: HashMap<String, FileDef> = program_map.get(&main_name)
        .map(|p| p.files.iter().map(|f| (f.name.clone(), f.clone())).collect())
        .unwrap_or_default();
    let reports: HashMap<String, ReportDef> = program_map.get(&main_name)
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
        call_state: RefCell::new(HashMap::new()),
        goto: RefCell::new(None),
        file_defs,
        files: RefCell::new(HashMap::new()),
        reports,
    };
    let main = ctx.programs.get(&main_name).expect("main program is registered");

    let mut out = Vec::new();
    let mut fields = build_program_fields(main, &ctx)?;
    run_program_body(main, &ctx, &mut fields, &mut out)?;
    let rc = read_return_code(&fields);
    let printer = ctx.printer.borrow().clone();
    Ok((out, printer, rc))
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
    linkage: Vec<ProgItem>,
    using: Vec<String>,
    /// `SELECT ... ASSIGN` + `FD` declared files (the subset: sequential / line-sequential).
    files: Vec<FileDef>,
    /// `RD` report descriptions (REPORT SECTION) by report name.
    reports: HashMap<String, ReportDef>,
    proc_toks: Vec<Tok>,
    /// `PROGRAM-ID. name IS INITIAL` -- the program's WORKING-STORAGE is re-initialized to its VALUE
    /// clauses on EVERY entry, rather than persisting (static) across CALLs.
    is_initial: bool,
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
}

/// A declared file: its `SELECT` name, the `FD` record field it reads/writes through, the optional
/// `FILE STATUS` field, the organization, and (for RELATIVE) the RELATIVE KEY field name.
#[derive(Debug, Clone)]
struct FileDef {
    name: String,
    /// The `ASSIGN TO` target -- the in-memory file store is keyed by this, so two SELECTs on the same
    /// physical name share records (a report written then re-read: the disk semantics the oracle has).
    assign: String,
    record: String,
    status: Option<String>,
    org: FileOrg,
    rel_key: Option<String>,
}

/// One printable report element: a `COLUMN n PIC p {SOURCE field | VALUE lit}` entry in a report group.
#[derive(Debug, Clone)]
struct RElem {
    column: usize,
    pic: String,
    source: Option<String>,
    value: Option<Tok>,
}

/// A `RD` report description: the file it writes to, and each report group's lines (each line a set of
/// column-placed elements). The minimal subset: groups of COLUMN + PIC + SOURCE/VALUE elements.
#[derive(Debug, Clone, Default)]
struct ReportDef {
    file: String,
    groups: HashMap<String, Vec<Vec<RElem>>>,
}

/// The live state of an OPEN file: its logical records, the next READ position, and the open mode.
#[derive(Debug, Clone, Default)]
struct FileState {
    records: Vec<Vec<u8>>,
    read_pos: usize,
    /// 0 = closed, 1 = INPUT, 2 = OUTPUT, 3 = EXTEND, 4 = I-O.
    mode: u8,
}

/// One `01`-level elementary item (its name, PIC, and optional VALUE literal) -- the field is built at run
/// time (so a CALL can build the callee's fields under the same dialect).
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
    condition: Option<(String, Vec<CondVal>)>,
    /// `OCCURS ... INDEXED BY idx [idx ...]` -- the table's index name(s). Each becomes an integer index
    /// field; the first is the table's implicit SEARCH index.
    indexed_by: Vec<String>,
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
    files: RefCell<HashMap<String, FileState>>,
    /// `RD` report descriptions by report name (from the main program), for INITIATE/GENERATE/TERMINATE.
    reports: HashMap<String, ReportDef>,
}

/// The UPSI switch environment: the live switch states (from `COB_SWITCH_n`) and the `SPECIAL-NAMES`
/// `SWITCH-n ON/OFF STATUS IS <name>` condition-name map.
struct SwitchEnv {
    /// `cob_switch[n]` -- index `n` from `SWITCH-n` (1-based); on/off.
    states: [bool; crate::common_misc::COB_SWITCH_COUNT],
    /// condition-name -> (switch index, expected ON when true).
    conds: HashMap<String, (usize, bool)>,
}

impl Default for SwitchEnv {
    fn default() -> Self {
        SwitchEnv { states: [false; crate::common_misc::COB_SWITCH_COUNT], conds: HashMap::new() }
    }
}

/// Parse the `SPECIAL-NAMES` switch declarations (`SWITCH-n [ON STATUS IS a] [OFF STATUS IS b]`) before
/// `before`, and load the switch states from the `COB_SWITCH_n` environment (`ON`/`1` -> on, else off --
/// the default is off), mirroring `cob_init`.
fn parse_switches(toks: &[Tok], before: usize) -> SwitchEnv {
    let mut conds: HashMap<String, (usize, bool)> = HashMap::new();
    let mut i = 0;
    while i < before {
        if let Some(Tok::Word(w)) = toks.get(i) {
            if let Some(n) = w.strip_prefix("SWITCH-").and_then(|s| s.parse::<usize>().ok()) {
                let mut k = i + 1;
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
    SwitchEnv { states, conds }
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
        if main_name.is_none() {
            main_name = Some(name.clone());
        }
        map.insert(name, def);
    }
    Ok((main_name.unwrap(), map))
}

/// Parse one program from `toks[start..end]` (start is its `PROGRAM-ID`).
fn parse_one_program(toks: &[Tok], start: usize, end: usize) -> Result<(String, ProgramDef), RunError> {
    // PROGRAM-ID. NAME.
    let mut k = start + 1;
    if matches!(toks.get(k), Some(Tok::Dot)) {
        k += 1;
    }
    let name = match toks.get(k) {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("expected program name after PROGRAM-ID".into())),
    };
    // PROGRAM-ID. name [IS] [INITIAL | COMMON | RECURSIVE]. -- scan the paragraph (to its '.') for INITIAL.
    let mut is_initial = false;
    let mut q = k + 1;
    while let Some(t) = toks.get(q) {
        match t {
            Tok::Dot => break,
            Tok::Word(w) if w == "INITIAL" => {
                is_initial = true;
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
    let (mut file_recs, file_rec, report_file) = parse_file_section(toks, start, proc_at)?;
    let files: Vec<FileDef> = file_control.into_iter().map(|(name, assign, org, status, rel_key)| {
        let record = file_rec.get(&name).cloned().unwrap_or_default();
        FileDef { name, assign, record, status, org, rel_key }
    }).collect();
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
                Some(Tok::Word(w)) if w == "BY" || w == "REFERENCE" || w == "CONTENT" || w == "VALUE" => {
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

    Ok((name, ProgramDef { ws, linkage, using, files, reports, proc_toks, is_initial }))
}

/// Parse `FILE-CONTROL` `SELECT name ASSIGN ... [ORGANIZATION [IS] {LINE SEQUENTIAL|SEQUENTIAL}]
/// [FILE STATUS [IS] status]` entries -> `(name, org, status)`. Unknown clauses are skipped.
fn parse_file_control(toks: &[Tok], start: usize, end: usize) -> Vec<(String, String, FileOrg, Option<String>, Option<String>)> {
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
                let name = match toks.get(i) { Some(Tok::Word(w)) => w.clone(), _ => break };
                i += 1;
                let mut org = FileOrg::Sequential;
                let mut status = None;
                let mut rel_key = None;
                let mut assign = name.clone();
                while i < end {
                    match toks.get(i) {
                        Some(Tok::Dot) => { i += 1; break; }
                        // ASSIGN [TO] [DYNAMIC] {"path" | word} -- the physical file the store keys on.
                        Some(Tok::Word(w)) if w == "ASSIGN" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "TO") { i += 1; }
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "DYNAMIC" || w == "EXTERNAL") { i += 1; }
                            match toks.get(i) {
                                Some(Tok::Str(s)) => { assign = String::from_utf8_lossy(s).to_string(); i += 1; }
                                Some(Tok::Word(w)) => { assign = w.clone(); i += 1; }
                                _ => {}
                            }
                        }
                        Some(Tok::Word(w)) if w == "ORGANIZATION" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") { i += 1; }
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "LINE") {
                                org = FileOrg::LineSequential;
                                i += 1;
                                if matches!(toks.get(i), Some(Tok::Word(w)) if w == "SEQUENTIAL") { i += 1; }
                            } else if matches!(toks.get(i), Some(Tok::Word(w)) if w == "RELATIVE") {
                                org = FileOrg::Relative;
                                i += 1;
                            } else if matches!(toks.get(i), Some(Tok::Word(w)) if w == "SEQUENTIAL") {
                                org = FileOrg::Sequential;
                                i += 1;
                            }
                        }
                        // RELATIVE KEY [IS] field
                        Some(Tok::Word(w)) if w == "RELATIVE" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "KEY") { i += 1; }
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") { i += 1; }
                            if let Some(Tok::Word(w)) = toks.get(i) { rel_key = Some(w.clone()); i += 1; }
                        }
                        Some(Tok::Word(w)) if w == "STATUS" => {
                            i += 1;
                            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") { i += 1; }
                            if let Some(Tok::Word(w)) = toks.get(i) { status = Some(w.clone()); i += 1; }
                        }
                        _ => i += 1,
                    }
                }
                out.push((name, assign, org, status, rel_key));
            }
            None => break,
            _ => i += 1,
        }
    }
    out
}

/// Parse the `FILE SECTION` `FD name [clauses]. 01 record ...` entries -> (the record items to add to the
/// field table, and a file-name -> record-name map). The subset is one `01` record per file.
fn parse_file_section(toks: &[Tok], start: usize, end: usize) -> Result<(Vec<ProgItem>, HashMap<String, String>, HashMap<String, String>), RunError> {
    let fs = match find_seq_in(toks, &["FILE", "SECTION"], start, end) {
        Some(i) => i + 2,
        None => return Ok((Vec::new(), HashMap::new(), HashMap::new())),
    };
    // the FILE SECTION ends at the next section (WORKING-STORAGE, LOCAL-STORAGE, LINKAGE, or REPORT).
    let ws_at = [
        find_seq_in(toks, &["WORKING-STORAGE", "SECTION"], fs, end),
        find_seq_in(toks, &["REPORT", "SECTION"], fs, end),
        find_seq_in(toks, &["LINKAGE", "SECTION"], fs, end),
    ].into_iter().flatten().min().unwrap_or(end);
    let mut recs = Vec::new();
    let mut file_rec = HashMap::new();
    let mut report_file = HashMap::new();
    let mut i = fs;
    while i < ws_at {
        match toks.get(i) {
            Some(Tok::Word(w)) if w == "FD" || w == "SD" => {
                i += 1;
                let fname = match toks.get(i) { Some(Tok::Word(w)) => w.clone(), _ => break };
                i += 1;
                // scan the FD clauses to the period; capture `REPORT[S] [IS|ARE] r1 [r2 ...]`.
                while i < ws_at && !matches!(toks.get(i), Some(Tok::Dot)) {
                    if matches!(toks.get(i), Some(Tok::Word(w)) if w == "REPORT" || w == "REPORTS") {
                        i += 1;
                        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS" || w == "ARE") { i += 1; }
                        while let Some(Tok::Word(r)) = toks.get(i) {
                            report_file.insert(r.clone(), fname.clone());
                            i += 1;
                        }
                        continue;
                    }
                    i += 1;
                }
                if matches!(toks.get(i), Some(Tok::Dot)) { i += 1; }
                let rec_start = i;
                let mut rec_end = i;
                while rec_end < ws_at && !matches!(toks.get(rec_end), Some(Tok::Word(w)) if w == "FD" || w == "SD") {
                    rec_end += 1;
                }
                let items = parse_items(toks, rec_start, rec_end)?;
                if let Some(first) = items.first() {
                    file_rec.insert(fname, first.name.clone());
                }
                recs.extend(items);
                i = rec_end;
            }
            _ => i += 1,
        }
    }
    Ok((recs, file_rec, report_file))
}

/// Parse the `REPORT SECTION` `RD r1. 01 group [TYPE ...]. ... COLUMN n PIC p {SOURCE id | VALUE lit} ...`
/// into report definitions. Minimal subset: each report group is a set of column-placed elements, one
/// output line per `LINE` clause (a group with no LINE clause is one line).
fn parse_report_section(toks: &[Tok], start: usize, end: usize, report_file: &HashMap<String, String>) -> HashMap<String, ReportDef> {
    let mut reports: HashMap<String, ReportDef> = HashMap::new();
    let rs = match find_seq_in(toks, &["REPORT", "SECTION"], start, end) {
        Some(i) => i + 2,
        None => return reports,
    };
    let mut cur_report: Option<String> = None;
    let mut cur_group: Option<String> = None;
    let mut i = rs;
    while i < end {
        let w = match toks.get(i) { Some(Tok::Word(w)) => w.clone(), _ => { i += 1; continue; } };
        if w == "PROCEDURE" { break; }
        if w == "RD" {
            i += 1;
            if let Some(Tok::Word(r)) = toks.get(i) {
                let file = report_file.get(r).cloned().unwrap_or_default();
                reports.entry(r.clone()).or_insert_with(|| ReportDef { file, groups: HashMap::new() });
                cur_report = Some(r.clone());
                cur_group = None;
                i += 1;
            }
            while i < end && !matches!(toks.get(i), Some(Tok::Dot)) { i += 1; }
            i += 1;
            continue;
        }
        // a level number starting a report item.
        if w == "01" {
            i += 1;
            let gname = match toks.get(i) { Some(Tok::Word(g)) => g.clone(), _ => { continue; } };
            i += 1;
            if let (Some(rep), grp) = (cur_report.clone(), &gname) {
                reports.entry(rep).or_default().groups.entry(grp.clone()).or_insert_with(|| vec![Vec::new()]);
            }
            cur_group = Some(gname);
            continue;
        }
        // a `LINE` clause starts a new output line in the current group.
        if w == "LINE" {
            if let (Some(rep), Some(grp)) = (&cur_report, &cur_group) {
                if let Some(rd) = reports.get_mut(rep) {
                    let lines = rd.groups.entry(grp.clone()).or_insert_with(|| vec![Vec::new()]);
                    if !lines.last().map(|l| l.is_empty()).unwrap_or(true) {
                        lines.push(Vec::new());
                    }
                }
            }
            i += 1;
            continue;
        }
        // a `COLUMN n ... PIC p ... {SOURCE id | VALUE lit}` printable element.
        if w == "COLUMN" || w == "COL" {
            i += 1;
            if matches!(toks.get(i), Some(Tok::Word(w)) if w == "NUMBER" || w == "IS" || w == "PLUS") { i += 1; }
            let column: usize = match toks.get(i) { Some(Tok::Word(n)) => n.parse().unwrap_or(1), _ => 1 };
            i += 1;
            let mut pic = String::new();
            let mut source = None;
            let mut value = None;
            while i < end && !matches!(toks.get(i), Some(Tok::Dot)) {
                match toks.get(i) {
                    Some(Tok::Word(w)) if w == "PIC" || w == "PICTURE" => {
                        i += 1;
                        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") { i += 1; }
                        if let Some(Tok::Word(p)) = toks.get(i) { pic = p.clone(); i += 1; }
                    }
                    Some(Tok::Word(w)) if w == "SOURCE" => {
                        i += 1;
                        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") { i += 1; }
                        if let Some(Tok::Word(s)) = toks.get(i) { source = Some(s.clone()); i += 1; }
                    }
                    Some(Tok::Word(w)) if w == "VALUE" => {
                        i += 1;
                        if matches!(toks.get(i), Some(Tok::Word(w)) if w == "IS") { i += 1; }
                        value = toks.get(i).cloned();
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
            if let (Some(rep), Some(grp)) = (&cur_report, &cur_group) {
                if let Some(rd) = reports.get_mut(rep) {
                    let lines = rd.groups.entry(grp.clone()).or_insert_with(|| vec![Vec::new()]);
                    if lines.is_empty() { lines.push(Vec::new()); }
                    lines.last_mut().unwrap().push(RElem { column, pic, source, value });
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
        // An `88`-level condition-name on the most recent data item.
        if level == "88" {
            k += 1;
            let cname = match toks.get(k) {
                Some(Tok::Word(w)) => w.clone(),
                _ => return Err(RunError::Unsupported("expected condition-name after 88".into())),
            };
            k += 1;
            let parent = last_item
                .clone()
                .ok_or_else(|| RunError::Unsupported("88 condition-name with no parent item".into()))?;
            // VALUE [IS] v [THRU h] [v2 [THRU h2] ...] .
            if matches!(toks.get(k), Some(Tok::Word(w)) if w == "VALUE" || w == "VALUES") {
                k += 1;
                if matches!(toks.get(k), Some(Tok::Word(w)) if w == "IS" || w == "ARE") {
                    k += 1;
                }
            }
            let mut values: Vec<CondVal> = Vec::new();
            while k < end {
                match toks.get(k) {
                    Some(Tok::Dot) => {
                        k += 1;
                        break;
                    }
                    Some(t) => {
                        let lo = tok_to_cond_word(t);
                        k += 1;
                        if matches!(toks.get(k), Some(Tok::Word(w)) if w == "THRU" || w == "THROUGH") {
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
                condition: Some((parent, values)),
                indexed_by: Vec::new(),
            });
            continue;
        }
        let lvl: u16 = level.parse().unwrap_or(0);
        if lvl == 0 || (lvl > 49 && lvl != 77) {
            // 01..49 group/elementary levels and 77 (independent elementary) are supported; 66 (RENAMES)
            // and other forms fail closed.
            return Err(RunError::Unsupported(format!("unsupported level number {level}")));
        }
        k += 1;
        let name = match toks.get(k) {
            Some(Tok::Word(w)) => w.clone(),
            _ => return Err(RunError::Unsupported("expected data name after a level number".into())),
        };
        last_item = Some(name.clone());
        k += 1;
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
                    if matches!(toks.get(k), Some(Tok::Word(w)) if w == "TIMES") {
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
                        if matches!(nm.as_str(), "PIC" | "PICTURE" | "VALUE" | "OCCURS" | "REDEFINES" | "TIMES") {
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
                        pic = Some(p.clone());
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
                }
                _ => k += 1,
            }
        }
        // a PIC-less item is a GROUP (its children follow at higher level numbers); resolved in build.
        let pic = pic.unwrap_or_default();
        items.push(ProgItem { level: lvl, name, pic, value, occurs, redefines, condition: None, indexed_by: indexed });
    }
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
fn build_program_fields(prog: &ProgramDef, ctx: &Ctx) -> Result<HashMap<String, Field>, RunError> {
    let mut fields = HashMap::new();
    for it in &prog.ws {
        // An 88-level condition-name carries no storage -- record its parent + values for cond_rel.
        if let Some((parent, values)) = &it.condition {
            fields.insert(
                it.name.clone(),
                Field {
                    storage: Storage::Condition { parent: parent.clone(), values: values.clone() },
                    bytes: Vec::new(),
                    occurs: 1,
                    redefines: None,
                },
            );
            continue;
        }
        // A group item (no PIC) is built after its leaves exist (second pass below).
        if it.pic.is_empty() {
            continue;
        }
        let mut f = make_field(&it.pic, it.value.as_ref(), ctx.currency, ctx.decimal_comma, ctx.dialect)?;
        if it.occurs > 1 {
            // A 01-level OCCURS table: replicate the element image `occurs` times (each element initialized
            // identically, per its VALUE or the dialect fill).
            let elem = f.bytes.clone();
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
        fields.insert(it.name.clone(), f);
    }
    // Second pass: group items (no PIC). A group's IMMEDIATE children are the items that follow it at the
    // first (smallest) level below it, up to the next item at <= its level; deeper items belong to a child.
    for (i, it) in prog.ws.iter().enumerate() {
        if it.level == 88 || !it.pic.is_empty() {
            continue;
        }
        let mut children = Vec::new();
        let mut child_level: Option<u16> = None;
        for sib in &prog.ws[i + 1..] {
            if sib.level <= it.level {
                break;
            }
            if sib.level == 88 {
                continue;
            }
            let cl = *child_level.get_or_insert(sib.level);
            if sib.level == cl {
                children.push(sib.name.clone());
            }
        }
        fields.insert(it.name.clone(), Field {
            storage: Storage::Group { children },
            bytes: Vec::new(),
            occurs: 1,
            redefines: None,
        });
    }
    // RETURN-CODE: the signed special register, initialised to 0 (modelled as S9(9) DISPLAY).
    fields.insert("RETURN-CODE".to_string(), make_return_code(0));
    // TALLY: the EXAMINE count register (unsigned 9(5) DISPLAY).
    if let Ok(t) = make_field("9(5)", None, ctx.currency, ctx.decimal_comma, ctx.dialect) {
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
    read_field(fields, name).ok().flatten().map(|f| f.bytes.len()).unwrap_or(0)
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
        let _ = write_field(fields, c, |f| { f.bytes = slice; Ok(()) });
        off += len;
    }
}

thread_local! {
    /// `OCCURS ... INDEXED BY` table -> its implicit SEARCH index name, populated as each program's fields
    /// are built. `SEARCH table` (without `VARYING`) varies this index.
    static TABLE_INDEX: std::cell::RefCell<HashMap<String, String>> = std::cell::RefCell::new(HashMap::new());
}

/// The implicit SEARCH index for an `OCCURS ... INDEXED BY` table, if one was declared.
fn table_index_lookup(table: &str) -> Option<String> {
    TABLE_INDEX.with(|m| m.borrow().get(table).cloned())
}

thread_local! {
    /// The currently-executing program body's paragraphs as `(name, start_token)` plus the body length,
    /// used by out-of-line `PERFORM para [THRU para2]` to find the token range to run. Saved/restored
    /// around each program body so a CALL does not clobber the caller's paragraphs.
    static CUR_PARAS: std::cell::RefCell<(Vec<(String, usize)>, usize)> = const { std::cell::RefCell::new((Vec::new(), 0)) };
    /// The current program body's tokens (`proc_toks`), so a verb that runs a paragraph range (SORT
    /// INPUT/OUTPUT PROCEDURE) can reach them. Saved/restored around each program body.
    static CUR_PROC: std::cell::RefCell<Vec<Tok>> = const { std::cell::RefCell::new(Vec::new()) };
    /// `ALTER`ed GO TO targets: the token index of a `GO` verb -> the paragraph it now proceeds to. Set by
    /// ALTER, consulted by the GO TO executor. Saved/restored per program body.
    static ALTERED: std::cell::RefCell<HashMap<usize, String>> = std::cell::RefCell::new(HashMap::new());
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
        let end = paras.iter().map(|(_, s)| *s).filter(|&s| s > p2start).min().unwrap_or(*plen);
        Some((start, end))
    })
}

/// Run the statements in `toks[start..end)` (a performed paragraph range). Returns `Ok(true)` if a halt
/// (STOP RUN / GOBACK / EXIT PROGRAM / pending GO TO) propagated out.
fn run_range(toks: &[Tok], start: usize, end: usize, fields: &mut HashMap<String, Field>, out: &mut Vec<u8>, ctx: &Ctx) -> Result<bool, RunError> {
    let mut pos = start;
    while pos < end {
        if matches!(toks.get(pos), Some(Tok::Dot)) {
            pos += 1;
            continue;
        }
        if run_block(toks, &mut pos, fields, out, true, ctx)? {
            return Ok(true);
        }
        if matches!(toks.get(pos), Some(Tok::Dot)) {
            pos += 1;
        }
    }
    Ok(false)
}

/// Execute a program's PROCEDURE DIVISION against `fields`, writing output to `out`. Returns when the body
/// ends (`STOP RUN` / `GOBACK` / `EXIT PROGRAM` / falling off the end).
fn run_program_body(
    prog: &ProgramDef,
    ctx: &Ctx,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
) -> Result<(), RunError> {
    let proc = &prog.proc_toks;
    let labels = paragraph_labels(proc);
    // Publish this body's paragraph ranges for out-of-line PERFORM, saving the caller's (CALL nesting).
    let paras_vec: Vec<(String, usize)> = labels.iter().map(|(n, s)| (n.clone(), *s)).collect();
    let prev_paras = CUR_PARAS.with(|c| c.replace((paras_vec, proc.len())));
    let prev_proc = CUR_PROC.with(|c| c.replace(proc.clone()));
    let prev_altered = ALTERED.with(|c| c.replace(HashMap::new()));
    let mut pos = 0;
    if matches!(proc.first(), Some(Tok::Dot)) {
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
            // A pending GO TO is not a real halt: resume at the named paragraph. STOP/GOBACK/EXIT leave
            // `goto` clear and genuinely end the body.
            let target = ctx.goto.borrow_mut().take();
            if let Some(label) = target {
                pos = *labels.get(&label)
                    .ok_or_else(|| RunError::Unsupported(format!("GO TO unknown paragraph `{label}`")))?;
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
    CUR_PARAS.with(|c| { *c.borrow_mut() = prev_paras; });
    CUR_PROC.with(|c| { *c.borrow_mut() = prev_proc; });
    ALTERED.with(|c| { *c.borrow_mut() = prev_altered; });
    Ok(())
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
            Tok::Dot => { at_start = true; i += 1; }
            Tok::Word(w) if at_start
                && matches!(proc.get(i + 1), Some(Tok::Dot))
                && !STMT_VERBS.contains(&w.as_str())
                && !SCOPE_ENDERS.contains(&w.as_str()) =>
            {
                m.entry(w.clone()).or_insert(i + 2);
                i += 2;
                at_start = true;
            }
            Tok::Word(w) if at_start
                && matches!(proc.get(i + 1), Some(Tok::Word(s)) if s == "SECTION")
                && matches!(proc.get(i + 2), Some(Tok::Dot)) =>
            {
                m.entry(w.clone()).or_insert(i + 3);
                i += 3;
                at_start = true;
            }
            _ => { at_start = false; i += 1; }
        }
    }
    m
}

/// Statement verbs that begin a new statement (so an operand list ends when one is seen).
const STMT_VERBS: &[&str] = &[
    "MOVE", "SET", "INITIALIZE", "INSPECT", "STRING", "UNSTRING", "ADD", "SUBTRACT", "MULTIPLY", "DIVIDE",
    "COMPUTE", "DISPLAY", "IF", "PERFORM", "STOP", "CONTINUE", "ACCEPT", "GO", "EVALUATE", "SEARCH", "CALL",
    "GOBACK", "EXIT", "CANCEL", "OPEN", "CLOSE", "READ", "WRITE", "REWRITE", "DELETE", "START", "UNLOCK",
    "COMMIT", "ROLLBACK", "SORT", "MERGE", "RELEASE", "RETURN", "JSON", "XML", "TRANSFORM", "RAISE",
    "VALIDATE", "DESTROY", "READY", "RESET", "EXHIBIT", "ALTER", "INITIATE", "TERMINATE", "SUPPRESS", "EXAMINE",
];
/// Scope terminators that end a block.
const SCOPE_ENDERS: &[&str] = &["ELSE", "END-IF", "END-PERFORM", "WHEN", "END-EVALUATE", "END-SEARCH", "END-READ", "END-RETURN"];

fn is_boundary(w: &str) -> bool {
    STMT_VERBS.contains(&w) || SCOPE_ENDERS.contains(&w)
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
            Some(Tok::Word(w)) => {
                // A paragraph label `NAME.` or section label `NAME SECTION.` in the run stream: skip it.
                // The following period ends this (empty) block; the program-body loop resumes after it.
                if matches!(toks.get(*pos + 1), Some(Tok::Dot))
                    && !STMT_VERBS.contains(&w.as_str())
                    && !SCOPE_ENDERS.contains(&w.as_str())
                {
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
                        // EXIT PROGRAM ends the body; a bare EXIT (paragraph exit) is a no-op.
                        let rest = collect_operands(toks, pos);
                        let is_prog = rest.iter().any(|t| matches!(t, Tok::Word(w) if w == "PROGRAM"));
                        if exec && is_prog {
                            return Ok(true);
                        }
                    }
                    "CONTINUE" | "NEXT" => { /* no-op */ }
                    // GO TO <paragraph>: set the pending-jump and end this block like a halt; the program
                    // body loop resolves the label and resumes there. `GO TO ... DEPENDING ON` is out of subset.
                    "GO" => {
                        let rest = collect_operands(toks, pos);
                        if exec {
                            if rest.iter().any(|t| matches!(t, Tok::Word(w) if w == "DEPENDING")) {
                                return Err(RunError::Unsupported("GO TO ... DEPENDING ON not in subset".into()));
                            }
                            // an ALTERed GO TO (this verb's position is in the override map) proceeds to the
                            // altered target; otherwise the written target.
                            let altered = ALTERED.with(|c| c.borrow().get(&verb_pos).cloned());
                            let label = altered.or_else(|| rest.iter().find_map(|t| match t {
                                Tok::Word(w) if w != "TO" => Some(w.clone()),
                                _ => None,
                            }));
                            match label {
                                Some(l) => { ctx.goto.borrow_mut().replace(l); return Ok(true); }
                                None => return Err(RunError::Unsupported("GO TO without a target paragraph".into())),
                            }
                        }
                    }
                    // CANCEL "NAME" ... -- drop each named program's persisted WORKING-STORAGE, so its
                    // next CALL rebuilds from VALUE (libcob un-initializes + unloads the module).
                    "CANCEL" => {
                        let rest = collect_operands(toks, pos);
                        if exec {
                            for t in &rest {
                                let nm = match t {
                                    Tok::Str(s) => Some(String::from_utf8_lossy(s).to_string()),
                                    Tok::Word(w) => Some(w.clone()),
                                    Tok::Dot => None,
                                };
                                if let Some(nm) = nm {
                                    ctx.call_state.borrow_mut().remove(&nm);
                                }
                            }
                        }
                    }
                    // Arithmetic verbs carry optional ON SIZE ERROR / NOT ON SIZE ERROR handler blocks
                    // (+ END-verb), so they are parsed here rather than via collect_operands/exec_stmt.
                    "ADD" | "SUBTRACT" | "MULTIPLY" | "DIVIDE" | "COMPUTE" => {
                        let stmt = collect_arith_operands(toks, pos);
                        let on_size = parse_on_size_handler(toks, pos, false);
                        let not_size = parse_on_size_handler(toks, pos, true);
                        let end_kw = format!("END-{verb}");
                        if matches!(toks.get(*pos), Some(Tok::Word(w)) if *w == end_kw) {
                            *pos += 1;
                        }
                        if exec {
                            let size_err = if verb == "COMPUTE" {
                                exec_compute(&stmt, fields)?
                            } else {
                                exec_arith(&verb, &stmt, fields)?
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
            Tok::Word(w) if w == "ON" || w == "NOT" || w.starts_with("END-") || is_boundary(w) => break,
            _ => *pos += 1,
        }
    }
    toks[start..*pos].to_vec()
}

/// True when `toks[p]` ends an arithmetic SIZE-ERROR handler block: end of input, `.`, an `END-verb`, an
/// outer scope terminator, or a following `NOT ON SIZE ERROR` clause.
fn at_size_terminator(toks: &[Tok], p: usize) -> bool {
    match toks.get(p) {
        None | Some(Tok::Dot) => true,
        Some(Tok::Word(w)) if w.starts_with("END-") || SCOPE_ENDERS.contains(&w.as_str()) => true,
        Some(Tok::Word(w)) if w == "NOT"
            && matches!(toks.get(p + 1), Some(Tok::Word(x)) if x == "ON")
            && matches!(toks.get(p + 2), Some(Tok::Word(x)) if x == "SIZE") =>
        {
            true
        }
        _ => false,
    }
}

/// Parse an `[NOT] ON SIZE ERROR <statements>` handler at `*pos` (when `is_not`, the `NOT ON SIZE ERROR`
/// form). Returns the handler statement tokens and advances `*pos` past them; `None` if the clause is absent.
fn parse_on_size_handler(toks: &[Tok], pos: &mut usize, is_not: bool) -> Option<Vec<Tok>> {
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
    while p < toks.len() && !at_size_terminator(toks, p) {
        p += 1;
    }
    let block = toks[start..p].to_vec();
    *pos = p;
    Some(block)
}

/// Run an arithmetic SIZE-ERROR handler block (its own statement sequence). Returns `true` on `STOP RUN` /
/// `GOBACK` inside it.
fn run_handler(block: &[Tok], fields: &mut HashMap<String, Field>, out: &mut Vec<u8>, ctx: &Ctx) -> Result<bool, RunError> {
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
            Tok::Word(w) if w == "THEN" || STMT_VERBS.contains(&w.as_str()) || SCOPE_ENDERS.contains(&w.as_str()) => break,
            _ => {
                cond.push(t.clone());
                *pos += 1;
            }
        }
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "THEN") {
        *pos += 1;
    }
    let truth = if exec { eval_cond(&cond, fields, &ctx.switches, ctx.collation.as_ref())? } else { false };

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
    let is_true = subject.len() == 1 && matches!(&subject[0], Tok::Word(w) if w == "TRUE" || w == "ANY");
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
    let table = match toks.get(*pos) {
        Some(Tok::Word(w)) if w == "ALL" => return Err(RunError::Unsupported("SEARCH ALL (binary search) not in subset".into())),
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("SEARCH: missing table name".into())),
    };
    *pos += 1;
    let mut varying: Option<String> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "VARYING") {
        *pos += 1;
        if let Some(Tok::Word(w)) = toks.get(*pos) { varying = Some(w.clone()); *pos += 1; }
    }
    let occurs = fields.get(&table).map(|f| f.occurs).filter(|&o| o > 1)
        .ok_or_else(|| RunError::Unsupported(format!("SEARCH `{table}` is not an OCCURS table")))?;
    let idx_name = varying.or_else(|| table_index_lookup(&table))
        .ok_or_else(|| RunError::Unsupported(format!("SEARCH `{table}`: no INDEXED BY or VARYING index")))?;
    // parse (do not yet run) the optional AT END block and the WHEN clauses, recording token ranges.
    let mut at_end: Option<usize> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "AT") {
        *pos += 1;
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END") { *pos += 1; }
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
                _ => { cond.push(t.clone()); *pos += 1; }
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
    // serial search: vary the index from its current value until a WHEN matches or it runs off the table.
    loop {
        let iv = resolve_int(&idx_name, fields).unwrap_or(0);
        if iv < 1 || iv as usize > occurs {
            if let Some(s) = at_end {
                let mut p = s;
                if run_block(toks, &mut p, fields, out, true, ctx)? { return Ok(true); }
            }
            return Ok(false);
        }
        for (cond, bstart) in &whens {
            if eval_cond(cond, fields, &ctx.switches, ctx.collation.as_ref())? {
                let mut p = *bstart;
                return run_block(toks, &mut p, fields, out, true, ctx);
            }
        }
        let mv = vec![Tok::Word((iv + 1).to_string()), Tok::Word("TO".to_string()), Tok::Word(idx_name.clone())];
        exec_move(&mv, fields, ctx.decimal_comma)?;
    }
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
        let lo = words.first().ok_or_else(|| RunError::Unsupported("THRU without lower bound".into()))?;
        let hi = words.get(i + 1).ok_or_else(|| RunError::Unsupported("THRU without upper bound".into()))?;
        let ge = cond_compare(subj, lo, fields, col)? != Ordering::Less;
        let le = cond_compare(subj, hi, fields, col)? != Ordering::Greater;
        return Ok(ge && le);
    }
    let val = words.first().ok_or_else(|| RunError::Unsupported("WHEN without a value".into()))?;
    Ok(cond_compare(subj, val, fields, col)? == Ordering::Equal)
}

/// `PERFORM <n> TIMES <stmts> END-PERFORM` or `PERFORM UNTIL <cond> <stmts> END-PERFORM` (the inline
/// forms). Out-of-line `PERFORM <paragraph>` is not in the subset (fail closed).
fn exec_perform(
    toks: &[Tok],
    pos: &mut usize,
    fields: &mut HashMap<String, Field>,
    out: &mut Vec<u8>,
    exec: bool,
    ctx: &Ctx,
) -> Result<bool, RunError> {
    // out-of-line form: PERFORM para [THRU para2] [ n TIMES | UNTIL cond ] -- run a named paragraph range.
    if let Some(Tok::Word(w)) = toks.get(*pos) {
        if para_exists(w) {
            let p1 = w.clone();
            *pos += 1;
            let mut p2 = p1.clone();
            if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "THRU" || w == "THROUGH") {
                *pos += 1;
                if let Some(Tok::Word(w)) = toks.get(*pos) { p2 = w.clone(); *pos += 1; }
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
                        Tok::Word(w) if STMT_VERBS.contains(&w.as_str()) || SCOPE_ENDERS.contains(&w.as_str()) => break,
                        _ => { ucond.push(t.clone()); *pos += 1; }
                    }
                }
            } else if matches!(toks.get(*pos), Some(Tok::Word(_)))
                && matches!(toks.get(*pos + 1), Some(Tok::Word(t)) if t == "TIMES")
            {
                if let Some(Tok::Word(w)) = toks.get(*pos) { times = Some(w.clone()); }
                *pos += 2;
            }
            if !exec {
                return Ok(false);
            }
            let (start, end) = para_range(&p1, &p2)
                .ok_or_else(|| RunError::Unsupported(format!("PERFORM: unknown paragraph `{p1}`/`{p2}`")))?;
            if until {
                let mut guard = 0u32;
                while !eval_cond(&ucond, fields, &ctx.switches, ctx.collation.as_ref())? {
                    if run_range(toks, start, end, fields, out, ctx)? { return Ok(true); }
                    guard += 1;
                    if guard > 1_000_000 { return Err(RunError::Runtime("PERFORM UNTIL exceeded 1e6 iterations".into())); }
                }
            } else {
                let n = times.as_deref().and_then(|w| resolve_int(w, fields)).unwrap_or(1);
                for _ in 0..n.max(0) {
                    if run_range(toks, start, end, fields, out, ctx)? { return Ok(true); }
                }
            }
            return Ok(false);
        }
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
                Tok::Word(w) if STMT_VERBS.contains(&w.as_str()) || SCOPE_ENDERS.contains(&w.as_str()) => break,
                _ => {
                    cond.push(t.clone());
                    *pos += 1;
                }
            }
        }
    } else {
        // PERFORM <n> TIMES
        if let Some(Tok::Word(w)) = toks.get(*pos) {
            times_word = Some(w.clone());
            *pos += 1;
        }
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "TIMES") {
            *pos += 1;
        } else {
            return Err(RunError::Unsupported("PERFORM form (only `n TIMES` / `UNTIL cond` inline)".into()));
        }
    }

    // record the body's start; we re-run it per iteration.
    let body_start = *pos;
    let mut body_end = *pos;
    // first pass: if not executing, just skip the body once to find END-PERFORM.
    {
        let mut scan = *pos;
        let _ = run_block(toks, &mut scan, fields, out, false, ctx)?;
        body_end = scan;
    }

    if exec {
        if is_until {
            // PERFORM UNTIL: test BEFORE each iteration (WITH TEST BEFORE, the default).
            let mut guard = 0u32;
            while !eval_cond(&cond, fields, &ctx.switches, ctx.collation.as_ref())? {
                let mut p = body_start;
                if run_block(toks, &mut p, fields, out, true, ctx)? {
                    return Ok(true);
                }
                guard += 1;
                if guard > 1_000_000 {
                    return Err(RunError::Runtime("PERFORM UNTIL exceeded 1e6 iterations".into()));
                }
            }
        } else {
            let n = times_word
                .as_deref()
                .and_then(|w| resolve_int(w, fields))
                .ok_or_else(|| RunError::Unsupported("PERFORM TIMES count not an integer".into()))?;
            for _ in 0..n {
                let mut p = body_start;
                if run_block(toks, &mut p, fields, out, true, ctx)? {
                    return Ok(true);
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
            let dec = source_to_decimal(&f.bytes, a).ok()?;
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
    fields: &HashMap<String, Field>,
    sw: &SwitchEnv,
    col: Option<&[u8; 256]>,
) -> Result<bool, RunError> {
    let words: Vec<String> = t
        .iter()
        .map(|tok| match tok {
            Tok::Word(w) => w.clone(),
            Tok::Str(s) => format!("\u{1}{}", String::from_utf8_lossy(s)), // mark string literal
            Tok::Dot => ".".into(),
        })
        .collect();
    let mut p = 0;
    let r = cond_or(&words, &mut p, fields, sw, col)?;
    if p != words.len() {
        return Err(RunError::Unsupported(format!("trailing tokens in condition at {}", words[p])));
    }
    Ok(r)
}

fn cond_or(w: &[String], p: &mut usize, f: &HashMap<String, Field>, sw: &SwitchEnv, col: Option<&[u8; 256]>) -> Result<bool, RunError> {
    let mut acc = cond_and(w, p, f, sw, col)?;
    while w.get(*p).map(|s| s.as_str()) == Some("OR") {
        *p += 1;
        let r = cond_and(w, p, f, sw, col)?;
        acc = acc || r;
    }
    Ok(acc)
}

fn cond_and(w: &[String], p: &mut usize, f: &HashMap<String, Field>, sw: &SwitchEnv, col: Option<&[u8; 256]>) -> Result<bool, RunError> {
    let mut acc = cond_rel(w, p, f, sw, col)?;
    while w.get(*p).map(|s| s.as_str()) == Some("AND") {
        *p += 1;
        let r = cond_rel(w, p, f, sw, col)?;
        acc = acc && r;
    }
    Ok(acc)
}

fn cond_rel(w: &[String], p: &mut usize, f: &HashMap<String, Field>, sw: &SwitchEnv, col: Option<&[u8; 256]>) -> Result<bool, RunError> {
    let left = w.get(*p).ok_or_else(|| RunError::Unsupported("condition: missing left operand".into()))?.clone();
    *p += 1;
    // A bare UPSI switch condition-name (SPECIAL-NAMES `SWITCH-n ON/OFF STATUS IS <name>`): its truth is
    // the switch's state matching the declared ON/OFF sense. No relational operator follows.
    if let Some(&(idx, on)) = sw.conds.get(&left) {
        return Ok(sw.states[idx] == on);
    }
    // A bare 88-level condition-name: true when its parent's value equals any listed value or range.
    if let Some(Field { storage: Storage::Condition { parent, values }, .. }) = f.get(&left) {
        for v in values {
            let hit = match v {
                CondVal::Single(val) => cond_compare(parent, val, f, col)? == std::cmp::Ordering::Equal,
                CondVal::Range(lo, hi) => {
                    cond_compare(parent, lo, f, col)? != std::cmp::Ordering::Less
                        && cond_compare(parent, hi, f, col)? != std::cmp::Ordering::Greater
                }
            };
            if hit {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    if w.get(*p).map(|s| s.as_str()) == Some("IS") {
        *p += 1;
    }
    let mut neg = false;
    if w.get(*p).map(|s| s.as_str()) == Some("NOT") {
        neg = true;
        *p += 1;
    }
    let op = match w.get(*p).map(|s| s.as_str()) {
        Some("=") => Rel::Eq,
        Some(">") => Rel::Gt,
        Some("<") => Rel::Lt,
        Some(">=") => Rel::Ge,
        Some("<=") => Rel::Le,
        Some("<>") => Rel::Ne,
        Some("GREATER") => {
            *p += 1;
            if w.get(*p).map(|s| s.as_str()) == Some("THAN") {
                *p += 1;
            }
            *p -= 1; // the loop below does +=1 once
            Rel::Gt
        }
        Some("LESS") => {
            *p += 1;
            if w.get(*p).map(|s| s.as_str()) == Some("THAN") {
                *p += 1;
            }
            *p -= 1;
            Rel::Lt
        }
        Some("EQUAL") => {
            *p += 1;
            if w.get(*p).map(|s| s.as_str()) == Some("TO") {
                *p += 1;
            }
            *p -= 1;
            Rel::Eq
        }
        other => return Err(RunError::Unsupported(format!("condition relop {other:?} (subset: = > < >= <= <> GREATER LESS EQUAL)"))),
    };
    *p += 1;
    let right = w.get(*p).ok_or_else(|| RunError::Unsupported("condition: missing right operand".into()))?.clone();
    *p += 1;
    let ord = cond_compare(&left, &right, f, col)?;
    let base = match op {
        Rel::Eq => ord == std::cmp::Ordering::Equal,
        Rel::Ne => ord != std::cmp::Ordering::Equal,
        Rel::Gt => ord == std::cmp::Ordering::Greater,
        Rel::Lt => ord == std::cmp::Ordering::Less,
        Rel::Ge => ord != std::cmp::Ordering::Less,
        Rel::Le => ord != std::cmp::Ordering::Greater,
    };
    Ok(if neg { !base } else { base })
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
fn cond_compare(a: &str, b: &str, f: &HashMap<String, Field>, col: Option<&[u8; 256]>) -> Result<std::cmp::Ordering, RunError> {
    let na = cond_numeric(a, f);
    let nb = cond_numeric(b, f);
    if let (Some(da), Some(db)) = (&na, &nb) {
        return Ok(dec_cmp(da, db));
    }
    // alphanumeric compare: space-pad the shorter, byte compare. Under PROGRAM COLLATING SEQUENCE the
    // bytes are weighted through `col` first (e.g. EBCDIC order: lowercase < uppercase < digits).
    let sa = cond_bytes(a, f);
    let sb = cond_bytes(b, f);
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
fn cond_numeric(w: &str, f: &HashMap<String, Field>) -> Option<Decimal> {
    if let Some(field) = read_field(f, w).ok().flatten() {
        if let Storage::Numeric(a) = &field.storage {
            return source_to_decimal(&field.bytes, a).ok();
        }
        return None;
    }
    if w.starts_with('\u{1}') {
        return None; // string literal -> alphanumeric
    }
    parse_num_literal(w).ok()
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
        return if na { Ordering::Less } else { Ordering::Greater };
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
) -> Result<Field, RunError> {
    match build_field(pic, Usage::Display, false, false) {
        Ok(pf) => {
            let is_alpha = !pf.attr.is_numeric();
            // Uninitialized storage (no VALUE) is filled per the dialect's `defaultbyte`: the category
            // default ('0'/space) under the default dialect, a single byte (0x00 ibm/mvs, space mf) else.
            // A VALUE clause always overrides the fill.
            let fill = dialect.defaultbyte.byte(is_alpha);
            let bytes = vec![fill; pf.size];
            let storage = if is_alpha { Storage::Alpha(pf.attr) } else { Storage::Numeric(pf.attr) };
            let mut field = Field { storage, bytes, occurs: 1, redefines: None };
            if let Some(v) = value {
                init_value(&mut field, v)?;
            }
            Ok(field)
        }
        Err(crate::pic::PicError::UnsupportedSymbol(_)) | Err(crate::pic::PicError::MixedCategory) => {
            // treat as numeric-edited: storage is the edited image, sized by edited_size. A non-'$'
            // CURRENCY SIGN is normalized to '$' for the size computation (the width is the same; the
            // '.'/',' role swap of DECIMAL-POINT IS COMMA is width-invariant too).
            let cur = (currency as char).to_ascii_uppercase();
            let pic_norm: String = if cur == '$' {
                pic.to_string()
            } else {
                pic.chars().map(|c| if c.to_ascii_uppercase() == cur { '$' } else { c }).collect()
            };
            let size = edited_size(&pic_norm).map_err(|e| RunError::Unsupported(format!("PIC {pic}: {e:?}")))?;
            let mut field =
                Field { storage: Storage::Edited(pic.to_string(), currency, decimal_comma), bytes: vec![b' '; size], occurs: 1, redefines: None };
            if let Some(v) = value {
                init_value(&mut field, v)?;
            }
            Ok(field)
        }
        Err(e) => Err(RunError::Unsupported(format!("PIC {pic}: {e:?}"))),
    }
}

/// Initialize a field from a VALUE literal (a numeric literal word, or a string).
fn init_value(field: &mut Field, v: &Tok) -> Result<(), RunError> {
    match v {
        Tok::Str(s) => {
            let src = s.clone();
            store_alnum(field, &src)
        }
        Tok::Word(w) => {
            // a numeric literal: digits with optional sign + decimal point.
            let dec = parse_num_literal(w)?;
            store_decimal(field, &dec)
        }
        Tok::Dot => Err(RunError::Unsupported("empty VALUE".into())),
    }
}

/// Parse a numeric literal like `-12.34` into a [`Decimal`].
fn parse_num_literal(w: &str) -> Result<Decimal, RunError> {
    let negative = w.starts_with('-');
    let body = w.trim_start_matches(['+', '-']);
    if body.is_empty() || !body.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Err(RunError::Unsupported(format!("not a numeric literal: {w}")));
    }
    let (int_p, frac_p) = body.split_once('.').unwrap_or((body, ""));
    let mut digits: Vec<u8> = Vec::new();
    for c in int_p.chars().chain(frac_p.chars()) {
        if c.is_ascii_digit() {
            digits.push(c as u8 - b'0');
        }
    }
    let scale = frac_p.chars().filter(|c| c.is_ascii_digit()).count() as i16;
    Ok(Decimal { negative, digits, scale })
}

/// Store a [`Decimal`] into a field (numeric -> zoned via the runtime move; edited -> encode).
fn store_decimal(field: &mut Field, dec: &Decimal) -> Result<(), RunError> {
    match &field.storage {
        Storage::Edited(pic, currency, decimal_comma) => {
            let pic = pic.clone();
            let cur = *currency;
            let dc = *decimal_comma;
            field.bytes = encode_edited_cfg(&pic, dec, cur, dc).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            Ok(())
        }
        Storage::Numeric(attr) => {
            // build a literal source field (zoned display) holding the decimal, then move it in.
            let attr = *attr;
            let (src, src_attr) = decimal_as_display(dec);
            let mut dst = field.bytes.clone();
            cob_move(&src, &src_attr, &mut dst, &attr).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            field.bytes = dst;
            Ok(())
        }
        Storage::Alpha(_) => {
            // numeric VALUE into alphanumeric: the digits as characters.
            let s: Vec<u8> = dec.digits.iter().map(|d| d + b'0').collect();
            store_alnum(field, &s)
        }
        Storage::Group { .. } => Err(RunError::Unsupported("a group MOVE is distributed across its leaves by write_field".into())),
        Storage::Condition { .. } => Err(RunError::Unsupported("cannot MOVE into an 88 condition-name".into())),
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
    let mut f = Field { storage: Storage::Numeric(attr), bytes: vec![b'0'; 9], occurs: 1, redefines: None };
    let mag: Vec<u8> = value.unsigned_abs().to_string().bytes().map(|b| b - b'0').collect();
    let _ = store_decimal(&mut f, &Decimal { negative: value < 0, digits: mag, scale: 0 });
    f
}

/// Format the RETURN-CODE register the way cobc DISPLAYs it: a leading `+`/`-` then 9 zero-padded digits
/// (`+000000042`, `+000000000`, `-000000007`).
fn display_return_code(f: &Field) -> Vec<u8> {
    let dec = match &f.storage {
        Storage::Numeric(a) => source_to_decimal(&f.bytes, a).ok(),
        _ => None,
    }
    .unwrap_or(Decimal { negative: false, digits: vec![0], scale: 0 });
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
        Storage::Alpha(attr) | Storage::Numeric(attr) => {
            let attr = *attr;
            let mut dst = field.bytes.clone();
            cob_move(src, &src_attr, &mut dst, &attr).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            field.bytes = dst;
            Ok(())
        }
        Storage::Group { .. } => Err(RunError::Unsupported("a group MOVE is distributed across its leaves by write_field".into())),
        Storage::Condition { .. } => Err(RunError::Unsupported("cannot MOVE into an 88 condition-name".into())),
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
        "SET" => exec_set(stmt, fields, ctx.decimal_comma),
        "INITIALIZE" => exec_initialize(stmt, fields, ctx.decimal_comma),
        "INSPECT" => exec_inspect(stmt, fields, ctx.decimal_comma),
        "STRING" => exec_string(stmt, fields, ctx.decimal_comma),
        "UNSTRING" => exec_unstring(stmt, fields),
        "ACCEPT" => exec_accept(stmt, fields),
        "OPEN" => exec_open(stmt, fields, ctx),
        "CLOSE" => exec_close(stmt, fields, ctx),
        "WRITE" => exec_write(stmt, fields, ctx),
        "REWRITE" => exec_rewrite(stmt, fields, ctx),
        "DELETE" => exec_delete(stmt, fields, ctx),
        "START" => exec_start(stmt, fields, ctx),
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
        // INITIATE/TERMINATE/SUPPRESS over the minimal report subset (DETAIL only) are no-ops.
        "INITIATE" | "TERMINATE" | "SUPPRESS" => Ok(()),
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
        // nondeterministic. Each fails closed with the specific reason (not a lazy TODO).
        "SEND" | "RECEIVE" | "PURGE" | "ENABLE" | "DISABLE" =>
            Err(RunError::Unsupported(format!("{verb}: message control requires a COMMUNICATION SECTION (CD); GnuCOBOL's CM is minimal and the front-end models WORKING-STORAGE / FILE / REPORT sections only"))),
        "MODIFY" | "INQUIRE" =>
            Err(RunError::Unsupported(format!("{verb}: an ACUCOBOL screen/GUI verb that requires a SCREEN SECTION the front-end does not model"))),
        "ALLOCATE" | "FREE" =>
            Err(RunError::Unsupported(format!("{verb}: BASED storage + POINTER -- the returned address is nondeterministic, so it is not oracle-reproducible"))),
        "USE" =>
            Err(RunError::Unsupported("USE: a DECLARATIVES error/exception handler; the front-end does not model the DECLARATIVES section or the file-not-found status that triggers it".into())),
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
    let callee = ctx.programs.get(&name).ok_or_else(|| {
        RunError::Unsupported(format!("CALL \"{name}\": not a contained program (external CALL is a boundary)"))
    })?;

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

    run_program_body(callee, ctx, &mut cfields, out)?;

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
fn exec_compute(stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<bool, RunError> {
    match exec_compute_inner(stmt, fields) {
        Ok(()) => Ok(false),
        Err(RunError::SizeError) => Ok(true),
        Err(e) => Err(e),
    }
}

fn exec_compute_inner(stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
    let eq = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "="))
        .ok_or_else(|| RunError::Unsupported("COMPUTE without '='".into()))?;
    // receivers = the names before '='; ROUNDED is not yet supported (fail closed).
    let mut receivers = Vec::new();
    for t in &stmt[..eq] {
        if let Tok::Word(w) = t {
            if w == "ROUNDED" {
                return Err(RunError::Unsupported("COMPUTE ... ROUNDED (deferred)".into()));
            }
            receivers.push(w.clone());
        }
    }
    if receivers.is_empty() {
        return Err(RunError::Unsupported("COMPUTE with no receiver".into()));
    }
    // tokenize the expression: split parentheses glued to operands; operators are space-separated.
    let mut etoks: Vec<String> = Vec::new();
    for t in &stmt[eq + 1..] {
        match t {
            Tok::Word(w) => split_parens(w, &mut etoks),
            Tok::Str(_) => return Err(RunError::Unsupported("string in COMPUTE".into())),
            Tok::Dot => {}
        }
    }
    let mut pos = 0;
    let (val, attr) = parse_expr(&etoks, &mut pos, fields)?;
    if pos != etoks.len() {
        return Err(RunError::Unsupported(format!("trailing tokens in COMPUTE expr at {}", etoks[pos])));
    }
    for r in receivers {
        let f = fields.get_mut(&r).ok_or_else(|| RunError::UndefinedName(r.clone()))?;
        // COMPUTE result is an already-decoded numeric value -> separator-independent store.
        move_into(f, &val, &attr, false)?;
    }
    Ok(())
}

/// Split a word into expression tokens, peeling leading `(` and trailing `)` (which may glue to an
/// operand, e.g. `(A` or `B)`); `**` / `+` / `-` / `*` / `/` and bare names pass through.
fn split_parens(w: &str, out: &mut Vec<String>) {
    let mut s = w;
    // Peel leading GROUPING '(' (e.g. `(E(1)` -> a group-open, then the operand).
    while let Some(rest) = s.strip_prefix('(') {
        out.push("(".into());
        s = rest;
    }
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
fn parse_expr(t: &[String], pos: &mut usize, f: &HashMap<String, Field>) -> Result<(Vec<u8>, FieldAttr), RunError> {
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
fn parse_term(t: &[String], pos: &mut usize, f: &HashMap<String, Field>) -> Result<(Vec<u8>, FieldAttr), RunError> {
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
                let wide = lit_num_attr(36, 18, true); // generous quotient scale; receiver store truncates.
                acc = cob_divide(&acc, &aattr, &b, &battr, &wide, Round::Truncate)
                    .map_err(map_arith_err)?;
                aattr = wide;
            }
            _ => break,
        }
    }
    Ok((acc, aattr))
}

/// `factor := primary ('**' integer)?` -- exponentiation by an integer literal (repeated multiply).
fn parse_factor(t: &[String], pos: &mut usize, f: &HashMap<String, Field>) -> Result<(Vec<u8>, FieldAttr), RunError> {
    let (base, battr) = parse_primary(t, pos, f)?;
    if t.get(*pos).map(|s| s.as_str()) == Some("**") {
        *pos += 1;
        let exp_word = t.get(*pos).ok_or_else(|| RunError::Unsupported("** without exponent".into()))?;
        *pos += 1;
        let e: u32 = exp_word.parse().map_err(|_| RunError::Unsupported(format!("** non-integer exponent {exp_word}")))?;
        // base ** e via repeated multiply; e==0 -> 1.
        if e == 0 {
            let (one, oa) = decimal_as_display(&Decimal { negative: false, digits: vec![1], scale: 0 });
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
    Ok((base, battr))
}

/// `primary := '(' expr ')' | '-' primary | operand`.
fn parse_primary(t: &[String], pos: &mut usize, f: &HashMap<String, Field>) -> Result<(Vec<u8>, FieldAttr), RunError> {
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
            let (zero, za) = decimal_as_display(&Decimal { negative: false, digits: vec![0], scale: 0 });
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
        None => Err(RunError::Unsupported("unexpected end of COMPUTE expr".into())),
    }
}

/// `DISPLAY op [op ...]` -- concatenate each operand's display bytes, then a newline.
fn exec_display(
    stmt: &[Tok],
    fields: &HashMap<String, Field>,
    out: &mut Vec<u8>,
    ctx: &Ctx,
) -> Result<(), RunError> {
    let mut operands: Vec<(Vec<u8>, FieldAttr)> = Vec::new();
    // `DISPLAY ... UPON PRINTER` (a built-in device mnemonic -- cobc accepts it even when SPECIAL-NAMES
    // does not declare it) is routed to the print redirect when active; UPON CONSOLE/SYSOUT and the
    // default stay on stdout.
    let mut upon_printer = false;
    let mut it = stmt.iter();
    while let Some(t) = it.next() {
        match t {
            Tok::Str(s) => operands.push((s.clone(), alnum_attr())),
            Tok::Word(w) => {
                if w == "UPON" {
                    if let Some(Tok::Word(dev)) = it.next() {
                        upon_printer = dev == "PRINTER";
                    }
                    continue;
                }
                if w == "WITH" || w == "NO" || w == "ADVANCING" {
                    // DISPLAY ... WITH NO ADVANCING handled below (no newline) -- mark it.
                    continue;
                }
                let f = read_field(fields, w)?.ok_or_else(|| RunError::UndefinedName(w.clone()))?;
                let bytes = if w == "RETURN-CODE" { display_return_code(&f) } else { display_bytes(&f, ctx.decimal_comma) };
                operands.push((bytes, alnum_attr()));
            }
            Tok::Dot => {}
        }
    }
    let no_adv = stmt.iter().any(|t| matches!(t, Tok::Word(w) if w=="ADVANCING"));
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
        Storage::Condition { .. } => Vec::new(), // a condition-name has no displayable value
    }
}

/// `MOVE src TO d1 [d2 ...]`.
fn exec_move(
    stmt: &[Tok],
    fields: &mut HashMap<String, Field>,
    decimal_comma: bool,
) -> Result<(), RunError> {
    // split at TO.
    let to = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w=="TO"))
        .ok_or_else(|| RunError::Unsupported("MOVE without TO".into()))?;
    let src_tok = stmt.first().ok_or_else(|| RunError::Unsupported("MOVE without source".into()))?;
    let dests: Vec<String> = stmt[to + 1..]
        .iter()
        .filter_map(|t| if let Tok::Word(w) = t { Some(w.clone()) } else { None })
        .collect();
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
fn exec_set(stmt: &[Tok], fields: &mut HashMap<String, Field>, decimal_comma: bool) -> Result<(), RunError> {
    // form: SET idx [idx ...] UP|DOWN BY n  (index arithmetic).
    if let Some(ud) = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "UP" || w == "DOWN")) {
        let up = matches!(stmt.get(ud), Some(Tok::Word(w)) if w == "UP");
        if !matches!(stmt.get(ud + 1), Some(Tok::Word(w)) if w == "BY") {
            return Err(RunError::Unsupported("SET ... UP/DOWN must be followed by BY".into()));
        }
        let amount = match stmt.get(ud + 2) {
            Some(Tok::Word(w)) => resolve_int(w, fields)
                .ok_or_else(|| RunError::Unsupported(format!("SET ... BY {w}: not an integer")))?,
            _ => return Err(RunError::Unsupported("SET ... BY: missing amount".into())),
        };
        for name in stmt[..ud].iter().filter_map(|t| if let Tok::Word(w) = t { Some(w.clone()) } else { None }) {
            let cur = resolve_int(&name, fields)
                .ok_or_else(|| RunError::Unsupported(format!("SET {name} UP/DOWN BY: not a numeric index")))?;
            let nv = if up { cur + amount } else { cur - amount };
            let mv = vec![Tok::Word(nv.to_string()), Tok::Word("TO".to_string()), Tok::Word(name)];
            exec_move(&mv, fields, decimal_comma)?;
        }
        return Ok(());
    }
    let to = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "TO"))
        .ok_or_else(|| RunError::Unsupported("SET subset is `SET name ... TO {TRUE|value}` / `SET idx UP|DOWN BY n`".into()))?;
    let targets: Vec<String> = stmt[..to].iter().filter_map(|t| match t {
        Tok::Word(w) => Some(w.clone()),
        _ => None,
    }).collect();
    if targets.is_empty() {
        return Err(RunError::Unsupported("SET: no target before TO".into()));
    }
    // form: SET idx [idx ...] TO value  (set an index/numeric item to a literal or another item's value).
    if !matches!(stmt.get(to + 1), Some(Tok::Word(w)) if w == "TRUE") {
        let src = stmt.get(to + 1).cloned()
            .ok_or_else(|| RunError::Unsupported("SET ... TO: missing value".into()))?;
        for name in &targets {
            match fields.get(name) {
                Some(Field { storage: Storage::Numeric(_), .. }) => {
                    let mv = vec![src.clone(), Tok::Word("TO".to_string()), Tok::Word(name.clone())];
                    exec_move(&mv, fields, decimal_comma)?;
                }
                Some(Field { storage: Storage::Condition { .. }, .. }) =>
                    return Err(RunError::Unsupported(format!("SET {name} TO <value>: an 88 condition-name is only `SET ... TO TRUE`"))),
                Some(_) => return Err(RunError::Unsupported(format!("SET {name} TO <value>: target is not a numeric/index item"))),
                None => return Err(RunError::UndefinedName(name.clone())),
            }
        }
        return Ok(());
    }
    // form: SET cond-name [cond-name ...] TO TRUE  (LEVEL-88 construction).
    for name in targets {
        let (parent, setword) = match fields.get(&name) {
            Some(Field { storage: Storage::Condition { parent, values }, .. }) => {
                let v = values.first()
                    .ok_or_else(|| RunError::Unsupported(format!("88 {name} has no VALUE to SET")))?;
                let w = match v { CondVal::Single(s) => s.clone(), CondVal::Range(lo, _) => lo.clone() };
                (parent.clone(), w)
            }
            Some(_) => return Err(RunError::Unsupported(format!("SET {name} TO TRUE: not an 88 condition-name"))),
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

/// `INITIALIZE item [item ...]` -- reset each named item to its category default, matching `cobc`'s
/// no-`REPLACING` INITIALIZE: a numeric item becomes ZERO, an alphanumeric/edited item becomes SPACES.
/// (VALUE clauses are deliberately NOT used, per the standard.) The subset is elementary items; the
/// `REPLACING`/`WITH`/group-traversal forms fail closed.
fn exec_initialize(stmt: &[Tok], fields: &mut HashMap<String, Field>, decimal_comma: bool) -> Result<(), RunError> {
    let mut names: Vec<String> = Vec::new();
    for t in stmt {
        match t {
            Tok::Word(w) if w == "REPLACING" || w == "WITH" || w == "ALL" || w == "THRU" || w == "TO" || w == "FILLER" => {
                return Err(RunError::Unsupported(format!("INITIALIZE ... {w} (subset: elementary items, no REPLACING/WITH)")));
            }
            Tok::Word(w) => names.push(w.clone()),
            _ => {}
        }
    }
    if names.is_empty() {
        return Err(RunError::Unsupported("INITIALIZE: no item named".into()));
    }
    for name in names {
        let kind = match fields.get(&name) {
            Some(f) => f.storage.clone(),
            None => return Err(RunError::UndefinedName(name)),
        };
        let src = match kind {
            Storage::Numeric(_) => Tok::Word("0".to_string()),
            Storage::Alpha(_) | Storage::Edited(..) => Tok::Str(vec![b' ']),
            // an 88 condition-name has no storage of its own -- INITIALIZE skips it (cobc does not reset it).
            Storage::Condition { .. } => continue,
            // INITIALIZE of a group resets each of its leaves to its category default.
            Storage::Group { children } => {
                let kids = children.clone();
                for c in kids {
                    exec_initialize(&[Tok::Word(c)], fields, decimal_comma)?;
                }
                continue;
            }
        };
        let mv = vec![src, Tok::Word("TO".to_string()), Tok::Word(name)];
        exec_move(&mv, fields, decimal_comma)?;
    }
    Ok(())
}

/// An `INSPECT` comparand operand -> its bytes: a string literal, the figuratives `SPACE`/`ZERO` (a single
/// character), or an identifier's current bytes. (Other figuratives/forms fail closed.)
fn inspect_operand(t: Option<&Tok>, fields: &HashMap<String, Field>) -> Result<Vec<u8>, RunError> {
    match t {
        Some(Tok::Str(s)) => Ok(s.clone()),
        Some(Tok::Word(w)) => match w.as_str() {
            "SPACE" | "SPACES" => Ok(vec![b' ']),
            "ZERO" | "ZEROS" | "ZEROES" => Ok(vec![b'0']),
            _ => match read_field(fields, w)? {
                Some(f) => Ok(f.bytes.clone()),
                None => Err(RunError::Unsupported(format!("INSPECT operand `{w}` (subset: literal / SPACE / ZERO / identifier)"))),
            },
        },
        _ => Err(RunError::Unsupported("INSPECT: missing operand".into())),
    }
}

/// Parse a trailing `INSPECT` region clause into `(kind, delim)` -- `0`=whole, `1`=`BEFORE INITIAL d`,
/// `2`=`AFTER INITIAL d` -- returning the delimiter bytes owned so the caller can build a `Region`.
fn inspect_region(rest: &[Tok], fields: &HashMap<String, Field>) -> Result<(u8, Vec<u8>), RunError> {
    match rest.first() {
        None => Ok((0, Vec::new())),
        Some(Tok::Word(w)) if w == "BEFORE" || w == "AFTER" => {
            let kind = if w == "AFTER" { 2 } else { 1 };
            let mut i = 1;
            if matches!(rest.get(i), Some(Tok::Word(x)) if x == "INITIAL") { i += 1; }
            Ok((kind, inspect_operand(rest.get(i), fields)?))
        }
        Some(t) => Err(RunError::Unsupported(format!("INSPECT region clause near {t:?}"))),
    }
}

/// `INSPECT target {TALLYING counter FOR <ALL|LEADING> lit | FOR CHARACTERS [region] | REPLACING
/// <ALL|LEADING|FIRST> x BY y [region] | CONVERTING from TO to [region]}` -- the byte effects of the
/// sealed `GNURUST.INSPECT.1` court. A single clause is in the subset; multi-clause/`ALL`-counter-chains
/// and figurative ranges fail closed.
fn exec_inspect(stmt: &[Tok], fields: &mut HashMap<String, Field>, decimal_comma: bool) -> Result<(), RunError> {
    use crate::inspect::{inspect_converting, inspect_replacing, inspect_tallying, Region, ReplaceMode, TallyMode};
    let target = match stmt.first() {
        Some(Tok::Word(w)) => w.clone(),
        _ => return Err(RunError::Unsupported("INSPECT: missing target".into())),
    };
    let target_bytes = read_field(fields, &target)?
        .ok_or_else(|| RunError::UndefinedName(target.clone()))?
        .bytes;
    match stmt.get(1) {
        Some(Tok::Word(w)) if w == "TALLYING" => {
            let counter = match stmt.get(2) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("INSPECT TALLYING: missing counter".into())) };
            if !matches!(stmt.get(3), Some(Tok::Word(w)) if w == "FOR") {
                return Err(RunError::Unsupported("INSPECT TALLYING: expected FOR".into()));
            }
            let modekw = match stmt.get(4) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("INSPECT TALLYING: missing FOR mode".into())) };
            let (item, rstart) = match modekw.as_str() {
                "CHARACTERS" => (Vec::new(), 5),
                "ALL" | "LEADING" => (inspect_operand(stmt.get(5), fields)?, 6),
                other => return Err(RunError::Unsupported(format!("INSPECT TALLYING FOR {other} (subset: ALL/LEADING/CHARACTERS)"))),
            };
            let (rk, d) = inspect_region(&stmt[rstart.min(stmt.len())..], fields)?;
            let region = match rk { 1 => Region::Before(&d), 2 => Region::After(&d), _ => Region::All };
            let mode = match modekw.as_str() {
                "CHARACTERS" => TallyMode::Characters,
                "ALL" => TallyMode::All(&item),
                _ => TallyMode::Leading(&item),
            };
            let count = inspect_tallying(&target_bytes, mode, region) as i64;
            let nv = resolve_int(&counter, fields).unwrap_or(0) + count;
            let mv = vec![Tok::Word(nv.to_string()), Tok::Word("TO".to_string()), Tok::Word(counter)];
            exec_move(&mv, fields, decimal_comma)
        }
        Some(Tok::Word(w)) if w == "REPLACING" => {
            let modekw = match stmt.get(2) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("INSPECT REPLACING: missing mode".into())) };
            if !matches!(modekw.as_str(), "ALL" | "LEADING" | "FIRST") {
                return Err(RunError::Unsupported(format!("INSPECT REPLACING {modekw} (subset: ALL/LEADING/FIRST x BY y)")));
            }
            let x = inspect_operand(stmt.get(3), fields)?;
            if !matches!(stmt.get(4), Some(Tok::Word(w)) if w == "BY") {
                return Err(RunError::Unsupported("INSPECT REPLACING: expected BY".into()));
            }
            let y = inspect_operand(stmt.get(5), fields)?;
            let (rk, d) = inspect_region(&stmt[6.min(stmt.len())..], fields)?;
            let region = match rk { 1 => Region::Before(&d), 2 => Region::After(&d), _ => Region::All };
            let mode = match modekw.as_str() {
                "ALL" => ReplaceMode::All(&x, &y),
                "LEADING" => ReplaceMode::Leading(&x, &y),
                _ => ReplaceMode::First(&x, &y),
            };
            let newb = inspect_replacing(&target_bytes, mode, region);
            write_field(fields, &target, |f| {
                if f.bytes.len() == newb.len() { f.bytes = newb; Ok(()) }
                else { Err(RunError::Runtime("INSPECT REPLACING changed field length".into())) }
            })
        }
        Some(Tok::Word(w)) if w == "CONVERTING" => {
            let from = inspect_operand(stmt.get(2), fields)?;
            if !matches!(stmt.get(3), Some(Tok::Word(w)) if w == "TO") {
                return Err(RunError::Unsupported("INSPECT CONVERTING: expected TO".into()));
            }
            let to = inspect_operand(stmt.get(4), fields)?;
            let (rk, d) = inspect_region(&stmt[5.min(stmt.len())..], fields)?;
            let region = match rk { 1 => Region::Before(&d), 2 => Region::After(&d), _ => Region::All };
            let newb = inspect_converting(&target_bytes, &from, &to, region);
            write_field(fields, &target, |f| {
                if f.bytes.len() == newb.len() { f.bytes = newb; Ok(()) }
                else { Err(RunError::Runtime("INSPECT CONVERTING changed field length".into())) }
            })
        }
        other => Err(RunError::Unsupported(format!("INSPECT clause {other:?} (subset: TALLYING/REPLACING/CONVERTING)"))),
    }
}

/// `EXHIBIT [NAMED] name [name ...]` -- the OS/VS debug display: each item as `NAME = <display value>`,
/// space-joined, one line. `CHANGED` (display-only-if-changed) is out of subset.
fn exec_exhibit(stmt: &[Tok], fields: &mut HashMap<String, Field>, out: &mut Vec<u8>, ctx: &Ctx) -> Result<(), RunError> {
    let mut i = 0;
    while let Some(Tok::Word(w)) = stmt.get(i) {
        match w.as_str() {
            "CHANGED" => return Err(RunError::Unsupported("EXHIBIT CHANGED not in subset".into())),
            "NAMED" => i += 1,
            _ => break,
        }
    }
    let names: Vec<String> = stmt[i..].iter().filter_map(|t| if let Tok::Word(w) = t { Some(w.clone()) } else { None }).collect();
    let mut line = Vec::new();
    for (j, name) in names.iter().enumerate() {
        if j > 0 { line.push(b' '); }
        line.extend_from_slice(name.as_bytes());
        line.extend_from_slice(b" = ");
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
    let words: Vec<String> = stmt.iter().filter_map(|t| if let Tok::Word(w) = t { Some(w.clone()) } else { None }).collect();
    let mut i = 0;
    while i < words.len() {
        let para = words[i].clone();
        i += 1;
        while i < words.len() && (words[i] == "TO" || words[i] == "PROCEED") { i += 1; }
        if i >= words.len() { break; }
        let target = words[i].clone();
        i += 1;
        if let Some((start, end)) = para_range(&para, &para) {
            for gi in start..end.min(proc.len()) {
                if matches!(proc.get(gi), Some(Tok::Word(w)) if w == "GO") {
                    ALTERED.with(|c| { c.borrow_mut().insert(gi, target.clone()); });
                    break;
                }
            }
        }
    }
    Ok(())
}

/// `EXAMINE id TALLYING {ALL|LEADING|UNTIL FIRST} lit [REPLACING BY lit2]` / `EXAMINE id REPLACING
/// {ALL|LEADING|FIRST} lit BY lit2` -- the COBOL-68 precursor of INSPECT (an OS/VS dialect verb). TALLYING
/// sets the `TALLY` register; reuses the sealed INSPECT TALLYING/REPLACING courts.
fn exec_examine(stmt: &[Tok], fields: &mut HashMap<String, Field>, decimal_comma: bool) -> Result<(), RunError> {
    use crate::inspect::{inspect_replacing, inspect_tallying, Region, ReplaceMode, TallyMode};
    let target = match stmt.first() { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("EXAMINE: missing field".into())) };
    let tbytes = read_field(fields, &target)?.map(|f| f.bytes).unwrap_or_default();
    let pos_of = |kw: &str| stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == kw));
    let write_target = |fields: &mut HashMap<String, Field>, newb: Vec<u8>| -> Result<(), RunError> {
        write_field(fields, &target, |f| {
            if f.bytes.len() == newb.len() { f.bytes = newb; Ok(()) }
            else { Err(RunError::Runtime("EXAMINE changed field length".into())) }
        })
    };
    if let Some(tp) = pos_of("TALLYING") {
        let mut i = tp + 1;
        let modekw = match stmt.get(i) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("EXAMINE TALLYING mode".into())) };
        i += 1;
        if modekw == "UNTIL" && matches!(stmt.get(i), Some(Tok::Word(w)) if w == "FIRST") { i += 1; }
        let lit = inspect_operand(stmt.get(i), fields)?;
        let tmode = match modekw.as_str() {
            "ALL" => TallyMode::All(&lit),
            "LEADING" => TallyMode::Leading(&lit),
            "UNTIL" => TallyMode::Characters,
            other => return Err(RunError::Unsupported(format!("EXAMINE TALLYING {other}"))),
        };
        let region = if modekw == "UNTIL" { Region::Before(&lit) } else { Region::All };
        let count = inspect_tallying(&tbytes, tmode, region) as i64;
        let mv = vec![Tok::Word(count.to_string()), Tok::Word("TO".to_string()), Tok::Word("TALLY".to_string())];
        exec_move(&mv, fields, decimal_comma)?;
        if let Some(rp) = stmt[tp..].iter().position(|t| matches!(t, Tok::Word(w) if w == "REPLACING")) {
            let mut j = tp + rp + 1;
            if matches!(stmt.get(j), Some(Tok::Word(w)) if w == "BY") { j += 1; }
            let lit2 = inspect_operand(stmt.get(j), fields)?;
            let rmode = match modekw.as_str() {
                "ALL" => ReplaceMode::All(&lit, &lit2),
                "LEADING" => ReplaceMode::Leading(&lit, &lit2),
                _ => return Err(RunError::Unsupported("EXAMINE TALLYING UNTIL ... REPLACING not in subset".into())),
            };
            let newb = inspect_replacing(&tbytes, rmode, Region::All);
            write_target(fields, newb)?;
        }
        return Ok(());
    }
    if let Some(rp) = pos_of("REPLACING") {
        let mut i = rp + 1;
        let modekw = match stmt.get(i) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("EXAMINE REPLACING mode".into())) };
        i += 1;
        if modekw == "UNTIL" && matches!(stmt.get(i), Some(Tok::Word(w)) if w == "FIRST") { i += 1; }
        let lit = inspect_operand(stmt.get(i), fields)?;
        i += 1;
        if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "BY") { i += 1; }
        let lit2 = inspect_operand(stmt.get(i), fields)?;
        let rmode = match modekw.as_str() {
            "ALL" => ReplaceMode::All(&lit, &lit2),
            "LEADING" => ReplaceMode::Leading(&lit, &lit2),
            "FIRST" => ReplaceMode::First(&lit, &lit2),
            other => return Err(RunError::Unsupported(format!("EXAMINE REPLACING {other}"))),
        };
        let newb = inspect_replacing(&tbytes, rmode, Region::All);
        write_target(fields, newb)?;
        return Ok(());
    }
    Err(RunError::Unsupported("EXAMINE: expected TALLYING or REPLACING".into()))
}

/// `TRANSFORM target FROM from TO to` -- the legacy form of `INSPECT target CONVERTING from TO to` (a
/// per-byte translation), reusing the sealed CONVERTING court.
fn exec_transform(stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
    use crate::inspect::{inspect_converting, Region};
    let target = match stmt.first() { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("TRANSFORM: missing target".into())) };
    let fp = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "FROM")).ok_or_else(|| RunError::Unsupported("TRANSFORM without FROM".into()))?;
    let from = inspect_operand(stmt.get(fp + 1), fields)?;
    let tp = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "TO")).ok_or_else(|| RunError::Unsupported("TRANSFORM without TO".into()))?;
    let to = inspect_operand(stmt.get(tp + 1), fields)?;
    let tb = read_field(fields, &target)?.map(|f| f.bytes).unwrap_or_default();
    let newb = inspect_converting(&tb, &from, &to, Region::All);
    write_field(fields, &target, |f| {
        if f.bytes.len() == newb.len() { f.bytes = newb; Ok(()) }
        else { Err(RunError::Runtime("TRANSFORM changed field length".into())) }
    })
}

/// `STRING <src [DELIMITED BY SIZE|lit]> ... INTO target [WITH POINTER p]` -- concatenate the sources into
/// the target at the 1-based pointer, preserving the unwritten tail (`GNURUST.STRING.UNSTRING.1`). The
/// `ON OVERFLOW` handler form is outside the subset (fails closed).
fn exec_string(stmt: &[Tok], fields: &mut HashMap<String, Field>, decimal_comma: bool) -> Result<(), RunError> {
    use crate::string_ops::{string_into, StringSource};
    if stmt.iter().any(|t| matches!(t, Tok::Word(w) if w == "OVERFLOW")) {
        return Err(RunError::Unsupported("STRING ... ON OVERFLOW not in subset".into()));
    }
    let into = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "INTO"))
        .ok_or_else(|| RunError::Unsupported("STRING without INTO".into()))?;
    let target = match stmt.get(into + 1) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("STRING: missing target".into())) };
    // optional WITH POINTER p (after the target)
    let mut pointer_name: Option<String> = None;
    let mut pointer = 1usize;
    if let Some(rel) = stmt[into + 2..].iter().position(|t| matches!(t, Tok::Word(w) if w == "POINTER")) {
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
                if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "BY") { i += 1; }
                let delim = match stmt.get(i) {
                    Some(Tok::Word(w)) if w == "SIZE" => None,
                    other => Some(inspect_operand(other, fields)?),
                };
                i += 1;
                for op in pending.drain(..) { srcs.push((op, delim.clone())); }
            }
            t => { pending.push(inspect_operand(Some(t), fields)?); i += 1; }
        }
    }
    for op in pending.drain(..) { srcs.push((op, None)); }
    let ss: Vec<StringSource> = srcs.iter().map(|(b, d)| match d {
        Some(d) => StringSource::Delimited(b, d),
        None => StringSource::Size(b),
    }).collect();
    let prefill = read_field(fields, &target)?.ok_or_else(|| RunError::UndefinedName(target.clone()))?.bytes;
    let res = string_into(&prefill, &ss, pointer);
    let newb = res.target;
    write_field(fields, &target, |f| {
        if f.bytes.len() == newb.len() { f.bytes = newb; Ok(()) }
        else { Err(RunError::Runtime("STRING changed target length".into())) }
    })?;
    if let Some(pn) = pointer_name {
        let mv = vec![Tok::Word(res.pointer.to_string()), Tok::Word("TO".to_string()), Tok::Word(pn)];
        exec_move(&mv, fields, decimal_comma)?;
    }
    Ok(())
}

/// `UNSTRING source [DELIMITED BY d] INTO f1 f2 ...` -- split the source by the delimiter (or by each
/// receiver's width when absent) into the alphanumeric receiving fields (`GNURUST.STRING.UNSTRING.1`).
/// `DELIMITER IN` / `COUNT IN` / `TALLYING` / `POINTER` / `OVERFLOW` and multi-delimiter forms fail closed.
fn exec_unstring(stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
    use crate::string_ops::unstring;
    for bad in ["DELIMITER", "COUNT", "TALLYING", "POINTER", "OVERFLOW", "OR"] {
        if stmt.iter().any(|t| matches!(t, Tok::Word(w) if w == bad)) {
            return Err(RunError::Unsupported(format!("UNSTRING ... {bad} not in subset")));
        }
    }
    let into = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "INTO"))
        .ok_or_else(|| RunError::Unsupported("UNSTRING without INTO".into()))?;
    let source = inspect_operand(stmt.first(), fields)?;
    // optional DELIMITED BY d between the source and INTO
    let mut delim: Option<Vec<u8>> = None;
    if let Some(dp) = stmt[..into].iter().position(|t| matches!(t, Tok::Word(w) if w == "DELIMITED")) {
        let mut j = dp + 1;
        if matches!(stmt.get(j), Some(Tok::Word(w)) if w == "BY") { j += 1; }
        delim = Some(inspect_operand(stmt.get(j), fields)?);
    }
    let recv: Vec<String> = stmt[into + 1..].iter().filter_map(|t| if let Tok::Word(w) = t { Some(w.clone()) } else { None }).collect();
    if recv.is_empty() {
        return Err(RunError::Unsupported("UNSTRING: no receiving field".into()));
    }
    let mut sizes = Vec::with_capacity(recv.len());
    for n in &recv {
        let f = read_field(fields, n)?.ok_or_else(|| RunError::UndefinedName(n.clone()))?;
        if !matches!(f.storage, Storage::Alpha(_)) {
            return Err(RunError::Unsupported(format!("UNSTRING into non-alphanumeric `{n}` not in subset")));
        }
        sizes.push(f.bytes.len());
    }
    let res = unstring(&source, delim.as_deref(), &sizes, 1);
    for (n, fld) in recv.iter().zip(res.fields.iter()) {
        let data = fld.data.clone();
        write_field(fields, n, |f| { f.bytes = data; Ok(()) })?;
    }
    Ok(())
}

/// Day-of-year (1..366) for a Gregorian `(year, month, day)`.
fn day_of_year(y: i32, m: i32, d: i32) -> i32 {
    let mdays = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let mut s = 0;
    for (i, &md) in mdays.iter().enumerate().take((m - 1).max(0) as usize) {
        s += md;
        if i == 1 && leap { s += 1; }
    }
    s + d
}

/// COBOL `DAY-OF-WEEK` (1 = Monday .. 7 = Sunday) for a Gregorian date (Sakamoto's algorithm).
fn day_of_week(y: i32, m: i32, d: i32) -> i32 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { y - 1 } else { y };
    let w = (yy + yy / 4 - yy / 100 + yy / 400 + t[(m - 1).max(0) as usize] + d) % 7; // 0 = Sunday
    if w == 0 { 7 } else { w }
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
        return Err(RunError::Unsupported("ACCEPT subset is `ACCEPT id FROM DATE|DAY|TIME|DAY-OF-WEEK` (terminal input not modelled)".into()));
    }
    let src = match stmt.get(2) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("ACCEPT FROM: missing source".into())) };
    let long_year = matches!(stmt.get(3), Some(Tok::Word(w)) if w == "YYYYMMDD" || w == "YYYYDDD");
    let raw = std::env::var("COB_CURRENT_DATE").map_err(|_| RunError::Unsupported(
        "ACCEPT FROM DATE/TIME requires a pinned COB_CURRENT_DATE (the live clock is a non-claim)".into()))?;
    let cd = crate::common_signal::check_current_date(raw.as_bytes());
    if cd.invalid || cd.year < 0 || cd.month < 0 || cd.day < 0 {
        return Err(RunError::Runtime("COB_CURRENT_DATE did not parse to a full date".into()));
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
        other => return Err(RunError::Unsupported(format!("ACCEPT FROM {other} (subset: DATE/DAY/TIME/DAY-OF-WEEK)"))),
    };
    let s = digits.into_bytes();
    let n = s.len();
    write_field(fields, &target, |f| match &f.storage {
        Storage::Numeric(_) | Storage::Alpha(_) if f.bytes.len() == n => { f.bytes = s; Ok(()) }
        _ => Err(RunError::Unsupported(format!("ACCEPT FROM {src}: receiver must be a {n}-digit numeric/alphanumeric item"))),
    })
}

/// The in-memory file-store key for a COBOL file name: its ASSIGN target (so two SELECTs on the same
/// physical name share storage), falling back to the name.
fn fkey(ctx: &Ctx, name: &str) -> String {
    ctx.file_defs.get(name).map(|d| d.assign.clone()).filter(|a| !a.is_empty()).unwrap_or_else(|| name.to_string())
}

/// Set a file's `FILE STATUS` field (if declared) to a 2-character code (`"00"` ok, `"10"` end-of-file).
fn set_file_status(fields: &mut HashMap<String, Field>, def: &FileDef, code: &str) {
    if let Some(s) = &def.status {
        let mv = vec![Tok::Str(code.as_bytes().to_vec()), Tok::Word("TO".to_string()), Tok::Word(s.clone())];
        let _ = exec_move(&mv, fields, false);
    }
}

/// `OPEN {INPUT|OUTPUT|EXTEND|I-O} file [file ...]` -- set each file's open mode (OUTPUT truncates the
/// logical file). The subset is a single mode keyword per statement.
fn exec_open(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx) -> Result<(), RunError> {
    let mode = match stmt.first() {
        Some(Tok::Word(w)) => match w.as_str() {
            "INPUT" => 1u8, "OUTPUT" => 2, "EXTEND" => 3, "I-O" => 4,
            other => return Err(RunError::Unsupported(format!("OPEN {other} (subset: INPUT/OUTPUT/EXTEND/I-O)"))),
        },
        _ => return Err(RunError::Unsupported("OPEN: missing mode".into())),
    };
    for name in stmt[1..].iter().filter_map(|t| if let Tok::Word(w) = t { Some(w.clone()) } else { None }) {
        let def = ctx.file_defs.get(&name).ok_or_else(|| RunError::Unsupported(format!("OPEN: `{name}` is not a declared file")))?;
        {
            let mut files = ctx.files.borrow_mut();
            let st = files.entry(fkey(ctx, &name)).or_default();
            if mode == 2 { st.records.clear(); }
            st.read_pos = 0;
            st.mode = mode;
        }
        set_file_status(fields, def, "00");
    }
    Ok(())
}

/// `CLOSE file [file ...]` -- mark each file closed (its logical records persist so a later OPEN INPUT can
/// re-read them within the same run).
fn exec_close(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx) -> Result<(), RunError> {
    for name in stmt.iter().filter_map(|t| if let Tok::Word(w) = t { Some(w.clone()) } else { None }) {
        let def = ctx.file_defs.get(&name).ok_or_else(|| RunError::Unsupported(format!("CLOSE: `{name}` is not a declared file")))?;
        if let Some(st) = ctx.files.borrow_mut().get_mut(&fkey(ctx, &name)) { st.mode = 0; }
        set_file_status(fields, def, "00");
    }
    Ok(())
}

/// `WRITE record [FROM id]` -- append the record's current bytes to its file (LINE SEQUENTIAL trims trailing
/// spaces, matching the oracle). The operand is the FD record name.
fn exec_write(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx) -> Result<(), RunError> {
    let rec = match stmt.first() { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("WRITE: missing record".into())) };
    // optional FROM id: MOVE id into the record first.
    if let Some(fp) = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "FROM")) {
        if let Some(src) = stmt.get(fp + 1) {
            let mv = vec![src.clone(), Tok::Word("TO".to_string()), Tok::Word(rec.clone())];
            exec_move(&mv, fields, ctx.decimal_comma)?;
        }
    }
    let def = ctx.file_defs.values().find(|d| d.record == rec)
        .ok_or_else(|| RunError::Unsupported(format!("WRITE `{rec}`: not an FD record")))?
        .clone();
    let mut bytes = read_field(fields, &rec)?.map(|f| f.bytes).unwrap_or_default();
    if def.org == FileOrg::LineSequential {
        while bytes.last() == Some(&b' ') { bytes.pop(); }
    }
    if def.org == FileOrg::Relative {
        // place the record at the 1-based RELATIVE KEY position (empty slots = absent records).
        let pos = relative_key_value(&def, fields)?;
        let mut files = ctx.files.borrow_mut();
        let st = files.entry(def.assign.clone()).or_default();
        if st.records.len() < pos { st.records.resize(pos, Vec::new()); }
        let occupied = !st.records[pos - 1].is_empty();
        if !occupied { st.records[pos - 1] = bytes; }
        drop(files);
        set_file_status(fields, &def, if occupied { "22" } else { "00" });
        return Ok(());
    }
    ctx.files.borrow_mut().entry(def.assign.clone()).or_default().records.push(bytes);
    set_file_status(fields, &def, "00");
    Ok(())
}

/// The current 1-based value of a RELATIVE file's RELATIVE KEY field (>= 1 required).
fn relative_key_value(def: &FileDef, fields: &HashMap<String, Field>) -> Result<usize, RunError> {
    let key = def.rel_key.as_ref()
        .ok_or_else(|| RunError::Unsupported(format!("RELATIVE file `{}` has no RELATIVE KEY", def.name)))?;
    let v = resolve_int(key, fields)
        .ok_or_else(|| RunError::Unsupported(format!("RELATIVE KEY `{key}` is not an integer")))?;
    if v < 1 {
        return Err(RunError::Runtime(format!("RELATIVE KEY `{key}` = {v} (< 1)")));
    }
    Ok(v as usize)
}

/// `REWRITE record [FROM id]` -- replace the record last READ (under OPEN I-O) with the record buffer's
/// current bytes. With no current record (no prior READ) it fails with status `"43"`.
fn exec_rewrite(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx) -> Result<(), RunError> {
    let rec = match stmt.first() { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("REWRITE: missing record".into())) };
    if let Some(fp) = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "FROM")) {
        if let Some(src) = stmt.get(fp + 1) {
            let mv = vec![src.clone(), Tok::Word("TO".to_string()), Tok::Word(rec.clone())];
            exec_move(&mv, fields, ctx.decimal_comma)?;
        }
    }
    let def = ctx.file_defs.values().find(|d| d.record == rec)
        .ok_or_else(|| RunError::Unsupported(format!("REWRITE `{rec}`: not an FD record")))?
        .clone();
    let mut bytes = read_field(fields, &rec)?.map(|f| f.bytes).unwrap_or_default();
    if def.org == FileOrg::LineSequential {
        while bytes.last() == Some(&b' ') { bytes.pop(); }
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
fn exec_unlock(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx) -> Result<(), RunError> {
    for name in stmt.iter().filter_map(|t| if let Tok::Word(w) = t { Some(w.clone()) } else { None }) {
        let def = ctx.file_defs.get(&name).ok_or_else(|| RunError::Unsupported(format!("UNLOCK: `{name}` is not a declared file")))?;
        set_file_status(fields, def, "00");
    }
    Ok(())
}

/// `SORT sd-file ON {ASCENDING|DESCENDING} KEY key USING in... GIVING out...` (and `MERGE`, same shape) --
/// read every record from the USING files, order them, and write them to the GIVING files. The subset is a
/// single **whole-record** KEY (sub-field keys need group items) with USING/GIVING; INPUT/OUTPUT PROCEDURE
/// (which drive RELEASE/RETURN) is out of subset.
fn exec_sort(stmt: &[Tok], fields: &mut HashMap<String, Field>, out: &mut Vec<u8>, ctx: &Ctx) -> Result<(), RunError> {
    let sf = match stmt.first() { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("SORT: missing sort file".into())) };
    let sd_def = ctx.file_defs.get(&sf).ok_or_else(|| RunError::Unsupported(format!("SORT: `{sf}` is not a declared file")))?.clone();
    let reclen = read_field(fields, &sd_def.record)?.map(|f| f.bytes.len()).unwrap_or(0);
    let kw = |w: &str| matches!(w, "ON" | "KEY" | "ASCENDING" | "DESCENDING" | "USING" | "GIVING" | "INPUT" | "OUTPUT" | "PROCEDURE" | "IS" | "THRU" | "THROUGH");
    let (mut descending, mut keys): (bool, Vec<String>) = (false, vec![]);
    let (mut using, mut giving): (Vec<String>, Vec<String>) = (vec![], vec![]);
    let (mut in_proc, mut out_proc): (Option<(String, String)>, Option<(String, String)>) = (None, None);
    let word = |i: usize| stmt.get(i).and_then(|t| if let Tok::Word(w) = t { Some(w.clone()) } else { None });
    let mut i = 1;
    while i < stmt.len() {
        match word(i).as_deref() {
            Some("ON") | Some("KEY") => i += 1,
            Some("ASCENDING") => { descending = false; i += 1; }
            Some("DESCENDING") => { descending = true; i += 1; }
            Some("USING") => { i += 1; while let Some(w) = word(i) { if kw(&w) { break; } using.push(w); i += 1; } }
            Some("GIVING") => { i += 1; while let Some(w) = word(i) { if kw(&w) { break; } giving.push(w); i += 1; } }
            Some("INPUT") | Some("OUTPUT") => {
                let is_in = word(i).as_deref() == Some("INPUT");
                i += 1;
                if word(i).as_deref() == Some("PROCEDURE") { i += 1; }
                if word(i).as_deref() == Some("IS") { i += 1; }
                let p1 = word(i).unwrap_or_default();
                i += 1;
                let mut p2 = p1.clone();
                if matches!(word(i).as_deref(), Some("THRU") | Some("THROUGH")) {
                    i += 1;
                    if let Some(w) = word(i) { p2 = w; i += 1; }
                }
                if is_in { in_proc = Some((p1, p2)); } else { out_proc = Some((p1, p2)); }
            }
            Some(w) => { keys.push(w.to_string()); i += 1; }
            None => i += 1,
        }
    }
    if keys.len() != 1 {
        return Err(RunError::Unsupported("SORT/MERGE subset: a single KEY".into()));
    }
    let (key_off, key_len) = sort_key_span(&sd_def.record, &keys[0], reclen, fields)
        .ok_or_else(|| RunError::Unsupported(format!("SORT/MERGE KEY `{}` is not a field of the sort record", keys[0])))?;
    // the current body's tokens, for running INPUT/OUTPUT PROCEDURE ranges.
    let proc = CUR_PROC.with(|c| c.borrow().clone());
    // ---- gather phase: INPUT PROCEDURE (RELEASE records into the sort file) or USING files ----
    let mut recs: Vec<Vec<u8>> = Vec::new();
    if let Some((p1, p2)) = &in_proc {
        ctx.files.borrow_mut().entry(fkey(ctx, &sf)).or_default().records.clear();
        let (start, end) = para_range(p1, p2)
            .ok_or_else(|| RunError::Unsupported(format!("SORT INPUT PROCEDURE: unknown paragraph `{p1}`")))?;
        run_range(&proc, start, end, fields, out, ctx)?;
        recs = ctx.files.borrow().get(&fkey(ctx, &sf)).map(|st| st.records.clone()).unwrap_or_default();
    } else if !using.is_empty() {
        let files = ctx.files.borrow();
        for f in &using {
            if let Some(st) = files.get(&fkey(ctx, f)) {
                for r in &st.records {
                    if r.is_empty() { continue; }
                    recs.push(r.clone());
                }
            }
        }
    } else {
        return Err(RunError::Unsupported("SORT/MERGE requires USING or INPUT PROCEDURE".into()));
    }
    for r in recs.iter_mut() { r.resize(reclen, b' '); }
    recs.sort_by(|a, b| {
        let ea = (key_off + key_len).min(a.len());
        let eb = (key_off + key_len).min(b.len());
        a[key_off.min(a.len())..ea].cmp(&b[key_off.min(b.len())..eb])
    });
    if descending { recs.reverse(); }
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
        let (start, end) = para_range(p3, p4)
            .ok_or_else(|| RunError::Unsupported(format!("SORT OUTPUT PROCEDURE: unknown paragraph `{p3}`")))?;
        run_range(&proc, start, end, fields, out, ctx)?;
    } else if !giving.is_empty() {
        let mut files = ctx.files.borrow_mut();
        for f in &giving {
            let gorg = ctx.file_defs.get(f).map(|d| d.org).unwrap_or(FileOrg::Sequential);
            let st = files.entry(fkey(ctx, f)).or_default();
            st.records = recs.iter().map(|r| {
                let mut b = r.clone();
                if gorg == FileOrg::LineSequential { while b.last() == Some(&b' ') { b.pop(); } }
                b
            }).collect();
            st.read_pos = 0;
            st.mode = 0;
        }
        set_file_status(fields, &sd_def, "00");
    } else {
        return Err(RunError::Unsupported("SORT/MERGE requires GIVING or OUTPUT PROCEDURE".into()));
    }
    Ok(())
}

/// Format a decimal as a JSON/XML number: sign (if negative non-zero), integer part with leading zeros
/// stripped, and the scale's fractional digits kept (e.g. `12.50`), matching GnuCOBOL `JSON/XML GENERATE`.
fn num_to_json(dec: &Decimal) -> String {
    let scale = dec.scale.max(0) as usize;
    let total = dec.digits.len();
    let intlen = total.saturating_sub(scale);
    let int_str: String = dec.digits[..intlen].iter().map(|d| (b'0' + d) as char).collect();
    let int_t = int_str.trim_start_matches('0');
    let int_final = if int_t.is_empty() { "0" } else { int_t };
    let frac: String = dec.digits[intlen..].iter().map(|d| (b'0' + d) as char).collect();
    let is_zero = int_final == "0" && frac.chars().all(|c| c == '0');
    let sign = if dec.negative && !is_zero { "-" } else { "" };
    if scale > 0 { format!("{sign}{int_final}.{frac}") } else { format!("{sign}{int_final}") }
}

/// A field's alphanumeric value with trailing spaces trimmed, with the `escape` map applied per byte.
fn trimmed_escaped(bytes: &[u8], escape: impl Fn(u8) -> Option<&'static str>) -> String {
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b' ' { end -= 1; }
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
fn json_value(name: &str, fields: &HashMap<String, Field>) -> String {
    match fields.get(name).map(|f| f.storage.clone()) {
        Some(Storage::Group { children }) => {
            let parts: Vec<String> = children.iter().map(|c| format!("\"{}\":{}", c, json_value(c, fields))).collect();
            format!("{{{}}}", parts.join(","))
        }
        Some(Storage::Numeric(attr)) => {
            let bytes = fields.get(name).map(|f| f.bytes.clone()).unwrap_or_default();
            source_to_decimal(&bytes, &attr).map(|d| num_to_json(&d)).unwrap_or_else(|_| "0".into())
        }
        _ => {
            let bytes = read_field(fields, name).ok().flatten().map(|f| f.bytes).unwrap_or_default();
            let esc = |b: u8| match b { b'"' => Some("\\\""), b'\\' => Some("\\\\"), 0x08 => Some("\\b"), 0x0c => Some("\\f"), b'\n' => Some("\\n"), b'\r' => Some("\\r"), b'\t' => Some("\\t"), _ => None };
            format!("\"{}\"", trimmed_escaped(&bytes, esc))
        }
    }
}

/// The XML element of a field: `<name>...</name>` with children nested, numeric/alnum content (XML-escaped),
/// recursively over the group tree (GnuCOBOL `XML GENERATE`, no declaration).
fn xml_value(name: &str, fields: &HashMap<String, Field>) -> String {
    let inner = match fields.get(name).map(|f| f.storage.clone()) {
        Some(Storage::Group { children }) => children.iter().map(|c| xml_value(c, fields)).collect::<String>(),
        Some(Storage::Numeric(attr)) => {
            let bytes = fields.get(name).map(|f| f.bytes.clone()).unwrap_or_default();
            source_to_decimal(&bytes, &attr).map(|d| num_to_json(&d)).unwrap_or_else(|_| "0".into())
        }
        _ => {
            let bytes = read_field(fields, name).ok().flatten().map(|f| f.bytes).unwrap_or_default();
            let esc = |b: u8| match b { b'&' => Some("&amp;"), b'<' => Some("&lt;"), b'>' => Some("&gt;"), _ => None };
            trimmed_escaped(&bytes, esc)
        }
    };
    format!("<{name}>{inner}</{name}>")
}

/// `{JSON|XML} GENERATE dest FROM source [COUNT IN c]` -- serialize the source group into `dest`. `NAME`/
/// `SUPPRESS`/`ON EXCEPTION` are out of subset.
fn exec_ml_generate(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx, xml: bool) -> Result<(), RunError> {
    if stmt.iter().any(|t| matches!(t, Tok::Word(w) if w == "NAME" || w == "SUPPRESS" || w == "EXCEPTION")) {
        return Err(RunError::Unsupported("JSON/XML GENERATE NAME/SUPPRESS/EXCEPTION not in subset".into()));
    }
    let dest = match stmt.get(1) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("JSON/XML GENERATE: missing destination".into())) };
    let fp = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "FROM")).ok_or_else(|| RunError::Unsupported("JSON/XML GENERATE without FROM".into()))?;
    let source = match stmt.get(fp + 1) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("JSON/XML GENERATE: missing source".into())) };
    let text = if xml { xml_value(&source, fields) } else { format!("{{\"{}\":{}}}", source, json_value(&source, fields)) };
    let bytes = text.into_bytes();
    let mv = vec![Tok::Str(bytes.clone()), Tok::Word("TO".to_string()), Tok::Word(dest)];
    exec_move(&mv, fields, ctx.decimal_comma)?;
    // optional COUNT IN counter -> the generated length.
    if let Some(cp) = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "COUNT")) {
        let mut i = cp + 1;
        if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "IN") { i += 1; }
        if let Some(Tok::Word(c)) = stmt.get(i) {
            let mv = vec![Tok::Word(bytes.len().to_string()), Tok::Word("TO".to_string()), Tok::Word(c.clone())];
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
fn format_relem(el: &RElem, fields: &HashMap<String, Field>, ctx: &Ctx) -> Result<Vec<u8>, RunError> {
    let mut temp = make_field(&el.pic, el.value.as_ref(), ctx.currency, ctx.decimal_comma, ctx.dialect)?;
    if let Some(src) = &el.source {
        let (sb, sa) = operand_value(&Tok::Word(src.clone()), fields)?;
        move_into(&mut temp, &sb, &sa, ctx.decimal_comma)?;
    }
    Ok(display_bytes(&temp, ctx.decimal_comma))
}

/// `GENERATE group-name` -- print a report group: for each of its lines, place each element's value at its
/// COLUMN and append the line to the report's file. (Headings/footings/SUM/control breaks are out of subset.)
fn exec_generate(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx) -> Result<(), RunError> {
    let gname = match stmt.first() { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("GENERATE: missing report group".into())) };
    for rd in ctx.reports.values() {
        if let Some(lines) = rd.groups.get(&gname) {
            let org = ctx.file_defs.get(&rd.file).map(|d| d.org).unwrap_or(FileOrg::LineSequential);
            for line_elems in lines {
                if line_elems.is_empty() { continue; }
                let mut buf: Vec<u8> = Vec::new();
                for el in line_elems {
                    let val = format_relem(el, fields, ctx)?;
                    let start = el.column.saturating_sub(1);
                    let endp = start + val.len();
                    if buf.len() < endp { buf.resize(endp, b' '); }
                    buf[start..endp].copy_from_slice(&val);
                }
                if org == FileOrg::LineSequential { while buf.last() == Some(&b' ') { buf.pop(); } }
                ctx.files.borrow_mut().entry(fkey(ctx, &rd.file)).or_default().records.push(buf);
            }
            return Ok(());
        }
    }
    Err(RunError::Unsupported(format!("GENERATE: `{gname}` is not a report group in the subset")))
}

/// `RELEASE record [FROM id]` -- write a record to its sort file during a SORT INPUT PROCEDURE (the records
/// accumulate, then SORT orders them).
fn exec_release(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx) -> Result<(), RunError> {
    let rec = match stmt.first() { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("RELEASE: missing record".into())) };
    if let Some(fp) = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "FROM")) {
        if let Some(src) = stmt.get(fp + 1) {
            let mv = vec![src.clone(), Tok::Word("TO".to_string()), Tok::Word(rec.clone())];
            exec_move(&mv, fields, ctx.decimal_comma)?;
        }
    }
    let def = ctx.file_defs.values().find(|d| d.record == rec)
        .ok_or_else(|| RunError::Unsupported(format!("RELEASE `{rec}`: not an SD/FD record")))?
        .clone();
    let bytes = read_field(fields, &rec)?.map(|f| f.bytes).unwrap_or_default();
    ctx.files.borrow_mut().entry(def.assign.clone()).or_default().records.push(bytes);
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
    let file = match toks.get(*pos) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("RETURN: missing sort file".into())) };
    *pos += 1;
    while matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "RECORD") { *pos += 1; }
    let mut into: Option<String> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "INTO") {
        *pos += 1;
        if let Some(Tok::Word(w)) = toks.get(*pos) { into = Some(w.clone()); *pos += 1; }
    }
    let mut at_end: Option<usize> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "AT") {
        *pos += 1;
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END") { *pos += 1; }
        at_end = Some(*pos);
        let mut scan = *pos;
        let _ = run_block(toks, &mut scan, fields, out, false, ctx)?;
        *pos = scan;
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-RETURN") { *pos += 1; }
    if !exec {
        return Ok(false);
    }
    let def = ctx.file_defs.get(&file).ok_or_else(|| RunError::Unsupported(format!("RETURN: `{file}` is not a declared file")))?.clone();
    let reclen = read_field(fields, &def.record)?.map(|f| f.bytes.len()).unwrap_or(0);
    let next = {
        let files = ctx.files.borrow();
        files.get(&fkey(ctx, &file)).and_then(|st| st.records.get(st.read_pos).cloned())
    };
    match next {
        Some(mut bytes) => {
            if let Some(st) = ctx.files.borrow_mut().get_mut(&fkey(ctx, &file)) { st.read_pos += 1; }
            bytes.resize(reclen, b' ');
            write_field(fields, &def.record, |f| { f.bytes = bytes; Ok(()) })?;
            if let Some(id) = into {
                let mv = vec![Tok::Word(def.record.clone()), Tok::Word("TO".to_string()), Tok::Word(id)];
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
fn sort_key_span(record: &str, key: &str, reclen: usize, fields: &HashMap<String, Field>) -> Option<(usize, usize)> {
    if key == record {
        return Some((0, reclen));
    }
    if let Some(Field { storage: Storage::Group { children }, .. }) = fields.get(record) {
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
    if kl == reclen { Some((0, reclen)) } else { None }
}

/// `DELETE file [RECORD]` -- remove the RELATIVE record at the current RELATIVE KEY (status `"23"` if no
/// such record). DELETE on a sequential file is invalid (out of subset).
fn exec_delete(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx) -> Result<(), RunError> {
    let file = match stmt.first() { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("DELETE: missing file".into())) };
    let def = ctx.file_defs.get(&file).ok_or_else(|| RunError::Unsupported(format!("DELETE: `{file}` is not a declared file")))?.clone();
    if def.org != FileOrg::Relative {
        return Err(RunError::Unsupported("DELETE is only supported on a RELATIVE file in this subset".into()));
    }
    let pos = relative_key_value(&def, fields)?;
    let deleted = {
        let mut files = ctx.files.borrow_mut();
        match files.get_mut(&fkey(ctx, &file)) {
            Some(st) if pos <= st.records.len() && !st.records[pos - 1].is_empty() => {
                st.records[pos - 1] = Vec::new();
                true
            }
            _ => false,
        }
    };
    set_file_status(fields, &def, if deleted { "00" } else { "23" });
    Ok(())
}

/// `START file [KEY [IS] {= | > | >= | < | <= | NOT < | NOT >} key-field]` -- position a RELATIVE file so
/// the next sequential READ returns the first record whose relative number satisfies the relation (default
/// `=` on the current RELATIVE KEY). Status `"23"` if no record qualifies. `INVALID KEY` is out of subset.
fn exec_start(stmt: &[Tok], fields: &mut HashMap<String, Field>, ctx: &Ctx) -> Result<(), RunError> {
    let file = match stmt.first() { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("START: missing file".into())) };
    let def = ctx.file_defs.get(&file).ok_or_else(|| RunError::Unsupported(format!("START: `{file}` is not a declared file")))?.clone();
    if def.org != FileOrg::Relative {
        return Err(RunError::Unsupported("START is only supported on a RELATIVE file in this subset".into()));
    }
    if stmt.iter().any(|t| matches!(t, Tok::Word(w) if w == "INVALID")) {
        return Err(RunError::Unsupported("START ... INVALID KEY not in subset".into()));
    }
    // default: EQUAL on the current RELATIVE KEY value.
    let mut rel = "=".to_string();
    let mut keyval = relative_key_value(&def, fields)?;
    if let Some(kp) = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w == "KEY")) {
        let mut i = kp + 1;
        if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "IS") { i += 1; }
        rel = match stmt.get(i).and_then(|t| if let Tok::Word(w) = t { Some(w.as_str()) } else { None }) {
            Some("=") | Some("EQUAL") => { i += 1; if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "TO") { i += 1; } "=".into() }
            Some(">=") => { i += 1; ">=".into() }
            Some("<=") => { i += 1; "<=".into() }
            Some(">") | Some("GREATER") => { i += 1; if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "THAN") { i += 1; } ">".into() }
            Some("<") | Some("LESS") => { i += 1; if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "THAN") { i += 1; } "<".into() }
            Some("NOT") => {
                i += 1;
                let r = match stmt.get(i).and_then(|t| if let Tok::Word(w) = t { Some(w.as_str()) } else { None }) {
                    Some("<") | Some("LESS") => ">=",
                    Some(">") | Some("GREATER") => "<=",
                    _ => return Err(RunError::Unsupported("START KEY NOT <relation>".into())),
                };
                i += 1;
                if matches!(stmt.get(i), Some(Tok::Word(w)) if w == "THAN") { i += 1; }
                r.into()
            }
            other => return Err(RunError::Unsupported(format!("START KEY relation {other:?}"))),
        };
        if let Some(Tok::Word(field)) = stmt.get(i) {
            keyval = resolve_int(field, fields).map(|v| v.max(0) as usize).unwrap_or(keyval);
        }
    }
    let foundpos = {
        let files = ctx.files.borrow();
        let mut fp = None;
        if let Some(st) = files.get(&fkey(ctx, &file)) {
            for n in 1..=st.records.len() {
                if st.records[n - 1].is_empty() { continue; }
                let ok = match rel.as_str() {
                    "=" => n == keyval,
                    ">" => n > keyval,
                    ">=" => n >= keyval,
                    "<" => n < keyval,
                    "<=" => n <= keyval,
                    _ => false,
                };
                if ok { fp = Some(n); break; }
            }
        }
        fp
    };
    match foundpos {
        Some(n) => {
            if let Some(st) = ctx.files.borrow_mut().get_mut(&fkey(ctx, &file)) { st.read_pos = n - 1; }
            set_file_status(fields, &def, "00");
        }
        None => set_file_status(fields, &def, "23"),
    }
    Ok(())
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
    let file = match toks.get(*pos) { Some(Tok::Word(w)) => w.clone(), _ => return Err(RunError::Unsupported("READ: missing file".into())) };
    *pos += 1;
    let mut had_next = false;
    while matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "NEXT" || w == "RECORD") {
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "NEXT") { had_next = true; }
        *pos += 1;
    }
    let mut into: Option<String> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "INTO") {
        *pos += 1;
        if let Some(Tok::Word(w)) = toks.get(*pos) { into = Some(w.clone()); *pos += 1; }
    }
    // optional AT END imperative (runs at EOF). Parsed as a block ending at END-READ / a period.
    let mut at_end: Option<usize> = None;
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "AT") {
        *pos += 1;
        if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END") { *pos += 1; }
        at_end = Some(*pos);
        let mut scan = *pos;
        let _ = run_block(toks, &mut scan, fields, out, false, ctx)?;
        *pos = scan;
    }
    if matches!(toks.get(*pos), Some(Tok::Word(w)) if w == "END-READ") { *pos += 1; }
    if !exec {
        return Ok(false);
    }
    let def = ctx.file_defs.get(&file).ok_or_else(|| RunError::Unsupported(format!("READ: `{file}` is not a declared file")))?.clone();
    let reclen = read_field(fields, &def.record)?.map(|f| f.bytes.len()).unwrap_or(0);
    let loaded: Option<Vec<u8>> = match def.org {
        // RELATIVE random read: by the RELATIVE KEY (no position advance).
        FileOrg::Relative if !had_next => {
            let pos = relative_key_value(&def, fields)?;
            let files = ctx.files.borrow();
            files.get(&fkey(ctx, &file)).and_then(|st| st.records.get(pos - 1)).filter(|r| !r.is_empty()).cloned()
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
                        let mv = vec![Tok::Word(relnum.to_string()), Tok::Word("TO".to_string()), Tok::Word(key.clone())];
                        exec_move(&mv, fields, ctx.decimal_comma)?;
                    }
                    Some(bytes)
                }
                None => None,
            }
        }
        // sequential / line-sequential: the next record.
        _ => {
            let bytes = { let files = ctx.files.borrow(); files.get(&fkey(ctx, &file)).and_then(|st| st.records.get(st.read_pos).cloned()) };
            if bytes.is_some() {
                if let Some(st) = ctx.files.borrow_mut().get_mut(&fkey(ctx, &file)) { st.read_pos += 1; }
            }
            bytes
        }
    };
    match loaded {
        Some(mut bytes) => {
            bytes.resize(reclen, b' ');
            write_field(fields, &def.record, |f| { f.bytes = bytes; Ok(()) })?;
            if let Some(id) = into {
                let mv = vec![Tok::Word(def.record.clone()), Tok::Word("TO".to_string()), Tok::Word(id)];
                exec_move(&mv, fields, ctx.decimal_comma)?;
            }
            set_file_status(fields, &def, "00");
            Ok(false)
        }
        None => {
            // a relative random miss is "23" (record not found); a sequential/relative-next end is "10".
            let code = if def.org == FileOrg::Relative && !had_next { "23" } else { "10" };
            set_file_status(fields, &def, code);
            if let Some(s) = at_end {
                let mut p = s;
                return run_block(toks, &mut p, fields, out, true, ctx);
            }
            Ok(false)
        }
    }
}

/// Resolve a single operand token to `(bytes, attr)` (identifier -> its stored numeric/alnum form;
/// string literal -> alnum; numeric literal -> zoned display).
fn operand_value(t: &Tok, fields: &HashMap<String, Field>) -> Result<(Vec<u8>, FieldAttr), RunError> {
    match t {
        Tok::Str(s) => Ok((s.clone(), alnum_attr())),
        Tok::Word(w) => {
            if let Some(f) = read_field(fields, w)? {
                match &f.storage {
                    Storage::Numeric(a) => Ok((f.bytes.clone(), *a)),
                    Storage::Alpha(a) => Ok((f.bytes.clone(), *a)),
                    Storage::Edited(..) => Ok((f.bytes.clone(), alnum_attr())),
                    // a group is an alphanumeric value of its concatenated leaves (read_field filled bytes).
                    Storage::Group { .. } => Ok((f.bytes.clone(), alnum_attr())),
                    Storage::Condition { .. } => {
                        Err(RunError::Unsupported("88 condition-name is not a value operand".into()))
                    }
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
        Storage::Alpha(_) | Storage::Edited(..) | Storage::Condition { .. } | Storage::Group { .. } => b' ',
    };
    Field { storage: storage.clone(), bytes: vec![fill; elem], occurs: 1, redefines: None }
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
    Ok(Field { storage: f.storage.clone(), bytes: f.bytes[start..start + elem].to_vec(), occurs: 1, redefines: None })
}

/// The bytes a field's storage operates on -- its own, or, for a `REDEFINES` alias, the target field's
/// bytes viewed at this field's size (the first `size` bytes; REDEFINES width <= target width). A single
/// alias hop (the common 01-level case).
fn aliased(fields: &HashMap<String, Field>, f: &Field) -> Field {
    match &f.redefines {
        Some(target) => {
            let size = f.bytes.len();
            let mut bytes = fields.get(target).map(|t| t.bytes.clone()).unwrap_or_default();
            bytes.resize(size, b' ');
            bytes.truncate(size);
            Field { storage: f.storage.clone(), bytes, occurs: f.occurs, redefines: None }
        }
        None => f.clone(),
    }
}

/// Resolve a (possibly subscripted) field reference word to an owned Field for READING. Returns `Ok(None)`
/// when `word` names no field (e.g. it is a numeric literal). The subscript may itself be a field (`E(I)`);
/// a `REDEFINES` field reads its target's storage (so an alias sees the other field's current bytes).
fn read_field(fields: &HashMap<String, Field>, word: &str) -> Result<Option<Field>, RunError> {
    let (base, sub) = split_subscript(word);
    let Some(f) = fields.get(base) else { return Ok(None) };
    // A group item reads as the concatenation of its leaves' current bytes (its live record image).
    if let Storage::Group { children } = &f.storage {
        let bytes = group_bytes(children, fields);
        return Ok(Some(Field {
            storage: Storage::Group { children: children.clone() },
            bytes,
            occurs: 1,
            redefines: None,
        }));
    }
    let f = aliased(fields, f);
    match sub {
        None => Ok(Some(f)),
        Some(s) => {
            let idx = resolve_int(s, fields)
                .ok_or_else(|| RunError::Unsupported(format!("subscript '{s}' is not an integer")))?;
            Ok(Some(table_element(&f, idx as usize, base)?))
        }
    }
}

/// Apply `apply` to a (possibly subscripted) field reference for WRITING: a bare `NAME` mutates the field;
/// `NAME(i)` extracts the element, applies, and writes the element bytes back into the table.
fn write_field(
    fields: &mut HashMap<String, Field>,
    word: &str,
    apply: impl FnOnce(&mut Field) -> Result<(), RunError>,
) -> Result<(), RunError> {
    let (base, sub) = split_subscript(word);
    // A group write distributes the result across its leaves: shape a temp alphanumeric field over the
    // group's current concatenation, apply, then split the bytes back into the leaves by length.
    if sub.is_none() {
        if let Some(Storage::Group { children }) = fields.get(base).map(|f| f.storage.clone()) {
            let concat = group_bytes(&children, fields);
            let mut tmp = Field { storage: Storage::Alpha(alnum_attr()), bytes: concat, occurs: 1, redefines: None };
            apply(&mut tmp)?;
            put_group_bytes(&children, tmp.bytes, fields);
            return Ok(());
        }
    }
    // A REDEFINES field writes THROUGH its alias into the target's storage: shape a temp with this field's
    // storage over the target's bytes, apply, and copy the result back into the target.
    if sub.is_none() {
        if let Some(target) = fields.get(base).and_then(|f| f.redefines.clone()) {
            let f = fields.get(base).expect("base present");
            let storage = f.storage.clone();
            let size = f.bytes.len();
            let occ = f.occurs;
            let mut bytes = fields.get(&target).map(|t| t.bytes.clone()).unwrap_or_default();
            bytes.resize(size, b' ');
            bytes.truncate(size);
            let mut tmp = Field { storage, bytes, occurs: occ, redefines: None };
            apply(&mut tmp)?;
            let t = fields.get_mut(&target).ok_or_else(|| RunError::UndefinedName(target.clone()))?;
            let n = tmp.bytes.len().min(t.bytes.len());
            t.bytes[..n].copy_from_slice(&tmp.bytes[..n]);
            return Ok(());
        }
    }
    match sub {
        None => {
            let f = fields.get_mut(base).ok_or_else(|| RunError::UndefinedName(base.to_string()))?;
            apply(f)
        }
        Some(s) => {
            let idx = resolve_int(s, fields)
                .ok_or_else(|| RunError::Unsupported(format!("subscript '{s}' is not an integer")))?
                as usize;
            let f = fields.get(base).ok_or_else(|| RunError::UndefinedName(base.to_string()))?;
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
        Storage::Edited(pic, currency, decimal_comma) => {
            // numeric/alnum source into a numeric-edited receiver: decode the source to a decimal,
            // then encode per the edited PIC (the move.c numeric->edited path).
            let pic = pic.clone();
            let cur = *currency;
            let dc = *decimal_comma;
            let dec = source_to_decimal(sbytes, sattr)?;
            f.bytes = encode_edited_cfg(&pic, &dec, cur, dc).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            Ok(())
        }
        Storage::Numeric(attr) | Storage::Alpha(attr) => {
            let attr = *attr;
            let mut dst = f.bytes.clone();
            // cob_move_cfg honors DECIMAL-POINT IS COMMA on the alphanumeric->numeric leaf (move.c reads
            // dec_pt/num_sep from the module): MOVE "12,34" under comma stores 12.34, not 1234.
            cob_move_cfg(sbytes, sattr, &mut dst, &attr, decimal_comma)
                .map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            f.bytes = dst;
            Ok(())
        }
        Storage::Group { .. } => Err(RunError::Unsupported("a group MOVE is distributed across its leaves by write_field".into())),
        Storage::Condition { .. } => Err(RunError::Unsupported("cannot MOVE into an 88 condition-name".into())),
    }
}

/// Decode a numeric DISPLAY (or alnum-of-digits) source `(bytes, attr)` to a [`Decimal`].
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
    Ok(Decimal { negative, digits, scale })
}

/// `ADD/SUBTRACT/MULTIPLY/DIVIDE ...` -- the `TO`/`FROM`/`BY`/`INTO`/`GIVING` forms over numeric
/// receivers, dispatched onto the sealed arithmetic primitives.
/// `ADD`/`SUBTRACT`/`MULTIPLY`/`DIVIDE` -- returns `true` if a SIZE ERROR (e.g. DIVIDE by zero) occurred,
/// leaving the receiver UNCHANGED. The caller dispatches the `ON SIZE ERROR` handler.
fn exec_arith(verb: &str, stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<bool, RunError> {
    match exec_arith_inner(verb, stmt, fields) {
        Ok(()) => Ok(false),
        Err(RunError::SizeError) => Ok(true),
        Err(e) => Err(e),
    }
}

fn exec_arith_inner(verb: &str, stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
    // find a GIVING receiver if present.
    let giving = stmt.iter().position(|t| matches!(t, Tok::Word(w) if w=="GIVING"));
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

    let op = match verb {
        "ADD" => Op::Add,
        "SUBTRACT" => Op::Subtract,
        "MULTIPLY" => Op::Multiply,
        "DIVIDE" => Op::Divide,
        _ => unreachable!(),
    };

    // the "target" operand (after kw): the in-place receiver, or the left/right side for GIVING.
    let target_word = kw_at.and_then(|p| stmt[p + 1..].iter().find_map(|t| match t {
        Tok::Word(w) if !is_kw(w) => Some(w.clone()),
        _ => None,
    }));

    // GIVING receiver name.
    let giving_name = giving.and_then(|p| stmt[p + 1..].iter().find_map(|t| match t {
        Tok::Word(w) if !is_kw(w) => Some(w.clone()),
        _ => None,
    }));

    // Fold the source operands into a single decimal-bearing (bytes, attr) accumulator.
    let (mut acc, mut acc_attr) = operand_value(sources[0], fields)?;
    for s in &sources[1..] {
        let (b, a) = operand_value(s, fields)?;
        acc = cob_arith(Op::Add, &acc, &acc_attr, &b, &a, Round::Truncate)
            .map_err(|e| RunError::Runtime(format!("{e:?}")))?;
        acc_attr = widen(&acc_attr, &a);
    }

    // The receiver name is the GIVING field, else the in-place target.
    let recv_name = giving_name.clone().or_else(|| target_word.clone())
        .ok_or_else(|| RunError::Unsupported(format!("{verb}: no receiver")))?;

    // Compute the result as a WIDE numeric (bytes, attr); `move_into` then truncates/edits it into
    // the receiver's exact format (numeric OR numeric-edited). This is the libcob pattern: arithmetic
    // is exact, the store/edit is the rounding/formatting point.
    let (rb, ra): (Vec<u8>, FieldAttr) = match (verb, &target_word, &giving_name) {
        // ADD a... TO t [GIVING c]:  result = sum(a...) + t
        ("ADD", Some(t), _) => {
            let (tb, ta) = operand_value(&Tok::Word(t.clone()), fields)?;
            wide_op(Op::Add, &acc, &acc_attr, &tb, &ta)?
        }
        // ADD a... GIVING c:  result = sum(a...)
        ("ADD", None, Some(_)) => (acc.clone(), acc_attr),
        // SUBTRACT a... FROM t [GIVING c]:  result = t - sum(a...)
        ("SUBTRACT", Some(t), _) => {
            let (tb, ta) = operand_value(&Tok::Word(t.clone()), fields)?;
            wide_op(Op::Subtract, &tb, &ta, &acc, &acc_attr)?
        }
        // MULTIPLY a BY t [GIVING c]:  result = a * t
        ("MULTIPLY", Some(t), _) => {
            let (tb, ta) = operand_value(&Tok::Word(t.clone()), fields)?;
            wide_op(Op::Multiply, &acc, &acc_attr, &tb, &ta)?
        }
        // DIVIDE a INTO t [GIVING c]: result = t / a ;  DIVIDE a BY t [GIVING c]: result = a / t
        ("DIVIDE", Some(t), _) => {
            let (tb, ta) = operand_value(&Tok::Word(t.clone()), fields)?;
            let (num, na, den, da) = if kw == "INTO" {
                (tb, ta, acc.clone(), acc_attr)
            } else {
                (acc.clone(), acc_attr, tb, ta)
            };
            let wide = lit_num_attr(36, 18, true); // generous quotient scale; move_into truncates.
            let q = cob_divide(&num, &na, &den, &da, &wide, Round::Truncate)
                .map_err(map_arith_err)?;
            (q, wide)
        }
        _ => return Err(RunError::Unsupported(format!("{verb} form (target/giving)"))),
    };

    // Store the wide result into the receiver -- cob_move (numeric) or encode_edited (edited).
    let f = fields.get_mut(&recv_name).ok_or_else(|| RunError::UndefinedName(recv_name.clone()))?;
    // arithmetic result is an already-decoded numeric value -> separator-independent store.
    move_into(f, &rb, &ra, false)
}

/// Compute `op(a, b)` exactly into a wide numeric DISPLAY `(bytes, attr)` -- 18 integer digits plus a
/// scale generous enough to be exact for add/subtract (max operand scale) and multiply (sum of
/// scales); the receiver store is the truncation point.
fn wide_op(op: Op, a: &[u8], aa: &FieldAttr, b: &[u8], ba: &FieldAttr) -> Result<(Vec<u8>, FieldAttr), RunError> {
    let scale = match op {
        Op::Multiply => (aa.scale.max(0) + ba.scale.max(0)).max(0),
        _ => aa.scale.max(ba.scale).max(0),
    };
    let wide = lit_num_attr(18 + scale as u16, scale, true);
    let wsize = wide.digits as usize;
    let mut a_wide = vec![b'0'; wsize.max(1)];
    cob_move(a, aa, &mut a_wide, &wide).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
    let r = cob_arith(op, &a_wide, &wide, b, ba, Round::Truncate)
        .map_err(|e| RunError::Runtime(format!("{e:?}")))?;
    Ok((r, wide))
}




/// Widen an accumulator attr to cover another operand (used while folding ADD sources).
fn widen(a: &FieldAttr, b: &FieldAttr) -> FieldAttr {
    let scale = a.scale.max(b.scale).max(0);
    let int = (a.digits as i16 - a.scale).max(b.digits as i16 - b.scale).max(0);
    lit_num_attr((int + scale) as u16 + 1, scale, true)
}



/// Is `w` an arithmetic keyword (not an operand name)?
fn is_kw(w: &str) -> bool {
    matches!(w, "TO" | "FROM" | "BY" | "INTO" | "GIVING" | "ROUNDED" | "REMAINDER")
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
    // KANIFOR: GNURUST.FRONTEND.1
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
        let on = run_program_dialect_with_rc(&prog(">>TURN EC-BOUND-SUBSCRIPT CHECKING ON\n"), Dialect::DEFAULT);
        assert!(on.is_err(), "EC-BOUND-SUBSCRIPT ON must raise on an out-of-range subscript");
        // EC-ALL CHECKING ON also enables it.
        let all = run_program_dialect_with_rc(&prog(">>TURN EC-ALL CHECKING ON\n"), Dialect::DEFAULT);
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
        let (out2, rc2) = run_program_dialect_with_rc(&prog("STOP RUN 9."), Dialect::DEFAULT).unwrap();
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
            run_program_dialect_with_rc(&src, Dialect::DEFAULT).unwrap().1
        };
        // MOVE n TO RETURN-CODE -> process exit code n (oracle: 5->5, 42->42).
        assert_eq!(rc("           MOVE 42 TO RETURN-CODE.\n           STOP RUN."), 42);
        assert_eq!(rc("           MOVE 5 TO RETURN-CODE.\n           STOP RUN."), 5);
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
        for s in ["", "garbage tokens here", "MOVE", "01 X PIC", "PROCEDURE DIVISION."] {
            let _ = run_program(s); // must not panic
        }
    }

    #[test]
    fn add_move_display() {
        let out = run(
            "       IDENTIFICATION DIVISION.\n\
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
                        STOP RUN.\n",
        );
        assert_eq!(out, b"TOTAL=   350\n");
    }

    #[test]
    fn multiply_giving() {
        let out = run(
            "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 P PIC 9(3) VALUE 12.\n\
                    01 Q PIC 9(3) VALUE 4.\n\
                    01 R PIC ZZ9.\n\
                    PROCEDURE DIVISION.\n\
                        MULTIPLY P BY Q GIVING R.\n\
                        DISPLAY \"P=\" R.\n\
                        STOP RUN.\n",
        );
        assert_eq!(out, b"P= 48\n");
    }

    #[test]
    fn compute_precedence_and_div() {
        // COMPUTE with operator precedence + division intermediate precision.
        let out = run(
            "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC 9(6) VALUE 22.\n\
                    01 B PIC 9(6) VALUE 7.\n\
                    01 R PIC 9.9(8).\n\
                    PROCEDURE DIVISION.\n\
                        COMPUTE R = A / B.\n\
                        DISPLAY \"PI=\" R.\n\
                        STOP RUN.\n",
        );
        assert_eq!(out, b"PI=3.14285714\n"); // 22/7 truncated to 8 fractional digits
    }

    #[test]
    fn if_else_and_perform() {
        // IF/ELSE branch selection + PERFORM UNTIL loop (factorial) + alphanumeric compare.
        let fac = run(
            "       IDENTIFICATION DIVISION.\n\
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
                        STOP RUN.\n",
        );
        assert_eq!(fac, b"     120\n"); // 5! = 120

        let branch = run(
            "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC 9 VALUE 3.\n\
                    01 R PIC X(3).\n\
                    PROCEDURE DIVISION.\n\
                        IF A > 5 MOVE \"BIG\" TO R ELSE MOVE \"LOW\" TO R END-IF.\n\
                        DISPLAY R.\n\
                        STOP RUN.\n",
        );
        assert_eq!(branch, b"LOW\n");
    }

    #[test]
    fn compute_rounded_fails_closed() {
        // ROUNDED is not yet in the subset -> fail closed, not a wrong answer.
        let r = run_program(
            "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 A PIC 9(4) VALUE 10.\n\
                    01 R PIC 9.\n\
                    PROCEDURE DIVISION.\n\
                        COMPUTE R ROUNDED = A / 3.\n\
                        STOP RUN.\n",
        );
        assert!(matches!(r, Err(RunError::Unsupported(_))));
    }

    #[test]
    fn string_move_and_display() {
        let out = run(
            "       IDENTIFICATION DIVISION.\n\
                    PROGRAM-ID. T.\n\
                    DATA DIVISION.\n\
                    WORKING-STORAGE SECTION.\n\
                    01 WS-N PIC X(5).\n\
                    PROCEDURE DIVISION.\n\
                        MOVE \"HI\" TO WS-N.\n\
                        DISPLAY \"[\" WS-N \"]\".\n\
                        STOP RUN.\n",
        );
        assert_eq!(out, b"[HI   ]\n");
    }
}
