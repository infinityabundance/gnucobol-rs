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

#[cfg(test)]
mod tests {
    use super::*;
    fn len(pic: &str, u: Usage) -> usize {
        intrinsic_length(pic, u).unwrap()
    }
    fn nv(s: &str) -> String {
        numval_display(&intrinsic_numval(s), 8, 4)
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
