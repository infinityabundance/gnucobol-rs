//! Intrinsic functions (`GNURUST.INTRINSIC.*`) — the first *implemented* intrinsic courts, split out of the
//! observed `GNURUST.INTRINSIC.ATLAS.1` map.
//!
//! `GNURUST.INTRINSIC.LENGTH.1`: `FUNCTION LENGTH(field)` returns the **storage byte length** of an
//! elementary item — the same byte count the sealed field model (`build_field`, `GNURUST.3`/`GNURUST.9`/
//! `GNURUST.14`) computes — proven equal to GnuCOBOL's `FUNCTION LENGTH` across DISPLAY / `COMP-3` / binary
//! field types. **Non-claims:** `LENGTH` of a group / table / reference-modified operand, `LENGTH OF`
//! (different from `FUNCTION LENGTH` for some operands), national/UTF-8 (character vs byte length), and all
//! dialects — those remain on the atlas, not implemented here.

use crate::pic::{build_field, PicError};
use crate::Usage;

/// `FUNCTION LENGTH(field)` for an elementary item: the storage byte length (== GnuCOBOL). For a `PIC X(n)`
/// this is `n`; for numeric `DISPLAY` the digit count; for `COMP-3` the packed byte count; for binary the
/// storage width.
pub fn intrinsic_length(pic: &str, usage: Usage) -> Result<usize, PicError> {
    build_field(pic, usage, false, false).map(|f| f.size)
}

/// A parsed `FUNCTION NUMVAL` value: `value = scaled * 10^(-scale)`, with the sign in `negative`.
///
/// `GNURUST.INTRINSIC.NUMVAL.1` admits the narrow form: optional leading/trailing spaces, an optional sign
/// (leading `+`/`-`, or trailing `+`/`-`/`CR`/`DB` — all of `-`/`CR`/`DB` mean negative), digits, and an
/// optional decimal point. **Non-claims:** `NUMVAL-C` (currency / thousands separators), national/UTF-8,
/// locale decimal/comma swap, malformed-input error semantics, and all dialects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Numval {
    pub negative: bool,
    pub scaled: u128,
    pub scale: u32,
}

fn pow10_u128(n: u32) -> u128 {
    (0..n).fold(1u128, |a, _| a * 10)
}

/// `FUNCTION NUMVAL(s)` for the narrow admitted form (see [`Numval`]).
pub fn intrinsic_numval(s: &str) -> Numval {
    let mut negative = false;
    let mut core = s.trim().to_string();
    let upper = core.to_ascii_uppercase();
    if upper.ends_with("CR") || upper.ends_with("DB") {
        negative = true;
        core = core[..core.len() - 2].trim().to_string();
    }
    if let Some(r) = core.strip_prefix('-') {
        negative = true;
        core = r.trim().to_string();
    } else if let Some(r) = core.strip_prefix('+') {
        core = r.trim().to_string();
    } else if let Some(r) = core.strip_suffix('-') {
        negative = true;
        core = r.trim().to_string();
    } else if let Some(r) = core.strip_suffix('+') {
        core = r.trim().to_string();
    }
    let (ip, fp) = core.split_once('.').unwrap_or((core.as_str(), ""));
    let int_digits: String = ip.chars().filter(|c| c.is_ascii_digit()).collect();
    let frac_digits: String = fp.chars().filter(|c| c.is_ascii_digit()).collect();
    let scaled: u128 = format!("{int_digits}{frac_digits}").parse().unwrap_or(0);
    if scaled == 0 {
        negative = false; // +0
    }
    Numval { negative, scaled, scale: frac_digits.len() as u32 }
}

/// `FUNCTION NUMVAL-C(s)` for the narrow admitted form: like [`intrinsic_numval`] but first strips the
/// default currency symbol `$` and thousands-separator commas (`GNURUST.INTRINSIC.NUMVAL-C.1`). So
/// `NUMVAL-C("$1,234.56") = 1234.56`. **Non-claims:** a non-default currency symbol (the 2-arg form),
/// `DECIMAL-POINT IS COMMA` / locale comma-decimal, national/UTF-8, and all dialects.
pub fn intrinsic_numval_c(s: &str) -> Numval {
    let cleaned: String = s.chars().filter(|&c| c != '$' && c != ',').collect();
    intrinsic_numval(&cleaned)
}

/// Render a [`Numval`] as a signed fixed `S9(int_digits)V9(frac_digits)` display string (the bytes a `MOVE`
/// into such a receiver produces): `sign + int_digits + "." + frac_digits`, truncating toward zero.
pub fn numval_display(nv: &Numval, int_digits: usize, frac_digits: usize) -> String {
    let target = frac_digits as u32;
    let v = if nv.scale <= target {
        nv.scaled * pow10_u128(target - nv.scale)
    } else {
        nv.scaled / pow10_u128(nv.scale - target)
    };
    let modulus = pow10_u128(frac_digits as u32);
    let int_part = v / modulus;
    let frac_part = v % modulus;
    let mut int_str = format!("{int_part}");
    if int_str.len() < int_digits {
        int_str = format!("{:0>width$}", int_str, width = int_digits);
    } else if int_str.len() > int_digits {
        int_str = int_str[int_str.len() - int_digits..].to_string(); // keep low int_digits (overflow trunc)
    }
    let frac_str = format!("{:0>width$}", frac_part, width = frac_digits);
    let sign = if nv.negative { '-' } else { '+' };
    format!("{sign}{int_str}.{frac_str}")
}

impl Numval {
    /// The signed unscaled magnitude as an `i128` (`value = signed_mag * 10^(-scale)`).
    pub fn signed_mag(&self) -> i128 {
        if self.negative {
            -(self.scaled as i128)
        } else {
            self.scaled as i128
        }
    }
}

/// `FUNCTION INTEGER-PART(x)`: the integer part, **truncated toward zero** (`GNURUST.INTRINSIC.INTEGER.1`).
/// `x = signed_mag * 10^(-scale)`. INTEGER-PART(-3.7) = -3.
pub fn intrinsic_integer_part(signed_mag: i128, scale: u32) -> i128 {
    signed_mag / (pow10_u128(scale) as i128)
}

/// `FUNCTION INTEGER(x)`: the greatest integer **not greater than** `x` (floor) — differs from
/// [`intrinsic_integer_part`] on negatives with a fractional part: INTEGER(-3.7) = -4.
pub fn intrinsic_integer(signed_mag: i128, scale: u32) -> i128 {
    let d = pow10_u128(scale) as i128;
    let q = signed_mag / d;
    let r = signed_mag % d;
    if r != 0 && signed_mag < 0 {
        q - 1
    } else {
        q
    }
}

/// `FUNCTION REM(a, b)` for integers: the C-style remainder `a - b * trunc(a/b)` — the result takes the
/// **dividend** sign (`GNURUST.INTRINSIC.MOD-REM.1`). `b == 0` is a non-claim (returns 0 rather than panic).
pub fn intrinsic_rem(a: i128, b: i128) -> i128 {
    if b == 0 {
        return 0;
    }
    a % b
}

/// `FUNCTION MOD(a, b)` for integers: `a - b * floor(a/b)` — the result takes the **divisor** sign
/// (mathematical modulo), unlike [`intrinsic_rem`]. `b == 0` is a non-claim (returns 0).
pub fn intrinsic_mod(a: i128, b: i128) -> i128 {
    if b == 0 {
        return 0;
    }
    let r = a % b;
    if r != 0 && (r < 0) != (b < 0) {
        r + b
    } else {
        r
    }
}

/// `FUNCTION UPPER-CASE(s)` — ASCII `a..z` → `A..Z`, every other byte unchanged, same length
/// (`GNURUST.INTRINSIC.CASE.1`). Non-claims: locale/national case folding (non-ASCII).
pub fn intrinsic_upper_case(s: &[u8]) -> Vec<u8> {
    s.iter().map(|b| b.to_ascii_uppercase()).collect()
}

/// `FUNCTION LOWER-CASE(s)` — ASCII `A..Z` → `a..z`, every other byte unchanged, same length.
pub fn intrinsic_lower_case(s: &[u8]) -> Vec<u8> {
    s.iter().map(|b| b.to_ascii_lowercase()).collect()
}

/// `FUNCTION REVERSE(s)` — the bytes in reverse order (including spaces), same length.
pub fn intrinsic_reverse(s: &[u8]) -> Vec<u8> {
    s.iter().rev().copied().collect()
}

/// `FUNCTION ORD(c)` — the **1-based** position of byte `c` in the (native ASCII) collating sequence:
/// `ORD(c) = c + 1` (`GNURUST.INTRINSIC.ORD-CHAR.1`). ORD('A') = 66, not 65. Non-claims: national/UTF-8,
/// non-default collating sequences, all dialects.
pub fn intrinsic_ord(c: u8) -> u32 {
    c as u32 + 1
}

/// `FUNCTION CHAR(n)` — the byte at **1-based** position `n` (`n` in `1..=256`): `CHAR(n) = n - 1`. The
/// 1-based inverse of [`intrinsic_ord`]: CHAR(66) = 'A'. `n` outside `1..=256` is a non-claim.
pub fn intrinsic_char(n: u32) -> u8 {
    n.saturating_sub(1) as u8
}

// --- Date-conversion intrinsics (`GNURUST.INTRINSIC.DATE.1`) -----------------------------------------------
// COBOL integer dates count days in the proleptic Gregorian calendar from a fixed epoch: 1601-01-01 is day 1
// (so the reference is 1600-12-31). These are DETERMINISTIC (pure calendar math) -- unlike the env-sensitive
// CURRENT-DATE / WHEN-COMPILED, which stay refused. Algorithm: Howard Hinnant's days_from_civil / civil_from_days.

/// Days since 1970-01-01 for a proleptic-Gregorian `(y, m, d)` (Hinnant).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// The proleptic-Gregorian `(y, m, d)` for `z` days since 1970-01-01 (Hinnant; inverse of [`days_from_civil`]).
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The COBOL day-1 reference (1600-12-31) as days since 1970-01-01.
fn cobol_epoch() -> i64 {
    days_from_civil(1601, 1, 1) - 1
}

/// `FUNCTION INTEGER-OF-DATE(YYYYMMDD)` — the integer day number (1601-01-01 = 1) for a Gregorian date.
pub fn intrinsic_integer_of_date(yyyymmdd: u32) -> i64 {
    let (y, m, d) = ((yyyymmdd / 10000) as i64, ((yyyymmdd / 100) % 100) as i64, (yyyymmdd % 100) as i64);
    days_from_civil(y, m, d) - cobol_epoch()
}

/// `FUNCTION DATE-OF-INTEGER(n)` — the Gregorian date `YYYYMMDD` for integer day number `n` (inverse of
/// [`intrinsic_integer_of_date`]).
pub fn intrinsic_date_of_integer(n: i64) -> u32 {
    let (y, m, d) = civil_from_days(cobol_epoch() + n);
    (y * 10000 + m * 100 + d) as u32
}

/// `FUNCTION INTEGER-OF-DAY(YYYYDDD)` — the integer day number for an ordinal (Julian) date.
pub fn intrinsic_integer_of_day(yyyyddd: u32) -> i64 {
    let (y, ddd) = ((yyyyddd / 1000) as i64, (yyyyddd % 1000) as i64);
    days_from_civil(y, 1, 1) - cobol_epoch() + ddd - 1
}

/// `FUNCTION DAY-OF-INTEGER(n)` — the ordinal date `YYYYDDD` for integer day number `n` (inverse of
/// [`intrinsic_integer_of_day`]).
pub fn intrinsic_day_of_integer(n: i64) -> u32 {
    let (y, _, _) = civil_from_days(cobol_epoch() + n);
    let ddd = n - (days_from_civil(y, 1, 1) - cobol_epoch()) + 1;
    (y as u32) * 1000 + ddd as u32
}

// ---- intrinsic.c `cob_intr_*` entry points + the result-field layer -----------------------------
// The C `cob_intr_*` functions return a transient result field (`curr_field`) built by the result-field
// allocator (`make_field_entry` / `cob_alloc_set_field_uint`/`_int`). This port returns the result field
// as an owned `(bytes, attr)` pair (the byte content is what the oracle observes); the rotating temp pool
// the C uses is just memory reuse — RAII here. The value logic is the already-sealed `intrinsic_*` fns.

use crate::attr::{FieldAttr, COB_FLAG_HAVE_SIGN, COB_TYPE_ALPHANUMERIC, COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_DISPLAY};

/// A `cob_intr_*` result field: the data bytes + their attribute.
pub type IntrField = (Vec<u8>, FieldAttr);

/// `make_field_entry (f)` (intrinsic.c): allocate the transient result field with `f`'s attribute and
/// `size` zeroed bytes (the C reuses a rotating pool; RAII makes that a plain allocation here).
pub fn make_field_entry(attr: &FieldAttr, size: usize) -> IntrField {
    (vec![0u8; size], *attr)
}

/// `cob_alloc_set_field_uint (val)` (intrinsic.c): a 4-byte native `BINARY` result field (`PIC 9(9) COMP`)
/// holding the unsigned value.
pub fn cob_alloc_set_field_uint(val: u32) -> IntrField {
    let attr = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: 0 };
    (val.to_ne_bytes().to_vec(), attr)
}

/// `cob_alloc_set_field_int (val)` (intrinsic.c): a 4-byte native `BINARY` result field; signed when
/// `val < 0`.
pub fn cob_alloc_set_field_int(val: i32) -> IntrField {
    let flags = if val < 0 { COB_FLAG_HAVE_SIGN } else { 0 };
    let attr = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags };
    (val.to_ne_bytes().to_vec(), attr)
}

const ALPHA1: FieldAttr = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };

/// `cob_intr_ord (srcfield)` (intrinsic.c): `FUNCTION ORD(c)` — `*data + 1` as a `BINARY` result.
pub fn cob_intr_ord(src: &[u8]) -> IntrField {
    cob_alloc_set_field_uint(src.first().copied().unwrap_or(0) as u32 + 1)
}

/// `cob_intr_char (srcfield)` (intrinsic.c): `FUNCTION CHAR(n)` — a 1-byte field holding `n-1` when
/// `n` is in `1..=256`, else `0`. `n` is read from the source as an integer.
pub fn cob_intr_char(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let i = crate::accessors::cob_get_int(src, src_attr);
    let mut r = make_field_entry(&ALPHA1, 1);
    r.0[0] = if !(1..=256).contains(&i) { 0 } else { (i - 1) as u8 };
    r
}

/// `cob_intr_byte_length (srcfield)` (intrinsic.c): `FUNCTION BYTE-LENGTH` — the field's byte size.
pub fn cob_intr_byte_length(src_size: usize) -> IntrField {
    cob_alloc_set_field_uint(src_size as u32)
}

/// `cob_intr_length (srcfield)` (intrinsic.c): `FUNCTION LENGTH` — the field's size (national fields are
/// divided by `COB_NATIONAL_SIZE`; this port handles the non-national case).
pub fn cob_intr_length(src_size: usize) -> IntrField {
    cob_alloc_set_field_uint(src_size as u32)
}

/// `cob_intr_upper_case (offset, length, srcfield)` (intrinsic.c): `FUNCTION UPPER-CASE` — the source
/// bytes ASCII-upper-cased (same size), optionally reference-modified by `(offset:length)`.
pub fn cob_intr_upper_case(offset: i32, length: i32, src: &[u8]) -> IntrField {
    intr_refmod(intrinsic_upper_case(src), offset, length)
}

/// `cob_intr_lower_case (offset, length, srcfield)` (intrinsic.c): `FUNCTION LOWER-CASE`.
pub fn cob_intr_lower_case(offset: i32, length: i32, src: &[u8]) -> IntrField {
    intr_refmod(intrinsic_lower_case(src), offset, length)
}

/// `cob_intr_reverse (offset, length, srcfield)` (intrinsic.c): `FUNCTION REVERSE`.
pub fn cob_intr_reverse(offset: i32, length: i32, src: &[u8]) -> IntrField {
    intr_refmod(intrinsic_reverse(src), offset, length)
}

/// Apply the `cob_intr_*` trailing `calc_ref_mod (curr_field, offset, length)` (`(offset:length)` on the
/// result) when `offset > 0`; otherwise return the whole result as an alphanumeric field.
fn intr_refmod(data: Vec<u8>, offset: i32, length: i32) -> IntrField {
    if offset > 0 {
        let start = (offset - 1).max(0) as usize;
        let len = if length > 0 { length as usize } else { data.len().saturating_sub(start) };
        let end = (start + len).min(data.len());
        let slice = if start <= data.len() { data[start..end].to_vec() } else { Vec::new() };
        (slice, ALPHA1)
    } else {
        (data, ALPHA1)
    }
}

// ---- numeric-result cob_intr_* (over the sealed CobDecimal layer) -------------------------------

use crate::cob_decimal::{cob_decimal_div, cob_decimal_get_field, cob_decimal_mul, cob_decimal_set_field, cob_decimal_sub, CobDecimal};
use crate::gmp::Mpz;

/// `cob_trim_decimal (d)` (intrinsic.c): strip trailing decimal zeros, lowering the scale (a zero value
/// becomes scale 0).
pub fn cob_trim_decimal(d: &mut CobDecimal) {
    if d.value.sgn() == 0 {
        d.scale = 0;
        return;
    }
    let ten = Mpz::from_i64(10);
    while d.scale > 0 {
        let (q, r) = d.value.tdiv_qr(&ten);
        if r.sgn() != 0 {
            break;
        }
        d.value = q;
        d.scale -= 1;
    }
}

/// `cob_alloc_field (d)` (intrinsic.c): choose the result field's attribute + size for a `cob_decimal` —
/// a 4-byte `BINARY` (fits 32 bits, scale < 10), an 8-byte `BINARY` (fits 64 bits, scale < 19), or a
/// `DISPLAY` field wide enough for the digits. Trims `d` first. Returns `(attr, size)`.
pub fn cob_alloc_field(d: &mut CobDecimal) -> (FieldAttr, usize) {
    cob_trim_decimal(d);
    let neg = d.value.sgn() < 0;
    let negsign = if neg { 1 } else { 0 };
    let flags = if neg { COB_FLAG_HAVE_SIGN } else { 0 };
    let bitnum = d.value.sizeinbase2();
    if bitnum < (33 - negsign) && d.scale < 10 {
        (FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: d.scale as i16, flags }, 4)
    } else if bitnum < (65 - negsign) && d.scale < 19 {
        (FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 20, scale: d.scale as i16, flags }, 8)
    } else {
        let digits10 = d.value.to_decimal_string().trim_start_matches('-').len();
        let size = digits10.max(d.scale.max(0) as usize);
        (FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: size as u16, scale: d.scale as i16, flags }, size)
    }
}

fn intr_decimal_result(mut d: CobDecimal) -> IntrField {
    let (attr, size) = cob_alloc_field(&mut d);
    let bytes = cob_decimal_get_field(d, &attr, size, crate::arith::Round::Truncate, false)
        .unwrap_or_else(|_| vec![0u8; size]);
    (bytes, attr)
}

/// `cob_intr_sign (srcfield)` (intrinsic.c): `FUNCTION SIGN(x)` — `-1`/`0`/`+1` as a `BINARY` result.
pub fn cob_intr_sign(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    cob_alloc_set_field_int(cob_decimal_set_field(src, src_attr).value.sgn())
}

/// `cob_intr_abs (srcfield)` (intrinsic.c): `FUNCTION ABS(x)` — `|x|` stored in a field with the source's
/// own attribute/size.
pub fn cob_intr_abs(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let mut d = cob_decimal_set_field(src, src_attr);
    d.value.abs();
    let bytes = cob_decimal_get_field(d, src_attr, src.len(), crate::arith::Round::Truncate, false)
        .unwrap_or_else(|_| vec![0u8; src.len()]);
    (bytes, *src_attr)
}

/// `cob_intr_integer (srcfield)` (intrinsic.c): `FUNCTION INTEGER(x)` — the floor (greatest integer not
/// greater than `x`).
pub fn cob_intr_integer(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let mut d = cob_decimal_set_field(src, src_attr);
    if d.scale < 0 {
        d.value = d.value.mul(&Mpz::ui_pow_ui(10, (-d.scale) as u32));
    } else if d.scale > 0 {
        let sign = d.value.sgn();
        let (q, r) = d.value.tdiv_qr(&Mpz::ui_pow_ui(10, d.scale as u32));
        d.value = q;
        if sign == -1 && r.sgn() != 0 {
            d.value = d.value.sub_ui(1); // floor adjust for negatives
        }
    }
    d.scale = 0;
    intr_decimal_result(d)
}

/// `cob_intr_integer_part (srcfield)` (intrinsic.c): `FUNCTION INTEGER-PART(x)` — truncation toward zero.
pub fn cob_intr_integer_part(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let mut d = cob_decimal_set_field(src, src_attr);
    if d.scale < 0 {
        d.value = d.value.mul(&Mpz::ui_pow_ui(10, (-d.scale) as u32));
    } else if d.scale > 0 {
        d.value = d.value.tdiv_q(&Mpz::ui_pow_ui(10, d.scale as u32));
    }
    d.scale = 0;
    intr_decimal_result(d)
}

/// Build a `CobDecimal` from a parsed [`Numval`] (`value = signed scaled * 10^(-scale)`).
fn numval_to_decimal(nv: &Numval) -> CobDecimal {
    let mut value = Mpz::from_u128(nv.scaled);
    if nv.negative {
        value.neg();
    }
    CobDecimal { value, scale: nv.scale as i32 }
}

/// `cob_intr_numval (srcfield)` (intrinsic.c): `FUNCTION NUMVAL(s)` — parse the field's text to a numeric
/// result field (wraps the sealed `intrinsic_numval`).
pub fn cob_intr_numval(src: &[u8]) -> IntrField {
    let s = String::from_utf8_lossy(src);
    intr_decimal_result(numval_to_decimal(&intrinsic_numval(&s)))
}

/// `cob_intr_numval_c (srcfield, currency)` (intrinsic.c): `FUNCTION NUMVAL-C(s)` — like
/// [`cob_intr_numval`] after stripping the default currency symbol + thousands commas.
pub fn cob_intr_numval_c(src: &[u8]) -> IntrField {
    let s = String::from_utf8_lossy(src);
    intr_decimal_result(numval_to_decimal(&intrinsic_numval_c(&s)))
}

/// `cob_mod_or_rem (f1, f2, func_is_rem)` (intrinsic.c): the shared `MOD`/`REM` core —
/// `f1 - q*f2` where `q` is `floor(f1/f2)` (MOD) or `trunc(f1/f2)` (REM). A zero divisor yields `0`.
pub fn cob_mod_or_rem(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr, func_is_rem: bool) -> IntrField {
    let mut q = cob_decimal_set_field(f1, a1);
    let d3 = cob_decimal_set_field(f2, a2);
    if d3.value.sgn() == 0 {
        return cob_alloc_set_field_uint(0);
    }
    let _ = cob_decimal_div(&mut q, &d3); // q = f1 / f2
    if q.scale < 0 {
        q.value = q.value.mul(&Mpz::ui_pow_ui(10, (-q.scale) as u32));
    } else if q.scale > 0 {
        let p = Mpz::ui_pow_ui(10, q.scale as u32);
        if func_is_rem {
            q.value = q.value.tdiv_q(&p); // REM uses INTEGER-PART (truncate)
        } else {
            let sign = q.value.sgn();
            let (quo, r) = q.value.tdiv_qr(&p); // MOD uses INTEGER (floor)
            q.value = quo;
            if sign == -1 && r.sgn() != 0 {
                q.value = q.value.sub_ui(1);
            }
        }
    }
    q.scale = 0;
    let f2dec = cob_decimal_set_field(f2, a2);
    cob_decimal_mul(&mut q, &f2dec); // q = q * f2
    let mut result = cob_decimal_set_field(f1, a1);
    cob_decimal_sub(&mut result, &q); // result = f1 - q*f2
    intr_decimal_result(result)
}

/// `cob_intr_mod (f1, f2)` (intrinsic.c): `FUNCTION MOD(a, b)` — remainder with the **divisor** sign.
pub fn cob_intr_mod(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr) -> IntrField {
    cob_mod_or_rem(f1, a1, f2, a2, false)
}

/// `cob_intr_rem (f1, f2)` (intrinsic.c): `FUNCTION REM(a, b)` — remainder with the **dividend** sign.
pub fn cob_intr_rem(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr) -> IntrField {
    cob_mod_or_rem(f1, a1, f2, a2, true)
}

// ---- date validators (intrinsic.c) + the date-conversion cob_intr_* wrappers -------------------

const NORMAL_MONTH_DAYS: [i32; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const LEAP_MONTH_DAYS: [i32; 13] = [0, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// `in_range (min, max, val)` (intrinsic.c): `min <= val <= max`.
pub fn in_range(min: i32, max: i32, val: i32) -> bool {
    min <= val && val <= max
}

/// `leap_year (year)` (intrinsic.c): Gregorian leap-year test.
pub fn leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// `days_in_year (year)` (intrinsic.c): 366 in a leap year, else 365.
pub fn days_in_year(year: i32) -> i32 {
    if leap_year(year) {
        366
    } else {
        365
    }
}

/// `valid_year (year)` (intrinsic.c): `1601..=9999`.
pub fn valid_year(year: i32) -> bool {
    in_range(1601, 9999, year)
}

/// `valid_month (month)` (intrinsic.c): `1..=12`.
pub fn valid_month(month: i32) -> bool {
    in_range(1, 12, month)
}

/// `valid_day_of_month (year, month, day)` (intrinsic.c): `day` within the month's length (leap-aware).
pub fn valid_day_of_month(year: i32, month: i32, day: i32) -> bool {
    if !valid_month(month) {
        return false;
    }
    let max = if leap_year(year) {
        LEAP_MONTH_DAYS[month as usize]
    } else {
        NORMAL_MONTH_DAYS[month as usize]
    };
    in_range(1, max, day)
}

/// `valid_day_of_year (year, doy)` (intrinsic.c): `doy` within the year's length.
pub fn valid_day_of_year(year: i32, doy: i32) -> bool {
    in_range(1, days_in_year(year), doy)
}

/// `valid_integer_date (days)` (intrinsic.c): the integer-date range `1..=3067671` (1601-01-01 base).
pub fn valid_integer_date(days: i32) -> bool {
    in_range(1, 3067671, days)
}

/// `cob_intr_integer_of_date (srcfield)` (intrinsic.c): `FUNCTION INTEGER-OF-DATE(YYYYMMDD)` — the day
/// number, or `0` (with an exception) for an invalid date.
pub fn cob_intr_integer_of_date(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let indate = crate::accessors::cob_get_int(src, src_attr);
    let year = indate / 10000;
    if !valid_year(year) {
        return cob_alloc_set_field_uint(0);
    }
    let md = indate % 10000;
    let month = md / 100;
    let day = md % 100;
    if !valid_month(month) || !valid_day_of_month(year, month, day) {
        return cob_alloc_set_field_uint(0);
    }
    cob_alloc_set_field_uint(intrinsic_integer_of_date(indate as u32) as u32)
}

/// `cob_intr_integer_of_day (srcfield)` (intrinsic.c): `FUNCTION INTEGER-OF-DAY(YYYYDDD)`.
pub fn cob_intr_integer_of_day(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let indate = crate::accessors::cob_get_int(src, src_attr);
    let year = indate / 1000;
    if !valid_year(year) {
        return cob_alloc_set_field_uint(0);
    }
    let doy = indate % 1000;
    if !valid_day_of_year(year, doy) {
        return cob_alloc_set_field_uint(0);
    }
    cob_alloc_set_field_uint(intrinsic_integer_of_day(indate as u32) as u32)
}

/// `cob_intr_date_of_integer (srcdays)` (intrinsic.c): `FUNCTION DATE-OF-INTEGER(days)` — an 8-digit
/// `YYYYMMDD` DISPLAY field, or `"00000000"` for an out-of-range day.
pub fn cob_intr_date_of_integer(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 8, scale: 0, flags: 0 };
    let days = crate::accessors::cob_get_int(src, src_attr);
    if !valid_integer_date(days) {
        return (b"00000000".to_vec(), attr);
    }
    (format!("{:08}", intrinsic_date_of_integer(days as i64)).into_bytes(), attr)
}

/// `cob_intr_day_of_integer (srcdays)` (intrinsic.c): `FUNCTION DAY-OF-INTEGER(days)` — a 7-digit
/// `YYYYDDD` DISPLAY field.
pub fn cob_intr_day_of_integer(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 7, scale: 0, flags: 0 };
    let days = crate::accessors::cob_get_int(src, src_attr);
    if !valid_integer_date(days) {
        return (b"0000000".to_vec(), attr);
    }
    (format!("{:07}", intrinsic_day_of_integer(days as i64)).into_bytes(), attr)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn len(pic: &str, u: Usage) -> usize {
        intrinsic_length(pic, u).unwrap()
    }
    fn nv(s: &str) -> String {
        numval_display(&intrinsic_numval(s), 8, 4)
    }
    fn nvc(s: &str) -> String {
        numval_display(&intrinsic_numval_c(s), 8, 4)
    }
    #[test]
    fn cob_intr_numeric_results() {
        use crate::attr::COB_FLAG_HAVE_SIGN;
        // S9(2)V99: -12.34 stored as "123t" (trailing negative overpunch '4'->'t'=0x74); +12.34 = "1234".
        let sattr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 4, scale: 2, flags: COB_FLAG_HAVE_SIGN };
        let neg = b"123t";
        let pos = b"1234";
        let bin = |d: &[u8], a: &FieldAttr| crate::accessors::cob_get_int(d, a);
        // SIGN
        assert_eq!(bin(&cob_intr_sign(neg, &sattr).0, &cob_intr_sign(neg, &sattr).1), -1);
        assert_eq!(bin(&cob_intr_sign(pos, &sattr).0, &cob_intr_sign(pos, &sattr).1), 1);
        // INTEGER (floor): -12.34 -> -13 ; 12.34 -> 12
        let (d, a) = cob_intr_integer(neg, &sattr);
        assert_eq!(bin(&d, &a), -13);
        let (d, a) = cob_intr_integer(pos, &sattr);
        assert_eq!(bin(&d, &a), 12);
        // INTEGER-PART (trunc): -12.34 -> -12 ; 12.34 -> 12
        let (d, a) = cob_intr_integer_part(neg, &sattr);
        assert_eq!(bin(&d, &a), -12);
        let (d, a) = cob_intr_integer_part(pos, &sattr);
        assert_eq!(bin(&d, &a), 12);
        // ABS(-12.34) -> +12.34 in the same S9(2)V99 field -> "1234"
        assert_eq!(cob_intr_abs(neg, &sattr).0, b"1234");
    }

    #[test]
    fn cob_intr_date_wrappers() {
        let disp = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 8, scale: 0, flags: 0 };
        // validators
        assert!(leap_year(2000));
        assert!(!leap_year(1900));
        assert!(leap_year(2024));
        assert!(valid_day_of_month(2024, 2, 29)); // leap Feb 29
        assert!(!valid_day_of_month(2023, 2, 29)); // non-leap
        assert!(!valid_month(13));
        assert!(valid_integer_date(1));
        assert!(!valid_integer_date(0));
        // INTEGER-OF-DATE(YYYYMMDD) then DATE-OF-INTEGER round-trips
        let (d, a) = cob_intr_integer_of_date(b"20240229", &disp);
        let days = crate::accessors::cob_get_int(&d, &a);
        assert!(days > 0);
        let days_disp = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 9, scale: 0, flags: 0 };
        let dbytes = format!("{days:09}").into_bytes();
        let (back, _) = cob_intr_date_of_integer(&dbytes, &days_disp);
        assert_eq!(back, b"20240229");
        // invalid date -> 0 / "00000000"
        assert_eq!(crate::accessors::cob_get_int(&cob_intr_integer_of_date(b"20240230", &disp).0,
            &FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: 0 }), 0);
        assert_eq!(cob_intr_date_of_integer(b"000000000", &days_disp).0, b"00000000");
    }

    #[test]
    fn cob_intr_result_fields() {
        use crate::attr::COB_TYPE_ALPHANUMERIC;
        let an = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };
        // ORD('A') = 66 -> 4-byte native binary
        let (d, a) = cob_intr_ord(b"A");
        assert_eq!(crate::accessors::cob_get_int(&d, &a), 66);
        // CHAR(66) = 'A'
        let (d, _) = cob_intr_char(b"\x42\x00\x00\x00", &FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: 0 });
        assert_eq!(d, b"A");
        // CHAR out of range -> 0
        let (d, _) = cob_intr_char(b"\x00\x00\x00\x00", &FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: 0 });
        assert_eq!(d, &[0u8]);
        // BYTE-LENGTH / LENGTH
        let (d, a) = cob_intr_byte_length(7);
        assert_eq!(crate::accessors::cob_get_int(&d, &a), 7);
        let (d, a) = cob_intr_length(5);
        assert_eq!(crate::accessors::cob_get_int(&d, &a), 5);
        // UPPER/LOWER/REVERSE
        assert_eq!(cob_intr_upper_case(0, 0, b"aB3").0, b"AB3");
        assert_eq!(cob_intr_lower_case(0, 0, b"aB3").0, b"ab3");
        assert_eq!(cob_intr_reverse(0, 0, b"abc").0, b"cba");
        // UPPER-CASE("hello")(2:3) -> "ELL"
        assert_eq!(cob_intr_upper_case(2, 3, b"hello").0, b"ELL");
        let _ = an;
    }

    #[test]
    fn numval_c_strips_currency_and_commas() {
        assert_eq!(nvc("$1,234.56"), "+00001234.5600");
        assert_eq!(nvc("1,234,567"), "+01234567.0000");
        assert_eq!(nvc("-$1,234.56"), "-00001234.5600");
        assert_eq!(nvc("$1,234.56CR"), "-00001234.5600");
        assert_eq!(nvc("  $42.00  "), "+00000042.0000");
    }
    #[test]
    fn date_conversion_intrinsics_match_oracle() {
        // INTEGER-OF-DATE: 1601-01-01=1, 2000-01-01=145732, 2024-02-29=154557 (leap), 1999-12-31=145731
        assert_eq!(intrinsic_integer_of_date(16010101), 1);
        assert_eq!(intrinsic_integer_of_date(20000101), 145732);
        assert_eq!(intrinsic_integer_of_date(20240229), 154557);
        assert_eq!(intrinsic_integer_of_date(19991231), 145731);
        // DATE-OF-INTEGER (inverse)
        assert_eq!(intrinsic_date_of_integer(1), 16010101);
        assert_eq!(intrinsic_date_of_integer(145732), 20000101);
        // INTEGER-OF-DAY / DAY-OF-INTEGER (ordinal dates)
        assert_eq!(intrinsic_integer_of_day(2000001), 145732);
        assert_eq!(intrinsic_integer_of_day(2024060), 154557);
        assert_eq!(intrinsic_day_of_integer(145732), 2000001);
        // round-trip over a range
        for n in 1..=200000i64 {
            assert_eq!(intrinsic_integer_of_date(intrinsic_date_of_integer(n)), n);
        }
    }
    #[test]
    fn ord_char_are_one_based_inverses() {
        assert_eq!([intrinsic_ord(b'A'), intrinsic_ord(b'0'), intrinsic_ord(b' '), intrinsic_ord(b'z')], [66, 49, 33, 123]);
        assert_eq!([intrinsic_char(66), intrinsic_char(49), intrinsic_char(1), intrinsic_char(256)], [b'A', b'0', 0, 255]);
        for n in 1u32..=256 { assert_eq!(intrinsic_ord(intrinsic_char(n)), n); } // round-trip
    }
    #[test]
    fn case_and_reverse_transforms() {
        assert_eq!(intrinsic_upper_case(b"aB3 z!"), b"AB3 Z!"); // non-alpha unchanged
        assert_eq!(intrinsic_lower_case(b"Ab3 Z!"), b"ab3 z!");
        assert_eq!(intrinsic_reverse(b"ab c"), b"c ba"); // spaces reversed too
        assert_eq!(intrinsic_reverse(b"12345"), b"54321");
    }
    #[test]
    fn integer_is_floor_integer_part_truncates() {
        let im = |s: &str| intrinsic_integer(intrinsic_numval(s).signed_mag(), intrinsic_numval(s).scale);
        let ip = |s: &str| intrinsic_integer_part(intrinsic_numval(s).signed_mag(), intrinsic_numval(s).scale);
        // INTEGER (floor): 3.7->3, -3.7->-4, 2.5->2, -2.5->-3, -3.0->-3, -0.1->-1
        assert_eq!([im("3.7"), im("-3.7"), im("2.5"), im("-2.5"), im("-3.0"), im("-0.1")], [3, -4, 2, -3, -3, -1]);
        // INTEGER-PART (truncate): 3.7->3, -3.7->-3, 2.5->2, -2.5->-2, 0.9->0
        assert_eq!([ip("3.7"), ip("-3.7"), ip("2.5"), ip("-2.5"), ip("0.9")], [3, -3, 2, -2, 0]);
    }
    #[test]
    fn mod_takes_divisor_sign_rem_takes_dividend_sign() {
        // MOD -> divisor sign: (17,5)=2 (-17,5)=3 (17,-5)=-3 (-17,-5)=-2
        assert_eq!([intrinsic_mod(17, 5), intrinsic_mod(-17, 5), intrinsic_mod(17, -5), intrinsic_mod(-17, -5)], [2, 3, -3, -2]);
        assert_eq!([intrinsic_mod(15, 5), intrinsic_mod(0, 5)], [0, 0]);
        // REM -> dividend sign: (17,5)=2 (-17,5)=-2 (17,-5)=2 (-17,-5)=-2
        assert_eq!([intrinsic_rem(17, 5), intrinsic_rem(-17, 5), intrinsic_rem(17, -5), intrinsic_rem(-17, -5)], [2, -2, 2, -2]);
    }
    #[test]
    fn numval_matches_oracle() {
        // oracle MOVE FUNCTION NUMVAL(...) TO S9(8)V9(4)
        assert_eq!(nv("123.45"), "+00000123.4500");
        assert_eq!(nv("  123  "), "+00000123.0000");
        assert_eq!(nv("-123.45"), "-00000123.4500");
        assert_eq!(nv("+123.45"), "+00000123.4500");
        assert_eq!(nv("123.45-"), "-00000123.4500");
        assert_eq!(nv("  -42 "), "-00000042.0000");
        assert_eq!(nv("123.45 CR"), "-00000123.4500");
        assert_eq!(nv("123.45 DB"), "-00000123.4500");
        assert_eq!(nv(".5"), "+00000000.5000");
        assert_eq!(nv("007"), "+00000007.0000");
        assert_eq!(nv("0"), "+00000000.0000");
    }
    #[test]
    fn length_matches_storage_bytes() {
        // oracle FUNCTION LENGTH: X(5)=5 · 9(5)=5 · S9(8)V99=10 · S9(3)COMP-3=2 · 9(4)COMP=2 ·
        //                         9(7)COMP-3=4 · S9(9)COMP=4 · X(1)=1
        assert_eq!(len("X(5)", Usage::Display), 5);
        assert_eq!(len("9(5)", Usage::Display), 5);
        assert_eq!(len("S9(8)V99", Usage::Display), 10);
        assert_eq!(len("S9(3)", Usage::Comp3), 2);
        assert_eq!(len("9(4)", Usage::Comp), 2);
        assert_eq!(len("9(7)", Usage::Comp3), 4);
        assert_eq!(len("S9(9)", Usage::Comp), 4);
        assert_eq!(len("X(1)", Usage::Display), 1);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // KANIFOR: GNURUST.INTRINSIC.ORD-CHAR.1
    /// ORD and CHAR are exact 1-based inverses over the whole collating sequence (unbounded).
    #[kani::proof]
    fn ord_char_round_trip() {
        let n: u32 = kani::any();
        kani::assume((1..=256).contains(&n));
        assert_eq!(intrinsic_ord(intrinsic_char(n)), n);
    }

    // KANIFOR: GNURUST.INTRINSIC.MOD-REM.1
    /// MOD is a valid residue with the DIVISOR sign and |MOD| < |b|; REM is the truncated remainder.
    #[kani::proof]
    fn mod_rem_invariants() {
        let a: i64 = kani::any();
        let b: i64 = kani::any();
        kani::assume(b != 0);
        let (a, b) = (a as i128, b as i128);
        let m = intrinsic_mod(a, b);
        let r = intrinsic_rem(a, b);
        assert_eq!(r, a % b);
        assert_eq!((a - m) % b, 0); // m is a residue of a mod b
        assert!(m == 0 || (m > 0) == (b > 0)); // divisor sign
        assert!(m.unsigned_abs() < b.unsigned_abs());
    }

    // KANIFOR: GNURUST.INTRINSIC.INTEGER.1
    /// INTEGER (floor) <= INTEGER-PART (truncate), and they differ by at most 1.
    #[kani::proof]
    fn integer_floor_le_part_trunc() {
        let mag: i64 = kani::any();
        let scale: u32 = kani::any();
        kani::assume(scale <= 9);
        let mag = mag as i128;
        let fl = intrinsic_integer(mag, scale);
        let tr = intrinsic_integer_part(mag, scale);
        assert!(fl <= tr);
        assert!(tr - fl <= 1);
    }

    // KANIFOR: GNURUST.INTRINSIC.DATE.1
    /// DATE-OF-INTEGER and INTEGER-OF-DATE are exact inverses across the admitted range.
    #[kani::proof]
    fn date_round_trip() {
        let d: i64 = kani::any();
        kani::assume(d >= 1 && d <= 3_000_000); // ~ years 1601..9999
        assert_eq!(intrinsic_integer_of_date(intrinsic_date_of_integer(d)), d);
    }

    // KANIFOR: GNURUST.INTRINSIC.CASE.1
    /// Case folding and REVERSE preserve length; REVERSE is an involution.
    #[kani::proof]
    fn case_reverse_preserve_length() {
        let s: [u8; 4] = kani::any();
        assert_eq!(intrinsic_upper_case(&s).len(), s.len());
        assert_eq!(intrinsic_lower_case(&s).len(), s.len());
        let r = intrinsic_reverse(&s);
        assert_eq!(r.len(), s.len());
        assert_eq!(intrinsic_reverse(&r), s.to_vec());
    }

    // KANIFOR: GNURUST.INTRINSIC.NUMVAL.1, GNURUST.INTRINSIC.NUMVAL-C.1
    /// NUMVAL / NUMVAL-C never panic on symbolic input and the scale is bounded by the input length.
    #[kani::proof]
    fn numval_no_panic_bounded_scale() {
        let s: [u8; 6] = kani::any();
        let st = core::str::from_utf8(&s);
        if let Ok(txt) = st {
            let nv = intrinsic_numval(txt);
            assert!(nv.scale as usize <= txt.len());
            let _ = intrinsic_numval_c(txt);
        }
    }

    // KANIFOR: GNURUST.INTRINSIC.LENGTH.1
    /// FUNCTION LENGTH equals the field model's storage size (here: X(n) is n bytes), never panics.
    #[kani::proof]
    fn length_of_alpha_is_n() {
        let n: usize = kani::any();
        kani::assume(n >= 1 && n <= 9);
        let pic = match n { 1=>"X(1)",2=>"X(2)",3=>"X(3)",4=>"X(4)",5=>"X(5)",6=>"X(6)",7=>"X(7)",8=>"X(8)",_=>"X(9)" };
        assert_eq!(intrinsic_length(pic, Usage::Display).unwrap(), n);
    }
}
