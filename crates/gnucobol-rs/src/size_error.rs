//! Arithmetic SIZE ERROR byte effects (`GNURUST.SIZE.ERROR.1`): what happens to a numeric receiver when an
//! arithmetic result does not fit, proven against GnuCOBOL 3.2. Implements the behavior the observed
//! `SIZE.ERROR.ATLAS.1` mapped.
//!
//! **Witnessed rules (from the oracle):** a *size error condition* occurs when the result's integer part has
//! **more significant digits than the receiver's integer capacity** (overflow), or on **divide by zero**.
//! - **Without `ON SIZE ERROR`** the receiver gets the **truncated** result: the **low-order** integer digits
//!   (most-significant digits dropped) and the fraction truncated toward zero to the receiver's scale — e.g.
//!   `999 + 999 = 1998` into `9(3)` stores `998`; `1234.567` into `9(3)V99` stores `234.56`; the sign is kept.
//! - **With `ON SIZE ERROR`** the receiver is **left unchanged** (its prior value) and the imperative runs.
//!
//! This module computes the size-error condition and the truncated digits (the no-`ON SIZE ERROR` store).
//! The "receiver unchanged" case is the caller's: when [`SizeErrorResult::size_error`] is set and the
//! statement codes `ON SIZE ERROR`, keep the prior receiver bytes instead of [`SizeErrorResult::truncated`].
//!
//! **Non-claims:** the arithmetic itself (that is `GNURUST.7`/`13`/`19`), `ROUNDED`, intermediate-result
//! precision rules, floating-point receivers, `SIZE ERROR` on `MOVE`, and all dialects.

/// The SIZE ERROR outcome for one arithmetic store into a fixed numeric receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizeErrorResult {
    /// A size error condition occurred (integer overflow or divide-by-zero).
    pub size_error: bool,
    /// The receiver's stored digit bytes when there is **no** `ON SIZE ERROR`: `recv_int + recv_scale` digit
    /// bytes (`'0'..'9'`, no sign, no `.`), low-order on overflow.
    pub truncated: Vec<u8>,
}

/// Apply an arithmetic result to a fixed numeric receiver. `int_digits` / `frac_digits` are the result's
/// integer and fractional magnitude digit bytes (sign handled by the caller); `recv_int` / `recv_scale` the
/// receiver's integer-digit count and scale. Returns the size-error condition and the truncated store.
pub fn arith_size_error(int_digits: &[u8], frac_digits: &[u8], recv_int: usize, recv_scale: usize) -> SizeErrorResult {
    let significant = int_digits.iter().skip_while(|&&b| b == b'0').count();
    let size_error = significant > recv_int;
    // low-order recv_int integer digits, left-padded with '0'
    let mut out: Vec<u8> = if int_digits.len() >= recv_int {
        int_digits[int_digits.len() - recv_int..].to_vec()
    } else {
        let mut v = vec![b'0'; recv_int - int_digits.len()];
        v.extend_from_slice(int_digits);
        v
    };
    // first recv_scale fractional digits (truncate toward zero), right-padded with '0'
    let mut frac: Vec<u8> = frac_digits.iter().take(recv_scale).copied().collect();
    frac.resize(recv_scale, b'0');
    out.extend_from_slice(&frac);
    SizeErrorResult { size_error, truncated: out }
}

/// Divide by zero is always a size error; the receiver is unchanged under `ON SIZE ERROR`. Provided for
/// completeness — there is no meaningful "truncated" store for divide-by-zero.
pub fn divide_by_zero_is_size_error() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn se(i: &[u8], f: &[u8], ri: usize, rs: usize) -> SizeErrorResult {
        arith_size_error(i, f, ri, rs)
    }
    #[test]
    fn overflow_truncates_low_order_digits() {
        // 999+999=1998 into 9(3) -> "998", size error
        assert_eq!(se(b"1998", b"", 3, 0), SizeErrorResult { size_error: true, truncated: b"998".to_vec() });
        // 1234.567 into 9(3)V99 -> "234"+"56", size error
        assert_eq!(se(b"1234", b"567", 3, 2), SizeErrorResult { size_error: true, truncated: b"23456".to_vec() });
        // 50000 into 9(3) -> "000", size error
        assert_eq!(se(b"50000", b"", 3, 0), SizeErrorResult { size_error: true, truncated: b"000".to_vec() });
        // 12.5 into 9(1)V9 -> "2"+"5", size error
        assert_eq!(se(b"12", b"5", 1, 1), SizeErrorResult { size_error: true, truncated: b"25".to_vec() });
    }
    #[test]
    fn fits_no_size_error() {
        // 46 into 9(3) -> "046", no error
        assert_eq!(se(b"46", b"", 3, 0), SizeErrorResult { size_error: false, truncated: b"046".to_vec() });
        // 7.89 into 9(1)V9 -> "7"+"8" (fraction truncated), no error
        assert_eq!(se(b"7", b"89", 1, 1), SizeErrorResult { size_error: false, truncated: b"78".to_vec() });
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.SIZE.ERROR.1
    /// The no-ON-SIZE-ERROR truncated store is ALWAYS exactly `recv_int + recv_scale` digit bytes.
    #[kani::proof]
    #[kani::unwind(13)]
    fn truncated_width_is_recv_width() {
        let int_digits: [u8; 6] = kani::any();
        let frac_digits: [u8; 4] = kani::any();
        let recv_int: usize = kani::any();
        let recv_scale: usize = kani::any();
        kani::assume(recv_int <= 8 && recv_scale <= 4);
        let r = arith_size_error(&int_digits, &frac_digits, recv_int, recv_scale);
        assert_eq!(r.truncated.len(), recv_int + recv_scale);
    }
}
