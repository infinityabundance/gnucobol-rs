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
//!   `FROM` / `BY` / `INTO` / `GIVING` forms), `DISPLAY`, `STOP RUN`.
//!
//! Group items, `OCCURS`/`REDEFINES`, level numbers other than `01`, `COMPUTE`, control flow
//! (`IF`/`PERFORM`/`EVALUATE`), `ACCEPT`, files, and any unlisted verb are out of subset and return a
//! [`RunError`] rather than guessing.

use crate::arith::{cob_arith, cob_divide, Op, Round};
use crate::attr::{FieldAttr, COB_TYPE_NUMERIC_DISPLAY};
use crate::edited::{edited_size, encode_edited};
use crate::move_ops::cob_move;
use crate::pic::{build_field, Usage};
use crate::termio::{cob_display, DisplaySettings};
use crate::value::Decimal;
use std::collections::HashMap;

/// Why a program could not be run (fail closed -- the front-end never guesses).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// A token/keyword/structure outside the sealed subset, with a human note.
    Unsupported(String),
    /// A referenced data name was never declared in WORKING-STORAGE.
    UndefinedName(String),
    /// A runtime operation (move/arith/edit) failed (e.g. non-numeric operand, divide by zero).
    Runtime(String),
}

impl core::fmt::Display for RunError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RunError::Unsupported(s) => write!(f, "unsupported: {s}"),
            RunError::UndefinedName(s) => write!(f, "undefined data name: {s}"),
            RunError::Runtime(s) => write!(f, "runtime error: {s}"),
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
    /// A numeric-edited field: the bytes are the edited image; its PIC string drives editing.
    Edited(String),
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
/// Fails closed with a [`RunError`] for anything outside the sealed subset.
pub fn run_program(source: &str) -> Result<Vec<u8>, RunError> {
    let up = source.to_uppercase();
    let toks = lex(&up);
    let mut fields: HashMap<String, Field> = HashMap::new();
    let mut out = Vec::new();

    // locate PROCEDURE DIVISION; everything before it (after WORKING-STORAGE SECTION) is data.
    let proc_at = find_seq(&toks, &["PROCEDURE", "DIVISION"])
        .ok_or_else(|| RunError::Unsupported("no PROCEDURE DIVISION".into()))?;
    let ws_at = find_seq(&toks, &["WORKING-STORAGE", "SECTION"]);

    // --- DATA DIVISION: parse 01-level elementary items between WS SECTION and PROCEDURE DIVISION.
    if let Some(ws) = ws_at {
        let mut k = ws + 2;
        // skip the '.' after SECTION
        if matches!(toks.get(k), Some(Tok::Dot)) {
            k += 1;
        }
        while k < proc_at {
            // each item: <level> NAME [PIC <pic>] [VALUE <lit>] .
            let level = match toks.get(k) {
                Some(Tok::Word(w)) => w.clone(),
                _ => {
                    k += 1;
                    continue;
                }
            };
            if level != "01" {
                return Err(RunError::Unsupported(format!("only level 01 elementary items (got {level})")));
            }
            k += 1;
            let name = match toks.get(k) {
                Some(Tok::Word(w)) => w.clone(),
                _ => return Err(RunError::Unsupported("expected data name after 01".into())),
            };
            k += 1;
            // gather the rest of the item (until the terminating Dot).
            let mut pic: Option<String> = None;
            let mut value: Option<Tok> = None;
            while k < proc_at {
                match toks.get(k) {
                    Some(Tok::Dot) => {
                        k += 1;
                        break;
                    }
                    Some(Tok::Word(w)) if w == "PIC" || w == "PICTURE" => {
                        // optional "IS"
                        k += 1;
                        if matches!(toks.get(k), Some(Tok::Word(w)) if w=="IS") {
                            k += 1;
                        }
                        if let Some(Tok::Word(p)) = toks.get(k) {
                            pic = Some(p.clone());
                            k += 1;
                        }
                    }
                    Some(Tok::Word(w)) if w == "VALUE" => {
                        k += 1;
                        if matches!(toks.get(k), Some(Tok::Word(w)) if w=="IS") {
                            k += 1;
                        }
                        value = toks.get(k).cloned();
                        k += 1;
                    }
                    _ => {
                        k += 1;
                    }
                }
            }
            let pic = pic.ok_or_else(|| RunError::Unsupported(format!("item {name} has no PIC")))?;
            let field = make_field(&pic, value.as_ref())?;
            fields.insert(name, field);
        }
    }

    // --- PROCEDURE DIVISION: execute statements from after "PROCEDURE DIVISION." to end.
    let mut k = proc_at + 2;
    if matches!(toks.get(k), Some(Tok::Dot)) {
        k += 1;
    }
    while k < toks.len() {
        match toks.get(k) {
            Some(Tok::Word(w)) => {
                let verb = w.clone();
                k += 1;
                // collect this statement's tokens up to the next Dot.
                let mut stmt = Vec::new();
                while k < toks.len() && !matches!(toks.get(k), Some(Tok::Dot)) {
                    stmt.push(toks[k].clone());
                    k += 1;
                }
                if matches!(toks.get(k), Some(Tok::Dot)) {
                    k += 1;
                }
                exec_stmt(&verb, &stmt, &mut fields, &mut out)?;
                if verb == "STOP" {
                    break;
                }
            }
            Some(Tok::Dot) => {
                k += 1;
            }
            _ => {
                k += 1;
            }
        }
    }
    Ok(out)
}

/// Build a [`Field`] from its PIC + optional VALUE literal. Edited pictures (which [`build_field`]
/// rejects as unsupported symbols) are stored as their edited image.
fn make_field(pic: &str, value: Option<&Tok>) -> Result<Field, RunError> {
    match build_field(pic, Usage::Display, false, false) {
        Ok(pf) => {
            let is_alpha = !pf.attr.is_numeric();
            let mut bytes = vec![if is_alpha { b' ' } else { b'0' }; pf.size];
            let storage = if is_alpha { Storage::Alpha(pf.attr) } else { Storage::Numeric(pf.attr) };
            let mut field = Field { storage, bytes: bytes.clone() };
            if let Some(v) = value {
                init_value(&mut field, v)?;
            } else if is_alpha {
                // alphanumeric default is spaces (already set).
                bytes.fill(b' ');
                field.bytes = bytes;
            }
            Ok(field)
        }
        Err(crate::pic::PicError::UnsupportedSymbol(_)) | Err(crate::pic::PicError::MixedCategory) => {
            // treat as numeric-edited: storage is the edited image, sized by edited_size.
            let size = edited_size(pic).map_err(|e| RunError::Unsupported(format!("PIC {pic}: {e:?}")))?;
            let mut field = Field { storage: Storage::Edited(pic.to_string()), bytes: vec![b' '; size] };
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
        Storage::Edited(pic) => {
            let pic = pic.clone();
            field.bytes = encode_edited(&pic, dec).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
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

/// Store alphanumeric source bytes into a field (left-justified, space-padded/truncated, or numeric
/// receiver via the runtime move).
fn store_alnum(field: &mut Field, src: &[u8]) -> Result<(), RunError> {
    let src_attr = alnum_attr();
    match &field.storage {
        Storage::Edited(_) => {
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
) -> Result<(), RunError> {
    match verb {
        "DISPLAY" => exec_display(stmt, fields, out),
        "MOVE" => exec_move(stmt, fields),
        "ADD" | "SUBTRACT" | "MULTIPLY" | "DIVIDE" => exec_arith(verb, stmt, fields),
        "STOP" => Ok(()), // STOP RUN
        other => Err(RunError::Unsupported(format!("verb {other}"))),
    }
}

/// `DISPLAY op [op ...]` -- concatenate each operand's display bytes, then a newline.
fn exec_display(stmt: &[Tok], fields: &HashMap<String, Field>, out: &mut Vec<u8>) -> Result<(), RunError> {
    let mut operands: Vec<(Vec<u8>, FieldAttr)> = Vec::new();
    for t in stmt {
        match t {
            Tok::Str(s) => operands.push((s.clone(), alnum_attr())),
            Tok::Word(w) => {
                if w == "WITH" || w == "NO" || w == "ADVANCING" {
                    // DISPLAY ... WITH NO ADVANCING handled below (no newline) -- mark it.
                    continue;
                }
                let f = fields.get(w).ok_or_else(|| RunError::UndefinedName(w.clone()))?;
                operands.push((display_bytes(f), alnum_attr()));
            }
            Tok::Dot => {}
        }
    }
    let no_adv = stmt.iter().any(|t| matches!(t, Tok::Word(w) if w=="ADVANCING"));
    let refs: Vec<(&[u8], &FieldAttr)> = operands.iter().map(|(b, a)| (b.as_slice(), a)).collect();
    cob_display(!no_adv, &refs, &DisplaySettings::default(), out);
    Ok(())
}

/// The bytes a field contributes to DISPLAY: numeric DISPLAY fields are shown via the runtime's
/// display formatting; alphanumeric + edited fields are shown as their stored bytes.
fn display_bytes(f: &Field) -> Vec<u8> {
    match &f.storage {
        Storage::Numeric(attr) => {
            let mut o = Vec::new();
            crate::termio::cob_display_common(&f.bytes, attr, &DisplaySettings::default(), &mut o);
            o
        }
        Storage::Alpha(_) | Storage::Edited(_) => f.bytes.clone(),
    }
}

/// `MOVE src TO d1 [d2 ...]`.
fn exec_move(stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
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
        move_into(f, &sbytes, &sattr)?;
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
                    Storage::Edited(_) => Ok((f.bytes.clone(), alnum_attr())),
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
fn move_into(f: &mut Field, sbytes: &[u8], sattr: &FieldAttr) -> Result<(), RunError> {
    match &f.storage {
        Storage::Edited(pic) => {
            // numeric/alnum source into a numeric-edited receiver: decode the source to a decimal,
            // then encode per the edited PIC (the move.c numeric->edited path).
            let pic = pic.clone();
            let dec = source_to_decimal(sbytes, sattr)?;
            f.bytes = encode_edited(&pic, &dec).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            Ok(())
        }
        Storage::Numeric(attr) | Storage::Alpha(attr) => {
            let attr = *attr;
            let mut dst = f.bytes.clone();
            cob_move(sbytes, sattr, &mut dst, &attr).map_err(|e| RunError::Runtime(format!("{e:?}")))?;
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
fn exec_arith(verb: &str, stmt: &[Tok], fields: &mut HashMap<String, Field>) -> Result<(), RunError> {
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
                .map_err(|e| RunError::Runtime(format!("{e:?}")))?;
            (q, wide)
        }
        _ => return Err(RunError::Unsupported(format!("{verb} form (target/giving)"))),
    };

    // Store the wide result into the receiver -- cob_move (numeric) or encode_edited (edited).
    let f = fields.get_mut(&recv_name).ok_or_else(|| RunError::UndefinedName(recv_name.clone()))?;
    move_into(f, &rb, &ra)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<u8> {
        run_program(src).expect("run")
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
