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
    "DISPLAY", "MOVE", "ADD", "SUBTRACT", "MULTIPLY", "DIVIDE", "COMPUTE", "IF", "PERFORM", "STOP",
    "CONTINUE",
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
}

/// A live field: its storage shape and its current bytes (always exactly the field's size).
#[derive(Debug, Clone)]
struct Field {
    storage: Storage,
    bytes: Vec<u8>,
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
    let up = pre.to_uppercase();
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
    let ctx = Ctx {
        programs: &program_map,
        dialect,
        currency,
        decimal_comma,
        switches,
        print_redirect: redirect_printer,
        printer: RefCell::new(Vec::new()),
        stop_run: Cell::new(false),
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
    proc_toks: Vec<Tok>,
}

/// One `01`-level elementary item (its name, PIC, and optional VALUE literal) -- the field is built at run
/// time (so a CALL can build the callee's fields under the same dialect).
struct ProgItem {
    name: String,
    pic: String,
    value: Option<Tok>,
}

/// The shared execution context: the program registry (for CALL resolution) + the dialect / SPECIAL-NAMES
/// needed to build any program's fields, and the UPSI switch environment.
struct Ctx<'a> {
    programs: &'a HashMap<String, ProgramDef>,
    dialect: crate::dialect::Dialect,
    currency: u8,
    decimal_comma: bool,
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
    let proc_at = find_seq_in(toks, &["PROCEDURE", "DIVISION"], start, end)
        .ok_or_else(|| RunError::Unsupported(format!("{name}: no PROCEDURE DIVISION")))?;
    let ws_at = find_seq_in(toks, &["WORKING-STORAGE", "SECTION"], start, proc_at);
    let link_at = find_seq_in(toks, &["LINKAGE", "SECTION"], start, proc_at);

    // WORKING-STORAGE items: from WS SECTION to LINKAGE SECTION (or PROCEDURE).
    let ws = match ws_at {
        Some(w) => parse_items(toks, w + 2, link_at.unwrap_or(proc_at))?,
        None => Vec::new(),
    };
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

    Ok((name, ProgramDef { ws, linkage, using, proc_toks }))
}

/// Parse the `01`-level elementary items in `toks[start..end]` (a WORKING-STORAGE or LINKAGE section body).
fn parse_items(toks: &[Tok], start: usize, end: usize) -> Result<Vec<ProgItem>, RunError> {
    let mut items = Vec::new();
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
        if level == "PROCEDURE" || level == "LINKAGE" || level == "DATA" {
            break;
        }
        if level != "01" {
            return Err(RunError::Unsupported(format!("only level 01 elementary items (got {level})")));
        }
        k += 1;
        let name = match toks.get(k) {
            Some(Tok::Word(w)) => w.clone(),
            _ => return Err(RunError::Unsupported("expected data name after 01".into())),
        };
        k += 1;
        let mut pic: Option<String> = None;
        let mut value: Option<Tok> = None;
        while k < end {
            match toks.get(k) {
                Some(Tok::Dot) => {
                    k += 1;
                    break;
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
        let pic = pic.ok_or_else(|| RunError::Unsupported(format!("item {name} has no PIC")))?;
        items.push(ProgItem { name, pic, value });
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
        let f = make_field(&it.pic, it.value.as_ref(), ctx.currency, ctx.decimal_comma, ctx.dialect)?;
        fields.insert(it.name.clone(), f);
    }
    // RETURN-CODE: the signed special register, initialised to 0 (modelled as S9(9) DISPLAY).
    fields.insert("RETURN-CODE".to_string(), make_return_code(0));
    Ok(fields)
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
    let mut pos = 0;
    if matches!(proc.first(), Some(Tok::Dot)) {
        pos = 1;
    }
    while pos < proc.len() {
        if matches!(proc.get(pos), Some(Tok::Dot)) {
            pos += 1;
            continue;
        }
        let halted = run_block(proc, &mut pos, fields, out, true, ctx)?;
        if halted {
            break; // STOP RUN / GOBACK / EXIT PROGRAM
        }
        if matches!(proc.get(pos), Some(Tok::Dot)) {
            pos += 1;
        }
    }
    Ok(())
}

/// Statement verbs that begin a new statement (so an operand list ends when one is seen).
const STMT_VERBS: &[&str] = &[
    "MOVE", "ADD", "SUBTRACT", "MULTIPLY", "DIVIDE", "COMPUTE", "DISPLAY", "IF", "PERFORM", "STOP",
    "CONTINUE", "ACCEPT", "GO", "EVALUATE", "CALL", "GOBACK", "EXIT",
];
/// Scope terminators that end a block.
const SCOPE_ENDERS: &[&str] = &["ELSE", "END-IF", "END-PERFORM", "WHEN", "END-EVALUATE"];

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
                let verb = w.clone();
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
    let truth = if exec { eval_cond(&cond, fields, &ctx.switches)? } else { false };

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
            while !eval_cond(&cond, fields, &ctx.switches)? {
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
fn eval_cond(t: &[Tok], fields: &HashMap<String, Field>, sw: &SwitchEnv) -> Result<bool, RunError> {
    let words: Vec<String> = t
        .iter()
        .map(|tok| match tok {
            Tok::Word(w) => w.clone(),
            Tok::Str(s) => format!("\u{1}{}", String::from_utf8_lossy(s)), // mark string literal
            Tok::Dot => ".".into(),
        })
        .collect();
    let mut p = 0;
    let r = cond_or(&words, &mut p, fields, sw)?;
    if p != words.len() {
        return Err(RunError::Unsupported(format!("trailing tokens in condition at {}", words[p])));
    }
    Ok(r)
}

fn cond_or(w: &[String], p: &mut usize, f: &HashMap<String, Field>, sw: &SwitchEnv) -> Result<bool, RunError> {
    let mut acc = cond_and(w, p, f, sw)?;
    while w.get(*p).map(|s| s.as_str()) == Some("OR") {
        *p += 1;
        let r = cond_and(w, p, f, sw)?;
        acc = acc || r;
    }
    Ok(acc)
}

fn cond_and(w: &[String], p: &mut usize, f: &HashMap<String, Field>, sw: &SwitchEnv) -> Result<bool, RunError> {
    let mut acc = cond_rel(w, p, f, sw)?;
    while w.get(*p).map(|s| s.as_str()) == Some("AND") {
        *p += 1;
        let r = cond_rel(w, p, f, sw)?;
        acc = acc && r;
    }
    Ok(acc)
}

fn cond_rel(w: &[String], p: &mut usize, f: &HashMap<String, Field>, sw: &SwitchEnv) -> Result<bool, RunError> {
    let left = w.get(*p).ok_or_else(|| RunError::Unsupported("condition: missing left operand".into()))?.clone();
    *p += 1;
    // A bare UPSI switch condition-name (SPECIAL-NAMES `SWITCH-n ON/OFF STATUS IS <name>`): its truth is
    // the switch's state matching the declared ON/OFF sense. No relational operator follows.
    if let Some(&(idx, on)) = sw.conds.get(&left) {
        return Ok(sw.states[idx] == on);
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
    let ord = cond_compare(&left, &right, f)?;
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
fn cond_compare(a: &str, b: &str, f: &HashMap<String, Field>) -> Result<std::cmp::Ordering, RunError> {
    let na = cond_numeric(a, f);
    let nb = cond_numeric(b, f);
    if let (Some(da), Some(db)) = (&na, &nb) {
        return Ok(dec_cmp(da, db));
    }
    // alphanumeric compare: space-pad the shorter, byte compare.
    let sa = cond_bytes(a, f);
    let sb = cond_bytes(b, f);
    let n = sa.len().max(sb.len());
    for i in 0..n {
        let ca = sa.get(i).copied().unwrap_or(b' ');
        let cb = sb.get(i).copied().unwrap_or(b' ');
        if ca != cb {
            return Ok(ca.cmp(&cb));
        }
    }
    Ok(std::cmp::Ordering::Equal)
}

/// If a condition operand is numeric (a numeric field or a numeric literal), decode it to a [`Decimal`].
fn cond_numeric(w: &str, f: &HashMap<String, Field>) -> Option<Decimal> {
    if let Some(field) = f.get(w) {
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
    if let Some(field) = f.get(w) {
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
            let mut field = Field { storage, bytes };
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
                Field { storage: Storage::Edited(pic.to_string(), currency, decimal_comma), bytes: vec![b' '; size] };
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
    let mut f = Field { storage: Storage::Numeric(attr), bytes: vec![b'0'; 9] };
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
        "CALL" => exec_call(stmt, fields, out, ctx),
        "STOP" => Ok(()), // STOP RUN
        // ADD/SUBTRACT/MULTIPLY/DIVIDE/COMPUTE are handled in run_block (they carry ON SIZE ERROR clauses).
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

    // Build the callee's fields (its WORKING-STORAGE + RETURN-CODE), then fill its LINKAGE USING params
    // from the caller's arguments (copy-in).
    let mut cfields = build_program_fields(callee, ctx)?;
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
    while let Some(rest) = s.strip_prefix('(') {
        out.push("(".into());
        s = rest;
    }
    // collect trailing ')'s.
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
                let f = fields.get(w).ok_or_else(|| RunError::UndefinedName(w.clone()))?;
                let bytes = if w == "RETURN-CODE" { display_return_code(f) } else { display_bytes(f, ctx.decimal_comma) };
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
        let f = fields.get_mut(&d).ok_or_else(|| RunError::UndefinedName(d.clone()))?;
        move_into(f, &sbytes, &sattr, decimal_comma)?;
    }
    Ok(())
}

/// Resolve a single operand token to `(bytes, attr)` (identifier -> its stored numeric/alnum form;
/// string literal -> alnum; numeric literal -> zoned display).
fn operand_value(t: &Tok, fields: &HashMap<String, Field>) -> Result<(Vec<u8>, FieldAttr), RunError> {
    match t {
        Tok::Str(s) => Ok((s.clone(), alnum_attr())),
        Tok::Word(w) => {
            if let Some(f) = fields.get(w) {
                match &f.storage {
                    Storage::Numeric(a) => Ok((f.bytes.clone(), *a)),
                    Storage::Alpha(a) => Ok((f.bytes.clone(), *a)),
                    Storage::Edited(..) => Ok((f.bytes.clone(), alnum_attr())),
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
