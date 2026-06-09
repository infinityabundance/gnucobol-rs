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
