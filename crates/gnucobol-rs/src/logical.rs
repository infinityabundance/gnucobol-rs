//! COBOL bit-logical operations: `B-AND`, `B-OR`, `B-XOR`, `B-NOT`, and the bit shifts
//! (`GNURUST.LOGICAL.1`). Faithful port of libcob's `cob_logical_and/or/xor/not/left/right`
//! (`numeric.c`):
//!
//! - each operand is reduced to its **absolute value modulo 2^64** (`mpz_get_ui` ignores the sign
//!   and keeps only the low 64 bits of the unscaled integer);
//! - the C bitwise operator is applied over `cob_u64_t`;
//! - shift counts use the C `<<` / `>>` on a 64-bit value, whose count is taken modulo 64 on the
//!   admitted x86-64 oracle (`wrapping_shl`/`wrapping_shr` reproduce that exactly);
//! - the unsigned 64-bit result becomes the new value (`cob_decimal_set_ullint`).
//!
//! Inputs are taken as integers (the unscaled magnitude); a fractional operand's scale is dropped,
//! matching `mpz_get_ui` over `d->value`.
#![forbid(unsafe_code)]

/// `|value| mod 2^64` — the operand reduction libcob applies via `mpz_get_ui` (sign ignored).
fn to_u64(v: i128) -> u64 {
    v.unsigned_abs() as u64
}

/// `B-AND`: `cob_logical_and`.
pub fn logical_and(a: i128, b: i128) -> u64 {
    to_u64(a) & to_u64(b)
}
/// `B-OR`: `cob_logical_or`.
pub fn logical_or(a: i128, b: i128) -> u64 {
    to_u64(a) | to_u64(b)
}
/// `B-XOR`: `cob_logical_xor`.
pub fn logical_xor(a: i128, b: i128) -> u64 {
    to_u64(a) ^ to_u64(b)
}
/// `B-NOT`: `cob_logical_not` (one's complement of the low 64 bits).
pub fn logical_not(b: i128) -> u64 {
    !to_u64(b)
}
/// Left bit shift: `cob_logical_left` (`u0 << u1`, count mod 64).
pub fn logical_left(a: i128, b: i128) -> u64 {
    to_u64(a).wrapping_shl(to_u64(b) as u32)
}
/// Right bit shift: `cob_logical_right` (`u0 >> u1`, count mod 64).
pub fn logical_right(a: i128, b: i128) -> u64 {
    to_u64(a).wrapping_shr(to_u64(b) as u32)
}

/// Fuzz target: every bit-logical op is total over arbitrary operand pairs.
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_logical(data: &[u8]) {
    if data.len() < 16 {
        return;
    }
    let a = i64::from_le_bytes(data[0..8].try_into().unwrap()) as i128;
    let b = i64::from_le_bytes(data[8..16].try_into().unwrap()) as i128;
    let _ = (
        logical_and(a, b),
        logical_or(a, b),
        logical_xor(a, b),
        logical_not(b),
        logical_left(a, b),
        logical_right(a, b),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_bit_ops() {
        assert_eq!(logical_and(0b1100, 0b1010), 0b1000);
        assert_eq!(logical_or(0b1100, 0b1010), 0b1110);
        assert_eq!(logical_xor(0b1100, 0b1010), 0b0110);
        assert_eq!(logical_not(0), u64::MAX);
        assert_eq!(logical_left(1, 4), 16);
        assert_eq!(logical_right(256, 4), 16);
    }

    #[test]
    fn sign_is_ignored_like_mpz_get_ui() {
        // mpz_get_ui uses the ABSOLUTE value's low 64 bits (sign ignored), NOT two's complement:
        // so B-AND(-1, 0xFF) = (|-1| & 0xFF) = 1, not 0xFF. (Verified against the libcob oracle by
        // the logical sweep, not assumed.)
        assert_eq!(logical_and(-1, 0xFF), 1);
        assert_eq!(logical_and(-255, 0xFF), 255); // |-255| = 0xFF
        assert_eq!(logical_not(-1), u64::MAX - 1); // !(|-1|=1) = ...FFFE
    }

    #[test]
    fn shift_count_is_mod_64() {
        assert_eq!(logical_left(1, 64), 1); // 64 mod 64 = 0 -> no shift
        assert_eq!(logical_left(1, 65), 2); // 65 mod 64 = 1
        assert_eq!(logical_right(0x8000_0000_0000_0000, 63), 1);
    }
}
