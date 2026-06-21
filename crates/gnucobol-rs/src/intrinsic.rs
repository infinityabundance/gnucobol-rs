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

/// `FUNCTION NUMVAL(s)` honoring `DECIMAL-POINT IS COMMA`: when `decimal_comma`, `,` is the decimal point
/// and `.` the grouping separator. Modelled by swapping the two characters into the standard `.`-decimal
/// form (the integer-part grouping is then dropped by [`intrinsic_numval`]'s non-digit filter), so
/// `NUMVAL("1.234,56")` under DECIMAL-POINT IS COMMA = `1234.56`. `decimal_comma == false` is exactly
/// [`intrinsic_numval`].
pub fn intrinsic_numval_cfg(s: &str, decimal_comma: bool) -> Numval {
    if decimal_comma {
        let swapped: String = s
            .chars()
            .map(|c| match c {
                '.' => ',',
                ',' => '.',
                x => x,
            })
            .collect();
        intrinsic_numval(&swapped)
    } else {
        intrinsic_numval(s)
    }
}

/// `FUNCTION NUMVAL-C(s)` for the narrow admitted form: like [`intrinsic_numval`] but first strips the
/// default currency symbol `$` and thousands-separator commas (`GNURUST.INTRINSIC.NUMVAL-C.1`). So
/// `NUMVAL-C("$1,234.56") = 1234.56`.
pub fn intrinsic_numval_c(s: &str) -> Numval {
    intrinsic_numval_c_cfg(s, '$', false)
}

/// `FUNCTION NUMVAL-C(s)` honoring the program's `CURRENCY SIGN` (`currency`) and `DECIMAL-POINT IS COMMA`
/// (`decimal_comma`): strips the currency symbol, then parses via [`intrinsic_numval_cfg`] (whose non-digit
/// filter drops the thousands separators). So `NUMVAL-C("F1.234,56")` under `CURRENCY SIGN IS "F"` +
/// `DECIMAL-POINT IS COMMA` = `1234.56`. The default (`'$'`, no comma) is the byte-unchanged
/// [`intrinsic_numval_c`].
pub fn intrinsic_numval_c_cfg(s: &str, currency: char, decimal_comma: bool) -> Numval {
    let stripped: String = s.chars().filter(|&c| c != currency).collect();
    intrinsic_numval_cfg(&stripped, decimal_comma)
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

/// `calc_ref_mod (field, offset, length)` (intrinsic.c): reference-modify a result field to `(offset:length)`
/// (1-based). The exact-name 1:1 alias of the shared [`intr_refmod`] used by every `cob_intr_*` tail.
#[allow(dead_code)]
fn calc_ref_mod(data: Vec<u8>, offset: i32, length: i32) -> IntrField {
    intr_refmod(data, offset, length)
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

use crate::accessors::cob_get_int;
use crate::int_pow::cob_s32_pow;
use crate::attr::{COB_TYPE_ALPHANUMERIC_ALL, COB_TYPE_ALPHANUMERIC_EDITED, COB_TYPE_NUMERIC_COMP5, COB_TYPE_NUMERIC_DOUBLE, COB_TYPE_NUMERIC_EDITED, COB_TYPE_NUMERIC_FLOAT, COB_TYPE_NUMERIC_L_DOUBLE, COB_TYPE_NUMERIC_PACKED};
use crate::cob_decimal::{cob_decimal_add, cob_decimal_cmp, cob_decimal_div, cob_decimal_get_field, cob_decimal_get_mpf, cob_decimal_mul, cob_decimal_set_field, cob_decimal_set_mpf, cob_decimal_sub, CobDecimal};
use crate::mpf::{cob_mpf_acos, cob_mpf_asin, cob_mpf_atan, cob_mpf_cos, cob_mpf_exp, cob_mpf_log, cob_mpf_log10, cob_mpf_sin, cob_mpf_tan, cob_pi, Mpf, COB_MPF_PREC};
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
        // `mpz_sizeinbase(value, 10)` — GMP estimates the base-10 digit count from the bit length and may
        // return the exact count OR one too many (it is exact only for power-of-two bases). The result
        // field width follows that estimate, so replicate the formula (chars_per_bit_exactly = log10(2)).
        let digits10 = (d.value.sizeinbase2() as f64 * 0.301_029_995_663_981_2_f64) as usize + 1;
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

/// `cob_intr_concatenate (offset, length, params, ...)` (intrinsic.c): `FUNCTION CONCATENATE` — the
/// operands' bytes joined, optionally reference-modified `(offset:length)`.
pub fn cob_intr_concatenate(offset: i32, length: i32, parts: &[&[u8]]) -> IntrField {
    let mut data = Vec::new();
    for p in parts {
        data.extend_from_slice(p);
    }
    intr_refmod(data, offset, length)
}

/// `cob_intr_sum (params, ...)` (intrinsic.c): `FUNCTION SUM` — the sum of the numeric operands.
pub fn cob_intr_sum(fields: &[(&[u8], &FieldAttr)]) -> IntrField {
    let mut d = CobDecimal { value: Mpz::from_u64(0), scale: 0 };
    for (f, a) in fields {
        let d2 = cob_decimal_set_field(f, a);
        cob_decimal_add(&mut d, &d2);
    }
    intr_decimal_result(d)
}

/// `cob_intr_max (params, ...)` (intrinsic.c): `FUNCTION MAX` — the operand with the greatest value,
/// returned in its own field (numeric compare).
pub fn cob_intr_max(fields: &[(&[u8], &FieldAttr)]) -> IntrField {
    let mut best = 0usize;
    for i in 1..fields.len() {
        if crate::cob_decimal::cob_numeric_cmp(fields[i].0, fields[i].1, fields[best].0, fields[best].1) > 0 {
            best = i;
        }
    }
    (fields[best].0.to_vec(), *fields[best].1)
}

/// `cob_intr_min (params, ...)` (intrinsic.c): `FUNCTION MIN` — the operand with the least value.
pub fn cob_intr_min(fields: &[(&[u8], &FieldAttr)]) -> IntrField {
    let mut best = 0usize;
    for i in 1..fields.len() {
        if crate::cob_decimal::cob_numeric_cmp(fields[i].0, fields[i].1, fields[best].0, fields[best].1) < 0 {
            best = i;
        }
    }
    (fields[best].0.to_vec(), *fields[best].1)
}

/// Index of the min and max operand (numeric compare) — the `get_min_and_max_of_args` helper.
/// `comp_field (m1, m2)` (intrinsic.c): the field comparator behind MAX/MIN/ORD-MAX/ORD-MIN/RANGE —
/// `cob_cmp(f1, f2)` (the numeric comparison the sealed statistics intrinsics use).
fn comp_field(f1: (&[u8], &FieldAttr), f2: (&[u8], &FieldAttr)) -> i32 {
    crate::cob_decimal::cob_numeric_cmp(f1.0, f1.1, f2.0, f2.1)
}

/// `get_min_and_max_of_args (num_args, args, min, max)` (intrinsic.c): the indices of the least and greatest
/// operands.
fn get_min_and_max_of_args(fields: &[(&[u8], &FieldAttr)]) -> (usize, usize) {
    let (mut mn, mut mx) = (0usize, 0usize);
    for i in 1..fields.len() {
        if comp_field(fields[i], fields[mn]) < 0 {
            mn = i;
        }
        if comp_field(fields[i], fields[mx]) > 0 {
            mx = i;
        }
    }
    (mn, mx)
}

/// `cob_intr_ord_min (params, ...)` (intrinsic.c): `FUNCTION ORD-MIN` — the **1-based** ordinal of the
/// least operand.
pub fn cob_intr_ord_min(fields: &[(&[u8], &FieldAttr)]) -> IntrField {
    cob_alloc_set_field_uint(get_min_and_max_of_args(fields).0 as u32 + 1)
}

/// `cob_intr_ord_max (params, ...)` (intrinsic.c): `FUNCTION ORD-MAX` — the 1-based ordinal of the
/// greatest operand.
pub fn cob_intr_ord_max(fields: &[(&[u8], &FieldAttr)]) -> IntrField {
    cob_alloc_set_field_uint(get_min_and_max_of_args(fields).1 as u32 + 1)
}

/// `cob_intr_range (params, ...)` (intrinsic.c): `FUNCTION RANGE` — `max - min`.
pub fn cob_intr_range(fields: &[(&[u8], &FieldAttr)]) -> IntrField {
    let (mn, mx) = get_min_and_max_of_args(fields);
    let mut d = cob_decimal_set_field(fields[mx].0, fields[mx].1);
    let dmin = cob_decimal_set_field(fields[mn].0, fields[mn].1);
    cob_decimal_sub(&mut d, &dmin);
    intr_decimal_result(d)
}

/// `cob_intr_midrange (params, ...)` (intrinsic.c): `FUNCTION MIDRANGE` — `(max + min) / 2`.
pub fn cob_intr_midrange(fields: &[(&[u8], &FieldAttr)]) -> IntrField {
    let (mn, mx) = get_min_and_max_of_args(fields);
    let mut d = cob_decimal_set_field(fields[mn].0, fields[mn].1);
    let dmax = cob_decimal_set_field(fields[mx].0, fields[mx].1);
    cob_decimal_add(&mut d, &dmax);
    let two = CobDecimal { value: Mpz::from_u64(2), scale: 0 };
    let _ = cob_decimal_div(&mut d, &two);
    intr_decimal_result(d)
}

/// `cob_intr_mean (params, ...)` (intrinsic.c): `FUNCTION MEAN` — the arithmetic mean of the operands.
pub fn cob_intr_mean(fields: &[(&[u8], &FieldAttr)]) -> IntrField {
    if fields.len() == 1 {
        return (fields[0].0.to_vec(), *fields[0].1);
    }
    let mut d = CobDecimal { value: Mpz::from_u64(0), scale: 0 };
    for (f, a) in fields {
        let d2 = cob_decimal_set_field(f, a);
        cob_decimal_add(&mut d, &d2);
    }
    let n = CobDecimal { value: Mpz::from_u64(fields.len() as u64), scale: 0 };
    let _ = cob_decimal_div(&mut d, &n);
    intr_decimal_result(d)
}

/// `cob_intr_median (params, ...)` (intrinsic.c): `FUNCTION MEDIAN` — the middle value (or the mean of the
/// two middle values) after sorting the operands.
pub fn cob_intr_median(fields: &[(&[u8], &FieldAttr)]) -> IntrField {
    let n = fields.len();
    if n == 1 {
        return (fields[0].0.to_vec(), *fields[0].1);
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        crate::cob_decimal::cob_numeric_cmp(fields[a].0, fields[a].1, fields[b].0, fields[b].1).cmp(&0)
    });
    let i = n / 2;
    if n % 2 == 1 {
        let k = order[i];
        (fields[k].0.to_vec(), *fields[k].1)
    } else {
        let mut d = cob_decimal_set_field(fields[order[i - 1]].0, fields[order[i - 1]].1);
        let d2 = cob_decimal_set_field(fields[order[i]].0, fields[order[i]].1);
        cob_decimal_add(&mut d, &d2);
        let two = CobDecimal { value: Mpz::from_u64(2), scale: 0 };
        let _ = cob_decimal_div(&mut d, &two);
        intr_decimal_result(d)
    }
}

/// `cob_intr_factorial (srcfield)` (intrinsic.c): `FUNCTION FACTORIAL(n)` — `n!` (0 for `n < 0`).
pub fn cob_intr_factorial(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let n = crate::accessors::cob_get_int(src, src_attr);
    if n < 0 {
        return cob_alloc_set_field_uint(0);
    }
    let mut value = Mpz::from_u64(1);
    for k in 2..=n as u64 {
        value = value.mul_ui(k);
    }
    intr_decimal_result(CobDecimal { value, scale: 0 })
}

/// `cob_intr_hex_of (srcfield)` (intrinsic.c): `FUNCTION HEX-OF` — each byte as two uppercase hex digits.
pub fn cob_intr_hex_of(src: &[u8]) -> IntrField {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = Vec::with_capacity(src.len() * 2);
    for &b in src {
        out.push(HEX[(b >> 4) as usize & 0xF]);
        out.push(HEX[(b & 0xF) as usize]);
    }
    (out, ALPHA1)
}

/// One hex digit's value (`0..15`), or `None` for a non-hex char.
fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c & 0x0F),
        b'A'..=b'F' => Some(c - b'A' + 10),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

/// `cob_intr_hex_to_char (srcfield)` (intrinsic.c): `FUNCTION HEX-TO-CHAR` — each hex pair → a byte
/// (`size/2` bytes; a non-hex digit contributes 0).
pub fn cob_intr_hex_to_char(src: &[u8]) -> IntrField {
    let size = src.len() / 2;
    let mut out = Vec::with_capacity(size);
    for i in 0..size {
        let hi = hex_digit(src[i * 2]).unwrap_or(0);
        let lo = hex_digit(src[i * 2 + 1]).unwrap_or(0);
        out.push(hi * 16 + lo);
    }
    (out, ALPHA1)
}

/// `cob_intr_bit_of (srcfield)` (intrinsic.c): `FUNCTION BIT-OF` — each byte as 8 `'0'`/`'1'` chars (MSB
/// first).
pub fn cob_intr_bit_of(src: &[u8]) -> IntrField {
    let mut out = Vec::with_capacity(src.len() * 8);
    for &b in src {
        for bit in (0..8).rev() {
            out.push(if b & (1 << bit) != 0 { b'1' } else { b'0' });
        }
    }
    (out, ALPHA1)
}

/// `has_bit_checked (byte)` (intrinsic.c): a bit char is set unless it is `'0'` (`'1'` is set; any other
/// char is treated as set, with an argument exception).
fn has_bit_checked(c: u8) -> bool {
    c != b'0'
}

/// `cob_intr_bit_to_char (srcfield)` (intrinsic.c): `FUNCTION BIT-TO-CHAR` — each group of 8 bit chars → a
/// byte (`size/8` bytes).
pub fn cob_intr_bit_to_char(src: &[u8]) -> IntrField {
    let size = src.len() / 8;
    let mut out = Vec::with_capacity(size);
    for i in 0..size {
        let mut byte = 0u8;
        for bit in 0..8 {
            if has_bit_checked(src[i * 8 + bit]) {
                byte |= 1 << (7 - bit);
            }
        }
        out.push(byte);
    }
    (out, ALPHA1)
}

/// `10^digits - 1` as a decimal at the given scale (the all-nines magnitude of a `digits`-digit field).
fn nines_decimal(digits: u16, scale: i16) -> CobDecimal {
    CobDecimal { value: Mpz::ui_pow_ui(10, digits as u32).sub_ui(1), scale: scale as i32 }
}

/// `cob_intr_lowest_algebraic (srcfield)` (intrinsic.c): `FUNCTION LOWEST-ALGEBRAIC` — the smallest value
/// the field can hold. Numeric unsigned → 0; signed `DISPLAY`/`PACKED`/`EDITED` → `-(10^digits - 1)`;
/// `BINARY`/`COMP-5` follow the binary-truncate default; `ALPHANUMERIC` → a `size`-byte field; float →
/// argument exception + 0.
pub fn cob_intr_lowest_algebraic(src_len: usize, attr: &FieldAttr) -> IntrField {
    match attr.field_type {
        COB_TYPE_ALPHANUMERIC | COB_TYPE_ALPHANUMERIC_ALL => (vec![0u8; src_len], ALPHA1),
        COB_TYPE_ALPHANUMERIC_EDITED => (vec![0u8; attr.digits as usize], ALPHA1),
        COB_TYPE_NUMERIC_BINARY | COB_TYPE_NUMERIC_COMP5 => {
            if attr.flags & COB_FLAG_HAVE_SIGN == 0 {
                return cob_alloc_set_field_uint(0);
            }
            let mut d = if attr.field_type == COB_TYPE_NUMERIC_COMP5 {
                let expo = (src_len as u32) * 8 - 1;
                CobDecimal { value: Mpz::ui_pow_ui(2, expo), scale: attr.scale as i32 }
            } else {
                nines_decimal(attr.digits, attr.scale)
            };
            d.value.neg();
            intr_decimal_result(d)
        }
        COB_TYPE_NUMERIC_FLOAT | COB_TYPE_NUMERIC_DOUBLE | COB_TYPE_NUMERIC_L_DOUBLE => {
            cob_alloc_set_field_uint(0)
        }
        _ => {
            // NUMERIC_DISPLAY / NUMERIC_PACKED / NUMERIC_EDITED
            if attr.flags & COB_FLAG_HAVE_SIGN == 0 {
                return cob_alloc_set_field_uint(0);
            }
            let mut d = nines_decimal(attr.digits, attr.scale);
            d.value.neg();
            intr_decimal_result(d)
        }
    }
}

/// `cob_intr_highest_algebraic (srcfield)` (intrinsic.c): `FUNCTION HIGHEST-ALGEBRAIC` — the largest value
/// the field can hold. Numeric `DISPLAY`/`PACKED`/`EDITED` → `10^digits - 1`; `BINARY`/`COMP-5` follow the
/// binary-truncate default; `ALPHANUMERIC` → a `size`-byte field of `0xFF`; float → argument exception + 0.
pub fn cob_intr_highest_algebraic(src_len: usize, attr: &FieldAttr) -> IntrField {
    match attr.field_type {
        COB_TYPE_ALPHANUMERIC | COB_TYPE_ALPHANUMERIC_ALL => (vec![0xFFu8; src_len], ALPHA1),
        COB_TYPE_ALPHANUMERIC_EDITED => (vec![0xFFu8; attr.digits as usize], ALPHA1),
        COB_TYPE_NUMERIC_BINARY | COB_TYPE_NUMERIC_COMP5 => {
            let d = if attr.field_type == COB_TYPE_NUMERIC_COMP5 {
                let mut expo = (src_len as u32) * 8;
                if attr.flags & COB_FLAG_HAVE_SIGN != 0 {
                    expo -= 1;
                }
                CobDecimal { value: Mpz::ui_pow_ui(2, expo).sub_ui(1), scale: attr.scale as i32 }
            } else {
                nines_decimal(attr.digits, attr.scale)
            };
            intr_decimal_result(d)
        }
        COB_TYPE_NUMERIC_FLOAT | COB_TYPE_NUMERIC_DOUBLE | COB_TYPE_NUMERIC_L_DOUBLE => {
            cob_alloc_set_field_uint(0)
        }
        _ => intr_decimal_result(nines_decimal(attr.digits, attr.scale)),
    }
}

/// `valid_decimal_time (seconds_from_midnight)` (intrinsic.c): the time-of-day is valid when the
/// seconds-from-midnight count does not exceed `SECONDS_IN_DAY` (86400).
fn valid_decimal_time(seconds_from_midnight: &CobDecimal) -> bool {
    let seconds_in_day = CobDecimal { value: { let mut m = Mpz::new(); m.set_ui(86400); m }, scale: 0 };
    cob_decimal_cmp(seconds_from_midnight, &seconds_in_day) <= 0
}

/// `cob_intr_combined_datetime (srcdays, srctime)` (intrinsic.c): `FUNCTION COMBINED-DATETIME` —
/// `integer-date + (seconds-from-midnight / 100000)`; invalid date or time → argument exception + 0.
pub fn cob_intr_combined_datetime(
    days: &[u8],
    days_attr: &FieldAttr,
    time: &[u8],
    time_attr: &FieldAttr,
) -> IntrField {
    let srdays = cob_get_int(days, days_attr);
    if !valid_integer_date(srdays) {
        return cob_alloc_set_field_uint(0);
    }
    let mut combined = CobDecimal { value: { let mut m = Mpz::new(); m.set_ui(srdays as u64); m }, scale: 0 };
    let mut srtime = cob_decimal_set_field(time, time_attr);
    if !valid_decimal_time(&srtime) {
        return cob_alloc_set_field_uint(0);
    }
    let hundred_thousand = CobDecimal { value: { let mut m = Mpz::new(); m.set_ui(100000); m }, scale: 0 };
    let _ = cob_decimal_div(&mut srtime, &hundred_thousand);
    cob_decimal_add(&mut combined, &srtime);
    intr_decimal_result(combined)
}

/// `cob_intr_fraction_part (srcfield)` (intrinsic.c): `FUNCTION FRACTION-PART` — the digits right of the
/// decimal point (`value mod 10^scale`, keeping the scale); an integer source yields 0.
pub fn cob_intr_fraction_part(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let mut d = cob_decimal_set_field(src, src_attr);
    if d.scale > 0 {
        let m = Mpz::ui_pow_ui(10, d.scale as u32);
        d.value = d.value.tdiv_r(&m);
    } else {
        d.value.set_ui(0);
        d.scale = 0;
    }
    intr_decimal_result(d)
}

/// `cob_intr_test_date_yyyymmdd (srcfield)` (intrinsic.c): `FUNCTION TEST-DATE-YYYYMMDD` — 0 if the
/// `YYYYMMDD` integer is a valid date, else 1 (bad year), 2 (bad month), or 3 (bad day-of-month).
pub fn cob_intr_test_date_yyyymmdd(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let mut indate = cob_get_int(src, src_attr);
    let year = indate / 10000;
    if !valid_year(year) {
        return cob_alloc_set_field_uint(1);
    }
    indate %= 10000;
    let month = indate / 100;
    if !valid_month(month) {
        return cob_alloc_set_field_uint(2);
    }
    let days = indate % 100;
    if !valid_day_of_month(year, month, days) {
        return cob_alloc_set_field_uint(3);
    }
    cob_alloc_set_field_uint(0)
}

/// `cob_intr_test_day_yyyyddd (srcfield)` (intrinsic.c): `FUNCTION TEST-DAY-YYYYDDD` — 0 if the `YYYYDDD`
/// integer is a valid ordinal date, else 1 (bad year) or 2 (bad day-of-year).
pub fn cob_intr_test_day_yyyyddd(src: &[u8], src_attr: &FieldAttr) -> IntrField {
    let indate = cob_get_int(src, src_attr);
    let year = indate / 1000;
    if !valid_year(year) {
        return cob_alloc_set_field_uint(1);
    }
    let days = indate % 1000;
    if !valid_day_of_year(year, days) {
        return cob_alloc_set_field_uint(2);
    }
    cob_alloc_set_field_uint(0)
}

/// `cob_intr_trim (offset, length, srcfield, direction)` (intrinsic.c): `FUNCTION TRIM` — strips spaces;
/// `direction` 0 trims both ends, 1 (LEADING) trims the left, 2 (TRAILING) trims the right. An all-space
/// source yields a zero-length result. Reference modification is applied when `offset > 0`.
pub fn cob_intr_trim(offset: i32, length: i32, src: &[u8], src_attr: &FieldAttr, direction: i32) -> IntrField {
    if src.iter().all(|&b| b == b' ') {
        return (Vec::new(), *src_attr);
    }
    let mut begin = 0usize;
    let mut end = src.len() - 1;
    if direction != 2 {
        while src[begin] == b' ' {
            begin += 1;
        }
    }
    if direction != 1 {
        while src[end] == b' ' {
            end -= 1;
        }
    }
    let out: Vec<u8> = src[begin..=end].to_vec();
    if offset > 0 {
        return intr_refmod(out, offset, length);
    }
    (out, *src_attr)
}

/// Whether two equal-length byte runs match, optionally folding ASCII case (mirrors `memcmp` vs
/// `strncasecmp` under `LC_ALL=C`).
fn run_eq(a: &[u8], b: &[u8], case_insensitive: bool) -> bool {
    if case_insensitive {
        a.iter().zip(b).all(|(&x, &y)| x.to_ascii_lowercase() == y.to_ascii_lowercase())
    } else {
        a == b
    }
}

/// `substitute (offset, length, params, cmp_func, ...)` (intrinsic.c): the shared engine behind
/// `FUNCTION SUBSTITUTE`/`SUBSTITUTE-CASE`. A single left-to-right pass replaces the first matching
/// `(match, replacement)` pair at each position (first pair wins), copying unmatched bytes verbatim. An
/// empty match operand is skipped (libcob would otherwise loop). Reference modification applies when
/// `offset > 0`.
/// Whether a `(match, replacement)` pair matches `original` at index `i`.
fn substitute_match_at(original: &[u8], i: usize, m: &[u8], case_insensitive: bool) -> bool {
    !m.is_empty() && i + m.len() <= original.len() && run_eq(&original[i..i + m.len()], m, case_insensitive)
}

/// `get_substituted_size (original, matches, reps, numreps, cmp)` (intrinsic.c): the length of the
/// SUBSTITUTE result (first matching pair at each position advances by its match length).
fn get_substituted_size(original: &[u8], pairs: &[(&[u8], &[u8])], case_insensitive: bool) -> usize {
    let mut size = 0;
    let mut i = 0;
    while i < original.len() {
        match pairs.iter().find(|(m, _)| substitute_match_at(original, i, m, case_insensitive)) {
            Some((m, r)) => {
                size += r.len();
                i += m.len();
            }
            None => {
                size += 1;
                i += 1;
            }
        }
    }
    size
}

/// `substitute_matches (original, matches, reps, numreps, cmp, replaced_begin)` (intrinsic.c): write the
/// SUBSTITUTE result into `out`.
fn substitute_matches(original: &[u8], pairs: &[(&[u8], &[u8])], case_insensitive: bool, out: &mut Vec<u8>) {
    let mut i = 0;
    while i < original.len() {
        match pairs.iter().find(|(m, _)| substitute_match_at(original, i, m, case_insensitive)) {
            Some((m, r)) => {
                out.extend_from_slice(r);
                i += m.len();
            }
            None => {
                out.push(original[i]);
                i += 1;
            }
        }
    }
}

/// `substitute (offset, length, params, cmp_func, ...)` (intrinsic.c): the shared engine behind
/// `FUNCTION SUBSTITUTE`/`SUBSTITUTE-CASE` — size the result, write the matches, optionally reference-modify.
/// An empty match operand is skipped (libcob would otherwise loop).
fn substitute(offset: i32, length: i32, original: &[u8], pairs: &[(&[u8], &[u8])], case_insensitive: bool) -> IntrField {
    let mut out = Vec::with_capacity(get_substituted_size(original, pairs, case_insensitive));
    substitute_matches(original, pairs, case_insensitive, &mut out);
    if offset > 0 {
        return intr_refmod(out, offset, length);
    }
    (out, ALPHA1)
}

/// `cob_intr_substitute (offset, length, params, ...)` (intrinsic.c): `FUNCTION SUBSTITUTE` — case-sensitive
/// pattern replacement.
pub fn cob_intr_substitute(offset: i32, length: i32, original: &[u8], pairs: &[(&[u8], &[u8])]) -> IntrField {
    substitute(offset, length, original, pairs, false)
}

/// `cob_intr_substitute_case (offset, length, params, ...)` (intrinsic.c): `FUNCTION SUBSTITUTE-CASE` —
/// case-insensitive pattern replacement.
pub fn cob_intr_substitute_case(offset: i32, length: i32, original: &[u8], pairs: &[(&[u8], &[u8])]) -> IntrField {
    substitute(offset, length, original, pairs, true)
}

/// The shared 2-digit-year windowing (`maxyear = current_year + interval`; pivot on `maxyear % 100`).
/// Returns the full year, or `None` if the window bounds are invalid.
fn window_year(year: i32, interval: i32, current_year: i32) -> Option<i32> {
    let maxyear = current_year + interval;
    if !valid_year(current_year) || maxyear < 1700 || maxyear > 9999 {
        return None;
    }
    Some(if maxyear % 100 >= year {
        year + 100 * (maxyear / 100)
    } else {
        year + 100 * ((maxyear / 100) - 1)
    })
}

/// `cob_intr_year_to_yyyy (params, ...)` (intrinsic.c): `FUNCTION YEAR-TO-YYYY(yy, interval, base-year)`.
/// The default base-year (localtime) is a non-claim; this takes it explicitly (the deterministic form).
pub fn cob_intr_year_to_yyyy(year: i32, interval: i32, current_year: i32) -> IntrField {
    if !(0..=99).contains(&year) {
        return cob_alloc_set_field_uint(0);
    }
    match window_year(year, interval, current_year) {
        Some(y) => cob_alloc_set_field_int(y),
        None => cob_alloc_set_field_uint(0),
    }
}

/// `cob_intr_date_to_yyyymmdd (params, ...)` (intrinsic.c): `FUNCTION DATE-TO-YYYYMMDD(yymmdd, …)`.
pub fn cob_intr_date_to_yyyymmdd(value: i32, interval: i32, current_year: i32) -> IntrField {
    let year = value / 10000;
    let mmdd = value % 10000;
    if !(0..=999999).contains(&year) {
        return cob_alloc_set_field_uint(0);
    }
    match window_year(year, interval, current_year) {
        Some(y) => cob_alloc_set_field_int(y * 10000 + mmdd),
        None => cob_alloc_set_field_uint(0),
    }
}

/// `cob_intr_day_to_yyyyddd (params, ...)` (intrinsic.c): `FUNCTION DAY-TO-YYYYDDD(yyddd, …)`.
pub fn cob_intr_day_to_yyyyddd(value: i32, interval: i32, current_year: i32) -> IntrField {
    let year = value / 1000;
    let ddd = value % 1000;
    if !(0..=999999).contains(&year) {
        return cob_alloc_set_field_uint(0);
    }
    match window_year(year, interval, current_year) {
        Some(y) => cob_alloc_set_field_int(y * 1000 + ddd),
        None => cob_alloc_set_field_uint(0),
    }
}

/// `cob_intr_num_decimal_point ()` (intrinsic.c): `FUNCTION NUMVAL`-context decimal point from
/// `localeconv()`. Under the pinned `LC_ALL=C.UTF-8` (C locale) this is `"."`. **Non-claim:** other
/// locales (the result is `localeconv()`-dependent).
pub fn cob_intr_num_decimal_point() -> IntrField {
    (b".".to_vec(), ALPHA1)
}

/// `cob_intr_num_thousands_sep ()` (intrinsic.c): `localeconv()->thousands_sep` — empty under the C
/// locale (a zero-size field). Non-claim: other locales.
pub fn cob_intr_num_thousands_sep() -> IntrField {
    (Vec::new(), ALPHA1)
}

/// `cob_intr_mon_decimal_point ()` (intrinsic.c): `localeconv()->mon_decimal_point` — empty under C.
pub fn cob_intr_mon_decimal_point() -> IntrField {
    (Vec::new(), ALPHA1)
}

/// `cob_intr_mon_thousands_sep ()` (intrinsic.c): `localeconv()->mon_thousands_sep` — empty under C.
pub fn cob_intr_mon_thousands_sep() -> IntrField {
    (Vec::new(), ALPHA1)
}

/// `cob_intr_currency_symbol ()` (intrinsic.c): `localeconv()->currency_symbol` — empty under C.
pub fn cob_intr_currency_symbol() -> IntrField {
    (Vec::new(), ALPHA1)
}

/// `cob_intr_stored_char_length (srcfield)` (intrinsic.c): `FUNCTION STORED-CHAR-LENGTH` — the field
/// size minus trailing spaces.
pub fn cob_intr_stored_char_length(src: &[u8]) -> IntrField {
    let mut count = src.len();
    while count > 0 && src[count - 1] == b' ' {
        count -= 1;
    }
    cob_alloc_set_field_uint(count as u32)
}

/// `error_not_implemented ()` (intrinsic.c): in libcob this raises `COB_EC_IMP_FEATURE_MISSING` and
/// **fatal-errors** (aborts). A library port cannot abort here, so this returns an empty field — the
/// documented not-implemented boundary (these `FUNCTION`s are genuinely unimplemented in GnuCOBOL 3.2).
pub fn error_not_implemented() -> IntrField {
    (Vec::new(), FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 })
}

/// `cob_intr_boolean_of_integer (f1, f2)` (intrinsic.c): unimplemented upstream — see
/// [`error_not_implemented`].
pub fn cob_intr_boolean_of_integer(_f1: &[u8], _a1: &FieldAttr, _f2: &[u8], _a2: &FieldAttr) -> IntrField {
    error_not_implemented()
}

/// `cob_intr_integer_of_boolean (srcfield)` (intrinsic.c): unimplemented upstream.
pub fn cob_intr_integer_of_boolean(_src: &[u8], _attr: &FieldAttr) -> IntrField {
    error_not_implemented()
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
    numval(src, false)
}

/// `numval (srcfield, currency, type)` (intrinsic.c): the shared NUMVAL / NUMVAL-C parser core — routed
/// through the sealed value-logic (`intrinsic_numval` / `intrinsic_numval_c`).
fn numval(src: &[u8], numval_c: bool) -> IntrField {
    let s = String::from_utf8_lossy(src);
    let nv = if numval_c { intrinsic_numval_c(&s) } else { intrinsic_numval(&s) };
    intr_decimal_result(numval_to_decimal(&nv))
}

/// `cob_intr_numval_c (srcfield, currency)` (intrinsic.c): `FUNCTION NUMVAL-C(s)` — like
/// [`cob_intr_numval`] after stripping the default currency symbol + thousands commas.
pub fn cob_intr_numval_c(src: &[u8]) -> IntrField {
    numval(src, true)
}

/// `cob_check_numval (srcfield, currency, chkcurr, anycase)` (intrinsic.c): validate that `srcfield` holds
/// a numeric string. Returns 0 when valid, otherwise the 1-based position of the first offending character
/// (or `size + 1` when the field contains no digit at all). `chkcurr` enables `NUMVAL-C` currency/comma
/// handling; `currency` overrides the symbol; `anycase` accepts lowercase `cr`/`db`. `dec_pt`/`currency_symbol`
/// come from the current module (the oracle's default config: `'.'` and `'$'`).
#[allow(unused_assignments)] // `break_needed` mirrors libcob's flag; some resets are redundant under Rust control flow
pub fn cob_check_numval(
    src: &[u8],
    currency: Option<&[u8]>,
    chkcurr: bool,
    anycase: bool,
    dec_pt: u8,
    currency_symbol: u8,
) -> i32 {
    const COB_MAX_DIGITS: usize = 38;
    let max_pos = src.len() as i32;
    if max_pos == 0 {
        return 1;
    }

    // Determine the currency token (begp / currcy_size).
    let mut begp: Option<Vec<u8>> = None;
    let mut currcy_size: i32 = 0;
    if let Some(cur) = currency {
        let cmax = cur.len();
        let mut begi: Option<usize> = None;
        let mut endi: Option<usize> = None;
        for pos in 0..cmax {
            match cur[pos] {
                b'0'..=b'9' | b'+' | b'-' | b'.' | b',' | b'*' => return 1,
                b' ' => {}
                _ => {
                    if pos < cmax - 1 && (&cur[pos..pos + 2] == b"CR" || &cur[pos..pos + 2] == b"DB") {
                        return 1;
                    }
                    if begi.is_none() {
                        begi = Some(pos);
                    }
                    endi = Some(pos);
                }
            }
        }
        match (begi, endi) {
            (Some(b), Some(e)) => {
                let sz = (e - b) as i32 + 1;
                if sz < max_pos {
                    begp = Some(cur[b..=e].to_vec());
                    currcy_size = sz;
                }
            }
            _ => return 1,
        }
    } else if chkcurr {
        begp = Some(vec![currency_symbol]);
        currcy_size = 1;
    }

    let mut plus_minus = false;
    let mut break_needed;
    let mut n: i32 = 0;

    // Leading positions (sign / spaces / currency before the first digit).
    break_needed = false;
    while n < max_pos {
        let c = src[n as usize];
        match c {
            b'0'..=b'9' => break_needed = true,
            b' ' => {
                n += 1;
                continue;
            }
            b'+' | b'-' => {
                if plus_minus {
                    return n + 1;
                }
                plus_minus = true;
                n += 1;
                continue;
            }
            b',' | b'.' => {
                if c != dec_pt {
                    return n + 1;
                }
                break_needed = true;
            }
            _ => {
                let mut matched_currency = false;
                if let Some(ref tok) = begp {
                    if n < max_pos - currcy_size {
                        let s = n as usize;
                        if src[s..s + currcy_size as usize] == tok[..] {
                            matched_currency = true;
                        }
                    }
                }
                if !matched_currency {
                    return n + 1;
                }
            }
        }
        if break_needed {
            break;
        }
        n += 1;
    }

    // End reached without a digit -> definitely not numeric.
    if n == max_pos {
        return max_pos + 1;
    }

    // Check the actual data.
    break_needed = false;
    let mut digits = 0usize;
    let mut decimal_seen = false;
    let mut space_seen = false;
    while n < max_pos {
        let c = src[n as usize];
        match c {
            b'0'..=b'9' => {
                digits += 1;
                if digits > COB_MAX_DIGITS || space_seen {
                    return n + 1;
                }
                n += 1;
                continue;
            }
            b',' | b'.' => {
                if decimal_seen || space_seen {
                    return n + 1;
                }
                if c == dec_pt {
                    decimal_seen = true;
                } else if !chkcurr {
                    return n + 1;
                }
                if digits > 0 {
                    let prev = src[(n - 1) as usize];
                    if !prev.is_ascii_digit() {
                        return n + 1;
                    }
                } else if n < max_pos - 1 {
                    let next = src[(n + 1) as usize];
                    if !next.is_ascii_digit() {
                        return n + 2;
                    }
                }
                n += 1;
                continue;
            }
            b' ' => {
                space_seen = true;
                n += 1;
                continue;
            }
            b'+' | b'-' => {
                if plus_minus {
                    return n + 1;
                }
                n += 1; // trailing sign consumed; only spaces may follow
                break_needed = true;
            }
            b'c' | b'C' => {
                if c == b'c' && !anycase {
                    return n + 1;
                }
                if plus_minus {
                    return n + 1;
                }
                if n < max_pos - 1 {
                    let nx = src[(n + 1) as usize];
                    if nx == b'R' || (anycase && nx == b'r') {
                        n += 2; // skip cR
                        break_needed = true;
                    } else {
                        return n + 2;
                    }
                } else {
                    return n + 2;
                }
            }
            b'd' | b'D' => {
                if c == b'd' && !anycase {
                    return n + 1;
                }
                if plus_minus {
                    return n + 1;
                }
                if n < max_pos - 1 {
                    let nx = src[(n + 1) as usize];
                    if nx == b'B' || (anycase && nx == b'b') {
                        n += 2; // skip dB
                        break_needed = true;
                    } else {
                        return n + 2;
                    }
                } else {
                    return n + 2;
                }
            }
            _ => return n + 1,
        }
        if break_needed {
            break;
        }
        n += 1;
    }

    // No digit -> definitely not numeric.
    if digits == 0 {
        return max_pos + 1;
    }

    // Trailing spaces only.
    while n < max_pos {
        if src[n as usize] != b' ' {
            return n + 1;
        }
        n += 1;
    }

    0
}

/// `cob_intr_test_numval (srcfield)` (intrinsic.c): `FUNCTION TEST-NUMVAL` — 0 if valid for `NUMVAL`, else
/// the 1-based position of the first invalid character.
pub fn cob_intr_test_numval(src: &[u8]) -> IntrField {
    cob_alloc_set_field_int(cob_check_numval(src, None, false, false, b'.', b'$'))
}

/// `cob_intr_test_numval_c (srcfield, currency)` (intrinsic.c): `FUNCTION TEST-NUMVAL-C` — like
/// [`cob_intr_test_numval`] with `NUMVAL-C` currency/comma handling.
pub fn cob_intr_test_numval_c(src: &[u8], currency: Option<&[u8]>) -> IntrField {
    cob_alloc_set_field_int(cob_check_numval(src, currency, true, false, b'.', b'$'))
}

/// `cob_check_numval_f (srcfield)` (intrinsic.c): validate a floating-point numeric string (`NUMVAL-F`
/// form: optional sign, digits, decimal point, then an `E±` exponent of up to 4 digits — the exponent sign
/// is mandatory). Returns 0 when valid, else the 1-based position of the first offending character.
#[allow(unused_assignments)] // `break_needed` mirrors libcob's flag; its initial reset is redundant under Rust control flow
pub fn cob_check_numval_f(src: &[u8], dec_pt: u8) -> i32 {
    const COB_MAX_DIGITS: usize = 38;
    let size = src.len() as i32;
    if size == 0 {
        return 1;
    }

    let mut plus_minus = false;
    let mut digits = 0usize;
    let mut decimal_seen = false;
    let mut space_seen = false;
    let mut e_seen = false;
    let mut exponent = 0usize;
    let mut e_plus_minus = false;
    let mut break_needed = false;
    let mut n: i32 = 0;

    // Check leading positions.
    while n < size {
        let c = src[n as usize];
        match c {
            b'0'..=b'9' => break_needed = true,
            b' ' => {
                n += 1;
                continue;
            }
            b'+' | b'-' => {
                if plus_minus {
                    return n + 1;
                }
                plus_minus = true;
                n += 1;
                continue;
            }
            b',' | b'.' => {
                if c != dec_pt {
                    return n + 1;
                }
                break_needed = true;
            }
            _ => return n + 1,
        }
        if break_needed {
            break;
        }
        n += 1;
    }

    if n == size {
        return n + 1;
    }

    while n < size {
        let c = src[n as usize];
        match c {
            b'0'..=b'9' => {
                if e_seen {
                    exponent += 1;
                    if exponent > 4 || !e_plus_minus {
                        return n + 1;
                    }
                } else {
                    digits += 1;
                    if digits > COB_MAX_DIGITS || space_seen {
                        return n + 1;
                    }
                }
            }
            b',' | b'.' => {
                if decimal_seen || space_seen || e_seen {
                    return n + 1;
                }
                if c == dec_pt {
                    decimal_seen = true;
                } else {
                    return n + 1;
                }
            }
            b' ' => space_seen = true,
            b'E' | b'e' => {
                if e_seen {
                    return n + 1;
                }
                e_seen = true;
            }
            b'+' | b'-' => {
                if e_seen {
                    if e_plus_minus {
                        return n + 1;
                    }
                    e_plus_minus = true;
                } else {
                    if plus_minus {
                        return n + 1;
                    }
                    plus_minus = true;
                }
            }
            _ => return n + 1,
        }
        n += 1;
    }

    if digits == 0 || (e_seen && exponent == 0) {
        return n + 1;
    }

    0
}

/// `cob_intr_test_numval_f (srcfield)` (intrinsic.c): `FUNCTION TEST-NUMVAL-F` — 0 if valid for `NUMVAL-F`,
/// else the 1-based position of the first invalid character.
pub fn cob_intr_test_numval_f(src: &[u8]) -> IntrField {
    cob_alloc_set_field_int(cob_check_numval_f(src, b'.'))
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

// ---- formatted date/time machinery (intrinsic.c) ------------------------------------------------
//
// The day-number basis is 1601-01-01 = 1. These are faithful 1:1 ports of the intrinsic.c date helpers
// (the project's existing `intrinsic_*` value-logic layer takes packed YYYYMMDD; these take split
// year/month/day components, exactly as the formatted-date intrinsics need).

const NORMAL_DAYS: [i32; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
const LEAP_DAYS: [i32; 13] = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];

/// `days_up_to_year (year)` (intrinsic.c): days from the 1601 base up to (not including) `year`.
fn days_up_to_year(year: i32) -> u32 {
    let mut totaldays = 0u32;
    let mut baseyear = 1601;
    while baseyear != year {
        totaldays += days_in_year(baseyear) as u32;
        baseyear += 1;
    }
    totaldays
}

/// `integer_of_date (year, month, days)` (intrinsic.c): the 1601-based day number for a calendar date.
fn integer_of_date(year: i32, month: i32, days: i32) -> u32 {
    let mut totaldays = days_up_to_year(year);
    totaldays += if leap_year(year) { LEAP_DAYS[(month - 1) as usize] } else { NORMAL_DAYS[(month - 1) as usize] } as u32;
    totaldays + days as u32
}

/// `integer_of_day (year, days)` (intrinsic.c): the day number for a `YYYY` + day-of-year.
fn integer_of_day(year: i32, days: i32) -> u32 {
    days_up_to_year(year) + days as u32
}

/// `day_of_integer (day_num)` (intrinsic.c): the `(year, day_of_year)` for a 1601-based day number.
fn day_of_integer(day_num: i32) -> (i32, i32) {
    let mut leapyear = 365;
    let mut days = day_num;
    let mut year = 1601;
    while days > leapyear {
        days -= leapyear;
        year += 1;
        leapyear = days_in_year(year);
    }
    (year, days)
}

/// `get_day_of_week (day_num)` (intrinsic.c): 0 (Monday) .. 6 (Sunday) for a day number.
fn get_day_of_week(day_num: i32) -> i32 {
    (day_num - 1) % 7
}

/// `get_iso_week_one (day_num, day_of_year)` (intrinsic.c): day number of the Monday of ISO week 1.
fn get_iso_week_one(day_num: i32, day_of_year: i32) -> i32 {
    let jan_4 = day_num - day_of_year + 4;
    let day_of_week = get_day_of_week(jan_4);
    jan_4 - day_of_week
}

/// `get_iso_week (day_num)` (intrinsic.c): the `(iso_year, iso_week)` for a day number.
fn get_iso_week(day_num: i32) -> (i32, i32) {
    let (mut year, day_of_year) = day_of_integer(day_num);
    let days_to_dec_29 = days_in_year(year) - 2;
    let dec_29 = day_num - day_of_year + days_to_dec_29;
    let week_one;
    if day_num >= dec_29 {
        let mut w1 = get_iso_week_one(day_num + days_in_year(year), day_of_year);
        if day_num < w1 {
            w1 = get_iso_week_one(day_num, day_of_year);
        } else {
            year += 1;
        }
        week_one = w1;
    } else {
        let mut w1 = get_iso_week_one(day_num, day_of_year);
        if day_num < w1 {
            year -= 1;
            w1 = get_iso_week_one(day_num - day_of_year, days_in_year(year));
        }
        week_one = w1;
    }
    (year, (day_num - week_one) / 7 + 1)
}

/// `max_week (year)` (intrinsic.c): the highest ISO week number (52 or 53) in `year`.
fn max_week(year: i32) -> i32 {
    let first_day = integer_of_date(year, 1, 1) as i32;
    let last_day = first_day + days_in_year(year) - 1;
    get_iso_week(last_day).1
}

/// `date[offset]` with C null-terminated-string semantics: out-of-bounds reads the `'\0'` terminator.
fn date_at(d: &[u8], off: i32) -> u8 {
    if off >= 0 && (off as usize) < d.len() {
        d[off as usize]
    } else {
        0
    }
}

/// `test_char_cond (cond, offset)` (intrinsic.c): advance on success (return 0), else return `offset+1`.
fn test_char_cond(cond: bool, offset: &mut i32) -> i32 {
    if cond {
        *offset += 1;
        0
    } else {
        *offset + 1
    }
}

/// `test_char (wanted, str, offset)` (intrinsic.c).
fn test_char(wanted: u8, d: &[u8], offset: &mut i32) -> i32 {
    test_char_cond(wanted == date_at(d, *offset), offset)
}

/// `test_char_in_range (min, max, ch, offset)` (intrinsic.c).
fn test_char_in_range(min: u8, max: u8, ch: u8, offset: &mut i32) -> i32 {
    test_char_cond(min <= ch && ch <= max, offset)
}

/// `test_digit (ch, offset)` (intrinsic.c): a `'0'..'9'` range check (locale-independent).
fn test_digit(ch: u8, offset: &mut i32) -> i32 {
    test_char_in_range(b'0', b'9', ch, offset)
}

macro_rules! return_if_not_zero {
    ($e:expr) => {{
        let r = $e;
        if r != 0 {
            return r;
        }
    }};
}

/// `test_millenium (date, offset, state)` (intrinsic.c): the thousands digit (`1..9`).
fn test_millenium(d: &[u8], offset: &mut i32, state: &mut i32) -> i32 {
    return_if_not_zero!(test_char_in_range(b'1', b'9', date_at(d, *offset), offset));
    *state = (date_at(d, *offset - 1) & 0x0F) as i32;
    0
}

/// `test_century (date, offset, state)` (intrinsic.c): hundreds digit (`6..9` when millennium is 1).
fn test_century(d: &[u8], offset: &mut i32, state: &mut i32) -> i32 {
    if *state != 1 {
        return_if_not_zero!(test_digit(date_at(d, *offset), offset));
    } else {
        return_if_not_zero!(test_char_in_range(b'6', b'9', date_at(d, *offset), offset));
    }
    *state = *state * 10 + (date_at(d, *offset - 1) & 0x0F) as i32;
    0
}

/// `test_decade (date, offset, state)` (intrinsic.c): tens digit.
fn test_decade(d: &[u8], offset: &mut i32, state: &mut i32) -> i32 {
    return_if_not_zero!(test_digit(date_at(d, *offset), offset));
    *state = *state * 10 + (date_at(d, *offset - 1) & 0x0F) as i32;
    0
}

/// `test_unit_year (date, offset, state)` (intrinsic.c): units digit (`1..9` when the year is 1600).
fn test_unit_year(d: &[u8], offset: &mut i32, state: &mut i32) -> i32 {
    if *state != 160 {
        return_if_not_zero!(test_digit(date_at(d, *offset), offset));
    } else {
        return_if_not_zero!(test_char_in_range(b'1', b'9', date_at(d, *offset), offset));
    }
    *state = *state * 10 + (date_at(d, *offset - 1) & 0x0F) as i32;
    0
}

/// `test_year (date, offset, state)` (intrinsic.c): the four `YYYY` digits, accumulating the year.
fn test_year(d: &[u8], offset: &mut i32, state: &mut i32) -> i32 {
    return_if_not_zero!(test_millenium(d, offset, state));
    return_if_not_zero!(test_century(d, offset, state));
    return_if_not_zero!(test_decade(d, offset, state));
    return_if_not_zero!(test_unit_year(d, offset, state));
    0
}

/// `test_hyphen_presence (with_hyphens, date, offset)` (intrinsic.c).
fn test_hyphen_presence(with_hyphens: bool, d: &[u8], offset: &mut i32) -> i32 {
    if with_hyphens {
        test_char(b'-', d, offset)
    } else {
        0
    }
}

/// `test_month (date, offset, month)` (intrinsic.c): the two `MM` digits (`01..12`).
fn test_month(d: &[u8], offset: &mut i32, month: &mut i32) -> i32 {
    return_if_not_zero!(test_char_cond(date_at(d, *offset) == b'0' || date_at(d, *offset) == b'1', offset));
    let first_digit = (date_at(d, *offset - 1) & 0x0F) as i32;
    if first_digit == 0 {
        return_if_not_zero!(test_char_in_range(b'1', b'9', date_at(d, *offset), offset));
    } else {
        return_if_not_zero!(test_char_in_range(b'0', b'2', date_at(d, *offset), offset));
    }
    *month = first_digit * 10 + (date_at(d, *offset - 1) & 0x0F) as i32;
    0
}

/// `test_day_of_month (date, year, month, offset)` (intrinsic.c): `DD` bounded by the month length.
fn test_day_of_month(d: &[u8], year: i32, month: i32, offset: &mut i32) -> i32 {
    let days_in_month = if leap_year(year) { LEAP_MONTH_DAYS[month as usize] } else { NORMAL_MONTH_DAYS[month as usize] };
    let max_first_digit = b'0' + (days_in_month / 10) as u8;
    let max_second_digit = b'0' + (days_in_month % 10) as u8;
    return_if_not_zero!(test_char_in_range(b'0', max_first_digit, date_at(d, *offset), offset));
    let first_digit = date_at(d, *offset - 1);
    if first_digit == b'0' {
        return_if_not_zero!(test_char_in_range(b'1', b'9', date_at(d, *offset), offset));
    } else if first_digit != max_first_digit {
        return_if_not_zero!(test_digit(date_at(d, *offset), offset));
    } else {
        return_if_not_zero!(test_char_in_range(b'0', max_second_digit, date_at(d, *offset), offset));
    }
    0
}

/// `test_day_of_year (date, year, offset)` (intrinsic.c): the three `DDD` digits (`001..365/366`).
fn test_day_of_year(d: &[u8], year: i32, offset: &mut i32) -> i32 {
    return_if_not_zero!(test_char_in_range(b'0', b'3', date_at(d, *offset), offset));
    let mut state = (date_at(d, *offset - 1) & 0x0F) as i32;
    if state != 3 {
        return_if_not_zero!(test_digit(date_at(d, *offset), offset));
    } else {
        return_if_not_zero!(test_char_in_range(b'0', b'6', date_at(d, *offset), offset));
    }
    state = state * 10 + (date_at(d, *offset - 1) & 0x0F) as i32;
    if state == 0 {
        return_if_not_zero!(test_char_in_range(b'1', b'9', date_at(d, *offset), offset));
    } else if state != 36 {
        return_if_not_zero!(test_digit(date_at(d, *offset), offset));
    } else {
        let max_last_digit = if leap_year(year) { b'6' } else { b'5' };
        return_if_not_zero!(test_char_in_range(b'0', max_last_digit, date_at(d, *offset), offset));
    }
    0
}

/// `test_w_presence (date, offset)` (intrinsic.c): the literal `'W'` of a week date.
fn test_w_presence(d: &[u8], offset: &mut i32) -> i32 {
    test_char(b'W', d, offset)
}

/// `test_week (date, year, offset)` (intrinsic.c): the two `ww` digits bounded by `max_week(year)`.
fn test_week(d: &[u8], year: i32, offset: &mut i32) -> i32 {
    return_if_not_zero!(test_char_in_range(b'0', b'5', date_at(d, *offset), offset));
    let first_digit = (date_at(d, *offset - 1) & 0x0F) as i32;
    if first_digit == 0 {
        return_if_not_zero!(test_char_in_range(b'1', b'9', date_at(d, *offset), offset));
    } else if first_digit != 5 {
        return_if_not_zero!(test_digit(date_at(d, *offset), offset));
    } else {
        let max_last_digit = if max_week(year) == 53 { b'3' } else { b'2' };
        return_if_not_zero!(test_char_in_range(b'0', max_last_digit, date_at(d, *offset), offset));
    }
    0
}

/// `test_day_of_week (date, offset)` (intrinsic.c): the single `d` digit (`1..7`).
fn test_day_of_week(d: &[u8], offset: &mut i32) -> i32 {
    test_char_in_range(b'1', b'7', date_at(d, *offset), offset)
}

/// `test_date_end (format, date, year, offset)` (intrinsic.c): the part after `YYYY` per the format kind.
fn test_date_end(format: DateFormat, d: &[u8], year: i32, offset: &mut i32) -> i32 {
    match format.days {
        DaysFormat::Mmdd => {
            let mut month = 0;
            return_if_not_zero!(test_month(d, offset, &mut month));
            return_if_not_zero!(test_hyphen_presence(format.with_hyphens, d, offset));
            return_if_not_zero!(test_day_of_month(d, year, month, offset));
        }
        DaysFormat::Ddd => {
            return_if_not_zero!(test_day_of_year(d, year, offset));
        }
        DaysFormat::Wwwd => {
            return_if_not_zero!(test_w_presence(d, offset));
            return_if_not_zero!(test_week(d, year, offset));
            return_if_not_zero!(test_hyphen_presence(format.with_hyphens, d, offset));
            return_if_not_zero!(test_day_of_week(d, offset));
        }
    }
    0
}

/// `test_no_trailing_junk (str, offset, end_of_string)` (intrinsic.c): trailing spaces ok at end-of-string.
fn test_no_trailing_junk(d: &[u8], mut offset: i32, end_of_string: bool) -> i32 {
    if end_of_string {
        while date_at(d, offset) != 0 {
            if date_at(d, offset) != b' ' {
                return offset + 1;
            }
            offset += 1;
        }
        0
    } else if date_at(d, offset) == 0 {
        0
    } else {
        offset + 1
    }
}

/// `test_formatted_date (format, date, end_of_string)` (intrinsic.c): 0 if `date` matches `format`, else
/// the 1-based position of the first offending character.
fn test_formatted_date(format: DateFormat, d: &[u8], end_of_string: bool) -> i32 {
    let mut offset = 0;
    let mut year = 0;
    return_if_not_zero!(test_year(d, &mut offset, &mut year));
    return_if_not_zero!(test_hyphen_presence(format.with_hyphens, d, &mut offset));
    return_if_not_zero!(test_date_end(format, d, year, &mut offset));
    return_if_not_zero!(test_no_trailing_junk(d, offset, end_of_string));
    0
}

/// The three calendar-date layouts (`enum days_format` in intrinsic.c).
#[derive(Clone, Copy, PartialEq)]
enum DaysFormat {
    Mmdd,
    Ddd,
    Wwwd,
}

/// `struct date_format` (intrinsic.c).
#[derive(Clone, Copy)]
struct DateFormat {
    days: DaysFormat,
    with_hyphens: bool,
}

/// `enum formatted_time_extra` (intrinsic.c).
#[derive(Clone, Copy, PartialEq)]
enum TimeExtra {
    None,
    Z,
    OffsetTime,
}

/// `struct time_format` (intrinsic.c).
#[derive(Clone, Copy)]
struct TimeFormat {
    with_colons: bool,
    decimal_places: usize,
    extra: TimeExtra,
}

/// `cob_valid_date_format (format)` (intrinsic.c): one of the six accepted date format strings.
fn cob_valid_date_format(format: &[u8]) -> bool {
    matches!(
        format,
        b"YYYYMMDD" | b"YYYY-MM-DD" | b"YYYYDDD" | b"YYYY-DDD" | b"YYYYWwwD" | b"YYYY-Www-D"
    )
}

/// `parse_date_format_string (format_str)` (intrinsic.c).
fn parse_date_format_string(format: &[u8]) -> DateFormat {
    let days = if format == b"YYYYMMDD" || format == b"YYYY-MM-DD" {
        DaysFormat::Mmdd
    } else if format == b"YYYYDDD" || format == b"YYYY-DDD" {
        DaysFormat::Ddd
    } else {
        DaysFormat::Wwwd
    };
    DateFormat { days, with_hyphens: format.get(4) == Some(&b'-') }
}

/// `decimal_places_for_seconds (str, point_pos)` (intrinsic.c): count of `'s'` after the decimal point.
fn decimal_places_for_seconds(s: &[u8], point_pos: usize) -> usize {
    let mut offset = point_pos;
    let mut decimal_places = 0;
    loop {
        offset += 1;
        if s.get(offset) == Some(&b's') {
            decimal_places += 1;
        } else {
            break;
        }
    }
    decimal_places
}

/// `rest_is_z (str)` (intrinsic.c): the remainder is the zone marker `"Z"`.
fn rest_is_z(s: &[u8]) -> bool {
    s == b"Z"
}

/// `rest_is_offset_format (str, with_colon)` (intrinsic.c): the remainder is a `+hh[:]mm` offset.
fn rest_is_offset_format(s: &[u8], with_colon: bool) -> bool {
    if with_colon {
        s == b"+hh:mm"
    } else {
        s == b"+hhmm"
    }
}

/// `cob_valid_time_format (format, decimal_point)` (intrinsic.c).
fn cob_valid_time_format(format: &[u8], decimal_point: u8) -> bool {
    let with_colons;
    let mut format_offset;
    if format.starts_with(b"hhmmss") {
        with_colons = false;
        format_offset = 6;
    } else if format.starts_with(b"hh:mm:ss") {
        with_colons = true;
        format_offset = 8;
    } else {
        return false;
    }
    if format.get(format_offset) == Some(&decimal_point) {
        let decimal_places = decimal_places_for_seconds(format, format_offset);
        format_offset += decimal_places + 1;
        if decimal_places == 0 || decimal_places > 9 {
            return false;
        }
    }
    if format.len() > format_offset {
        let rest = &format[format_offset..];
        if !rest_is_z(rest) && !rest_is_offset_format(rest, with_colons) {
            return false;
        }
    }
    true
}

/// `parse_time_format_string (str)` (intrinsic.c).
fn parse_time_format_string(s: &[u8]) -> TimeFormat {
    let with_colons;
    let mut offset;
    if s.starts_with(b"hhmmss") {
        with_colons = false;
        offset = 6;
    } else {
        with_colons = true;
        offset = 8;
    }
    let decimal_places = if s.get(offset) == Some(&b'.') || s.get(offset) == Some(&b',') {
        let dp = decimal_places_for_seconds(s, offset);
        offset += dp + 1;
        dp
    } else {
        0
    };
    let extra = if s.len() > offset {
        if rest_is_z(&s[offset..]) {
            TimeExtra::Z
        } else {
            TimeExtra::OffsetTime
        }
    } else {
        TimeExtra::None
    };
    TimeFormat { with_colons, decimal_places, extra }
}

/// `split_around_t (str, first, second)` (intrinsic.c): split `<date>T<time>` around the `'T'`, capping the
/// date at 10 and time at 25 chars. Returns `(date_part, time_part, overflow_indicator)`.
fn split_around_t(s: &[u8]) -> (Vec<u8>, Vec<u8>, i32) {
    const COB_DATESTR_MAX: usize = 10;
    const COB_TIMESTR_MAX: usize = 25;
    let mut ret = 0i32;
    let i = s.iter().position(|&c| c == b'T').unwrap_or(s.len());
    let first_length = if i > COB_DATESTR_MAX {
        ret = (COB_DATESTR_MAX + 1) as i32;
        COB_DATESTR_MAX
    } else {
        i
    };
    let first = s[..first_length].to_vec();
    let mut second = Vec::new();
    if i < s.len() {
        let rest = &s[i + 1..];
        let mut second_length = rest.len();
        if second_length != 0 {
            if second_length > COB_TIMESTR_MAX {
                second_length = COB_TIMESTR_MAX;
                ret = (COB_TIMESTR_MAX + 1 + i) as i32;
            }
            second = rest[..second_length].to_vec();
        }
    }
    (first, second, ret)
}

/// `cob_valid_datetime_format (format, decimal_point)` (intrinsic.c): a valid `<date>T<time>` whose date
/// and time agree on the separator style (hyphens iff colons).
fn cob_valid_datetime_format(format: &[u8], decimal_point: u8) -> bool {
    let (date_str, time_str, ret) = split_around_t(format);
    if ret != 0 {
        return false;
    }
    if !cob_valid_date_format(&date_str) || !cob_valid_time_format(&time_str, decimal_point) {
        return false;
    }
    let date_format = parse_date_format_string(&date_str);
    let time_format = parse_time_format_string(&time_str);
    date_format.with_hyphens == time_format.with_colons
}

/// Read up to `max` ASCII digits starting at `start` (the `sscanf("%Nd")` the intrinsics use on already
/// validated date strings).
fn scan_uint(s: &[u8], start: usize, max: usize) -> i32 {
    let mut v = 0i32;
    let mut k = 0;
    while k < max {
        match s.get(start + k) {
            Some(c) if c.is_ascii_digit() => {
                v = v * 10 + (c - b'0') as i32;
                k += 1;
            }
            _ => break,
        }
    }
    v
}

/// `integer_of_mmdd (format, year, final_part)` (intrinsic.c): `YYYYMMDD` → day number.
fn integer_of_mmdd(format: DateFormat, year: i32, final_part: &[u8]) -> u32 {
    let month = scan_uint(final_part, 0, 2);
    let day_start = if format.with_hyphens { 3 } else { 2 };
    let day = scan_uint(final_part, day_start, 2);
    integer_of_date(year, month, day)
}

/// `integer_of_ddd (year, final_part)` (intrinsic.c): `YYYYDDD` → day number.
fn integer_of_ddd(year: i32, final_part: &[u8]) -> u32 {
    integer_of_day(year, scan_uint(final_part, 0, 3))
}

/// `integer_of_wwwd (format, year, final_part)` (intrinsic.c): ISO `YYYYWwwD` → day number.
fn integer_of_wwwd(format: DateFormat, year: i32, final_part: &[u8]) -> u32 {
    let first_week_monday = get_iso_week_one((days_up_to_year(year) + 1) as i32, 1);
    let week = scan_uint(final_part, 1, 2);
    let dow_idx = if format.with_hyphens { 4 } else { 3 };
    let day_of_week = (date_at(final_part, dow_idx) - b'0') as i32;
    (first_week_monday + (week - 1) * 7 + day_of_week - 1) as u32
}

/// `integer_of_formatted_date (format, formatted_date)` (intrinsic.c): parse a validated formatted date to
/// its 1601-based day number.
fn integer_of_formatted_date(format: DateFormat, formatted_date: &[u8]) -> u32 {
    let year = scan_uint(formatted_date, 0, 4);
    let final_part_start = 4 + format.with_hyphens as usize;
    let final_part = &formatted_date[final_part_start.min(formatted_date.len())..];
    match format.days {
        DaysFormat::Mmdd => integer_of_mmdd(format, year, final_part),
        DaysFormat::Ddd => integer_of_ddd(year, final_part),
        DaysFormat::Wwwd => integer_of_wwwd(format, year, final_part),
    }
}

/// `num_leading_nonspace (str, str_len)` (intrinsic.c): characters before the first whitespace.
fn num_leading_nonspace(s: &[u8]) -> usize {
    s.iter()
        .position(|&c| matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        .unwrap_or(s.len())
}

/// `copy_data_to_null_terminated_str (f, out, max)` (intrinsic.c): the field text up to the first
/// whitespace, capped at `max`.
fn copy_data_to_null_terminated_str(f: &[u8], max: usize) -> Vec<u8> {
    let length = num_leading_nonspace(f).min(max);
    f[..length].to_vec()
}

/// `cob_intr_integer_of_formatted_date (format_field, date_field)` (intrinsic.c):
/// `FUNCTION INTEGER-OF-FORMATTED-DATE(format, date)` — the 1601-based day number for a date (or the date
/// part of a datetime) given its format string; an invalid format or date yields 0.
pub fn cob_intr_integer_of_formatted_date(fmt: &[u8], date: &[u8]) -> IntrField {
    const COB_DATETIMESTR_MAX: usize = 36;
    let original_format = copy_data_to_null_terminated_str(fmt, COB_DATETIMESTR_MAX);
    let original_date = copy_data_to_null_terminated_str(date, COB_DATETIMESTR_MAX);

    let is_date = cob_valid_date_format(&original_format);
    let format_str: Vec<u8> = if is_date {
        original_format.clone()
    } else if cob_valid_datetime_format(&original_format, b'.') {
        split_around_t(&original_format).0
    } else {
        return cob_alloc_set_field_uint(0);
    };
    let date_fmt = parse_date_format_string(&format_str);

    let date_str: Vec<u8> = if is_date { original_date.clone() } else { split_around_t(&original_date).0 };
    if test_formatted_date(date_fmt, &date_str, true) != 0 {
        return cob_alloc_set_field_uint(0);
    }
    cob_alloc_set_field_uint(integer_of_formatted_date(date_fmt, &date_str))
}

/// `date_of_integer (day_num)` (intrinsic.c): the `(year, month, day_of_month)` for a 1601-based day number.
fn date_of_integer(day_num: i32) -> (i32, i32, i32) {
    let mut days = day_num;
    let mut baseyear = 1601;
    let mut leapyear = 365;
    while days > leapyear {
        days -= leapyear;
        baseyear += 1;
        leapyear = days_in_year(baseyear);
    }
    let mut i = 0i32;
    while i < 13 {
        if leap_year(baseyear) {
            if i != 0 && days <= LEAP_DAYS[i as usize] {
                days -= LEAP_DAYS[(i - 1) as usize];
                break;
            }
        } else if i != 0 && days <= NORMAL_DAYS[i as usize] {
            days -= NORMAL_DAYS[(i - 1) as usize];
            break;
        }
        i += 1;
    }
    (baseyear, i, days)
}

/// `format_as_yyyymmdd (day_num, with_hyphen)` (intrinsic.c).
fn format_as_yyyymmdd(day_num: i32, with_hyphen: bool) -> Vec<u8> {
    let (year, month, day) = date_of_integer(day_num);
    if with_hyphen {
        format!("{year:04}-{month:02}-{day:02}")
    } else {
        format!("{year:04}{month:02}{day:02}")
    }
    .into_bytes()
}

/// `format_as_yyyyddd (day_num, with_hyphen)` (intrinsic.c).
fn format_as_yyyyddd(day_num: i32, with_hyphen: bool) -> Vec<u8> {
    let (year, doy) = day_of_integer(day_num);
    if with_hyphen {
        format!("{year:04}-{doy:03}")
    } else {
        format!("{year:04}{doy:03}")
    }
    .into_bytes()
}

/// `format_as_yyyywwwd (day_num, with_hyphen)` (intrinsic.c): the ISO week year + week + day-of-week.
fn format_as_yyyywwwd(day_num: i32, with_hyphen: bool) -> Vec<u8> {
    let (year, week) = get_iso_week(day_num);
    let day_of_week = get_day_of_week(day_num) + 1;
    if with_hyphen {
        format!("{year:04}-W{week:02}-{day_of_week:01}")
    } else {
        format!("{year:04}W{week:02}{day_of_week:01}")
    }
    .into_bytes()
}

/// `format_date (format, days, buff)` (intrinsic.c): render a day number per the date format kind.
fn format_date(format: DateFormat, days: i32) -> Vec<u8> {
    match format.days {
        DaysFormat::Mmdd => format_as_yyyymmdd(days, format.with_hyphens),
        DaysFormat::Ddd => format_as_yyyyddd(days, format.with_hyphens),
        DaysFormat::Wwwd => format_as_yyyywwwd(days, format.with_hyphens),
    }
}

/// `valid_day_and_format (day, format)` (intrinsic.c).
fn valid_day_and_format(day: i32, format: &[u8]) -> bool {
    valid_integer_date(day) && cob_valid_date_format(format)
}

/// `cob_intr_formatted_date (offset, length, format_field, days_field)` (intrinsic.c):
/// `FUNCTION FORMATTED-DATE(format, integer-date)` — render the day number as a date string of the format's
/// own width; an invalid day or format yields all spaces. Reference modification applies when `offset > 0`.
pub fn cob_intr_formatted_date(offset: i32, length: i32, fmt: &[u8], days: &[u8], days_attr: &FieldAttr) -> IntrField {
    const COB_DATESTR_MAX: usize = 10;
    let format_str = copy_data_to_null_terminated_str(fmt, COB_DATESTR_MAX);
    let field_length = format_str.len();
    let days_val = cob_get_int(days, days_attr);

    let out: Vec<u8> = if !valid_day_and_format(days_val, &format_str) {
        vec![b' '; field_length]
    } else {
        let mut buff = format_date(parse_date_format_string(&format_str), days_val);
        buff.truncate(field_length);
        buff
    };
    if offset > 0 {
        return intr_refmod(out, offset, length);
    }
    (out, ALPHA1)
}

/// `test_hour (time, offset)` (intrinsic.c): the two `hh` digits (`00..23`).
fn test_hour(t: &[u8], offset: &mut i32) -> i32 {
    return_if_not_zero!(test_char_in_range(b'0', b'2', date_at(t, *offset), offset));
    let first_digit = (date_at(t, *offset - 1) & 0x0F) as i32;
    if first_digit != 2 {
        return_if_not_zero!(test_digit(date_at(t, *offset), offset));
    } else {
        return_if_not_zero!(test_char_in_range(b'0', b'3', date_at(t, *offset), offset));
    }
    0
}

/// `test_less_than_60 (time, offset)` (intrinsic.c): two digits forming `00..59`.
fn test_less_than_60(t: &[u8], offset: &mut i32) -> i32 {
    return_if_not_zero!(test_char_in_range(b'0', b'5', date_at(t, *offset), offset));
    return_if_not_zero!(test_digit(date_at(t, *offset), offset));
    0
}

/// `test_minute (time, offset)` (intrinsic.c).
fn test_minute(t: &[u8], offset: &mut i32) -> i32 {
    test_less_than_60(t, offset)
}

/// `test_second (time, offset)` (intrinsic.c).
fn test_second(t: &[u8], offset: &mut i32) -> i32 {
    test_less_than_60(t, offset)
}

/// `test_colon_presence (with_colons, time, offset)` (intrinsic.c).
fn test_colon_presence(with_colons: bool, t: &[u8], offset: &mut i32) -> i32 {
    if with_colons {
        return_if_not_zero!(test_char(b':', t, offset));
    }
    0
}

/// `test_decimal_places (num_decimal_places, decimal_point, time, offset)` (intrinsic.c).
fn test_decimal_places(num: usize, dec_pt: u8, t: &[u8], offset: &mut i32) -> i32 {
    if num != 0 {
        return_if_not_zero!(test_char(dec_pt, t, offset));
        for _ in 0..num {
            return_if_not_zero!(test_digit(date_at(t, *offset), offset));
        }
    }
    0
}

/// `test_z_presence (time, offset)` (intrinsic.c): the UTC marker `'Z'`.
fn test_z_presence(t: &[u8], offset: &mut i32) -> i32 {
    test_char(b'Z', t, offset)
}

/// `test_two_zeroes (str, offset)` (intrinsic.c): the literal `"00"`.
fn test_two_zeroes(s: &[u8], offset: &mut i32) -> i32 {
    return_if_not_zero!(test_char(b'0', s, offset));
    return_if_not_zero!(test_char(b'0', s, offset));
    0
}

/// `test_offset_time (format, time, offset)` (intrinsic.c): a `±hh[:]mm` offset, or a literal `0000`.
fn test_offset_time(format: TimeFormat, t: &[u8], offset: &mut i32) -> i32 {
    let c = date_at(t, *offset);
    if c == b'+' || c == b'-' {
        *offset += 1;
        return_if_not_zero!(test_hour(t, offset));
        return_if_not_zero!(test_colon_presence(format.with_colons, t, offset));
        return_if_not_zero!(test_minute(t, offset));
    } else if c == b'0' {
        *offset += 1;
        return_if_not_zero!(test_two_zeroes(t, offset));
        return_if_not_zero!(test_colon_presence(format.with_colons, t, offset));
        return_if_not_zero!(test_two_zeroes(t, offset));
    } else {
        return *offset + 1;
    }
    0
}

/// `test_time_end (format, time, offset)` (intrinsic.c): the optional `Z`/offset zone after the seconds.
fn test_time_end(format: TimeFormat, t: &[u8], offset: &mut i32) -> i32 {
    if format.extra == TimeExtra::Z {
        return_if_not_zero!(test_z_presence(t, offset));
    } else if format.extra == TimeExtra::OffsetTime {
        return_if_not_zero!(test_offset_time(format, t, offset));
    }
    0
}

/// `test_formatted_time (format, time, decimal_point)` (intrinsic.c): 0 if `time` matches `format`, else
/// the 1-based position of the first offending character.
fn test_formatted_time(format: TimeFormat, t: &[u8], dec_pt: u8) -> i32 {
    let mut offset = 0;
    return_if_not_zero!(test_hour(t, &mut offset));
    return_if_not_zero!(test_colon_presence(format.with_colons, t, &mut offset));
    return_if_not_zero!(test_minute(t, &mut offset));
    return_if_not_zero!(test_colon_presence(format.with_colons, t, &mut offset));
    return_if_not_zero!(test_second(t, &mut offset));
    return_if_not_zero!(test_decimal_places(format.decimal_places, dec_pt, t, &mut offset));
    return_if_not_zero!(test_time_end(format, t, &mut offset));
    return_if_not_zero!(test_no_trailing_junk(t, offset, true));
    0
}

/// `cob_intr_test_formatted_datetime (format_field, datetime_field)` (intrinsic.c):
/// `FUNCTION TEST-FORMATTED-DATETIME(format, value)` — 0 if `value` matches `format` (date, time, or
/// `<date>T<time>`), else the 1-based position of the first offending character. An invalid format yields 0.
pub fn cob_intr_test_formatted_datetime(fmt: &[u8], datetime: &[u8]) -> IntrField {
    const COB_DATETIMESTR_MAX: usize = 36;
    let dec_pt = b'.';
    let datetime_format_str = copy_data_to_null_terminated_str(fmt, COB_DATETIMESTR_MAX);
    let formatted_datetime = copy_data_to_null_terminated_str(datetime, COB_DATETIMESTR_MAX);

    let (date_present, time_present) = if cob_valid_date_format(&datetime_format_str) {
        (true, false)
    } else if cob_valid_time_format(&datetime_format_str, dec_pt) {
        (false, true)
    } else if cob_valid_datetime_format(&datetime_format_str, dec_pt) {
        (true, true)
    } else {
        return cob_alloc_set_field_uint(0);
    };

    let (date_format_str, time_format_str) = if date_present && time_present {
        let (d, t, _) = split_around_t(&datetime_format_str);
        (d, t)
    } else if date_present {
        (datetime_format_str.clone(), Vec::new())
    } else {
        (Vec::new(), datetime_format_str.clone())
    };

    let (formatted_date, formatted_time) = if date_present && time_present {
        let (d, t, _) = split_around_t(&formatted_datetime);
        (d, t)
    } else if date_present {
        (formatted_datetime.clone(), Vec::new())
    } else {
        (Vec::new(), formatted_datetime.clone())
    };

    let time_part_offset = if date_present { formatted_date.len() as i32 + 1 } else { 0 };

    if date_present {
        let error_pos = test_formatted_date(parse_date_format_string(&date_format_str), &formatted_date, !time_present);
        if error_pos != 0 {
            return cob_alloc_set_field_uint(error_pos as u32);
        }
    }
    if date_present && time_present && date_at(&formatted_datetime, formatted_date.len() as i32) != b'T' {
        return cob_alloc_set_field_uint(formatted_date.len() as u32 + 1);
    }
    if time_present {
        let error_pos = test_formatted_time(parse_time_format_string(&time_format_str), &formatted_time, dec_pt);
        if error_pos != 0 {
            return cob_alloc_set_field_uint((time_part_offset + error_pos) as u32);
        }
    }
    cob_alloc_set_field_uint(0)
}

/// `calculate_start_end_for_numval (srcfield, pp, pp_end)` (intrinsic.c): the relevant `[start, end]` byte
/// range after trimming trailing spaces/low-values and leading spaces/zeros; `None` for an empty field.
fn calculate_start_end_for_numval(src: &[u8]) -> Option<(usize, usize)> {
    if src.is_empty() {
        return None;
    }
    let mut p = 0usize;
    let mut p_end = src.len() - 1;
    while p != p_end {
        if src[p_end] != b' ' && src[p_end] != 0 {
            break;
        }
        p_end -= 1;
    }
    while p != p_end {
        if src[p] != b' ' && src[p] != b'0' {
            break;
        }
        p += 1;
    }
    Some((p, p_end))
}

/// `cob_intr_numval_f (srcfield)` (intrinsic.c): `FUNCTION NUMVAL-F(s)` — parse a floating-point numeric
/// string (`±mantissa[.frac][E±exp]`) to an exact decimal (`mantissa * 10^(±exp - frac_digits)`); no
/// transcendental math. Parses "as valid as possible" (the default 3.2 build does not pre-validate).
///
/// `dec_pt` is the decimal-point character. GnuCOBOL reads it from `COB_MODULE_PTR->decimal_point`
/// (intrinsic.c:4958), so under `DECIMAL-POINT IS COMMA` the separator is `,` not `.`. The default-config
/// oracle passes `b'.'`; the caller threads the module setting (was previously hardcoded to `b'.'`).
pub fn cob_intr_numval_f(src: &[u8], dec_pt: u8) -> IntrField {
    const COB_MAX_DIGITS: usize = 38;
    let (start, p_end) = match calculate_start_end_for_numval(src) {
        Some(se) => se,
        None => return cob_alloc_set_field_uint(0),
    };

    let mut final_buff: Vec<u8> = Vec::new();
    let mut plus_minus = 0i32;
    let mut digits = 0usize;
    let mut decimal_digits = 0usize;
    let mut decimal_seen = false;
    let mut e_seen = false;
    let mut exponent: u64 = 0;
    let mut e_plus_minus = 0i32;

    let mut p = start;
    'parse: while p <= p_end {
        let c = src[p];
        match c {
            b'0'..=b'9' => {
                if c == b'0' && digits == 0 && !decimal_seen && exponent == 0 {
                    p += 1;
                    continue;
                }
                if e_seen {
                    exponent = exponent * 10 + (c & 0x0F) as u64;
                } else {
                    if decimal_seen {
                        decimal_digits += 1;
                    }
                    final_buff.push(c);
                    digits += 1;
                    if digits > COB_MAX_DIGITS {
                        break 'parse;
                    }
                }
            }
            b'+' => {
                if e_seen {
                    if e_plus_minus == 0 {
                        e_plus_minus = 1;
                    }
                } else if plus_minus == 0 {
                    plus_minus = 1;
                }
            }
            b'-' => {
                if e_seen {
                    if e_plus_minus == 0 {
                        e_plus_minus = -1;
                    }
                } else if plus_minus == 0 {
                    plus_minus = -1;
                }
            }
            b'e' | b'E' => {
                if !e_seen {
                    if digits == 0 && decimal_digits == 0 {
                        break 'parse;
                    }
                    e_seen = true;
                }
            }
            b' ' => {}
            _ => {
                if c == dec_pt && !decimal_seen {
                    decimal_seen = true;
                }
            }
        }
        p += 1;
    }

    if digits == 0 {
        final_buff.push(b'0');
    }
    let mut d = CobDecimal { value: Mpz::from_decimal_string(&String::from_utf8_lossy(&final_buff)), scale: 0 };
    if exponent > 9999 {
        exponent = 9999;
    }

    if d.value.sgn() == 0 {
        d.scale = 0;
        return intr_decimal_result(d);
    }
    if plus_minus == -1 {
        d.value.neg();
    }
    if exponent != 0 {
        if e_plus_minus == -1 {
            d.scale = (decimal_digits as u64 + exponent) as i32;
        } else if decimal_digits as u64 >= exponent {
            d.scale = (decimal_digits as u64 - exponent) as i32;
        } else {
            let extra = exponent - decimal_digits as u64;
            d.value = d.value.mul(&Mpz::ui_pow_ui(10, extra as u32));
            d.scale = 0;
        }
    } else {
        d.scale = decimal_digits as i32;
    }
    intr_decimal_result(d)
}

/// `valid_time (seconds_from_midnight)` (intrinsic.c): `0 <= s <= SECONDS_IN_DAY` (86400).
fn valid_time(seconds_from_midnight: i32) -> bool {
    in_range(0, 86400, seconds_from_midnight)
}

/// `seconds_from_formatted_time (format, str)` (intrinsic.c): parse a validated `hh[:]mm[:]ss[.frac]` time
/// to a decimal count of seconds since midnight (the trailing zone/offset is ignored here).
///
/// CHARACTERIZED DIVERGENCE: libcob's decimal branch sets `seconds_decimal->value` but not its scale, so
/// it reads the leftover scale of the shared scratch decimal `cob_d1` — its fractional result is therefore
/// call-history-dependent (e.g. 86399.125 vs a contaminated 8640.025 after a prior op left `cob_d1` at
/// scale 1). gnucobol-rs uses a fresh scale-0 base, yielding the well-defined mathematically-correct value;
/// byte-equality holds whenever `cob_d1` is clean (scale 0) at the call, which the oracle battery arranges.
fn seconds_from_formatted_time(format: TimeFormat, s: &[u8]) -> CobDecimal {
    let (hpos, mpos, spos) = if format.with_colons { (0, 3, 6) } else { (0, 2, 4) };
    let hours = scan_uint(s, hpos, 2);
    let minutes = scan_uint(s, mpos, 2);
    let seconds = scan_uint(s, spos, 2);
    let total_seconds = hours * 3600 + minutes * 60 + seconds;

    if format.decimal_places != 0 {
        let offset = if format.with_colons { 9 } else { 7 };
        let mut unscaled_fraction = 0i64;
        for k in 0..format.decimal_places {
            unscaled_fraction = unscaled_fraction * 10 + (date_at(s, (offset + k) as i32) & 0x0F) as i64;
        }
        let frac = CobDecimal {
            value: {
                let mut m = Mpz::new();
                m.set_ui(unscaled_fraction as u64);
                m
            },
            scale: format.decimal_places as i32,
        };
        let mut total = CobDecimal { value: { let mut m = Mpz::new(); m.set_ui(total_seconds as u64); m }, scale: 0 };
        cob_decimal_add(&mut total, &frac);
        total
    } else {
        CobDecimal { value: { let mut m = Mpz::new(); m.set_ui(total_seconds as u64); m }, scale: 0 }
    }
}

/// `cob_intr_seconds_from_formatted_time (format_field, time_field)` (intrinsic.c):
/// `FUNCTION SECONDS-FROM-FORMATTED-TIME(format, value)` — the seconds-since-midnight for a formatted time
/// (or the time part of a datetime); an invalid format or time yields 0.
pub fn cob_intr_seconds_from_formatted_time(fmt: &[u8], time: &[u8]) -> IntrField {
    let dec_pt = b'.';
    let str_length = num_leading_nonspace(fmt);
    let format_str = fmt[..str_length.min(fmt.len())].to_vec();

    let is_datetime = if cob_valid_datetime_format(&format_str, dec_pt) {
        true
    } else if !cob_valid_time_format(&format_str, dec_pt) {
        return cob_alloc_set_field_uint(0);
    } else {
        false
    };

    let (time_format_str, time_str): (Vec<u8>, Vec<u8>) = if is_datetime {
        (split_around_t(&format_str).1, split_around_t(time).1)
    } else {
        (format_str.clone(), time[..str_length.min(time.len())].to_vec())
    };

    let time_fmt = parse_time_format_string(&time_format_str);
    if test_formatted_time(time_fmt, &time_str, dec_pt) != 0 {
        return cob_alloc_set_field_uint(0);
    }
    intr_decimal_result(seconds_from_formatted_time(time_fmt, &time_str))
}

/// `get_fractional_seconds (time, fraction)` (intrinsic.c): the sub-second part of a numeric time field
/// (`decimal(time) - floor(time)`).
fn get_fractional_seconds(time: &[u8], attr: &FieldAttr) -> CobDecimal {
    let seconds = cob_get_int(time, attr);
    let whole = CobDecimal { value: { let mut m = Mpz::new(); m.set_ui(seconds as u64); m }, scale: 0 };
    let mut fraction = cob_decimal_set_field(time, attr);
    cob_decimal_sub(&mut fraction, &whole);
    fraction
}

/// `valid_offset_time (offset)` (intrinsic.c): `|offset| < 1440` minutes (one day).
fn valid_offset_time(offset: i32) -> bool {
    offset.abs() < 1440
}

/// `try_get_valid_offset_time (offset_time_field)` (intrinsic.c): `Some(offset)` (0 when the field is
/// absent), or `None` when the supplied offset is out of range.
fn try_get_valid_offset_time(field: Option<(&[u8], &FieldAttr)>) -> Option<i32> {
    match field {
        Some((d, a)) => {
            let off = cob_get_int(d, a);
            if valid_offset_time(off) {
                Some(off)
            } else {
                None
            }
        }
        None => Some(0),
    }
}

/// `get_system_offset_time_ptr` (intrinsic.c) resolves the host UTC offset from the system clock/timezone.
/// That is environment-dependent — the clock-deferral boundary (as for `FUNCTION CURRENT-DATE`); gnucobol-rs
/// reports the offset as unknown, matching libcob's `offset_known == 0` path.
fn get_system_offset_time_ptr() -> Option<i32> {
    None
}

/// `add_decimal_digits (decimal_places, second_fraction, buff, buff_pos)` (intrinsic.c): append the decimal
/// point and `decimal_places` fractional digits of `second_fraction`, right-padded with zeros.
fn add_decimal_digits(decimal_places: usize, second_fraction: &CobDecimal, buff: &mut Vec<u8>, dec_pt: u8) {
    let mut scale = second_fraction.scale;
    let mut fraction = second_fraction.value.get_ui();
    buff.push(dec_pt);
    let mut places = decimal_places as i32;
    while scale != 0 && places != 0 {
        scale -= 1;
        let power_of_ten = cob_s32_pow(10, scale).unwrap_or(1) as u64;
        buff.push(b'0' + (fraction / power_of_ten) as u8);
        fraction %= power_of_ten;
        places -= 1;
    }
    for _ in 0..places {
        buff.push(b'0');
    }
}

/// `add_offset_time (with_colon, offset_time, buff_pos, buff)` (intrinsic.c): append the 6-byte `±hh[:]mm`
/// zone offset (or `"00000"` + NUL when the offset pointer is null).
fn add_offset_time(with_colon: bool, offset_time: Option<i32>, buff: &mut Vec<u8>) {
    if let Some(off) = offset_time {
        let hours = off / 60;
        let minutes = (off % 60).abs();
        let s = if with_colon {
            format!("{hours:+03}:{minutes:02}")
        } else {
            format!("{hours:+03}{minutes:02}")
        };
        let mut bytes = s.into_bytes();
        bytes.resize(6, 0);
        buff.extend_from_slice(&bytes[..6]);
    } else {
        buff.extend_from_slice(b"00000\0");
    }
}

/// `format_time (format, time, second_fraction, offset_time, buff)` (intrinsic.c): render seconds-since-
/// midnight as a time string; returns the date overflow (-1/0/+1) a `Z` zone shift may introduce.
fn format_time(format: TimeFormat, time: i32, second_fraction: &CobDecimal, offset_time: Option<i32>, dec_pt: u8) -> (i32, Vec<u8>) {
    let mut hours = time / 3600;
    let rem = time % 3600;
    let mut minutes = rem / 60;
    let seconds = rem % 60;
    let mut date_overflow = 0;

    if format.extra == TimeExtra::Z {
        let off = match offset_time {
            Some(o) => o,
            None => return (0, Vec::new()), // EC_IMP_UTC_UNKNOWN: libcob leaves the buffer untouched
        };
        hours -= off / 60;
        minutes -= off % 60;
        if minutes >= 60 {
            minutes -= 60;
            hours += 1;
        } else if minutes < 0 {
            minutes += 60;
            hours -= 1;
        }
        if hours >= 24 {
            hours -= 24;
            date_overflow = 1;
        } else if hours < 0 {
            hours += 24;
            date_overflow = -1;
        }
    }

    let mut buff: Vec<u8> = if format.with_colons {
        format!("{hours:02}:{minutes:02}:{seconds:02}").into_bytes()
    } else {
        format!("{hours:02}{minutes:02}{seconds:02}").into_bytes()
    };
    if format.decimal_places != 0 {
        add_decimal_digits(format.decimal_places, second_fraction, &mut buff, dec_pt);
    }
    if format.extra == TimeExtra::Z {
        buff.push(b'Z');
    } else if format.extra == TimeExtra::OffsetTime {
        add_offset_time(format.with_colons, offset_time, &mut buff);
    }
    (date_overflow, buff)
}

/// `format_datetime (date_fmt, time_fmt, days, whole_seconds, frac, offset_time, buff)` (intrinsic.c):
/// `<date>T<time>`, the date adjusted by any `Z`-offset day overflow.
fn format_datetime(date_fmt: DateFormat, time_fmt: TimeFormat, days: i32, whole_seconds: i32, fractional: &CobDecimal, offset_time: Option<i32>, dec_pt: u8) -> Vec<u8> {
    let (overflow, formatted_time) = format_time(time_fmt, whole_seconds, fractional, offset_time, dec_pt);
    let mut buff = format_date(date_fmt, days + overflow);
    buff.push(b'T');
    buff.extend_from_slice(&formatted_time);
    buff
}

/// An all-spaces result of the given width (the formatted-date/time invalid-args path), reference-modified
/// when `offset > 0`.
fn formatted_spaces(field_length: usize, offset: i32, length: i32) -> IntrField {
    let out = vec![b' '; field_length];
    if offset > 0 {
        intr_refmod(out, offset, length)
    } else {
        (out, ALPHA1)
    }
}

/// `cob_intr_formatted_time (offset, length, params, ...)` (intrinsic.c):
/// `FUNCTION FORMATTED-TIME(format, seconds [, offset-minutes])` — render seconds-since-midnight per the
/// time format. The `use_system_offset` argument selects the host UTC offset (the clock-deferral boundary);
/// the explicit-offset path is sealed. Invalid time/format/offset yields all spaces.
#[allow(clippy::too_many_arguments)]
pub fn cob_intr_formatted_time(offset: i32, length: i32, fmt: &[u8], time: &[u8], time_attr: &FieldAttr, offset_time_field: Option<(&[u8], &FieldAttr)>, use_system_offset: bool) -> IntrField {
    const COB_TIMESTR_MAX: usize = 25;
    let dec_pt = b'.';
    let format_str = copy_data_to_null_terminated_str(fmt, COB_TIMESTR_MAX);
    let field_length = format_str.len();

    let whole_seconds = cob_get_int(time, time_attr);
    if !valid_time(whole_seconds) {
        return formatted_spaces(field_length, offset, length);
    }
    let fractional = get_fractional_seconds(time, time_attr);
    if !cob_valid_time_format(&format_str, dec_pt) {
        return formatted_spaces(field_length, offset, length);
    }
    let format = parse_time_format_string(&format_str);
    let offset_time = if use_system_offset {
        get_system_offset_time_ptr()
    } else {
        match try_get_valid_offset_time(offset_time_field) {
            Some(o) => Some(o),
            None => return formatted_spaces(field_length, offset, length),
        }
    };
    let (_overflow, mut buff) = format_time(format, whole_seconds, &fractional, offset_time, dec_pt);
    buff.resize(field_length, 0);
    if offset > 0 {
        return intr_refmod(buff, offset, length);
    }
    (buff, ALPHA1)
}

/// `cob_intr_formatted_datetime (offset, length, params, ...)` (intrinsic.c):
/// `FUNCTION FORMATTED-DATETIME(format, integer-date, seconds [, offset-minutes])` — `<date>T<time>` with
/// the date carried by any `Z`-offset day overflow. `use_system_offset` is the clock-deferral boundary;
/// the explicit-offset path is sealed. Invalid format/date/time/offset yields all spaces.
#[allow(clippy::too_many_arguments)]
pub fn cob_intr_formatted_datetime(offset: i32, length: i32, fmt: &[u8], days: &[u8], days_attr: &FieldAttr, time: &[u8], time_attr: &FieldAttr, offset_time_field: Option<(&[u8], &FieldAttr)>, use_system_offset: bool) -> IntrField {
    const COB_DATETIMESTR_MAX: usize = 36;
    let dec_pt = b'.';
    let fmt_str = copy_data_to_null_terminated_str(fmt, COB_DATETIMESTR_MAX);
    let field_length = fmt_str.len();

    if !cob_valid_datetime_format(&fmt_str, dec_pt) {
        return formatted_spaces(field_length, offset, length);
    }
    let days_val = cob_get_int(days, days_attr);
    let whole_seconds = cob_get_int(time, time_attr);
    if !valid_integer_date(days_val) || !valid_time(whole_seconds) {
        return formatted_spaces(field_length, offset, length);
    }
    let (date_fmt_str, time_fmt_str, ret) = split_around_t(&fmt_str);
    if ret != 0 {
        return formatted_spaces(field_length, offset, length);
    }
    let time_fmt = parse_time_format_string(&time_fmt_str);
    let offset_time = if use_system_offset {
        get_system_offset_time_ptr()
    } else {
        match try_get_valid_offset_time(offset_time_field) {
            Some(o) => Some(o),
            None => return formatted_spaces(field_length, offset, length),
        }
    };
    let date_fmt = parse_date_format_string(&date_fmt_str);
    let fractional = get_fractional_seconds(time, time_attr);
    let mut buff = format_datetime(date_fmt, time_fmt, days_val, whole_seconds, &fractional, offset_time, dec_pt);
    buff.resize(field_length, 0);
    if offset > 0 {
        return intr_refmod(buff, offset, length);
    }
    (buff, ALPHA1)
}

/// `COB_DECIMAL_NAN` (coblocal.h:137): the sentinel scale marking a not-a-number `cob_decimal`.
const COB_DECIMAL_NAN: i32 = -32768;

/// `cob_decimal_pow (pd1, pd2)` (intrinsic.c): raise `pd1` to the power `pd2` in place. Integer powers use
/// repeated squaring (with reciprocal for a negative power); fractional powers go through
/// `exp(pd2 * log(pd1))`, with an `mpf_sqrt` shortcut for the exponent `0.5`. A negative base with a
/// non-integer power yields NaN.
pub fn cob_decimal_pow(pd1: &mut CobDecimal, pd2: &mut CobDecimal) {
    let sign = pd1.value.sgn();
    if pd1.scale == COB_DECIMAL_NAN {
        return;
    }
    if pd2.scale == COB_DECIMAL_NAN {
        pd1.scale = COB_DECIMAL_NAN;
        return;
    }
    if pd2.value.sgn() == 0 {
        // Exponent is zero -> 1 (0^0 also yields 1, with an exception not modelled in the result bytes).
        pd1.value = Mpz::from_u64(1);
        pd1.scale = 0;
        return;
    }
    if sign == 0 {
        // Base is zero.
        pd1.scale = 0;
        return;
    }
    cob_trim_decimal(pd2);
    if sign == -1 && pd2.scale != 0 {
        // Negative base, non-integer power.
        pd1.scale = COB_DECIMAL_NAN;
        return;
    }
    cob_trim_decimal(pd1);
    if pd2.scale == 0 {
        // Integer power.
        if pd2.value.to_i128() == Some(1) {
            return; // power 1
        }
        if let Some(v) = pd2.value.to_i128() {
            if v < 0 && v >= i64::MIN as i128 {
                // Negative power: pd1 = pd1^|v|, then reciprocal.
                let n = (-v) as u64;
                pd1.value = pd1.value.pow_ui(n as u32);
                if pd1.scale != 0 {
                    pd1.scale *= n as i32;
                    cob_trim_decimal(pd1);
                }
                pd2.value = pd1.value.clone();
                pd2.scale = pd1.scale;
                pd1.value = Mpz::from_u64(1);
                pd1.scale = 0;
                let _ = cob_decimal_div(pd1, pd2);
                cob_trim_decimal(pd1);
                return;
            }
            if v >= 0 && v <= u64::MAX as i128 {
                // Positive power.
                let n = v as u64;
                pd1.value = pd1.value.pow_ui(n as u32);
                if pd1.scale != 0 {
                    pd1.scale *= n as i32;
                    cob_trim_decimal(pd1);
                }
                return;
            }
        }
    }

    // Fractional power via mpf.
    if sign == -1 {
        pd1.value.abs();
    }
    let base = cob_decimal_get_mpf(pd1);
    let result = if pd2.scale == 1 && pd2.value.to_i128() == Some(5) {
        base.sqrt()
    } else {
        let exponent = cob_decimal_get_mpf(pd2);
        cob_mpf_exp(&cob_mpf_log(&base).mul(&exponent))
    };
    *pd1 = cob_decimal_set_mpf(&result);
    if sign == -1 {
        pd1.value.neg();
    }
}

/// `cob_intr_sqrt (srcfield)` (intrinsic.c): `FUNCTION SQRT(x)` — the square root (`x ** 0.5`); a negative
/// argument yields an argument exception and 0.
pub fn cob_intr_sqrt(src: &[u8], attr: &FieldAttr) -> IntrField {
    let mut d1 = cob_decimal_set_field(src, attr);
    if d1.value.sgn() == -1 {
        return cob_alloc_set_field_uint(0);
    }
    let mut d2 = CobDecimal { value: Mpz::from_u64(5), scale: 1 };
    cob_trim_decimal(&mut d1);
    cob_decimal_pow(&mut d1, &mut d2);
    intr_decimal_result(d1)
}

/// `base ** exp` for a general (non-integer / fractional) exponent, via the sealed `cob_decimal_pow`
/// (same engine as `FUNCTION SQRT`/`EXP10`). The COMPUTE front-end keeps its exact repeated-multiply path
/// for non-negative integer exponents and routes everything else here.
pub fn cob_intr_pow(base: &[u8], battr: &FieldAttr, exp: &[u8], eattr: &FieldAttr) -> IntrField {
    let mut d1 = cob_decimal_set_field(base, battr);
    let mut d2 = cob_decimal_set_field(exp, eattr);
    cob_decimal_pow(&mut d1, &mut d2);
    intr_decimal_result(d1)
}

/// `cob_intr_exp (srcfield)` (intrinsic.c): `FUNCTION EXP(x)` — `e^x`; `EXP(0) = 1`.
pub fn cob_intr_exp(src: &[u8], attr: &FieldAttr) -> IntrField {
    let d1 = cob_decimal_set_field(src, attr);
    if d1.value.sgn() == 0 {
        return cob_alloc_set_field_uint(1);
    }
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_exp(&cob_decimal_get_mpf(&d1))))
}

/// `cob_intr_exp10 (srcfield)` (intrinsic.c): `FUNCTION EXP10(x)` — `10^x`; integer powers use exact
/// scaling, others go through `cob_decimal_pow`.
pub fn cob_intr_exp10(src: &[u8], attr: &FieldAttr) -> IntrField {
    let mut d1 = cob_decimal_set_field(src, attr);
    let sign = d1.value.sgn();
    if sign == 0 {
        return cob_alloc_set_field_uint(1);
    }
    cob_trim_decimal(&mut d1);
    if d1.scale == 0 {
        if let Some(v) = d1.value.to_i128() {
            if sign == -1 && v >= i32::MIN as i128 && v <= i32::MAX as i128 {
                // 10^(-n) = 1 with scale n.
                let n = (-v) as i32;
                return intr_decimal_result(CobDecimal { value: Mpz::from_u64(1), scale: n });
            }
            if sign == 1 && v <= u64::MAX as i128 {
                return intr_decimal_result(CobDecimal { value: Mpz::ui_pow_ui(10, v as u32), scale: 0 });
            }
        }
    }
    let mut d2 = CobDecimal { value: Mpz::from_u64(10), scale: 0 };
    cob_decimal_pow(&mut d2, &mut d1);
    intr_decimal_result(d2)
}

/// `cob_intr_log (srcfield)` (intrinsic.c): `FUNCTION LOG(x)` — natural log; `x <= 0` is an exception (0),
/// `LOG(1) = 0`.
pub fn cob_intr_log(src: &[u8], attr: &FieldAttr) -> IntrField {
    let mut d1 = cob_decimal_set_field(src, attr);
    if d1.value.sgn() != 1 {
        return cob_alloc_set_field_uint(0);
    }
    if d1.scale != 0 {
        cob_trim_decimal(&mut d1);
    }
    if d1.scale == 0 && d1.value.to_i128() == Some(1) {
        return cob_alloc_set_field_uint(0);
    }
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_log(&cob_decimal_get_mpf(&d1))))
}

/// `cob_intr_log10 (srcfield)` (intrinsic.c): `FUNCTION LOG10(x)` — base-10 log; `x <= 0` is an exception
/// (0), `LOG10(1) = 0`.
pub fn cob_intr_log10(src: &[u8], attr: &FieldAttr) -> IntrField {
    let mut d1 = cob_decimal_set_field(src, attr);
    if d1.value.sgn() != 1 {
        return cob_alloc_set_field_uint(0);
    }
    if d1.scale != 0 {
        cob_trim_decimal(&mut d1);
    }
    if d1.scale == 0 && d1.value.to_i128() == Some(1) {
        return cob_alloc_set_field_uint(0);
    }
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_log10(&cob_decimal_get_mpf(&d1))))
}

/// `cob_intr_sin (srcfield)` (intrinsic.c): `FUNCTION SIN(x)`.
pub fn cob_intr_sin(src: &[u8], attr: &FieldAttr) -> IntrField {
    let d1 = cob_decimal_set_field(src, attr);
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_sin(&cob_decimal_get_mpf(&d1))))
}

/// `cob_intr_cos (srcfield)` (intrinsic.c): `FUNCTION COS(x)`.
pub fn cob_intr_cos(src: &[u8], attr: &FieldAttr) -> IntrField {
    let d1 = cob_decimal_set_field(src, attr);
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_cos(&cob_decimal_get_mpf(&d1))))
}

/// `cob_intr_tan (srcfield)` (intrinsic.c): `FUNCTION TAN(x)`.
pub fn cob_intr_tan(src: &[u8], attr: &FieldAttr) -> IntrField {
    let d1 = cob_decimal_set_field(src, attr);
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_tan(&cob_decimal_get_mpf(&d1))))
}

/// `cob_intr_atan (srcfield)` (intrinsic.c): `FUNCTION ATAN(x)`; `ATAN(0) = 0`.
pub fn cob_intr_atan(src: &[u8], attr: &FieldAttr) -> IntrField {
    let d1 = cob_decimal_set_field(src, attr);
    if d1.value.sgn() == 0 {
        return cob_alloc_set_field_uint(0);
    }
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_atan(&cob_decimal_get_mpf(&d1))))
}

/// `cob_intr_asin (srcfield)` (intrinsic.c): `FUNCTION ASIN(x)`; `|x| > 1` is an exception (0),
/// `ASIN(0) = 0`.
pub fn cob_intr_asin(src: &[u8], attr: &FieldAttr) -> IntrField {
    let d1 = cob_decimal_set_field(src, attr);
    let neg1 = CobDecimal { value: Mpz::from_i64(-1), scale: 0 };
    let pos1 = CobDecimal { value: Mpz::from_u64(1), scale: 0 };
    if cob_decimal_cmp(&d1, &neg1) < 0 || cob_decimal_cmp(&d1, &pos1) > 0 {
        return cob_alloc_set_field_uint(0);
    }
    if d1.value.sgn() == 0 {
        return cob_alloc_set_field_uint(0);
    }
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_asin(&cob_decimal_get_mpf(&d1))))
}

/// `cob_intr_acos (srcfield)` (intrinsic.c): `FUNCTION ACOS(x)`; `|x| > 1` is an exception (0).
pub fn cob_intr_acos(src: &[u8], attr: &FieldAttr) -> IntrField {
    let d1 = cob_decimal_set_field(src, attr);
    let neg1 = CobDecimal { value: Mpz::from_i64(-1), scale: 0 };
    let pos1 = CobDecimal { value: Mpz::from_u64(1), scale: 0 };
    if cob_decimal_cmp(&d1, &neg1) < 0 || cob_decimal_cmp(&d1, &pos1) > 0 {
        return cob_alloc_set_field_uint(0);
    }
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_acos(&cob_decimal_get_mpf(&d1))))
}

/// `cob_intr_pi ()` (intrinsic.c): `FUNCTION PI` — the constant pi.
pub fn cob_intr_pi() -> IntrField {
    intr_decimal_result(cob_decimal_set_mpf(&cob_pi()))
}

/// `cob_intr_e ()` (intrinsic.c): `FUNCTION E` — Euler's number, `exp(1)`.
pub fn cob_intr_e() -> IntrField {
    intr_decimal_result(cob_decimal_set_mpf(&cob_mpf_exp(&Mpf::set_ui(1, COB_MPF_PREC))))
}

/// `cob_intr_binop (f1, op, f2)` (intrinsic.c): the runtime binary operator behind compiler-generated
/// arithmetic. Bitwise ops (`a`/`o`/`e`/`l`/`r`/`n`) work on `cob_get_int`; arithmetic ops
/// (`+`/`-`/`*`/`/`/`^`) on the decimal value (`/` by zero yields 0; `^` via [`cob_decimal_pow`]).
pub fn cob_intr_binop(f1: &[u8], a1: &FieldAttr, op: u8, f2: &[u8], a2: &FieldAttr) -> IntrField {
    match op {
        b'a' => return cob_alloc_set_field_uint((cob_get_int(f1, a1) & cob_get_int(f2, a2)) as u32),
        b'o' => return cob_alloc_set_field_uint((cob_get_int(f1, a1) | cob_get_int(f2, a2)) as u32),
        b'e' => return cob_alloc_set_field_uint((cob_get_int(f1, a1) ^ cob_get_int(f2, a2)) as u32),
        b'l' => return cob_alloc_set_field_uint(cob_get_int(f1, a1).wrapping_shl(cob_get_int(f2, a2) as u32) as u32),
        b'r' => return cob_alloc_set_field_uint(cob_get_int(f1, a1).wrapping_shr(cob_get_int(f2, a2) as u32) as u32),
        b'n' => return cob_alloc_set_field_uint(!cob_get_int(f2, a2) as u32),
        _ => {}
    }
    let mut d1 = cob_decimal_set_field(f1, a1);
    let mut d2 = cob_decimal_set_field(f2, a2);
    match op {
        b'+' => cob_decimal_add(&mut d1, &d2),
        b'-' => cob_decimal_sub(&mut d1, &d2),
        b'*' => cob_decimal_mul(&mut d1, &d2),
        b'/' => {
            if d2.value.sgn() == 0 {
                d1 = CobDecimal { value: Mpz::new(), scale: 0 };
            } else {
                let _ = cob_decimal_div(&mut d1, &d2);
            }
        }
        b'^' => cob_decimal_pow(&mut d1, &mut d2),
        _ => {}
    }
    intr_decimal_result(d1)
}

/// `cob_intr_annuity (P1, P2)` (intrinsic.c): `FUNCTION ANNUITY` — the annuity factor
/// `P1 / (1 - (1 + P1) ^ -P2)`; `P1 == 0` degenerates to `1/P2`. `P1 < 0`, or `P2 <= 0`/non-integer, is an
/// argument exception (0).
pub fn cob_intr_annuity(p1: &[u8], a1: &FieldAttr, p2: &[u8], a2: &FieldAttr) -> IntrField {
    let mut d1 = cob_decimal_set_field(p1, a1);
    let mut d2 = cob_decimal_set_field(p2, a2);
    let sign = d1.value.sgn();
    if sign < 0 || d2.value.sgn() <= 0 || d2.scale != 0 {
        return cob_alloc_set_field_uint(0);
    }
    if sign == 0 {
        d1 = CobDecimal { value: Mpz::from_u64(1), scale: 0 };
        let _ = cob_decimal_div(&mut d1, &d2);
        return intr_decimal_result(d1);
    }
    d2.value.neg(); // -P2
    let mut d3 = CobDecimal { value: d1.value.clone(), scale: d1.scale };
    cob_decimal_add(&mut d3, &CobDecimal { value: Mpz::from_u64(1), scale: 0 }); // 1 + P1
    cob_trim_decimal(&mut d3);
    cob_trim_decimal(&mut d2);
    cob_decimal_pow(&mut d3, &mut d2); // (1 + P1) ^ -P2
    let mut d4 = CobDecimal { value: Mpz::from_u64(1), scale: 0 };
    cob_decimal_sub(&mut d4, &d3); // 1 - (1 + P1) ^ -P2
    cob_trim_decimal(&mut d4);
    cob_trim_decimal(&mut d1);
    let _ = cob_decimal_div(&mut d1, &d4); // P1 / (...)
    intr_decimal_result(d1)
}

/// `cob_intr_present_value (rate, flows...)` (intrinsic.c): `FUNCTION PRESENT-VALUE` —
/// `sum_i flow_i / (1 + rate)^i` (i from 1).
pub fn cob_intr_present_value(rate: &[u8], rate_attr: &FieldAttr, flows: &[(&[u8], &FieldAttr)]) -> IntrField {
    let mut base = cob_decimal_set_field(rate, rate_attr);
    cob_decimal_add(&mut base, &CobDecimal { value: Mpz::from_u64(1), scale: 0 }); // 1 + rate
    let mut acc = CobDecimal { value: Mpz::new(), scale: 0 };
    for (i, (data, attr)) in flows.iter().enumerate() {
        let idx = i + 1;
        let mut flow = cob_decimal_set_field(data, attr);
        let mut denom = CobDecimal { value: base.value.clone(), scale: base.scale };
        if idx > 1 {
            denom.value = denom.value.pow_ui(idx as u32);
            denom.scale *= idx as i32;
        }
        let _ = cob_decimal_div(&mut flow, &denom);
        cob_decimal_add(&mut acc, &flow);
    }
    intr_decimal_result(acc)
}

/// `calc_mean_of_args (args)` (intrinsic.c): the arithmetic mean `sum(args) / n`.
fn calc_mean_of_args(args: &[(&[u8], &FieldAttr)]) -> CobDecimal {
    let mut mean = CobDecimal { value: Mpz::new(), scale: 0 };
    for (data, attr) in args {
        cob_decimal_add(&mut mean, &cob_decimal_set_field(data, attr));
    }
    let n = CobDecimal { value: Mpz::from_u64(args.len() as u64), scale: 0 };
    let _ = cob_decimal_div(&mut mean, &n);
    mean
}

/// `calc_variance_of_args (args, mean)` (intrinsic.c): the population variance `sum((arg - mean)^2) / n`
/// (`n == 1` -> 0).
fn calc_variance_of_args(args: &[(&[u8], &FieldAttr)], mean: &CobDecimal) -> CobDecimal {
    if args.len() == 1 {
        return CobDecimal { value: Mpz::new(), scale: 0 };
    }
    let mut sum = CobDecimal { value: Mpz::new(), scale: 0 };
    for (data, attr) in args {
        let mut diff = cob_decimal_set_field(data, attr);
        cob_decimal_sub(&mut diff, mean);
        let sq = diff.clone();
        cob_decimal_mul(&mut diff, &sq);
        cob_decimal_add(&mut sum, &diff);
    }
    let n = CobDecimal { value: Mpz::from_u64(args.len() as u64), scale: 0 };
    let _ = cob_decimal_div(&mut sum, &n);
    sum
}

/// `GET_VARIANCE` (intrinsic.c): the population variance — mean, then the variance about it.
fn get_variance(args: &[(&[u8], &FieldAttr)]) -> CobDecimal {
    let mean = calc_mean_of_args(args);
    calc_variance_of_args(args, &mean)
}

/// `cob_intr_variance (args...)` (intrinsic.c): `FUNCTION VARIANCE`.
pub fn cob_intr_variance(args: &[(&[u8], &FieldAttr)]) -> IntrField {
    intr_decimal_result(get_variance(args))
}

/// `cob_intr_standard_deviation (args...)` (intrinsic.c): `FUNCTION STANDARD-DEVIATION` — the square root
/// of the variance.
pub fn cob_intr_standard_deviation(args: &[(&[u8], &FieldAttr)]) -> IntrField {
    let mut d1 = get_variance(args);
    cob_trim_decimal(&mut d1);
    let mut half = CobDecimal { value: Mpz::from_u64(5), scale: 1 };
    cob_decimal_pow(&mut d1, &mut half);
    intr_decimal_result(d1)
}

/// `cob_intr_display_of (offset, length, params, ...)` (intrinsic.c): `FUNCTION DISPLAY-OF` — unimplemented
/// in GnuCOBOL 3.2 (it `error_not_implemented`s — fatal-errors upstream; the library boundary returns the
/// empty not-implemented field).
pub fn cob_intr_display_of(_offset: i32, _length: i32, _args: &[(&[u8], &FieldAttr)]) -> IntrField {
    error_not_implemented()
}

/// `cob_intr_national_of (offset, length, params, ...)` (intrinsic.c): `FUNCTION NATIONAL-OF` —
/// unimplemented in GnuCOBOL 3.2; see [`error_not_implemented`].
pub fn cob_intr_national_of(_offset: i32, _length: i32, _args: &[(&[u8], &FieldAttr)]) -> IntrField {
    error_not_implemented()
}

/// `cob_intr_char_national (srcfield)` (intrinsic.c): `FUNCTION CHAR-NATIONAL` — unimplemented in
/// GnuCOBOL 3.2; see [`error_not_implemented`].
pub fn cob_intr_char_national(_src: &[u8], _attr: &FieldAttr) -> IntrField {
    error_not_implemented()
}

/// `cob_intr_standard_compare (params, ...)` (intrinsic.c): `FUNCTION STANDARD-COMPARE` — unimplemented in
/// GnuCOBOL 3.2; see [`error_not_implemented`].
pub fn cob_intr_standard_compare(_args: &[(&[u8], &FieldAttr)]) -> IntrField {
    error_not_implemented()
}

/// `cob_intr_when_compiled (offset, length, f)` (intrinsic.c): `FUNCTION WHEN-COMPILED` — returns the
/// compile-time stamp the compiler supplies in `f` (with optional reference modification).
pub fn cob_intr_when_compiled(offset: i32, length: i32, f: &[u8], attr: &FieldAttr) -> IntrField {
    let out = f.to_vec();
    if offset > 0 {
        return intr_refmod(out, offset, length);
    }
    (out, *attr)
}

// ---- clock family (intrinsic.c + the cob_get_current_datetime dependency from common.c) ---------
//
// cob_get_current_datetime honors the COB_CURRENT_DATE override (a fixed Y/M/D H:M:S.ns+-tz), which makes
// CURRENT-DATE / FORMATTED-CURRENT-DATE deterministic and oracle-testable. The no-override path reads the
// system clock (UTC; std has no timezone API, so offset is reported unknown) — faithful but not byte-tested.

/// `struct cob_time` (common.h): a broken-down date/time. `-1` marks a component to take from the system
/// (the `COB_CURRENT_DATE` template convention).
#[derive(Clone, Copy)]
struct CobTime {
    year: i32,
    month: i32,
    day_of_month: i32,
    hour: i32,
    minute: i32,
    second: i32,
    nanosecond: i32,
    utc_offset: i32,
    offset_known: bool,
}

/// `civil_from_days` (Howard Hinnant): days-since-1970-01-01 -> `(year, month, day)` (proleptic Gregorian).
/// `cob_get_current_date_and_time_from_os` (common.c): the system clock as UTC (std exposes no timezone, so
/// `offset_known` is false). Non-deterministic — used only when `COB_CURRENT_DATE` is unset.
fn cob_get_current_date_and_time_from_os(want_nano: bool) -> CobTime {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = dur.as_secs() as i64;
    let (y, m, d) = civil_from_days(secs.div_euclid(86400));
    let rem = secs.rem_euclid(86400);
    CobTime {
        year: y as i32,
        month: m as i32,
        day_of_month: d as i32,
        hour: (rem / 3600) as i32,
        minute: ((rem % 3600) / 60) as i32,
        second: (rem % 60) as i32,
        nanosecond: if want_nano { dur.subsec_nanos() as i32 } else { 0 },
        utc_offset: 0,
        offset_known: false,
    }
}

/// Read up to `max` ASCII digits at `*p`, advancing `*p`; returns `(value, digit_count)`.
fn scan_date_digits(s: &[u8], p: &mut usize, max: usize) -> (i32, usize) {
    let mut v = 0i32;
    let mut i = 0;
    while *p < s.len() && s[*p].is_ascii_digit() {
        v = v * 10 + (s[*p] & 0x0F) as i32;
        *p += 1;
        i += 1;
        if i == max {
            break;
        }
    }
    (v, i)
}

/// `check_current_date` (common.c): parse the `COB_CURRENT_DATE` override string into a [`CobTime`] constant
/// (`-1` for a templated/absent component). Returns `None` when the variable is empty.
fn parse_current_date(s: &[u8]) -> Option<CobTime> {
    let mut t = CobTime { year: -1, month: -1, day_of_month: -1, hour: -1, minute: -1, second: -1, nanosecond: -1, utc_offset: 0, offset_known: false };
    let mut p = 0usize;
    while p < s.len() && (s[p] == 0x27 || s[p] == 0x22 || s[p].is_ascii_whitespace()) {
        p += 1;
    }
    if p >= s.len() {
        return None;
    }
    if s[p] == b'@' {
        // @seconds-since-epoch
        p += 1;
        let mut seconds = 0i64;
        while p < s.len() && s[p].is_ascii_digit() {
            seconds = seconds * 10 + (s[p] & 0x0F) as i64;
            p += 1;
        }
        let (y, m, d) = civil_from_days(seconds.div_euclid(86400));
        let rem = seconds.rem_euclid(86400);
        t.year = y as i32;
        t.month = m as i32;
        t.day_of_month = d as i32;
        t.hour = (rem / 3600) as i32;
        t.minute = ((rem % 3600) / 60) as i32;
        t.second = (rem % 60) as i32;
        return Some(t);
    }
    // date
    if p < s.len() {
        let (yr, i) = scan_date_digits(s, &mut p, 4);
        if i != 2 && i != 4 {
            if p < s.len() && s[p] == b'Y' {
                while p < s.len() && s[p] == b'Y' {
                    p += 1;
                }
            }
            t.year = -1;
        } else {
            t.year = if yr < 100 { yr + 2000 } else { yr };
        }
        if p < s.len() && (s[p] == b'/' || s[p] == b'-') {
            p += 1;
        }
    }
    if p < s.len() {
        let (mm, i) = scan_date_digits(s, &mut p, 2);
        t.month = if i == 2 { mm } else { -1 };
        if p < s.len() && (s[p] == b'/' || s[p] == b'-') {
            p += 1;
        }
    }
    if p < s.len() {
        let (dd, i) = scan_date_digits(s, &mut p, 2);
        t.day_of_month = if i == 2 { dd } else { -1 };
    }
    // time
    while p < s.len() && s[p].is_ascii_whitespace() {
        p += 1;
    }
    if p < s.len() {
        let (hh, i) = scan_date_digits(s, &mut p, 2);
        t.hour = if i == 2 { hh } else { -1 };
        if p < s.len() && (s[p] == b':' || s[p] == b'-') {
            p += 1;
        }
    }
    if p < s.len() {
        let (mi, i) = scan_date_digits(s, &mut p, 2);
        t.minute = if i == 2 { mi } else { -1 };
        if p < s.len() && (s[p] == b':' || s[p] == b'-') {
            p += 1;
        }
    }
    if p < s.len() && s[p] != b'Z' && s[p] != b'+' && s[p] != b'-' {
        let (ss, i) = scan_date_digits(s, &mut p, 2);
        t.second = if i == 2 { ss } else { -1 };
    }
    // nanoseconds
    if p < s.len() && s[p] != b'Z' && s[p] != b'+' && s[p] != b'-' {
        if s[p] == b'.' || s[p] == b':' {
            p += 1;
        }
        let (ns, i) = scan_date_digits(s, &mut p, 9);
        if i > 0 {
            t.nanosecond = ns;
        }
    }
    // UTC offset
    if p < s.len() && s[p] == b'Z' {
        t.utc_offset = 0;
        t.offset_known = true;
    } else if p < s.len() && (s[p] == b'+' || s[p] == b'-') {
        let neg = s[p] == b'-';
        let mut digs: Vec<u8> = Vec::new();
        let mut q = p + 1;
        while q < s.len() && digs.len() < 4 {
            if s[q] == b':' {
                q += 1;
                continue;
            }
            if !s[q].is_ascii_digit() {
                break;
            }
            digs.push(s[q] & 0x0F);
            q += 1;
        }
        if digs.len() >= 2 {
            // "+HH" alone -> minutes 00
            while digs.len() < 4 {
                digs.push(0);
            }
            let off = digs[0] as i32 * 600 + digs[1] as i32 * 60 + digs[2] as i32 * 10 + digs[3] as i32;
            t.utc_offset = if neg { -off } else { off };
            t.offset_known = true;
        }
    }
    Some(t)
}

/// Apply a `COB_CURRENT_DATE` override string onto a base time, component-by-component (`-1` keeps the
/// base component), then clamp a leap second to 59. Shared by the env-driven and the explicit (runtime.cfg)
/// override paths.
fn apply_current_date_override(mut t: CobTime, value: &[u8]) -> CobTime {
    if let Some(c) = parse_current_date(value) {
        if c.hour != -1 {
            t.hour = c.hour;
        }
        if c.minute != -1 {
            t.minute = c.minute;
        }
        if c.second != -1 {
            t.second = c.second;
        }
        if c.nanosecond != -1 {
            t.nanosecond = c.nanosecond;
        }
        if c.offset_known {
            t.offset_known = true;
            t.utc_offset = c.utc_offset;
        }
        if c.year != -1 {
            t.year = c.year;
        }
        if c.month != -1 {
            t.month = c.month;
        }
        if c.day_of_month != -1 {
            t.day_of_month = c.day_of_month;
        }
    }
    if t.second >= 60 {
        t.second = 59;
    }
    t
}

/// `cob_get_current_datetime (res)` (common.c): the system time, with any `COB_CURRENT_DATE` override
/// applied component-by-component (leap second clamped to 59).
fn cob_get_current_datetime(want_nano: bool) -> CobTime {
    let t = cob_get_current_date_and_time_from_os(want_nano);
    match std::env::var("COB_CURRENT_DATE") {
        Ok(env) => apply_current_date_override(t, env.as_bytes()),
        Err(_) => {
            let mut t = t;
            if t.second >= 60 {
                t.second = 59;
            }
            t
        }
    }
}

/// As [`cob_get_current_datetime`] but with an **explicit** `COB_CURRENT_DATE` override value -- e.g. the
/// `current_date` setting loaded from `runtime.cfg` (the `cli-runtime-cfg` wiring) -- instead of the env.
/// `None` means no override: the raw system clock.
fn cob_get_current_datetime_with(want_nano: bool, override_val: Option<&[u8]>) -> CobTime {
    let t = cob_get_current_date_and_time_from_os(want_nano);
    match override_val {
        Some(v) => apply_current_date_override(t, v),
        None => {
            let mut t = t;
            if t.second >= 60 {
                t.second = 59;
            }
            t
        }
    }
}

/// `get_seconds_past_midnight ()` (intrinsic.c): seconds since midnight from the **real** local clock
/// (does NOT honor `COB_CURRENT_DATE` — non-deterministic, hence not in the byte-oracle battery).
fn get_seconds_past_midnight() -> i32 {
    let t = cob_get_current_date_and_time_from_os(false);
    t.hour * 3600 + t.minute * 60 + t.second
}

/// Render a [`CobTime`] as the `FUNCTION CURRENT-DATE` field (`YYYYMMDDHHMMSShh+-HHMM`, 21 chars).
fn current_date_field(time: CobTime, offset: i32, length: i32) -> IntrField {
    let mut buff = format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}{:02}",
        time.year,
        time.month,
        time.day_of_month,
        time.hour,
        time.minute,
        time.second,
        time.nanosecond / 10000000
    )
    .into_bytes();
    add_offset_time(false, Some(time.utc_offset), &mut buff);
    buff.resize(21, b' ');
    if offset != 0 {
        return intr_refmod(buff, offset, length);
    }
    (buff, ALPHA1)
}

/// `cob_intr_current_date (offset, length)` (intrinsic.c): `FUNCTION CURRENT-DATE` —
/// `YYYYMMDDHHMMSShh+-HHMM` (21 chars). Honors the `COB_CURRENT_DATE` env override.
pub fn cob_intr_current_date(offset: i32, length: i32) -> IntrField {
    let want_nano = !(offset == 1 && length <= 14);
    current_date_field(cob_get_current_datetime(want_nano), offset, length)
}

/// As [`cob_intr_current_date`] but honoring an **explicit** `COB_CURRENT_DATE` override value -- e.g. the
/// `current_date` setting loaded from `runtime.cfg` (see `crate::common_configload::cob_runtime_config_value`).
/// `None` -> the system clock. This is the consuming end of the `cli-runtime-cfg` file-load wiring.
pub fn cob_intr_current_date_cfg(offset: i32, length: i32, override_val: Option<&[u8]>) -> IntrField {
    let want_nano = !(offset == 1 && length <= 14);
    current_date_field(cob_get_current_datetime_with(want_nano, override_val), offset, length)
}

/// `cob_intr_seconds_past_midnight ()` (intrinsic.c): `FUNCTION SECONDS-PAST-MIDNIGHT` (real-clock based).
pub fn cob_intr_seconds_past_midnight() -> IntrField {
    cob_alloc_set_field_int(get_seconds_past_midnight())
}

/// `format_current_date (date_fmt, time_fmt, buff)` (intrinsic.c): render the current datetime per the
/// parsed formats (over [`format_datetime`]).
fn format_current_date(date_fmt: DateFormat, time_fmt: TimeFormat) -> Vec<u8> {
    let time = cob_get_current_datetime(true);
    let days = integer_of_date(time.year, time.month, time.day_of_month) as i32;
    let seconds_from_midnight = time.hour * 3600 + time.minute * 60 + time.second;
    let fractional = CobDecimal { value: { let mut m = Mpz::new(); m.set_ui(time.nanosecond as u64); m }, scale: 9 };
    let offset_time = if time.offset_known { Some(time.utc_offset) } else { None };
    format_datetime(date_fmt, time_fmt, days, seconds_from_midnight, &fractional, offset_time, b'.')
}

/// `cob_intr_formatted_current_date (offset, length, format_field)` (intrinsic.c):
/// `FUNCTION FORMATTED-CURRENT-DATE(format)` — the current datetime in `format`; an invalid format -> spaces.
pub fn cob_intr_formatted_current_date(offset: i32, length: i32, fmt: &[u8]) -> IntrField {
    const COB_DATETIMESTR_MAX: usize = 36;
    let dec_pt = b'.';
    let format_str = copy_data_to_null_terminated_str(fmt, COB_DATETIMESTR_MAX);
    let field_length = format_str.len();
    if !cob_valid_datetime_format(&format_str, dec_pt) {
        return formatted_spaces(field_length, offset, length);
    }
    let (date_fmt_str, time_fmt_str, _) = split_around_t(&format_str);
    let date_fmt = parse_date_format_string(&date_fmt_str);
    let time_fmt = parse_time_format_string(&time_fmt_str);
    let mut buff = format_current_date(date_fmt, time_fmt);
    buff.resize(field_length, 0);
    if offset > 0 {
        return intr_refmod(buff, offset, length);
    }
    (buff, ALPHA1)
}

/// `cob_alloc_set_field_str (str, offset, length)` (intrinsic.c): an alphanumeric field holding `str`
/// (length = its byte length), with optional reference modification.
pub fn cob_alloc_set_field_str(s: &[u8], offset: i32, length: i32) -> IntrField {
    let out = s.to_vec();
    if offset > 0 {
        return intr_refmod(out, offset, length);
    }
    (out, ALPHA1)
}

/// `cob_alloc_set_field_spaces (n)` (intrinsic.c): an `n`-byte all-space alphanumeric field.
pub fn cob_alloc_set_field_spaces(n: usize) -> IntrField {
    (vec![b' '; n], ALPHA1)
}

// ---- module / exception introspection (intrinsic.c) --------------------------------------------
//
// These read the running module / last-exception global state (COB_MODULE_PTR, cobglobptr). gnucobol-rs is
// a function library with no such runtime, so each is ported as a parameterized formatter of the same
// state: the FORMATTING is 1:1, and the deterministic cases (a given module field, or the no-exception
// defaults) are oracle-testable.

/// `cob_intr_module_id ()` (intrinsic.c): `FUNCTION MODULE-ID` — the current module name.
pub fn cob_intr_module_id(module_name: &[u8]) -> IntrField {
    cob_alloc_set_field_str(module_name, 0, 0)
}

/// `cob_intr_module_source ()` (intrinsic.c): `FUNCTION MODULE-SOURCE`.
pub fn cob_intr_module_source(module_source: &[u8]) -> IntrField {
    cob_alloc_set_field_str(module_source, 0, 0)
}

/// `cob_intr_module_formatted_date ()` (intrinsic.c): `FUNCTION MODULE-FORMATTED-DATE`.
pub fn cob_intr_module_formatted_date(module_formatted_date: &[u8]) -> IntrField {
    cob_alloc_set_field_str(module_formatted_date, 0, 0)
}

/// `cob_intr_module_caller_id ()` (intrinsic.c): `FUNCTION MODULE-CALLER-ID` — the caller module name, or a
/// zero-length field when there is no caller.
pub fn cob_intr_module_caller_id(caller_name: Option<&[u8]>) -> IntrField {
    match caller_name {
        Some(name) => cob_alloc_set_field_str(name, 0, 0),
        None => (Vec::new(), ALPHA1),
    }
}

/// `cob_intr_module_path ()` (intrinsic.c): `FUNCTION MODULE-PATH` — the module path, or a zero-length
/// field when absent/empty.
pub fn cob_intr_module_path(path: Option<&[u8]>) -> IntrField {
    match path {
        Some(p) if !p.is_empty() => cob_alloc_set_field_str(p, 0, 0),
        _ => (Vec::new(), ALPHA1),
    }
}

/// `cob_intr_module_date ()` (intrinsic.c): `FUNCTION MODULE-DATE` — the module's numeric date as an
/// 8-digit `DISPLAY` field.
pub fn cob_intr_module_date(module_date: u32) -> IntrField {
    let mut b = format!("{module_date:08}").into_bytes();
    b.truncate(8);
    (b, FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 8, scale: 0, flags: 0 })
}

/// `cob_intr_module_time ()` (intrinsic.c): `FUNCTION MODULE-TIME` — a 6-digit `DISPLAY` field.
pub fn cob_intr_module_time(module_time: u32) -> IntrField {
    let mut b = format!("{module_time:06}").into_bytes();
    b.truncate(6);
    (b, FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 6, scale: 0, flags: 0 })
}

/// `cob_intr_exception_status ()` (intrinsic.c): `FUNCTION EXCEPTION-STATUS` — the last exception's name in
/// a 31-byte field (spaces when no exception is active).
pub fn cob_intr_exception_status(exception_name: Option<&[u8]>) -> IntrField {
    let mut out = vec![b' '; 31];
    if let Some(name) = exception_name {
        let n = name.len().min(31);
        out[..n].copy_from_slice(&name[..n]);
    }
    (out, ALPHA1)
}

/// `cob_intr_exception_statement ()` (intrinsic.c): `FUNCTION EXCEPTION-STATEMENT` — the last exception's
/// statement name in a 31-byte field (spaces when none).
pub fn cob_intr_exception_statement(statement: Option<&[u8]>) -> IntrField {
    let mut out = vec![b' '; 31];
    if let Some(s) = statement {
        let n = s.len().min(31);
        out[..n].copy_from_slice(&s[..n]);
    }
    (out, ALPHA1)
}

/// `cob_intr_exception_location ()` (intrinsic.c): `FUNCTION EXCEPTION-LOCATION` — `id; para OF section;
/// line` (with the variants when section/paragraph are absent); a single space when no exception is active.
pub fn cob_intr_exception_location(state: Option<(&[u8], Option<&[u8]>, Option<&[u8]>, u32)>) -> IntrField {
    match state {
        None => (vec![b' '], ALPHA1),
        Some((id, section, paragraph, line)) => {
            let s = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
            let buff = match (section, paragraph) {
                (Some(sec), Some(par)) => format!("{}; {} OF {}; {}", s(id), s(par), s(sec), line),
                (Some(sec), None) => format!("{}; {}; {}", s(id), s(sec), line),
                (None, Some(par)) => format!("{}; {}; {}", s(id), s(par), line),
                (None, None) => format!("{}; ; {}", s(id), line),
            };
            cob_alloc_set_field_str(buff.as_bytes(), 0, 0)
        }
    }
}

/// `cob_intr_exception_file ()` (intrinsic.c): `FUNCTION EXCEPTION-FILE` — the 2-char file status + the
/// select name on an I/O exception, else `"00"`.
pub fn cob_intr_exception_file(state: Option<(&[u8], &[u8])>) -> IntrField {
    match state {
        None => (b"00".to_vec(), ALPHA1),
        Some((status, select)) => {
            let mut out = Vec::with_capacity(2 + select.len());
            out.extend_from_slice(&status[..2.min(status.len())]);
            out.extend_from_slice(select);
            (out, ALPHA1)
        }
    }
}

// ---- locale-formatted date/time (intrinsic.c) --------------------------------------------------
//
// These format via the OS LC_TIME / LC_COLLATE. gnucobol-rs reproduces the C/POSIX-locale formats the
// oracle build uses (D_FMT `%m/%d/%y`, T_FMT `%H:%M:%S`, byte collation); a non-C `locale_field` selects a
// different OS-locale-DB format/collation — the OS-locale boundary, not modelled. An invalid argument
// yields a 10-space field (and an argument exception, not part of the result bytes).

/// True for the numeric field types `COB_FIELD_IS_NUMERIC` accepts.
fn field_is_numeric(attr: &FieldAttr) -> bool {
    matches!(
        attr.field_type,
        COB_TYPE_NUMERIC_DISPLAY | COB_TYPE_NUMERIC_BINARY | COB_TYPE_NUMERIC_PACKED | COB_TYPE_NUMERIC_COMP5 | COB_TYPE_NUMERIC_EDITED
    )
}

fn locale_derror() -> IntrField {
    cob_alloc_set_field_spaces(10)
}

/// `locale_time (hours, minutes, seconds, locale_field, buff)` (intrinsic.c): the C/POSIX-locale T_FMT
/// `%H:%M:%S`.
fn locale_time(hours: i32, minutes: i32, seconds: i32) -> Vec<u8> {
    format!("{hours:02}:{minutes:02}:{seconds:02}").into_bytes()
}

/// `cob_intr_locale_date (offset, length, srcfield, locale_field)` (intrinsic.c): `FUNCTION LOCALE-DATE` —
/// a `YYYYMMDD` rendered per LC_TIME's D_FMT (`mm/dd/yy` in the C locale).
pub fn cob_intr_locale_date(offset: i32, length: i32, src: &[u8], attr: &FieldAttr, _locale: Option<&[u8]>) -> IntrField {
    let indate = if field_is_numeric(attr) {
        cob_get_int(src, attr)
    } else {
        if src.len() < 8 {
            return locale_derror();
        }
        let mut v = 0i32;
        for &b in &src[..8] {
            if b.is_ascii_digit() {
                v = v * 10 + (b & 0x0F) as i32;
            } else {
                return locale_derror();
            }
        }
        v
    };
    let year = indate / 10000;
    if !valid_year(year) {
        return locale_derror();
    }
    let md = indate % 10000;
    let month = md / 100;
    if !valid_month(month) {
        return locale_derror();
    }
    let days = md % 100;
    if !valid_day_of_month(year, month, days) {
        return locale_derror();
    }
    cob_alloc_set_field_str(format!("{month:02}/{days:02}/{:02}", year % 100).as_bytes(), offset, length)
}

/// `cob_intr_locale_time (offset, length, srcfield, locale_field)` (intrinsic.c): `FUNCTION LOCALE-TIME` —
/// an `HHMMSS` rendered per LC_TIME's T_FMT (`hh:mm:ss` in the C locale).
pub fn cob_intr_locale_time(offset: i32, length: i32, src: &[u8], attr: &FieldAttr, _locale: Option<&[u8]>) -> IntrField {
    let indate = if field_is_numeric(attr) {
        cob_get_int(src, attr)
    } else {
        if src.len() < 6 {
            return locale_derror();
        }
        let mut v = 0i32;
        for &b in &src[..6] {
            if b.is_ascii_digit() {
                v = v * 10 + (b & 0x0F) as i32;
            } else {
                return locale_derror();
            }
        }
        v
    };
    let hours = indate / 10000;
    if !(0..=24).contains(&hours) {
        return locale_derror();
    }
    let minutes = (indate / 100) % 100;
    if minutes > 59 {
        return locale_derror();
    }
    let seconds = indate % 100;
    if seconds > 59 {
        return locale_derror();
    }
    cob_alloc_set_field_str(&locale_time(hours, minutes, seconds), offset, length)
}

/// `cob_intr_lcl_time_from_secs (offset, length, srcfield, locale_field)` (intrinsic.c):
/// `FUNCTION LOCALE-TIME-FROM-SECONDS` — seconds-since-midnight rendered per LC_TIME's T_FMT.
pub fn cob_intr_lcl_time_from_secs(offset: i32, length: i32, src: &[u8], attr: &FieldAttr, _locale: Option<&[u8]>) -> IntrField {
    if !field_is_numeric(attr) {
        return locale_derror();
    }
    let indate = cob_get_int(src, attr);
    if !valid_time(indate) {
        return locale_derror();
    }
    let hours = indate / 3600;
    let rem = indate % 3600;
    cob_alloc_set_field_str(&locale_time(hours, rem / 60, rem % 60), offset, length)
}

/// `cob_intr_locale_compare (params, ...)` (intrinsic.c): `FUNCTION LOCALE-COMPARE` — `'<'`/`'='`/`'>'` from
/// the LC_COLLATE comparison of the two (trailing-space-trimmed) operands (byte order in the C locale).
pub fn cob_intr_locale_compare(f1: &[u8], f2: &[u8], _locale: Option<&[u8]>) -> IntrField {
    let trim = |f: &[u8]| -> usize {
        let mut n = f.len();
        while n > 1 && f[n - 1] == b' ' {
            n -= 1;
        }
        n
    };
    let a = &f1[..trim(f1)];
    let b = &f2[..trim(f2)];
    let ch = match a.cmp(b) {
        std::cmp::Ordering::Less => b'<',
        std::cmp::Ordering::Greater => b'>',
        std::cmp::Ordering::Equal => b'=',
    };
    (vec![ch], ALPHA1)
}

/// `cob_init_intrinsic (lptr)` (intrinsic.c): module init — the C allocates a calc-struct pool + the global
/// scratch decimals/mpf; gnucobol-rs creates working values on demand (RAII), so this is a no-op.
pub fn cob_init_intrinsic() {}

/// `cob_exit_intrinsic (void)` (intrinsic.c): module teardown — a no-op in Rust (RAII frees on drop).
pub fn cob_exit_intrinsic() {}

/// `int_strncasecmp (s1, s2, n)` (intrinsic.c): ASCII case-insensitive comparison of the first `n` bytes.
pub fn int_strncasecmp(s1: &[u8], s2: &[u8], n: usize) -> i32 {
    for i in 0..n {
        let a = s1.get(i).copied().unwrap_or(0).to_ascii_lowercase();
        let b = s2.get(i).copied().unwrap_or(0).to_ascii_lowercase();
        if a != b {
            return a as i32 - b as i32;
        }
        if a == 0 {
            break;
        }
    }
    0
}

/// `add_z (buff_pos, buff)` (intrinsic.c): append the UTC `'Z'` marker to a time buffer.
pub fn add_z(buff: &mut Vec<u8>) {
    buff.push(b'Z');
}

/// `space_left (p, p_end)` (intrinsic.c): the inclusive count of bytes from `p` to `p_end` (`p_end - p + 1`).
pub fn space_left(p: usize, p_end: usize) -> usize {
    p_end - p + 1
}

/// `at_cr_or_db (p)` (intrinsic.c): whether the two bytes at `p` are `CR`/`cr` or `DB`/`db` (case-folded).
pub fn at_cr_or_db(p: &[u8]) -> bool {
    if p.len() < 2 {
        return false;
    }
    let a = p[0].to_ascii_uppercase();
    let b = p[1].to_ascii_uppercase();
    (a == b'C' && b == b'R') || (a == b'D' && b == b'B')
}

/// `cob_switch_value (id)` (intrinsic.c): `FUNCTION SWITCH-VALUE` — the program switch's value as an integer
/// field (the switch state is runtime-global; the value is supplied here).
pub fn cob_switch_value(value: i32) -> IntrField {
    cob_alloc_set_field_int(value)
}

/// Read a COBOL `POINTER` field's stored address (native byte order) from its (up to 8) data bytes.
fn read_pointer(b: &[u8]) -> u64 {
    let mut v = 0u64;
    for (i, &byte) in b.iter().take(8).enumerate() {
        v |= (byte as u64) << (i * 8);
    }
    v
}

/// `cob_intr_content_length (srcfield)` (intrinsic.c): `FUNCTION CONTENT-LENGTH(pointer)` — `strlen` of the
/// pointed-to C string, or 0 for a null pointer. A non-null pointer addresses live program memory the
/// library does not own; following it would require raw pointer dereference, which this crate forbids
/// (`#![forbid(unsafe_code)]`) — so only the well-defined null case is realised (the declared boundary).
pub fn cob_intr_content_length(ptr: &[u8]) -> IntrField {
    let _addr = read_pointer(ptr); // null -> 0; non-null -> unsafe-deref boundary, also 0 here
    cob_alloc_set_field_uint(0)
}

/// `cob_intr_content_of (offset, length, params, ...)` (intrinsic.c): `FUNCTION CONTENT-OF(pointer[, len])`
/// — the pointed-to bytes, or a zero-length field for a null pointer. The non-null dereference is the
/// `#![forbid(unsafe_code)]` boundary (see [`cob_intr_content_length`]); the null case is realised.
pub fn cob_intr_content_of(offset: i32, length: i32, ptr: &[u8], _request_len: u32) -> IntrField {
    let _addr = read_pointer(ptr);
    let out: Vec<u8> = Vec::new(); // null -> empty; non-null -> unsafe boundary
    if offset > 0 {
        return intr_refmod(out, offset, length);
    }
    (out, ALPHA1)
}

/// `get_interval_and_current_year_from_args (num_args, args, interval, current_year)` (intrinsic.c): the
/// windowing parameters for YEAR-TO-YYYY / DATE-TO-YYYYMMDD / DAY-TO-YYYYDDD — `interval` defaults to 50,
/// `current_year` to the system year.
#[allow(dead_code)] // exact-name 1:1 helper; equivalent logic is inlined at the call sites
fn get_interval_and_current_year_from_args(interval_arg: Option<i32>, current_year_arg: Option<i32>) -> (i32, i32) {
    let interval = interval_arg.unwrap_or(50);
    let current_year = current_year_arg.unwrap_or_else(|| cob_get_current_date_and_time_from_os(false).year);
    (interval, current_year)
}

/// `cob_put_indirect_field (f)` (intrinsic.c): stash a field for a later indirect get. libcob keeps a global
/// `move_field`; gnucobol-rs has no such runtime, so the stored copy is returned to the caller.
#[allow(dead_code)] // exact-name 1:1 helper; equivalent logic is inlined at the call sites
fn cob_put_indirect_field(f: &[u8], attr: &FieldAttr) -> (Vec<u8>, FieldAttr) {
    (f.to_vec(), *attr)
}

/// `cob_get_indirect_field (f)` (intrinsic.c): `cob_move` the stashed field into `f`.
#[allow(dead_code)] // exact-name 1:1 helper; equivalent logic is inlined at the call sites
fn cob_get_indirect_field(stored: &[u8], stored_attr: &FieldAttr, dst_attr: &FieldAttr, dst_len: usize) -> Vec<u8> {
    let mut out = vec![0u8; dst_len];
    let _ = crate::move_ops::cob_move(stored, stored_attr, &mut out, dst_attr);
    out
}

/// `cob_decimal_move_temp (src, dst)` (intrinsic.c): move a numeric `src` through a signed-`DISPLAY`
/// temporary sized to its trimmed magnitude into `dst`.
#[allow(dead_code)] // exact-name 1:1 helper; equivalent logic is inlined at the call sites
fn cob_decimal_move_temp(src: &[u8], src_attr: &FieldAttr, dst_attr: &FieldAttr, dst_len: usize) -> Vec<u8> {
    let mut d = cob_decimal_set_field(src, src_attr);
    cob_trim_decimal(&mut d);
    let size10 = (d.value.sizeinbase2() as f64 * 0.301_029_995_663_981_2_f64) as usize + 1;
    let size = size10.max(d.scale.max(0) as usize);
    let tattr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: size as u16, scale: d.scale as i16, flags: COB_FLAG_HAVE_SIGN };
    let temp = cob_decimal_get_field(d, &tattr, size, crate::arith::Round::Truncate, false).unwrap_or_else(|_| vec![0u8; size]);
    let mut out = vec![0u8; dst_len];
    let _ = crate::move_ops::cob_move(&temp, &tattr, &mut out, dst_attr);
    out
}

// ---- FUNCTION RANDOM (intrinsic.c) -------------------------------------------------------------
//
// libcob's RANDOM delegates to GMP: gmp_randinit_mt (Mersenne Twister) + gmp_randseed_ui + mpf_urandomb(63)
// + mpf_get_d, returning a COMP-2 double. The randomness is GMP's INTERNAL RNG (its non-reference MT seeding
// reduces the seed mod 2^19937-1 and fills the state by a GMP-specific procedure -- verified NOT to match
// the textbook MT19937 init_by_array). Reproducing GMP's exact bit-stream is reimplementing GMP's RNG, which
// is the project's declared GMP-substrate boundary (we port libcob's ALGORITHMS, not GMP's internals; we do
// not link libgmp). So this is ported as the faithful structure -- seed handling + a deterministic Mersenne
// Twister + the COMP-2 result -- with the exact value being the GMP-RNG substrate boundary (hence NOT in the
// byte-oracle battery). Output is reproducible per seed, just not GMP-bit-identical.

thread_local! {
    static RANDOM_STATE: std::cell::RefCell<Option<Mt19937>> = const { std::cell::RefCell::new(None) };
}

/// A standard MT19937 (Matsumoto/Nishimura 2002) -- the algorithm `gmp_randinit_mt` selects (GMP's
/// seeding/extraction differ in bit-exact detail; see the module note).
struct Mt19937 {
    mt: [u32; 624],
    mti: usize,
}

impl Mt19937 {
    fn seeded(seed: u32) -> Self {
        let mut mt = [0u32; 624];
        mt[0] = seed;
        for i in 1..624 {
            mt[i] = 1_812_433_253u32.wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_add(i as u32);
        }
        Mt19937 { mt, mti: 624 }
    }
    fn next_u32(&mut self) -> u32 {
        const MAG01: [u32; 2] = [0, 0x9908_b0df];
        if self.mti >= 624 {
            for kk in 0..227 {
                let y = (self.mt[kk] & 0x8000_0000) | (self.mt[kk + 1] & 0x7fff_ffff);
                self.mt[kk] = self.mt[kk + 397] ^ (y >> 1) ^ MAG01[(y & 1) as usize];
            }
            for kk in 227..623 {
                let y = (self.mt[kk] & 0x8000_0000) | (self.mt[kk + 1] & 0x7fff_ffff);
                self.mt[kk] = self.mt[kk + 397 - 624] ^ (y >> 1) ^ MAG01[(y & 1) as usize];
            }
            let y = (self.mt[623] & 0x8000_0000) | (self.mt[0] & 0x7fff_ffff);
            self.mt[623] = self.mt[396] ^ (y >> 1) ^ MAG01[(y & 1) as usize];
            self.mti = 0;
        }
        let mut y = self.mt[self.mti];
        self.mti += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }
    /// A 63-bit random fraction in `[0, 1)` (`mpf_urandomb(_, _, 63)` then `mpf_get_d`).
    fn next_f64(&mut self) -> f64 {
        let hi = self.next_u32() as u64;
        let lo = self.next_u32() as u64;
        let bits63 = ((hi << 31) | (lo >> 1)) & ((1u64 << 63) - 1);
        bits63 as f64 / (1u64 << 63) as f64
    }
}

/// `cob_intr_random (params, ...)` (intrinsic.c): `FUNCTION RANDOM([seed])` -- a COMP-2 (double) pseudo-random
/// value in `[0, 1)`. A negative seed raises an argument exception (and is ignored). See the module note:
/// the value is a deterministic Mersenne-Twister draw, with GMP's exact bit-stream the declared boundary.
pub fn cob_intr_random(seed: Option<i64>) -> IntrField {
    let val = RANDOM_STATE.with(|st| {
        let mut st = st.borrow_mut();
        if let Some(s) = seed {
            if s >= 0 {
                *st = Some(Mt19937::seeded(s as u32));
            }
        }
        if st.is_none() {
            *st = Some(Mt19937::seeded(0));
        }
        st.as_mut().unwrap().next_f64()
    });
    let attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DOUBLE, digits: 20, scale: 9, flags: COB_FLAG_HAVE_SIGN };
    (val.to_le_bytes().to_vec(), attr)
}

/// `cob_intr_exception_file_n ()` (intrinsic.c): unimplemented in GnuCOBOL 3.2; see [`error_not_implemented`].
pub fn cob_intr_exception_file_n() -> IntrField {
    error_not_implemented()
}

/// `cob_intr_exception_location_n ()` (intrinsic.c): unimplemented in GnuCOBOL 3.2; see
/// [`error_not_implemented`].
pub fn cob_intr_exception_location_n() -> IntrField {
    error_not_implemented()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn len(pic: &str, u: Usage) -> usize {
        intrinsic_length(pic, u).unwrap()
    }

    #[test]
    fn current_date_and_time_from_os_in_range() {
        // Reads the system clock (a runtime boundary); assert the decoded UTC components are in range.
        let t = cob_get_current_date_and_time_from_os(true);
        assert!(t.year >= 2020 && t.year < 3000);
        assert!((1..=12).contains(&t.month));
        assert!((1..=31).contains(&t.day_of_month));
        assert!((0..=23).contains(&t.hour));
        assert!((0..=59).contains(&t.minute));
        assert!((0..=60).contains(&t.second));
        assert!(!t.offset_known); // std exposes no timezone -> UTC, offset unknown
        let t2 = cob_get_current_date_and_time_from_os(false);
        assert_eq!(t2.nanosecond, 0); // want_nano=false zeroes the nanosecond field
    }
    #[test]
    fn current_date_offset_from_override_is_deterministic() {
        // FUNCTION CURRENT-DATE positions 17-21 carry the UTC offset. With an explicit COB_CURRENT_DATE
        // override the offset is pinned deterministically and matches the built-cobc oracle byte-for-byte
        // (the live-clock TZ offset -- override with no offset -- is the acknowledged OS-clock boundary).
        // Oracle: "...+0200" / "...-0500" / "Z"->"+0000". The override also pins the date; the live-clock
        // hundredths (positions 15-16) are not asserted.
        let off = |v: &[u8]| {
            let (b, _a) = cob_intr_current_date_cfg(0, 0, Some(v));
            assert_eq!(b.len(), 21);
            assert_eq!(&b[0..14], b"20260615143045", "override pins the date/time");
            String::from_utf8(b[16..21].to_vec()).unwrap()
        };
        assert_eq!(off(b"2026/06/15 14:30:45+0200"), "+0200");
        assert_eq!(off(b"2026/06/15 14:30:45-0500"), "-0500");
        assert_eq!(off(b"2026/06/15 14:30:45Z"), "+0000");
        assert_eq!(off(b"2026/06/15 14:30:45+05:30"), "+0530"); // colon form, half-hour zone
    }

    fn nv(s: &str) -> String {
        numval_display(&intrinsic_numval(s), 8, 4)
    }
    fn nvc(s: &str) -> String {
        numval_display(&intrinsic_numval_c(s), 8, 4)
    }
    #[test]
    fn stored_char_length_and_unimplemented() {
        let u3 = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: 0 };
        let (d, a) = cob_intr_stored_char_length(b"HI   ");
        assert_eq!(crate::accessors::cob_get_int(&d, &a), 2);
        let (d, _) = cob_intr_stored_char_length(b"     ");
        assert_eq!(crate::accessors::cob_get_int(&d, &u3), 0);
        // BOOLEAN-OF-INTEGER / INTEGER-OF-BOOLEAN are unimplemented in GnuCOBOL 3.2 -> empty result
        assert!(cob_intr_integer_of_boolean(b"1", &u3).0.is_empty());
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
    fn locale_case_and_compare_match_cutf8_oracle() {
        // Under the admitted C.UTF-8 oracle, UPPER-CASE/LOWER-CASE fold ONLY ASCII; a byte >=128 is left
        // untouched -- built-cobc: UPPER-CASE of `E9 61 0A` -> `e9 41 0a`. (Locale-sensitive 8-bit folding
        // only occurs under a non-C LC_CTYPE, outside the pinned locale -- so the port is faithful here.)
        assert_eq!(cob_intr_upper_case(0, 0, &[0xE9, b'a', 0x0A]).0, vec![0xE9, b'A', 0x0A]);
        assert_eq!(cob_intr_lower_case(0, 0, &[0xC9, b'A']).0, vec![0xC9, b'a']);
        // LOCALE-COMPARE uses LC_COLLATE = byte order under C.UTF-8 (strcoll == memcmp): built-cobc gives
        // `a`<`b` and `b`>`a`. So the port's bytewise compare is faithful under the pinned locale.
        assert_eq!(cob_intr_locale_compare(b"a", b"b", None).0, b"<");
        assert_eq!(cob_intr_locale_compare(b"b", b"a", None).0, b">");
        assert_eq!(cob_intr_locale_compare(b"a", b"a", None).0, b"=");
        // LOCALE-DATE renders per LC_TIME's D_FMT = `%m/%d/%y` under C.UTF-8: built-cobc LOCALE-DATE of
        // 20200615 -> `06/15/20`. The port hardcodes that oracle D_FMT, so it is faithful.
        let datt = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 8, scale: 0, flags: 0 };
        let ld = cob_intr_locale_date(0, 0, b"20200615", &datt, None).0;
        assert_eq!(&ld[..8], b"06/15/20");
        // LOCALE-TIME per LC_TIME's T_FMT = `%H:%M:%S` under C.UTF-8: 123456 -> `12:34:56`.
        let tatt = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 6, scale: 0, flags: 0 };
        let lt = cob_intr_locale_time(0, 0, b"123456", &tatt, None).0;
        assert_eq!(&lt[..8], b"12:34:56");
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
    fn numval_honors_decimal_comma_and_currency_sign() {
        // DECIMAL-POINT IS COMMA: built-cobc NUMVAL("1.234,56") = 1234.56 ('.' grouping, ',' decimal).
        assert_eq!(numval_display(&intrinsic_numval_cfg("1.234,56", true), 8, 4), "+00001234.5600");
        // CURRENCY SIGN IS "F" + DECIMAL-POINT IS COMMA: NUMVAL-C("F1.234,56") = 1234.56.
        assert_eq!(numval_display(&intrinsic_numval_c_cfg("F1.234,56", 'F', true), 8, 4), "+00001234.5600");
        // The default ($/'.'/no-comma) wrappers are byte-unchanged.
        assert_eq!(numval_display(&intrinsic_numval_c("$1,234.56"), 8, 4), "+00001234.5600");
        assert_eq!(numval_display(&intrinsic_numval_cfg("1,234.56", false), 8, 4), "+00001234.5600");
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

    // --- Date/time validation + formatting helper evidence (libcob intrinsic.c) ---

    #[test]
    fn char_and_digit_primitives() {
        // test_char_cond: advance on true (return 0), else offset+1 (offset unchanged).
        let mut o = 0;
        assert_eq!(test_char_cond(true, &mut o), 0);
        assert_eq!(o, 1);
        let mut o = 3;
        assert_eq!(test_char_cond(false, &mut o), 4);
        assert_eq!(o, 3);
        // test_char: match the wanted byte at offset.
        let mut o = 0;
        assert_eq!(test_char(b'A', b"A", &mut o), 0);
        assert_eq!(o, 1);
        let mut o = 0;
        assert_eq!(test_char(b'A', b"B", &mut o), 1);
        // test_char_in_range
        let mut o = 0;
        assert_eq!(test_char_in_range(b'a', b'z', b'm', &mut o), 0);
        let mut o = 0;
        assert_eq!(test_char_in_range(b'a', b'z', b'Z', &mut o), 1);
        // test_digit
        let mut o = 0;
        assert_eq!(test_digit(b'7', &mut o), 0);
        assert_eq!(o, 1);
        let mut o = 0;
        assert_eq!(test_digit(b'x', &mut o), 1);
    }

    #[test]
    fn year_components_accumulate() {
        // test_century / test_decade / test_unit_year accumulate `state`.
        let mut o = 0;
        let mut st = 0;
        assert_eq!(test_century(b"5", &mut o, &mut st), 0);
        assert_eq!(st, 5);
        let mut o = 0;
        let mut st = 16;
        assert_eq!(test_decade(b"0", &mut o, &mut st), 0);
        assert_eq!(st, 160);
        // test_unit_year: when state==160 the units digit must be 1..9 (year 1600 is invalid).
        let mut o = 0;
        let mut st = 160;
        assert_eq!(test_unit_year(b"0", &mut o, &mut st), 1); // 1600-01-01 not representable
        let mut o = 0;
        let mut st = 160;
        assert_eq!(test_unit_year(b"1", &mut o, &mut st), 0);
        assert_eq!(st, 1601);
        // test_year: full YYYY accumulates and validates.
        let mut o = 0;
        let mut st = 0;
        assert_eq!(test_year(b"2026", &mut o, &mut st), 0);
        assert_eq!((o, st), (4, 2026));
        let mut o = 0;
        let mut st = 0;
        assert_eq!(test_year(b"0500", &mut o, &mut st), 1); // millennium 0 invalid
    }

    #[test]
    fn month_day_and_separator_validators() {
        // test_month: 01..12 valid, 13 invalid (fails on the 2nd digit -> offset+1 == 2).
        let mut o = 0;
        let mut m = 0;
        assert_eq!(test_month(b"03", &mut o, &mut m), 0);
        assert_eq!(m, 3);
        let mut o = 0;
        let mut m = 0;
        assert_eq!(test_month(b"12", &mut o, &mut m), 0);
        assert_eq!(m, 12);
        let mut o = 0;
        let mut m = 0;
        assert_eq!(test_month(b"13", &mut o, &mut m), 2);
        // test_day_of_month: Feb 29 valid in a leap year, invalid otherwise.
        let mut o = 0;
        assert_eq!(test_day_of_month(b"29", 2024, 2, &mut o), 0);
        let mut o = 0;
        assert_eq!(test_day_of_month(b"29", 2023, 2, &mut o), 2);
        // test_day_of_year: 366 valid in leap year, invalid otherwise.
        let mut o = 0;
        assert_eq!(test_day_of_year(b"366", 2024, &mut o), 0);
        let mut o = 0;
        assert_eq!(test_day_of_year(b"366", 2023, &mut o), 3);
        let mut o = 0;
        assert_eq!(test_day_of_year(b"000", 2024, &mut o), 3); // 000 invalid
        // test_hyphen_presence
        let mut o = 0;
        assert_eq!(test_hyphen_presence(true, b"-", &mut o), 0);
        assert_eq!(o, 1);
        let mut o = 5;
        assert_eq!(test_hyphen_presence(false, b"x", &mut o), 0); // no hyphen expected
        assert_eq!(o, 5);
    }

    #[test]
    fn week_and_day_of_week_validators() {
        // test_w_presence: literal 'W'.
        let mut o = 0;
        assert_eq!(test_w_presence(b"W", &mut o), 0);
        let mut o = 0;
        assert_eq!(test_w_presence(b"x", &mut o), 1);
        // test_week: 2024 has 52 ISO weeks -> "53" invalid, "52" valid.
        let mut o = 0;
        assert_eq!(test_week(b"52", 2024, &mut o), 0);
        let mut o = 0;
        assert_eq!(test_week(b"00", 2024, &mut o), 2); // week 00 invalid
        // test_day_of_week: 1..7.
        let mut o = 0;
        assert_eq!(test_day_of_week(b"7", &mut o), 0);
        let mut o = 0;
        assert_eq!(test_day_of_week(b"8", &mut o), 1);
        let mut o = 0;
        assert_eq!(test_day_of_week(b"0", &mut o), 1);
    }

    #[test]
    fn date_end_and_trailing_junk() {
        // test_date_end for a YYYYMMDD-shaped tail (mmdd).
        let fmt_mmdd = DateFormat { days: DaysFormat::Mmdd, with_hyphens: false };
        let mut o = 0;
        assert_eq!(test_date_end(fmt_mmdd, b"0229", 2024, &mut o), 0);
        assert_eq!(o, 4);
        // test_date_end for ddd.
        let fmt_ddd = DateFormat { days: DaysFormat::Ddd, with_hyphens: false };
        let mut o = 0;
        assert_eq!(test_date_end(fmt_ddd, b"060", 2024, &mut o), 0);
        // test_no_trailing_junk: trailing spaces OK at end-of-string.
        assert_eq!(test_no_trailing_junk(b"   ", 0, true), 0);
        assert_eq!(test_no_trailing_junk(b"  x", 0, true), 3); // non-space -> 1-based pos
        assert_eq!(test_no_trailing_junk(b"", 0, false), 0); // not end-of-string but at NUL
    }

    #[test]
    fn time_component_validators() {
        // test_hour: 00..23.
        let mut o = 0;
        assert_eq!(test_hour(b"23", &mut o), 0);
        let mut o = 0;
        assert_eq!(test_hour(b"24", &mut o), 2);
        // test_less_than_60 / test_minute / test_second.
        let mut o = 0;
        assert_eq!(test_less_than_60(b"59", &mut o), 0);
        let mut o = 0;
        assert_eq!(test_less_than_60(b"60", &mut o), 1);
        let mut o = 0;
        assert_eq!(test_minute(b"00", &mut o), 0);
        let mut o = 0;
        assert_eq!(test_second(b"30", &mut o), 0);
        let mut o = 0;
        assert_eq!(test_second(b"99", &mut o), 1);
    }

    #[test]
    fn time_separator_and_zone_validators() {
        // test_colon_presence
        let mut o = 0;
        assert_eq!(test_colon_presence(true, b":", &mut o), 0);
        assert_eq!(o, 1);
        let mut o = 7;
        assert_eq!(test_colon_presence(false, b"x", &mut o), 0);
        assert_eq!(o, 7);
        // test_decimal_places: '.' then `num` digits.
        let mut o = 0;
        assert_eq!(test_decimal_places(3, b'.', b".500", &mut o), 0);
        assert_eq!(o, 4);
        let mut o = 0;
        assert_eq!(test_decimal_places(2, b'.', b".5x", &mut o), 3); // non-digit
        let mut o = 9;
        assert_eq!(test_decimal_places(0, b'.', b"", &mut o), 0); // zero places: no-op
        assert_eq!(o, 9);
        // test_z_presence
        let mut o = 0;
        assert_eq!(test_z_presence(b"Z", &mut o), 0);
        let mut o = 0;
        assert_eq!(test_z_presence(b"z", &mut o), 1);
        // test_two_zeroes
        let mut o = 0;
        assert_eq!(test_two_zeroes(b"00", &mut o), 0);
        assert_eq!(o, 2);
        let mut o = 0;
        assert_eq!(test_two_zeroes(b"01", &mut o), 2);
    }

    #[test]
    fn offset_time_validators() {
        let fmt_off = TimeFormat { with_colons: false, decimal_places: 0, extra: TimeExtra::OffsetTime };
        // "+0130" valid offset.
        let mut o = 0;
        assert_eq!(test_offset_time(fmt_off, b"+0130", &mut o), 0);
        assert_eq!(o, 5);
        // literal "00000" valid (leading '0' then two "00" zero-pairs).
        let mut o = 0;
        assert_eq!(test_offset_time(fmt_off, b"00000", &mut o), 0);
        assert_eq!(o, 5);
        // leading char neither +/-/0 -> immediate failure.
        let mut o = 0;
        assert_eq!(test_offset_time(fmt_off, b"x000", &mut o), 1);
        // test_time_end with a Z zone.
        let fmt_z = TimeFormat { with_colons: false, decimal_places: 0, extra: TimeExtra::Z };
        let mut o = 0;
        assert_eq!(test_time_end(fmt_z, b"Z", &mut o), 0);
        let fmt_none = TimeFormat { with_colons: false, decimal_places: 0, extra: TimeExtra::None };
        let mut o = 4;
        assert_eq!(test_time_end(fmt_none, b"", &mut o), 0); // None: no zone
        // valid_offset_time: |offset| < 1440 minutes.
        assert!(valid_offset_time(0));
        assert!(valid_offset_time(1439));
        assert!(valid_offset_time(-1439));
        assert!(!valid_offset_time(1440));
        assert!(!valid_offset_time(-1440));
    }

    #[test]
    fn date_format_string_helpers() {
        // decimal_places_for_seconds: count of 's' after the decimal point position.
        assert_eq!(decimal_places_for_seconds(b"hhmmss.sss", 6), 3);
        assert_eq!(decimal_places_for_seconds(b"hhmmss.s", 6), 1);
        // rest_is_z / rest_is_offset_format
        assert!(rest_is_z(b"Z"));
        assert!(!rest_is_z(b"+hhmm"));
        assert!(rest_is_offset_format(b"+hhmm", false));
        assert!(rest_is_offset_format(b"+hh:mm", true));
        assert!(!rest_is_offset_format(b"+hh:mm", false));
    }

    #[test]
    fn day_of_week_and_iso_week_helpers() {
        // get_day_of_week: day 1 (1601-01-01) is a Monday -> index 0.
        assert_eq!(get_day_of_week(1), 0);
        assert_eq!(get_day_of_week(8), 0);
        assert_eq!(get_day_of_week(7), 6);
        // get_iso_week_one: Monday of ISO week 1 is <= Jan 4 of that year.
        let day_jan4_2024 = integer_of_date(2024, 1, 4) as i32;
        let (_, doy) = day_of_integer(day_jan4_2024);
        let w1 = get_iso_week_one(day_jan4_2024, doy);
        assert_eq!(get_day_of_week(w1), 0); // it is a Monday
        assert!(w1 <= day_jan4_2024);
        // get_iso_week: Jan 4 is always in ISO week 1.
        assert_eq!(get_iso_week(day_jan4_2024), (2024, 1));
        // get_iso_week round-trips with format_as_yyyywwwd below.
        // max_week: 2020 has 53 ISO weeks, 2021 has 52.
        assert_eq!(max_week(2020), 53);
        assert_eq!(max_week(2021), 52);
    }

    #[test]
    fn date_formatters_render_known_dates() {
        let d = integer_of_date(2024, 2, 29) as i32;
        assert_eq!(format_as_yyyymmdd(d, false), b"20240229");
        assert_eq!(format_as_yyyymmdd(d, true), b"2024-02-29");
        let d2 = integer_of_date(2024, 3, 1) as i32; // day-of-year 061 in leap 2024
        assert_eq!(format_as_yyyyddd(d2, false), b"2024061");
        assert_eq!(format_as_yyyyddd(d2, true), b"2024-061");
        // format_as_yyyywwwd: 2024-01-01 is ISO 2024-W01-1 (a Monday).
        let d3 = integer_of_date(2024, 1, 1) as i32;
        assert_eq!(format_as_yyyywwwd(d3, false), b"2024W011");
        assert_eq!(format_as_yyyywwwd(d3, true), b"2024-W01-1");
    }

    #[test]
    fn integer_of_formatted_parts() {
        let fmt_mmdd = DateFormat { days: DaysFormat::Mmdd, with_hyphens: false };
        // integer_of_mmdd: parse "0229" with year 2024 -> same as integer_of_date(2024,2,29).
        assert_eq!(integer_of_mmdd(fmt_mmdd, 2024, b"0229"), integer_of_date(2024, 2, 29));
        let fmt_mmdd_h = DateFormat { days: DaysFormat::Mmdd, with_hyphens: true };
        assert_eq!(integer_of_mmdd(fmt_mmdd_h, 2024, b"02-29"), integer_of_date(2024, 2, 29));
        // integer_of_ddd: "061" of 2024 -> 2024-03-01.
        assert_eq!(integer_of_ddd(2024, b"061"), integer_of_date(2024, 3, 1));
        // integer_of_wwwd: ISO 2024-W01-1 == 2024-01-01.
        let fmt_w = DateFormat { days: DaysFormat::Wwwd, with_hyphens: false };
        assert_eq!(integer_of_wwwd(fmt_w, 2024, b"W011"), integer_of_date(2024, 1, 1));
    }

    #[test]
    fn substitute_helpers() {
        let pairs: &[(&[u8], &[u8])] = &[(b"ab".as_slice(), b"X".as_slice())];
        // "abcab" -> "XcX": size 3.
        assert_eq!(get_substituted_size(b"abcab", pairs, false), 3);
        let mut out = Vec::new();
        substitute_matches(b"abcab", pairs, false, &mut out);
        assert_eq!(out, b"XcX");
        // case-insensitive
        let pairs2: &[(&[u8], &[u8])] = &[(b"AB".as_slice(), b"Y".as_slice())];
        assert_eq!(get_substituted_size(b"abcab", pairs2, true), 3);
        let mut out2 = Vec::new();
        substitute_matches(b"abcab", pairs2, true, &mut out2);
        assert_eq!(out2, b"YcY");
    }

    #[test]
    fn add_z_and_add_decimal_digits() {
        // add_z appends the UTC marker.
        let mut buff = b"12".to_vec();
        add_z(&mut buff);
        assert_eq!(buff, b"12Z");
        // add_decimal_digits: '.' then `decimal_places` fraction digits, right-padded with zeros.
        // second_fraction = 0.5 (value 5 scale 1) -> ".500" for 3 places.
        let frac = CobDecimal { value: { let mut m = Mpz::new(); m.set_ui(5); m }, scale: 1 };
        let mut buff = Vec::new();
        add_decimal_digits(3, &frac, &mut buff, b'.');
        assert_eq!(buff, b".500");
        // 0 places: just the decimal point.
        let mut buff = Vec::new();
        add_decimal_digits(0, &frac, &mut buff, b'.');
        assert_eq!(buff, b".");
    }

    #[test]
    fn clock_based_intrinsics_run() {
        // get_seconds_past_midnight is a real-clock read in [0, 86400).
        let s = get_seconds_past_midnight();
        assert!((0..86400).contains(&s));
        // cob_intr_seconds_past_midnight wraps it into a BINARY result field that decodes to the same range.
        let (d, a) = cob_intr_seconds_past_midnight();
        let v = crate::accessors::cob_get_int(&d, &a);
        assert!((0..86400).contains(&v));
        // cob_intr_random: reseed(42) -> deterministic value in [0, 1); reseeding repeats it.
        let (d1, _) = cob_intr_random(Some(42));
        let r1 = f64::from_le_bytes(d1.try_into().unwrap());
        assert!((0.0..1.0).contains(&r1));
        let (d2, _) = cob_intr_random(Some(42));
        let r2 = f64::from_le_bytes(d2.try_into().unwrap());
        assert_eq!(r1, r2); // same seed -> same first draw
    }

    #[test]
    fn unimplemented_intrinsics_return_empty() {
        let an = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };
        // These FUNCTIONs are unimplemented in GnuCOBOL 3.2 -> empty not-implemented field.
        assert!(cob_intr_boolean_of_integer(b"1", &an, b"8", &an).0.is_empty());
        assert!(cob_intr_char_national(b"A", &an).0.is_empty());
        assert!(cob_intr_display_of(0, 0, &[(b"A".as_slice(), &an)]).0.is_empty());
        assert!(cob_intr_national_of(0, 0, &[(b"A".as_slice(), &an)]).0.is_empty());
        assert!(cob_intr_standard_compare(&[(b"A".as_slice(), &an)]).0.is_empty());
        assert!(cob_intr_exception_file_n().0.is_empty());
        assert!(cob_intr_exception_location_n().0.is_empty());
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
