//! 1:1 port of termio.c — `DISPLAY` / `ACCEPT` terminal I/O plus the `-fdump` debug feature. The C writes
//! to `FILE *` streams; this port writes to a `&mut Vec<u8>` sink (so the emitted bytes are testable) and
//! carries the runtime state the C keeps in `cobglobptr`/`cobsetptr`/`COB_MODULE_PTR` in explicit structs.
//!
//! The byte-verifiable heart is [`cob_display_common`] — the exact bytes `DISPLAY` emits for a field of
//! any type (signed/edited numeric, `COMP` realbin, `COMP-1`/`COMP-2` float, pointer, alphanumeric). The
//! dump feature ([`cob_dump_field_ext`] et al.) formats fields for a debug dump.
#![forbid(unsafe_code)]

use crate::attr::{
    FieldAttr, COB_FLAG_HAVE_SIGN, COB_FLAG_SIGN_LEADING, COB_FLAG_SIGN_SEPARATE, COB_TYPE_GROUP,
    COB_TYPE_NUMERIC_COMP5, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_DOUBLE,
    COB_TYPE_NUMERIC_FLOAT, COB_TYPE_NUMERIC_FP_DEC128, COB_TYPE_NUMERIC_FP_DEC64,
    COB_TYPE_NUMERIC_L_DOUBLE,
};
use crate::value::Decimal;

/// `COB_MEDIUM_MAX` (`common.h:586`): `COB_MEDIUM_BUFF - 1`.
const COB_MEDIUM_MAX: i32 = 8191;
/// `bin_digits` (termio.c:55): the decimal-digit count for a binary field of byte-size 0..8.
const BIN_DIGITS: [u16; 9] = [1, 3, 5, 8, 10, 13, 15, 17, 20];

/// The `COB_MODULE_PTR` settings `cob_display_common` reads: the `DECIMAL POINT` character and whether
/// `pretty-display` is on (GnuCOBOL's default config has `pretty-display: yes`).
#[derive(Debug, Clone, Copy)]
pub struct DisplaySettings {
    pub decimal_point: u8,
    pub pretty_display: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        DisplaySettings { decimal_point: b'.', pretty_display: true }
    }
}

/// `clean_double (wrk)` (termio.c:222): normalise the alternate NaN spellings to `NaN` and strip the
/// leading zero from a 2-digit exponent (`E+02` → `E+2`), matching the C post-processing of `%G`.
fn clean_double(mut wrk: String) -> String {
    if let Some(pos) = wrk.rfind('E') {
        // C: pos += 2 (skip "E±"); if pos[0]=='0' memmove one char away (removes the leading zero).
        let idx = pos + 2;
        if wrk.as_bytes().get(idx) == Some(&b'0') {
            wrk.remove(idx);
        }
        return wrk;
    }
    match wrk.as_str() {
        "-NAN" | "-NaNQ" | "-NaN" | "NAN" | "NaNQ" => "NaN".to_string(),
        _ => wrk,
    }
}

/// Strip the trailing zeros + trailing decimal point of a `%G` body (the C printf `%G` does this unless
/// the `#` flag is set).
fn strip_g(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// C `sprintf("%-.*G", prec, val)` — significant-digit float formatting: choose `%f` when the decimal
/// exponent `X` satisfies `-4 <= X < prec`, else `%e` (uppercase `E`, signed, ≥2-digit exponent), then
/// strip trailing zeros. Paired with [`clean_double`] to match the C exactly.
fn c_format_g(val: f64, prec: usize) -> String {
    let p = if prec == 0 { 1 } else { prec };
    if val.is_nan() {
        return "NAN".to_string();
    }
    if val.is_infinite() {
        return if val < 0.0 { "-INF".to_string() } else { "INF".to_string() };
    }
    // Decimal exponent X, as the %e form (p-1 fractional digits) reports it.
    let e = format!("{:.*E}", p - 1, val);
    let epos = e.find('E').unwrap();
    let x: i32 = e[epos + 1..].parse().unwrap();
    if x >= -4 && x < p as i32 {
        let fprec = (p as i32 - 1 - x).max(0) as usize;
        strip_g(&format!("{:.*}", fprec, val))
    } else {
        let mant = strip_g(&e[..epos]);
        let sign = if x < 0 { '-' } else { '+' };
        format!("{mant}E{sign}{:02}", x.abs())
    }
}

/// `display_numeric (f, fp)` (termio.c:63): emit a numeric field's bytes by converting to USAGE DISPLAY
/// with `SIGN SEPARATE` (the non-pretty path). The receiver is `digits + (scale<0?scale:0) + has_sign`
/// bytes; the sign is leading when the source is sign-leading or not a plain DISPLAY field.
fn display_numeric(data: &[u8], attr: &FieldAttr, out: &mut Vec<u8>) {
    let digits = attr.digits as i32;
    let scale = attr.scale as i32;
    let has_sign = if attr.have_sign() { 1 } else { 0 };
    let size = digits + if scale < 0 { scale } else { 0 } + has_sign;
    if size >= COB_MEDIUM_MAX {
        out.extend_from_slice(b"(Not representable)");
        return;
    }
    let mut flags = 0u16;
    if has_sign == 1 {
        flags = COB_FLAG_HAVE_SIGN | COB_FLAG_SIGN_SEPARATE;
        if attr.sign_leading() || attr.field_type != COB_TYPE_NUMERIC_DISPLAY {
            flags |= COB_FLAG_SIGN_LEADING;
        }
    }
    let dst_attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: attr.digits, scale: attr.scale, flags };
    let mut buf = vec![0u8; size.max(0) as usize];
    let _ = crate::move_ops::cob_move(data, attr, &mut buf, &dst_attr);
    out.extend_from_slice(&buf);
}

/// `pretty_display_numeric (f, fp)` (termio.c:109): emit a numeric field via a NUMERIC-EDITED conversion —
/// a leading/trailing sign, the integer `9`s, the `DECIMAL POINT`, and the fractional `9`s — i.e. the
/// human-readable form (`-012.34`). This is GnuCOBOL's default (`pretty-display: yes`).
fn pretty_display_numeric(data: &[u8], attr: &FieldAttr, decimal_point: u8, out: &mut Vec<u8>) {
    let mut digits = attr.digits as i32;
    let scale = attr.scale as i32;
    let has_sign = attr.have_sign();
    // termio.c splits `scale == 0` (PIC 999) and `scale < 0` (PIC P99) — both give the same size.
    let size: i32 = if scale <= 0 {
        digits + has_sign as i32
    } else {
        if digits < scale {
            digits = scale;
        }
        digits + has_sign as i32 + 1
    };
    if size > COB_MEDIUM_MAX {
        out.extend_from_slice(b"(Not representable)");
        return;
    }
    let dp = decimal_point as char;
    let mut pic = String::new();
    let trailing_sep = has_sign && attr.sign_separate() && !attr.sign_leading();
    if has_sign && !trailing_sep {
        pic.push('+');
    }
    if scale > 0 {
        if digits - scale > 0 {
            for _ in 0..(digits - scale) {
                pic.push('9');
            }
        }
        pic.push(dp);
        for _ in 0..scale {
            pic.push('9');
        }
    } else {
        for _ in 0..digits {
            pic.push('9');
        }
    }
    if trailing_sep {
        pic.push('+');
    }

    let value = decimal_of(data, attr);
    match crate::edited::encode_edited(&pic, &value) {
        Ok(bytes) => out.extend_from_slice(&bytes),
        Err(_) => out.extend_from_slice(b"(Not representable)"),
    }
}

/// Read a numeric field's value into a [`Decimal`] (the source side of the C `cob_move (f, edited)`).
fn decimal_of(data: &[u8], attr: &FieldAttr) -> Decimal {
    match attr.field_type {
        crate::attr::COB_TYPE_NUMERIC_PACKED => Decimal::from_packed(data, attr),
        crate::attr::COB_TYPE_NUMERIC_BINARY => Decimal::from_binary(data, attr),
        _ => Decimal::from_display(data, attr),
    }
}

/// `display_alnum (f, fp)` (termio.c:204): emit a field's raw bytes verbatim.
fn display_alnum(data: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(data);
}

/// `is_field_display (f)` (termio.c:467): every byte is printable 7-bit ASCII (`' ' <= b <= 0x7F`).
pub fn is_field_display(data: &[u8]) -> bool {
    data.iter().all(|&b| (b' '..=0x7f).contains(&b))
}

/// `cob_display_common (f, fp)` (termio.c:244): emit the `DISPLAY` bytes of a field of any type into
/// `out`. Numeric DISPLAY/PACKED/BINARY go through the pretty (edited) or sign-separate path per
/// `settings`; `COMP-5`/real-binary print the full binary value; `COMP-1`/`COMP-2`/`long double` use the
/// `%G` float form; `FLOAT-DECIMAL` uses the IEEE-decimal print; a pointer prints big-endian `0x` hex;
/// everything else is emitted verbatim.
pub fn cob_display_common(data: &[u8], attr: &FieldAttr, settings: &DisplaySettings, out: &mut Vec<u8>) {
    if data.is_empty() {
        return;
    }
    match attr.field_type {
        COB_TYPE_NUMERIC_L_DOUBLE => {
            // host-specific x87 80-bit; read via the f64 approximation (declared non-claim).
            let v = crate::cob_decimal::extended80_to_f64(data);
            out.extend_from_slice(clean_double(c_format_g(v, 32)).as_bytes());
            return;
        }
        COB_TYPE_NUMERIC_DOUBLE => {
            let mut b = [0u8; 8];
            b.copy_from_slice(&data[..8.min(data.len())]);
            let v = f64::from_le_bytes(b);
            out.extend_from_slice(clean_double(c_format_g(v, 16)).as_bytes());
            return;
        }
        COB_TYPE_NUMERIC_FLOAT => {
            let mut b = [0u8; 4];
            b.copy_from_slice(&data[..4.min(data.len())]);
            let v = f32::from_le_bytes(b) as f64;
            out.extend_from_slice(clean_double(c_format_g(v, 8)).as_bytes());
            return;
        }
        COB_TYPE_NUMERIC_FP_DEC64 | COB_TYPE_NUMERIC_FP_DEC128 => {
            out.extend_from_slice(crate::cob_decimal::cob_print_ieeedec(data, attr).as_bytes());
            return;
        }
        _ => {}
    }
    if attr.is_pointer() {
        out.extend_from_slice(b"0x");
        // big-endian display order of the pointer's bytes (little-endian host -> reverse).
        for &byte in data.iter().rev() {
            out.push(hex_nibble(byte >> 4));
            out.push(hex_nibble(byte & 0xF));
        }
        return;
    }
    if attr.is_numeric() {
        if attr.field_type == COB_TYPE_NUMERIC_COMP5 {
            out.extend_from_slice(crate::cob_decimal::cob_print_realbin(data, attr, attr.digits as usize).as_bytes());
            return;
        }
        if attr.real_binary()
            || (attr.field_type == crate::attr::COB_TYPE_NUMERIC_BINARY && !settings.pretty_display)
        {
            let bd = BIN_DIGITS.get(data.len()).copied().unwrap_or(20) as usize;
            out.extend_from_slice(crate::cob_decimal::cob_print_realbin(data, attr, bd).as_bytes());
            return;
        }
        if settings.pretty_display {
            pretty_display_numeric(data, attr, settings.decimal_point, out);
            return;
        }
        display_numeric(data, attr, out);
        return;
    }
    display_alnum(data, out);
}

fn hex_nibble(n: u8) -> u8 {
    if n < 10 {
        b'0' + n
    } else {
        b'a' + (n - 10)
    }
}

/// `cob_display (to_device, newline, varcnt, ...)` (termio.c:323): the `DISPLAY` statement. This port
/// produces the emitted bytes of the SYSOUT/SYSERR path — each operand's [`cob_display_common`] bytes
/// concatenated, then a single `'\n'` when `newline`. Device routing (printer/punch files, screen
/// redirect) is the surrounding I/O and is the caller's concern.
pub fn cob_display(newline: bool, operands: &[(&[u8], &FieldAttr)], settings: &DisplaySettings, out: &mut Vec<u8>) {
    for (data, attr) in operands {
        cob_display_common(data, attr, settings, out);
    }
    if newline {
        out.push(b'\n');
    }
}

/// `cob_accept (f)` (termio.c:1025): the non-screen `ACCEPT` — read one input line and `MOVE` it into the
/// receiver as an alphanumeric source (left-justified, space-padded, truncated to the field width). For a
/// `NUMERIC DISPLAY` receiver the source is first truncated to the field size (the C special case). The
/// input line is supplied as a slice (the testable form of `getchar` to newline).
pub fn cob_accept(input_line: &[u8], f: &mut [u8], f_attr: &FieldAttr) {
    let size = input_line.len().min(COB_MEDIUM_MAX as usize);
    let mut temp_size = size;
    if f_attr.field_type == COB_TYPE_NUMERIC_DISPLAY && temp_size > f.len() {
        temp_size = f.len();
    }
    let src_attr = FieldAttr { field_type: crate::attr::COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };
    let _ = crate::move_ops::cob_move(&input_line[..temp_size], &src_attr, f, f_attr);
}

/// `cob_init_termio (lptr, sptr)` (termio.c:1095): module init binding the runtime globals. This port
/// returns the default [`DisplaySettings`] (the C stores the pointers; the observable settings are the
/// `DECIMAL POINT` + `pretty-display`).
pub fn cob_init_termio() -> DisplaySettings {
    DisplaySettings::default()
}

// ---- DUMP feature (cobc -fdump) ----------------------------------------------------------------

/// `open_mode` of the file in [`cob_dump_file`].
#[derive(Debug, Clone, Copy)]
pub enum DumpOpenMode {
    Closed,
    Locked,
    Open,
}

/// The dump's module-static state (termio.c:683-800): the OCCURS-collapse bookkeeping that lets the dump
/// print `(1..N) same as (0)` instead of repeating identical table occurrences. Carried explicitly here.
#[derive(Default)]
pub struct DumpState {
    dump_null_adrs: bool,
    pending_dump_name: Vec<u8>,
    dump_index: usize,
    dump_idx: Vec<u32>,
    dump_idx_first: Vec<u32>,
    dump_idx_last: Vec<u32>,
    dump_skip: Vec<u32>,
    dump_prev_data: Vec<Option<Vec<u8>>>,
    dump_compat: bool,
}

impl DumpState {
    fn ensure(&mut self, n: usize) {
        let need = n + 2;
        if self.dump_idx.len() < need {
            self.dump_idx.resize(need, 0);
            self.dump_idx_first.resize(need, 0);
            self.dump_idx_last.resize(need, 0);
            self.dump_skip.resize(need, 0);
            self.dump_prev_data.resize(need, None);
        }
    }
}

/// `is_field_display` thin alias used by the dump path is [`is_field_display`].
///
/// `setup_varname_with_indices (buffer, subscript, indexes, name, closing_paren)` (termio.c:741): format
/// `name` plus any subscripts `(i,j,...)` into `buffer`; returns the written length.
pub fn setup_varname_with_indices(
    subscript: &[u32],
    indexes: u32,
    name: &str,
    closing_paren: bool,
) -> Vec<u8> {
    if indexes != 0 {
        let mut s = format!("{name} ({}", subscript[0]);
        for &sub in &subscript[1..indexes as usize] {
            s.push_str(&format!(",{sub}"));
        }
        if closing_paren {
            s.push(')');
        }
        s.into_bytes()
    } else {
        name.as_bytes().to_vec()
    }
}

/// `setup_lvlwrk_and_dump_null_adrs (lvlwrk, level, data_ptr)` (termio.c:764): format the level marker
/// (`01`/`77` two-digit, `0` → `   INDEX`, else indented level) and set/clear the null-address flag.
/// Returns the `lvlwrk` string (empty when a parent had no address and printing is to be skipped).
pub fn setup_lvlwrk_and_dump_null_adrs(state: &mut DumpState, level: i32, has_data: bool) -> Vec<u8> {
    if level == 77 || level == 1 {
        state.dump_null_adrs = !has_data;
        format!("{level:02}").into_bytes()
    } else if state.dump_null_adrs {
        Vec::new()
    } else if level == 0 {
        b"   INDEX".to_vec()
    } else {
        let mut indent = level / 2;
        if indent > 7 {
            indent = 7;
        }
        format!("{:indent$}{level:02}", "", indent = indent as usize).into_bytes()
    }
}

/// `dump_pending_output (fp)` (termio.c:802): flush a deferred `... same as (n)` line for collapsed
/// identical table occurrences.
pub fn dump_pending_output(state: &mut DumpState, out: &mut Vec<u8>) {
    if state.pending_dump_name.is_empty() {
        return;
    }
    out.extend_from_slice(&state.pending_dump_name);
    let di = state.dump_index;
    if state.dump_idx_last.get(di) != state.dump_idx_first.get(di) {
        out.extend_from_slice(format!("..{}", state.dump_idx_last[di]).as_bytes());
    }
    out.extend_from_slice(format!(") same as ({})\n", state.dump_idx[di]).as_bytes());
    state.pending_dump_name.clear();
}

/// `cob_dump_output (str)` (termio.c:686): emit a dump section header.
pub fn cob_dump_output(state: &mut DumpState, s: &str, out: &mut Vec<u8>) {
    dump_pending_output(state, out);
    out.extend_from_slice(format!("\n{s}\n**********************\n").as_bytes());
}

/// `cob_dump_file (name, fl)` (termio.c:702): emit a dump file header (open mode + 2-char file status).
pub fn cob_dump_file(state: &mut DumpState, name: Option<&str>, open_mode: DumpOpenMode, file_status: &[u8], out: &mut Vec<u8>) {
    dump_pending_output(state, out);
    let mode = match open_mode {
        DumpOpenMode::Closed => "CLOSED",
        DumpOpenMode::Locked => "LOCKED",
        DumpOpenMode::Open => "OPEN",
    };
    if let Some(n) = name {
        out.extend_from_slice(format!("\n{n}\n**********************\n").as_bytes());
    }
    out.extend_from_slice(format!("   File is {mode}\n").as_bytes());
    let st: String = file_status.iter().take(2).map(|&b| b as char).collect();
    out.extend_from_slice(format!("   FILE STATUS  '{st:.2}'\n").as_bytes());
}

/// `display_alnum_dump (f, fp, indent, max_width)` (termio.c:479): render a field's content for a dump —
/// the `ALL SPACES`/`ALL ZEROES`/`ALL LOW-VALUES`/`ALL HIGH-VALUES` shorthands, the trailing-LOW-VALUES
/// note, quoted printable text wrapped to the column width (with run-dedup), the escaped form for
/// delimiter-bearing data, and otherwise a side-by-side char + hex dump.
pub fn display_alnum_dump(data: &[u8], out: &mut Vec<u8>, indent: usize, max_width: usize) {
    let fsize = data.len();
    let colsize = max_width.saturating_sub(indent).saturating_sub(2);
    let (mut lowv, mut highv, mut spacev, mut zerov, mut printv, mut delv) = (0, 0, 0, 0, 0, 0);
    for &b in data {
        if b == 0x00 {
            lowv += 1;
            delv += 1;
        } else if b == 0xFF {
            highv += 1;
        } else if b == b' ' {
            spacev += 1;
            printv += 1;
        } else if b == b'0' {
            zerov += 1;
            printv += 1;
        } else if matches!(b, 0x08 | 0x0c | b'\n' | b'\r' | b'\t') {
            delv += 1;
        } else if b >= b' ' && is_print(b) {
            printv += 1;
        }
    }
    if spacev == fsize {
        out.extend_from_slice(b"ALL SPACES");
        return;
    }
    if zerov == fsize {
        out.extend_from_slice(b"ALL ZEROES");
        return;
    }
    if lowv == fsize {
        out.extend_from_slice(b"ALL LOW-VALUES");
        return;
    }
    if highv == fsize {
        out.extend_from_slice(b"ALL HIGH-VALUES");
        return;
    }
    // remove trailing LOW-VALUES with a note
    if lowv != 0 && (lowv + printv) == fsize {
        let mut len = fsize;
        while len != 0 && data[len - 1] == 0x00 {
            len -= 1;
        }
        if len + lowv == fsize {
            let mut i = 0;
            while len > colsize {
                out.extend_from_slice(b"'");
                out.extend_from_slice(&data[i..i + colsize]);
                out.extend_from_slice(format!("'\n{:indent$}", "", indent = indent).as_bytes());
                i += colsize;
                len -= colsize;
            }
            if len <= colsize {
                out.extend_from_slice(b"'");
                out.extend_from_slice(&data[i..i + len]);
                out.push(b'\'');
            }
            out.extend_from_slice(format!("\n{:w$} trailing LOW-VALUES", "", w = indent.saturating_sub(8)).as_bytes());
            return;
        }
    }
    // trailing SPACES are ignored
    let mut len = fsize;
    while len != 0 && data[len - 1] == b' ' {
        len -= 1;
    }
    if printv == fsize {
        let mut i = 0usize;
        while len > colsize {
            if i != 0 {
                out.extend_from_slice(format!("{i:5}:").as_bytes());
            }
            out.extend_from_slice(b"'");
            out.extend_from_slice(&data[i..i + colsize]);
            out.extend_from_slice(format!("'\n{:w$}", "", w = indent.saturating_sub(6)).as_bytes());
            i += colsize;
            len -= colsize;
        }
        if i != 0 {
            out.extend_from_slice(format!("{i:5}:").as_bytes());
        }
        if len <= colsize {
            out.extend_from_slice(b"'");
            out.extend_from_slice(&data[i..i + len]);
            out.push(b'\'');
            return;
        }
    }
    // escaped form when only printable + the named delimiters
    if (delv + printv) == fsize {
        let mut i = 0usize;
        while i < fsize {
            let mut j = 0usize;
            while j < colsize && i < fsize {
                match data[i] {
                    0x00 => { out.extend_from_slice(b"\\0"); j += 1; }
                    b'\\' => { out.extend_from_slice(b"\\\\"); j += 1; }
                    b'\r' => { out.extend_from_slice(b"\\r"); j += 1; }
                    b'\n' => { out.extend_from_slice(b"\\n"); j += 1; }
                    b'\t' => { out.extend_from_slice(b"\\t"); j += 1; }
                    0x08 => { out.extend_from_slice(b"\\b"); j += 1; }
                    0x0c => { out.extend_from_slice(b"\\f"); j += 1; }
                    c => out.push(c),
                }
                j += 1;
                i += 1;
            }
            if i < fsize {
                out.extend_from_slice(format!("\n{:w$}{:5} : ", "", i + 1, w = indent.saturating_sub(8)).as_bytes());
            }
        }
        return;
    }
    // side-by-side char + hex dump
    let mut colsize = colsize.min(199);
    if colsize > 9 {
        colsize = (colsize / 9) * 9;
    }
    colsize = (colsize / 4) * 4;
    let mut i = 0usize;
    while i < fsize {
        let pos = i + 1;
        let mut wrk = String::new();
        let mut j = 0usize;
        while j < colsize && i < fsize {
            if (b' '..=0x7F).contains(&data[i]) {
                out.push(b' ');
                out.push(data[i]);
            } else {
                out.extend_from_slice(b"  ");
            }
            wrk.push_str(&format!("{:02X}", data[i]));
            if (j + 2) < colsize && ((i + 1) % 4) == 0 && (i + 1) < fsize {
                out.push(b' ');
                wrk.push(' ');
                j += 1;
            }
            j += 2;
            i += 1;
        }
        out.extend_from_slice(format!("\n{:w$}{pos:5} x {wrk}", "", w = indent.saturating_sub(8)).as_bytes());
        if i < fsize {
            out.extend_from_slice(format!("\n{:indent$}", "", indent = indent).as_bytes());
        }
    }
}

fn is_print(b: u8) -> bool {
    (0x20..=0x7e).contains(&b)
}

/// `dump_field_internal (level, name, f_addr, field_offset, indexes, ...)` (termio.c:817) — the per-field
/// dump line. This port takes the subscripts + per-index element sizes explicitly (the C `va_list`), the
/// field `(data, attr)`, and a [`DumpState`] for the OCCURS-collapse bookkeeping, and writes the
/// `level name value` line (groups end with `.`; numeric values via [`cob_display_common`], wide/edited/
/// non-display content via [`display_alnum_dump`]).
#[allow(clippy::too_many_arguments)]
pub fn dump_field_internal(
    state: &mut DumpState,
    level: i32,
    name: &str,
    data: Option<&[u8]>,
    attr: &FieldAttr,
    indexes: u32,
    subscript: &[u32],
    settings: &DisplaySettings,
    dump_width: usize,
    out: &mut Vec<u8>,
) {
    state.ensure(indexes as usize + 1);
    let has_data = data.is_some();
    let lvlwrk = setup_lvlwrk_and_dump_null_adrs(state, level, has_data);
    let vname = setup_varname_with_indices(subscript, indexes, name, true);
    let name_length = if indexes != 0 {
        setup_varname_with_indices(subscript, indexes, name, true).len()
    } else {
        name.len()
    };

    if state.dump_null_adrs {
        if level == 1 || level == 77 {
            let mut vn = vname.clone();
            if attr.field_type == COB_TYPE_GROUP {
                vn.push(b'.');
            }
            out.extend_from_slice(format!("{:<10}{:<30} <NULL> address\n", String::from_utf8_lossy(&lvlwrk), String::from_utf8_lossy(&vn)).as_bytes());
        }
        return;
    }
    if attr.field_type == COB_TYPE_GROUP {
        out.extend_from_slice(format!("{:<10}{}.\n", String::from_utf8_lossy(&lvlwrk), String::from_utf8_lossy(&vname)).as_bytes());
        return;
    }
    out.extend_from_slice(format!("{:<10}{:<30} ", String::from_utf8_lossy(&lvlwrk), String::from_utf8_lossy(&vname)).as_bytes());
    if name_length > 30 {
        out.extend_from_slice(format!("\n{:41}", "").as_bytes());
    }
    let data = match data {
        None => {
            out.extend_from_slice(b"<CODEGEN ERROR, PLEASE REPORT THIS!>\n");
            return;
        }
        Some(d) => d,
    };
    // termio.c uses two `if`/`else if` clauses that both dump as alnum: a DISPLAY/EDITED-numeric whose
    // bytes are not all printable, OR an alphanumeric / oversized (>39) field.
    let is_disp_numeric = matches!(attr.field_type, COB_TYPE_NUMERIC_DISPLAY)
        || attr.field_type == crate::attr::COB_TYPE_NUMERIC_EDITED;
    if (is_disp_numeric && !is_field_display(data))
        || attr.field_type == crate::attr::COB_TYPE_ALPHANUMERIC
        || attr.field_type == crate::attr::COB_TYPE_ALPHANUMERIC_EDITED
        || data.len() > 39
    {
        display_alnum_dump(data, out, 41, dump_width);
    } else {
        out.push(b' ');
        cob_display_common(data, attr, settings, out);
    }
    out.push(b'\n');
}

/// `cob_dump_field_ext (level, name, f_addr, field_offset, indexes, ...)` (termio.c:1008): the dump entry
/// point. (`cob_dump_field` is the 3.1-compat alias that also handles the `level < 0` directives.)
#[allow(clippy::too_many_arguments)]
pub fn cob_dump_field_ext(
    state: &mut DumpState,
    level: i32,
    name: &str,
    data: Option<&[u8]>,
    attr: &FieldAttr,
    indexes: u32,
    subscript: &[u32],
    settings: &DisplaySettings,
    dump_width: usize,
    out: &mut Vec<u8>,
) {
    dump_field_internal(state, level, name, data, attr, indexes, subscript, settings, dump_width, out);
}

/// `cob_dump_field (level, name, f_addr, offset, indexes, ...)` (termio.c:978): the 3.1-compat dump entry
/// — negative `level` selects a directive (`-1` → [`cob_dump_output`], `-2` → [`cob_dump_file`]),
/// otherwise it dumps the field in compat mode (no OCCURS collapse).
#[allow(clippy::too_many_arguments)]
pub fn cob_dump_field(
    state: &mut DumpState,
    level: i32,
    name: &str,
    data: Option<&[u8]>,
    attr: &FieldAttr,
    indexes: u32,
    subscript: &[u32],
    settings: &DisplaySettings,
    dump_width: usize,
    out: &mut Vec<u8>,
) {
    state.dump_compat = true;
    dump_field_internal(state, level, name, data, attr, indexes, subscript, settings, dump_width, out);
    state.dump_compat = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn an(_n: usize) -> FieldAttr {
        FieldAttr { field_type: crate::attr::COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 }
    }

    #[test]
    fn display_alnum_verbatim() {
        let mut out = Vec::new();
        cob_display_common(b"HELLO", &an(5), &DisplaySettings::default(), &mut out);
        assert_eq!(out, b"HELLO");
    }

    #[test]
    fn float_g_and_clean_double() {
        // C "%.8G" of 1.5 = "1.5"; of 100000000 (1e8, X=8>=8) -> "1E+08" -> clean -> "1E+8".
        assert_eq!(clean_double(c_format_g(1.5, 8)), "1.5");
        assert_eq!(clean_double(c_format_g(1.0e8, 8)), "1E+8");
        assert_eq!(clean_double(c_format_g(0.0001, 8)), "0.0001"); // X=-4 -> f form
        assert_eq!(clean_double(c_format_g(0.00001, 8)), "1E-5"); // X=-5 -> e form, E-05 -> E-5
        assert_eq!(clean_double(c_format_g(-2.5, 8)), "-2.5");
        assert_eq!(clean_double(c_format_g(0.0, 8)), "0");
    }

    #[test]
    fn dump_formatters() {
        // ALL-shorthands
        let mut o = Vec::new();
        display_alnum_dump(b"     ", &mut o, 41, 100);
        assert_eq!(o, b"ALL SPACES");
        o.clear();
        display_alnum_dump(b"00000", &mut o, 41, 100);
        assert_eq!(o, b"ALL ZEROES");
        o.clear();
        display_alnum_dump(&[0u8; 4], &mut o, 41, 100);
        assert_eq!(o, b"ALL LOW-VALUES");
        o.clear();
        display_alnum_dump(&[0xFFu8; 4], &mut o, 41, 100);
        assert_eq!(o, b"ALL HIGH-VALUES");
        // short printable -> quoted (trailing spaces ignored)
        o.clear();
        display_alnum_dump(b"HELLO ", &mut o, 41, 100);
        assert_eq!(o, b"'HELLO'");
        // varname with indices
        assert_eq!(setup_varname_with_indices(&[1, 2], 2, "TBL", true), b"TBL (1,2)");
        assert_eq!(setup_varname_with_indices(&[], 0, "FLD", true), b"FLD");
        // level markers
        let mut st = DumpState::default();
        assert_eq!(setup_lvlwrk_and_dump_null_adrs(&mut st, 1, true), b"01");
        assert_eq!(setup_lvlwrk_and_dump_null_adrs(&mut st, 0, true), b"   INDEX");
    }

    #[test]
    fn cob_display_wrapper_concatenates_with_newline() {
        let a = an(3);
        let mut out = Vec::new();
        cob_display(true, &[(b"AB", &a), (b"CD", &a)], &DisplaySettings::default(), &mut out);
        assert_eq!(out, b"ABCD\n");
    }

    #[test]
    fn pretty_unsigned_scaled() {
        // PIC 9(3)V99 value 12.34 (zoned "01234") -> pretty "012.34" (default pretty-display).
        let attr =
            FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 5, scale: 2, flags: 0 };
        let mut out = Vec::new();
        cob_display_common(b"01234", &attr, &DisplaySettings::default(), &mut out);
        assert_eq!(out, b"012.34");
    }
}
