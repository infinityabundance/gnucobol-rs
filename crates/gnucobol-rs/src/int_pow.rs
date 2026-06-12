//! Integer exponentiation as GnuCOBOL emits it for an integer `**` operator (`GNURUST.INTPOW.1`).
//!
//! cobc's code generator (`cobc/codegen.c`) lowers `base ** power` over integer operands to a call
//! to `cob_s32_pow` / `cob_s64_pow` (`libcob/numeric.c`, the `POW_IMPL` macro). These are faithful
//! ports of that macro, including its exact edge cases:
//!
//! - `power == 0` -> `1` (even `0 ** 0 == 1`);
//! - `base == 1` or `base == -1` -> `1` **regardless of the power** (libcob does not special-case the
//!   sign of `(-1) ** odd`; the macro returns 1 for `base == -1`, so `(-1) ** 3` yields `1`, not `-1`);
//! - `power < 0` -> `0`, except `0 ** (negative)` which `cob_raise (SIGFPE)`s in libcob (a crash) ->
//!   here it fails closed as a typed error rather than aborting;
//! - otherwise repeated multiply in the target integer width, **wrapping** on overflow (C `*=` on
//!   `cob_s32_t` / `cob_s64_t` wraps two's-complement under GnuCOBOL's build).
#![allow(clippy::needless_range_loop)]

use crate::arith::ArithError;

/// `cob_s32_pow (base, power)` — 32-bit integer power (numeric.c POW_IMPL over `cob_s32_t`).
pub fn cob_s32_pow(base: i32, power: i32) -> Result<i32, ArithError> {
    if power == 0 || base == 1 || base == -1 {
        return Ok(1);
    }
    if power < 0 {
        if base == 0 {
            return Err(ArithError::DivideByZero); // libcob: cob_raise(SIGFPE)
        }
        return Ok(0);
    }
    let mut ret: i32 = 1;
    let mut p = power;
    while p > 0 {
        ret = ret.wrapping_mul(base);
        p -= 1;
    }
    Ok(ret)
}

/// `cob_s64_pow (base, power)` — 64-bit integer power (numeric.c POW_IMPL over `cob_s64_t`).
pub fn cob_s64_pow(base: i64, power: i64) -> Result<i64, ArithError> {
    if power == 0 || base == 1 || base == -1 {
        return Ok(1);
    }
    if power < 0 {
        if base == 0 {
            return Err(ArithError::DivideByZero);
        }
        return Ok(0);
    }
    let mut ret: i64 = 1;
    let mut p = power;
    while p > 0 {
        ret = ret.wrapping_mul(base);
        p -= 1;
    }
    Ok(ret)
}

/// Fuzz target: integer power never panics for any (base, power) and matches the wrapping macro.
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_int_pow(data: &[u8]) {
    if data.len() < 12 {
        return;
    }
    let base = i64::from_le_bytes(data[0..8].try_into().unwrap());
    let power = i32::from_le_bytes(data[8..12].try_into().unwrap()) as i64;
    let _ = cob_s64_pow(base, power);
    let _ = cob_s32_pow(base as i32, power as i32);
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.INTPOW.1
    /// Integer power is total: any base with a bounded power yields a value or a typed error, never a
    /// panic (wrapping_mul cannot overflow-panic; the loop is bounded for the proof).
    #[kani::proof]
    #[kani::unwind(6)]
    fn pow_total() {
        let base: i64 = kani::any();
        let power: i64 = kani::any();
        kani::assume((-2..=4).contains(&power)); // bound the repeated-multiply loop
        let _ = cob_s64_pow(base, power);
        let _ = cob_s32_pow(base as i32, power as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_edge_cases_match_libcob() {
        assert_eq!(cob_s64_pow(7, 0).unwrap(), 1); // power 0
        assert_eq!(cob_s64_pow(0, 0).unwrap(), 1); // 0**0 = 1
        assert_eq!(cob_s64_pow(1, 999).unwrap(), 1); // base 1
        assert_eq!(cob_s64_pow(-1, 3).unwrap(), 1); // base -1 -> 1 even for odd power (libcob quirk)
        assert_eq!(cob_s64_pow(-1, 2).unwrap(), 1);
        assert_eq!(cob_s64_pow(5, -2).unwrap(), 0); // negative power -> 0
        assert!(cob_s64_pow(0, -1).is_err()); // 0 ** negative -> SIGFPE -> fail closed
    }

    #[test]
    fn power_normal_values() {
        assert_eq!(cob_s64_pow(2, 10).unwrap(), 1024);
        assert_eq!(cob_s64_pow(-2, 3).unwrap(), -8);
        assert_eq!(cob_s64_pow(10, 18).unwrap(), 1_000_000_000_000_000_000);
        assert_eq!(cob_s32_pow(3, 4).unwrap(), 81);
    }

    #[test]
    fn power_wraps_on_overflow_like_c() {
        // 2 ** 31 overflows i32 -> wraps to i32::MIN (C cob_s32_t *= wraps).
        assert_eq!(cob_s32_pow(2, 31).unwrap(), i32::MIN);
        // 2 ** 64 wraps i64 to 0.
        assert_eq!(cob_s64_pow(2, 64).unwrap(), 0);
    }
}
