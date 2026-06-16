//! Decimal arithmetic (`GNURUST.7`): `ADD` / `SUBTRACT` / `MULTIPLY` on numeric fields, with
//! truncation and `ROUNDED` (nearest, ties away from zero), proven byte-identical against libcob's
//! `cob_add` / `cob_sub` / `cob_mul`.
//!
//! **Pure-Rust integer decimal.** libcob's `cob_decimal` is GMP-backed (`mpz` value + scale); this
//! port reproduces the **integer-decimal** result with a fixed-precision `i128` magnitude + scale —
//! zero runtime dependencies, `#![forbid(unsafe_code)]`, no floating point. The store path mirrors
//! `cob_decimal_get_field` (`numeric.c:2055`): adjust to the field scale (truncating `shift_decimal`
//! or `cob_decimal_do_round` for `ROUNDED`), then render to a DISPLAY temp and `cob_move` into the
//! target type — reusing the sealed [`crate::cob_move`].
//!
//! **Sealed subset:** `op := a (op) b` for `op ∈ {ADD, SUBTRACT, MULTIPLY}`, DISPLAY / COMP-3
//! operands and receiver, truncation or `ROUNDED` (nearest-away-from-zero). `ADD`/`SUBTRACT` into a
//! **PACKED** receiver take libcob's separate `cob_add_bcd` path (`GNURUST.13`) — the same
//! integer-decimal `compute`/`store` produces its bytes, with one extra rule: a negative result that
//! truncates to zero keeps its sign (`-0`), unlike the DISPLAY path. **Fail closed** (typed
//! [`ArithError`]) when an operand or intermediate exceeds the `i128` integer-decimal range
//! (`>38` significant digits / product overflow) — those need the GMP-grade bignum, a future court
//! (`GNURUST.ARITH-BIGNUM.0`). `DIVIDE`, the other six rounding modes, and `ON SIZE ERROR` exception
//! semantics are deferred.

use crate::attr::{FieldAttr, COB_FLAG_HAVE_SIGN, COB_TYPE_NUMERIC_DISPLAY};
use crate::move_ops::cob_move;
use crate::sign;
use crate::value::Decimal;

/// A binary decimal operation (`f1 := f1 op f2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    /// `DIVIDE` (`GNURUST.19`): quotient into a **GIVING** receiver — use [`cob_divide`], not
    /// [`cob_arith`] (division needs an explicit receiving field + scale, not `f1 := f1 / f2`).
    Divide,
}

/// Rounding when storing a result narrower than the computed scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Round {
    /// Default (no `ROUNDED`): drop low digits toward zero (`COB_STORE_TRUNCATION`, also the
    /// explicit `ROUNDED MODE IS TRUNCATION`).
    Truncate,
    /// `ROUNDED` default mode: nearest, ties away from zero (`COB_STORE_NEAR_AWAY_FROM_ZERO`).
    NearAwayFromZero,
    /// `ROUNDED MODE IS AWAY-FROM-ZERO`: round magnitude up whenever any digit is dropped
    /// (`COB_STORE_AWAY_FROM_ZERO`).
    AwayFromZero,
    /// `ROUNDED MODE IS NEAREST-TOWARD-ZERO`: nearest, exact ties truncate toward zero
    /// (`COB_STORE_NEAR_TOWARD_ZERO`).
    NearTowardZero,
    /// `ROUNDED MODE IS NEAREST-EVEN` (banker's rounding): nearest, exact ties go to the even digit
    /// (`COB_STORE_NEAR_EVEN`).
    NearEven,
    /// `ROUNDED MODE IS TOWARD-GREATER` (ceiling): round toward +infinity
    /// (`COB_STORE_TOWARD_GREATER`).
    TowardGreater,
    /// `ROUNDED MODE IS TOWARD-LESSER` (floor): round toward -infinity (`COB_STORE_TOWARD_LESSER`).
    TowardLesser,
    /// `ROUNDED MODE IS PROHIBITED`: any dropped non-zero digit is a size error
    /// (`COB_STORE_PROHIBITED`); fail closed.
    Prohibited,
}

/// Why an operation could not be performed within the sealed (i128) range.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArithError {
    /// An operand or intermediate exceeds the i128 integer-decimal range (needs a bignum — future).
    OutOfRange,
    /// A field's attributes are self-inconsistent.
    InvalidAttr,
    /// Retained for API stability. Previously signalled that `ADD`/`SUBTRACT` into a PACKED field
    /// was deferred; that path is now sealed (`GNURUST.13`, libcob `cob_add_bcd`), so this is no
    /// longer produced.
    PackedAddSubDeferred,
    /// `DIVIDE` by zero (`GNURUST.19`): fail closed. `ON SIZE ERROR` exception semantics are a
    /// separate future court, so division by zero is rejected rather than signalled.
    DivideByZero,
    /// `ROUNDED MODE IS PROHIBITED` with a dropped non-zero digit: libcob raises
    /// `COB_EC_SIZE_TRUNCATION` (`cob_decimal_do_round` returns 1). Fail closed.
    RoundingProhibited,
}

impl core::fmt::Display for ArithError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ArithError::OutOfRange => write!(f, "operand/intermediate exceeds i128 decimal range (needs bignum; deferred)"),
            ArithError::InvalidAttr => write!(f, "invalid field attribute"),
            ArithError::PackedAddSubDeferred => write!(f, "ADD/SUBTRACT into a PACKED field uses libcob's cob_add_bcd path (deferred, GNURUST.ARITH-BCD.0)"),
            ArithError::DivideByZero => write!(f, "DIVIDE by zero (fail closed; ON SIZE ERROR is a future court)"),
            ArithError::RoundingProhibited => write!(f, "ROUNDED MODE IS PROHIBITED: dropped a non-zero digit (size error, fail closed)"),
        }
    }
}

impl std::error::Error for ArithError {}

/// A fixed-precision decimal: `value = mag * 10^(-scale)`. `mag` carries the sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Dec {
    mag: i128,
    scale: i32,
}

/// Decode a numeric field's value (via the sealed decoders) into a `Dec`.
fn decode(data: &[u8], attr: &FieldAttr) -> Result<Dec, ArithError> {
    // PACKED (COMP-3/COMP-6) value decode goes through the faithful cob_decimal_set_packed
    // (numeric.c:1144), which accumulates `val*100 + PACK_TO_BIN[byte]`. For valid BCD this equals the
    // nibble split (proven by set_packed_agrees_with_sealed_from_packed_decoder); for INVALID nibbles
    // (>9) the byte-table folding -- carries included, e.g. PACK_TO_BIN[0x2F]=25, the verbatim
    // GnuCOBOL quirk -- is what libcob computes, which a per-nibble `*10+nibble` cannot reproduce.
    if attr.field_type == crate::attr::COB_TYPE_NUMERIC_PACKED {
        let cd = crate::packed::cob_decimal_set_packed(data, attr);
        let mag = cd.value.to_i128().ok_or(ArithError::OutOfRange)?;
        return Ok(Dec { mag, scale: cd.scale });
    }
    let d: Decimal = match attr.field_type {
        crate::attr::COB_TYPE_NUMERIC_DISPLAY => Decimal::from_display(data, attr),
        _ => return Err(ArithError::InvalidAttr),
    };
    let mut mag: i128 = 0;
    for &digit in &d.digits {
        mag = mag.checked_mul(10).ok_or(ArithError::OutOfRange)?;
        mag = mag
            .checked_add(digit as i128)
            .ok_or(ArithError::OutOfRange)?;
    }
    if d.negative {
        mag = -mag;
    }
    Ok(Dec {
        mag,
        scale: d.scale as i32,
    })
}

/// 10^n as i128, or `None` on overflow.
fn pow10(n: u32) -> Option<i128> {
    let mut r: i128 = 1;
    for _ in 0..n {
        r = r.checked_mul(10)?;
    }
    Some(r)
}

/// Rescale `d` up to `target_scale >= d.scale` (append zeros), or error on overflow.
fn upscale(d: Dec, target_scale: i32) -> Result<i128, ArithError> {
    let n = (target_scale - d.scale) as u32;
    let f = pow10(n).ok_or(ArithError::OutOfRange)?;
    d.mag.checked_mul(f).ok_or(ArithError::OutOfRange)
}

/// `f1 op f2`, returning the result Dec (exact, no scale narrowing yet).
fn compute(a: Dec, b: Dec, op: Op) -> Result<Dec, ArithError> {
    match op {
        Op::Add | Op::Subtract => {
            let scale = a.scale.max(b.scale);
            let am = upscale(a, scale)?;
            let bm = upscale(b, scale)?;
            let mag = match op {
                Op::Add => am.checked_add(bm),
                _ => am.checked_sub(bm),
            }
            .ok_or(ArithError::OutOfRange)?;
            Ok(Dec { mag, scale })
        }
        Op::Multiply => {
            let mag = a.mag.checked_mul(b.mag).ok_or(ArithError::OutOfRange)?;
            Ok(Dec {
                mag,
                scale: a.scale + b.scale,
            })
        }
        // DIVIDE has no exact Dec (the quotient depends on the receiving scale) — use cob_divide.
        Op::Divide => Err(ArithError::InvalidAttr),
    }
}

/// `a / b` evaluated at exactly `result_scale` fractional digits, truncating toward zero (the COBOL
/// DIVIDE truncation). `result_scale = a/b * 10^result_scale = a.mag * 10^(result_scale+b.scale-a.scale)
/// / b.mag`. Fails closed on divide-by-zero (`GNURUST.19`; `ON SIZE ERROR` is a future court).
/// Returns `(quotient at result_scale, inexact)` where `inexact` is true iff the division dropped a
/// nonzero remainder below `result_scale`. The remainder is the sticky bit the rounding step needs: with
/// a single guard digit, the round digit alone cannot distinguish an exact half (e.g. `x.5`) from
/// `x.5…nonzero`, so `do_round` ORs `inexact` into its sticky/exact tests. GnuCOBOL gets this for free
/// because `cob_decimal_div` shifts the dividend by `COB_MAX_DIGITS` before `mpz_tdiv_q` (numeric.c:2260),
/// carrying full precision into `cob_decimal_get_field`'s rounder.
fn compute_divide(a: Dec, b: Dec, result_scale: i32) -> Result<(Dec, bool), ArithError> {
    if b.mag == 0 {
        return Err(ArithError::DivideByZero);
    }
    let k = result_scale + b.scale - a.scale;
    let (mag, inexact) = if k >= 0 {
        // Fast path: scale the dividend in i128. If 10^k or the product overflows i128 (the scaled
        // dividend exceeds ~38 digits, as GnuCOBOL's GMP shift to COB_MAX_DIGITS routinely produces,
        // numeric.c:2260), fall back to the arbitrary-precision Mpz divide so we never fail closed.
        match pow10(k as u32).and_then(|p| a.mag.checked_mul(p)) {
            Some(num) => (num / b.mag, num % b.mag != 0), // i128 division truncates toward zero
            None => divide_via_mpz(a.mag, k as u32, b.mag, true)?,
        }
    } else {
        match pow10((-k) as u32).and_then(|p| b.mag.checked_mul(p)) {
            Some(den) => (a.mag / den, a.mag % den != 0),
            None => divide_via_mpz(b.mag, (-k) as u32, a.mag, false)?,
        }
    };
    Ok((Dec { mag, scale: result_scale }, inexact))
}

/// Arbitrary-precision divide fallback for when the i128 operand scaling in [`compute_divide`] overflows
/// (the scaled dividend/divisor exceeds ~38 digits). Mirrors GnuCOBOL's GMP `cob_decimal_div`: the
/// quotient is truncated toward zero and the (nonzero) remainder is the sticky bit. The quotient itself
/// fits i128 whenever the receiving field does; if it does not, that is a genuine size overflow
/// (`OutOfRange`), the same boundary the fast path reports. `scale_num = true` -> `(op*10^pow)/other`
/// (k>=0); `false` -> `op/(other*10^pow)` (k<0, op = dividend, other = divisor).
fn divide_via_mpz(op: i128, pow: u32, other: i128, scale_num: bool) -> Result<(i128, bool), ArithError> {
    use crate::gmp::Mpz;
    let ten_pow = Mpz::ui_pow_ui(10, pow);
    let (num, den) = if scale_num {
        (Mpz::from_i128(op).mul(&ten_pow), Mpz::from_i128(other))
    } else {
        (Mpz::from_i128(op), Mpz::from_i128(other).mul(&ten_pow))
    };
    let (q, r) = num.tdiv_qr(&den);
    Ok((q.to_i128().ok_or(ArithError::OutOfRange)?, r.sgn() != 0))
}

/// `DIVIDE` quotient into a GIVING receiver (`GNURUST.19`): `receiver := lhs / rhs`, returning the
/// receiver's field bytes — matching libcob's `cob_div`/`cob_decimal_div` + store. Truncation toward
/// zero by default; `ROUNDED` is nearest-away-from-zero (one guard digit, computed then rounded by
/// the store path). Divide-by-zero fails closed. (For `DIVIDE a BY b GIVING c`, `lhs=a, rhs=b`; for
/// `DIVIDE a INTO b GIVING c`, `lhs=b, rhs=a`.) The `REMAINDER` receiver is a sealed sibling
/// (`GNURUST.REMAINDER.1`) — see [`cob_divide_remainder`]. **Non-claims:** `ON SIZE ERROR`, `COMPUTE`,
/// expression evaluation, binary/edited receivers, and business correctness.
pub fn cob_divide(
    lhs: &[u8],
    lhs_attr: &FieldAttr,
    rhs: &[u8],
    rhs_attr: &FieldAttr,
    recv_attr: &FieldAttr,
    round: Round,
) -> Result<Vec<u8>, ArithError> {
    let dl = decode(lhs, lhs_attr)?;
    let dr = decode(rhs, rhs_attr)?;
    // One guard digit for EVERY rounding mode (was only NEAREST-AWAY) plus the division's `inexact`
    // sticky bit — so do_round can distinguish an exact half from `…5…nonzero` for NEAREST-EVEN,
    // NEAREST-TOWARD-ZERO, PROHIBITED, AWAY-FROM-ZERO and TOWARD-GREATER/LESSER. TRUNCATION drops the
    // guard digit (unchanged); NEAREST-AWAY ignores the sticky (≥5 always rounds away) — both stay
    // byte-identical to the sealed divide_sweep.
    let (q, inexact) = compute_divide(dl, dr, recv_attr.scale as i32 + 1)?;
    store(q, recv_attr, round, false, inexact)
}

/// `DIVIDE` with a `REMAINDER` receiver (`GNURUST.REMAINDER.1`): for `DIVIDE a BY b GIVING q REMAINDER r`
/// (and the `INTO` form via `lhs`/`rhs` swap), returns `(quotient_bytes, remainder_bytes)`. The remainder
/// is the COBOL definition — **dividend − (quotient-as-stored × divisor)** — so it depends on the quotient
/// receiver's scale/truncation: the quotient is truncated toward zero to `quot_attr.scale` (the `REMAINDER`
/// forms use the **un-rounded** quotient), then `r = a − q·b` is computed exactly and stored (truncated)
/// into the remainder receiver. Divide-by-zero fails closed. **Non-claims:** `ON SIZE ERROR` / `NOT ON SIZE
/// ERROR` control flow, `COMPUTE`, expression evaluation, Procedure Division execution, float, binary/edited
/// receivers, and business correctness — the remainder's sign/scale are not inferred, they are witnessed.
pub fn cob_divide_remainder(
    lhs: &[u8],
    lhs_attr: &FieldAttr,
    rhs: &[u8],
    rhs_attr: &FieldAttr,
    quot_attr: &FieldAttr,
    rem_attr: &FieldAttr,
) -> Result<(Vec<u8>, Vec<u8>), ArithError> {
    let dl = decode(lhs, lhs_attr)?;
    let dr = decode(rhs, rhs_attr)?;
    // quotient truncated toward zero to the quotient receiver's scale (the value as stored in GIVING);
    // REMAINDER uses the un-rounded quotient, so the division's inexactness is irrelevant here.
    let (q, _inexact) = compute_divide(dl, dr, quot_attr.scale as i32)?;
    // remainder = dividend − (stored quotient × divisor), exact, then truncated into the remainder receiver
    let qd = compute(q, dr, Op::Multiply)?;
    let r = compute(dl, qd, Op::Subtract)?;
    let quot_bytes = store(q, quot_attr, Round::Truncate, false, false)?;
    let rem_bytes = store(r, rem_attr, Round::Truncate, false, false)?;
    Ok((quot_bytes, rem_bytes))
}

/// Truncating division of `mag` by 10^k, toward zero (`cob_div_by_pow_10`).
fn tdiv_pow10(mag: i128, k: u32) -> Result<i128, ArithError> {
    let f = pow10(k).ok_or(ArithError::OutOfRange)?;
    Ok(mag / f) // i128 division truncates toward zero
}

/// Truncated remainder of `mag` by 10^k (`mpz_tdiv_r`): the dropped low-`k`-digit part, with the
/// sign of `mag`. Used by the rounding modes to decide whether the value is exact.
fn trem_pow10(mag: i128, k: u32) -> Result<i128, ArithError> {
    let f = pow10(k).ok_or(ArithError::OutOfRange)?;
    Ok(mag % f) // i128 remainder follows the dividend's sign, like mpz_tdiv_r
}

/// `5 * 10^k`, checked.
fn five_pow10(k: u32) -> Result<i128, ArithError> {
    pow10(k)
        .and_then(|p| p.checked_mul(5))
        .ok_or(ArithError::OutOfRange)
}

/// Full 256-bit product of two `u128` (schoolbook over 64-bit halves). Returns `(hi, lo)` with
/// `a*b = hi*2^128 + lo`. Used by the bignum MULTIPLY fallback (GNURUST.BIGNUM.1) when the i128
/// product overflows; libcob computes the exact product in GMP.
fn mul_u256(a: u128, b: u128) -> (u128, u128) {
    let (a0, a1) = (a & u64::MAX as u128, a >> 64);
    let (b0, b1) = (b & u64::MAX as u128, b >> 64);
    let p00 = a0 * b0;
    let p01 = a0 * b1;
    let p10 = a1 * b0;
    let p11 = a1 * b1;
    let (mid, carry1) = p01.overflowing_add(p10); // mid = a0*b1 + a1*b0 (may carry into bit 192)
    let (lo, carry2) = p00.overflowing_add(mid << 64);
    // hi = p11 + (mid >> 64) + carry2 + (carry1 << 64); the true product's hi 128 bits, so it fits.
    let hi = p11
        .wrapping_add(mid >> 64)
        .wrapping_add(carry2 as u128)
        .wrapping_add((carry1 as u128) << 64);
    (hi, lo)
}

/// `(hi*2^128 + lo) / d` and remainder, by binary long division. `d` is a power of ten <= 10^18
/// (< 2^60), so `rem << 1` never overflows `u128`. Returns `(q_hi, q_lo, rem)`.
fn u256_divmod_u128(hi: u128, lo: u128, d: u128) -> (u128, u128, u128) {
    let mut rem: u128 = 0;
    let (mut q_hi, mut q_lo): (u128, u128) = (0, 0);
    let mut i = 256;
    while i > 0 {
        i -= 1;
        q_hi = (q_hi << 1) | (q_lo >> 127);
        q_lo <<= 1;
        let bit = if i >= 128 { (hi >> (i - 128)) & 1 } else { (lo >> i) & 1 };
        rem = (rem << 1) | bit;
        if rem >= d {
            rem -= d;
            q_lo |= 1;
        }
    }
    (q_hi, q_lo, rem)
}

/// The exact decimal digits of a 256-bit magnitude, most-significant first (no leading zeros; `0`
/// renders as `[0]`). Peels 18 digits at a time via [`u256_divmod_u128`].
fn u256_to_decimal(mut hi: u128, mut lo: u128) -> Vec<u8> {
    if hi == 0 && lo == 0 {
        return vec![0];
    }
    const BASE: u128 = 1_000_000_000_000_000_000; // 10^18
    let mut chunks: Vec<u64> = Vec::new(); // 18-digit chunks, least-significant first
    while hi != 0 || lo != 0 {
        let (qh, ql, r) = u256_divmod_u128(hi, lo, BASE);
        chunks.push(r as u64);
        hi = qh;
        lo = ql;
    }
    let mut out = Vec::new();
    for (idx, &c) in chunks.iter().rev().enumerate() {
        let s = if idx == 0 { format!("{c}") } else { format!("{c:018}") };
        out.extend(s.bytes().map(|b| b - b'0'));
    }
    out
}

/// Whether to round the magnitude up by one, given the first dropped digit, whether anything below
/// it is non-zero (sticky), and the last kept digit — a digit-form restatement of `do_round` (and
/// proven equal to it by the round sweep). PROHIBITED with a dropped non-zero digit is an error.
fn round_up_decimal(
    mode: Round,
    positive: bool,
    round_digit: u8,
    sticky: bool,
    last_keep: u8,
) -> Result<bool, ArithError> {
    let any = round_digit != 0 || sticky;
    Ok(match mode {
        Round::Truncate => false,
        Round::Prohibited => {
            if any {
                return Err(ArithError::RoundingProhibited);
            }
            false
        }
        Round::AwayFromZero => any,
        Round::TowardGreater => positive && any,
        Round::TowardLesser => !positive && any,
        Round::NearAwayFromZero => round_digit >= 5,
        Round::NearTowardZero => round_digit > 5 || (round_digit == 5 && sticky),
        Round::NearEven => {
            round_digit > 5 || (round_digit == 5 && (sticky || last_keep % 2 == 1))
        }
    })
}

/// Increment a most-significant-first decimal digit vector by one, propagating the carry.
fn decimal_inc(d: &mut Vec<u8>) {
    let mut i = d.len();
    loop {
        if i == 0 {
            d.insert(0, 1);
            return;
        }
        i -= 1;
        if d[i] == 9 {
            d[i] = 0;
        } else {
            d[i] += 1;
            return;
        }
    }
}

/// MULTIPLY when `a.mag * b.mag` overflows i128. libcob keeps the exact product in GMP; gnucobol-rs
/// carries it as the full 256-bit product (sufficient: two <=38-digit operands give a <=76-digit
/// product < 2^256), converts to exact decimal, rounds to the receiver scale and truncates to its
/// low `digits` digits (all digit-array work, no big-int arithmetic), then hands the <=38-digit
/// result to the unchanged `store()`. No deferral within the binary-multiply domain (GNURUST.BIGNUM.1).
fn mul_store_big(
    a: Dec,
    b: Dec,
    recv: &FieldAttr,
    round: Round,
    bcd_path: bool,
) -> Result<Vec<u8>, ArithError> {
    let positive = (a.mag < 0) == (b.mag < 0);
    let (hi, lo) = mul_u256(a.mag.unsigned_abs(), b.mag.unsigned_abs());
    let digits = u256_to_decimal(hi, lo);
    let ps = a.scale + b.scale; // product scale
    let tr = recv.scale as i32; // receiver scale
    let eff = if bcd_path { bcd_round_mode(round) } else { round };

    // Round the exact decimal to `tr` fractional digits.
    let keep: Vec<u8> = if ps <= tr {
        // widen: append (tr - ps) zero fractional digits, no rounding.
        let mut k = digits;
        for _ in 0..(tr - ps) {
            k.push(0);
        }
        k
    } else {
        let drop = (ps - tr) as usize;
        let total = digits.len();
        let keep_count = total.saturating_sub(drop);
        let mut k = digits[..keep_count].to_vec();
        let round_digit = if keep_count < total { digits[keep_count] } else { 0 };
        let sticky = keep_count + 1 < total && digits[keep_count + 1..].iter().any(|&x| x != 0);
        let last_keep = k.last().copied().unwrap_or(0);
        if round_up_decimal(eff, positive, round_digit, sticky, last_keep)? {
            decimal_inc(&mut k);
        }
        k
    };

    // Low `recv.digits` digits -> i128 (<=38 digits, so < 10^38 < i128::MAX; the store re-applies the
    // modulus and renders). Building MSB->LSB never overflows because each prefix is < 10^38.
    let dg = recv.digits as usize;
    let start = keep.len().saturating_sub(dg);
    let mut mag: i128 = 0;
    for &d in &keep[start..] {
        mag = mag * 10 + d as i128;
    }
    let mag = if positive { mag } else { -mag };
    store(Dec { mag, scale: tr }, recv, Round::Truncate, bcd_path, false)
}

/// Faithful port of `cob_decimal_do_round` (numeric.c:1936): round `(mag, scale)` toward the target
/// `tgt` scale per `round`, returning the adjusted `(mag, scale)`. The caller then truncates to the
/// field scale (matching libcob's post-round `shift_decimal` to `COB_FIELD_SCALE`). Only invoked
/// when the value is non-zero and actually narrows (`tgt < scale`); other cases are a no-op upstream.
fn do_round(mag: i128, scale: i32, tgt: i32, round: Round, inexact: bool) -> Result<(i128, i32), ArithError> {
    // `inexact` (a divide's dropped remainder below `scale`) is a sticky bit: it makes a value that
    // *looks* exact at `scale` actually non-exact. It ORs into the "any dropped digit" tests
    // (AWAY/TOWARD-GREATER/LESSER/PROHIBITED) and breaks the exact-half tie tests (NEAR-EVEN,
    // NEAR-TOWARD-ZERO). NEAREST-AWAY and TRUNCATION never consult it, so `inexact: false` (every
    // non-divide caller) reproduces the original do_round byte-for-byte.
    let sign: i128 = if mag > 0 { 1 } else { -1 };
    match round {
        // COB_STORE_TRUNCATION: drop the low digits (handled by the caller's adjust step).
        Round::Truncate => Ok((mag, scale)),
        // COB_STORE_PROHIBITED: a dropped non-zero digit is a size error.
        Round::Prohibited => {
            if trem_pow10(mag, (scale - tgt) as u32)? != 0 || inexact {
                Err(ArithError::RoundingProhibited)
            } else {
                Ok((mag, scale))
            }
        }
        // COB_STORE_AWAY_FROM_ZERO: if inexact, push the magnitude past the boundary.
        Round::AwayFromZero => {
            let divisor = pow10((scale - tgt) as u32).ok_or(ArithError::OutOfRange)?;
            let mag = if mag % divisor != 0 || inexact {
                mag.checked_add(sign * divisor).ok_or(ArithError::OutOfRange)?
            } else {
                mag
            };
            Ok((mag, scale))
        }
        // COB_STORE_TOWARD_GREATER (ceiling): only positive inexact values move up.
        Round::TowardGreater => {
            let divisor = pow10((scale - tgt) as u32).ok_or(ArithError::OutOfRange)?;
            let mag = if (mag % divisor != 0 || inexact) && sign == 1 {
                mag.checked_add(divisor).ok_or(ArithError::OutOfRange)?
            } else {
                mag
            };
            Ok((mag, scale))
        }
        // COB_STORE_TOWARD_LESSER (floor): only negative inexact values move down.
        Round::TowardLesser => {
            let divisor = pow10((scale - tgt) as u32).ok_or(ArithError::OutOfRange)?;
            let mag = if (mag % divisor != 0 || inexact) && sign == -1 {
                mag.checked_sub(divisor).ok_or(ArithError::OutOfRange)?
            } else {
                mag
            };
            Ok((mag, scale))
        }
        // COB_STORE_NEAR_TOWARD_ZERO: nearest, exact ties truncate toward zero. libcob's `exact`
        // test is `value mod (5*10^(cur-tgt-1)) == 0` (true for both an exact value and an exact
        // half), computed on the value *before* the shift. A divide's sticky remainder breaks the tie.
        Round::NearTowardZero => {
            let exact = !inexact && mag % five_pow10((scale - tgt - 1) as u32)? == 0;
            let k = (scale - tgt - 1) as u32;
            let mut mag = if k > 0 { tdiv_pow10(mag, k)? } else { mag };
            let scale = tgt + 1;
            if !exact {
                mag = mag.checked_add(sign * 5).ok_or(ArithError::OutOfRange)?;
            }
            Ok((mag, scale))
        }
        // COB_STORE_NEAR_EVEN (banker's): nearest, exact ties go to the even kept digit. Same
        // `exact` test, then the kept (post-shift) digit pair {05,25,45,65,85} = even kept digit.
        Round::NearEven => {
            let exact = !inexact && mag % five_pow10((scale - tgt - 1) as u32)? == 0;
            let k = (scale - tgt - 1) as u32;
            let mut mag = if k > 0 { tdiv_pow10(mag, k)? } else { mag };
            let scale = tgt + 1;
            // On an exact tie, only round up when the kept digit is odd (so the result lands even).
            let round_up = if exact {
                let last_two = (mag % 100).unsigned_abs(); // |value| mod 100, like mpz_tdiv_ui
                !matches!(last_two, 5 | 25 | 45 | 65 | 85)
            } else {
                true
            };
            if round_up {
                mag = mag.checked_add(sign * 5).ok_or(ArithError::OutOfRange)?;
            }
            Ok((mag, scale))
        }
        // COB_STORE_NEAR_AWAY_FROM_ZERO (default ROUNDED): nearest, ties away from zero.
        Round::NearAwayFromZero => {
            let k = (scale - tgt - 1) as u32;
            let mut mag = if k > 0 { tdiv_pow10(mag, k)? } else { mag };
            let scale = tgt + 1;
            mag = mag.checked_add(sign * 5).ok_or(ArithError::OutOfRange)?;
            Ok((mag, scale))
        }
    }
}

/// Store `d` into the field `attr` with rounding `round`, returning the field bytes. Mirrors
/// `cob_decimal_get_field` (`numeric.c:2055`): round (if requested) then truncate/append to the
/// field scale, truncate overflow digits, render a DISPLAY temp, and `cob_move` to the target type.
/// Remap a rounding mode for the packed `cob_add_bcd` path (numeric.c:2826+, GNURUST.ROUND.2). That
/// nibble-level rounding matches `cob_decimal_do_round` for every mode except NEAREST-EVEN, which it
/// resolves away from zero (it does *not* round ties to even); so on this path NEAREST-EVEN behaves
/// as NEAREST-AWAY-FROM-ZERO. Proven by `round_sweep` over packed receivers.
fn bcd_round_mode(round: Round) -> Round {
    match round {
        Round::NearEven => Round::NearAwayFromZero,
        other => other,
    }
}

fn store(
    d: Dec,
    attr: &FieldAttr,
    round: Round,
    bcd_path: bool,
    inexact: bool,
) -> Result<Vec<u8>, ArithError> {
    let target_scale = attr.scale as i32;
    let target_digits = attr.digits as usize;
    // The computed value's sign, before any scale truncation. libcob's `cob_add_bcd` (packed
    // ADD/SUBTRACT) keeps this sign even when the result truncates to zero magnitude
    // (`-0.6` -> `-0`); the `cob_decimal`/DISPLAY path does not (it yields `+0`).
    let pre_negative = d.mag < 0;
    let mut mag = d.mag;
    let mut scale = d.scale;

    // ROUNDED (cob_decimal_do_round, numeric.c:1936). Only narrowing a non-zero value rounds; the
    // mode dispatch lives in `do_round`. TRUNCATION is a no-op here (the adjust step below drops the
    // low digits toward zero, matching COB_STORE_TRUNCATION). On the packed ADD/SUBTRACT cob_add_bcd
    // path the rounding mode is remapped (GNURUST.ROUND.2): that nibble-rounding (numeric.c:2826+)
    // diverges from cob_decimal only at NEAREST-EVEN, which it resolves away-from-zero (no to-even).
    if mag != 0 && target_scale < scale {
        let eff = if bcd_path { bcd_round_mode(round) } else { round };
        let (m, s) = do_round(mag, scale, target_scale, eff, inexact)?;
        mag = m;
        scale = s;
    }

    // Adjust to the field scale (truncating shift for narrowing; append zeros for widening).
    if scale != target_scale {
        if target_scale > scale {
            let f = pow10((target_scale - scale) as u32).ok_or(ArithError::OutOfRange)?;
            mag = mag.checked_mul(f).ok_or(ArithError::OutOfRange)?;
        } else {
            mag = tdiv_pow10(mag, (scale - target_scale) as u32)?;
        }
    }

    // Overflow: keep the low `target_digits` digits (TRUNC_ON_OVERFLOW default; SIZE error is a
    // future court). magnitude is at the field scale now.
    let negative = if mag != 0 {
        mag < 0
    } else {
        // zero magnitude: keep the pre-truncation sign only on the cob_add_bcd path.
        bcd_path && pre_negative
    };
    let mut abs = mag.unsigned_abs();
    let modulus = pow10(target_digits as u32).ok_or(ArithError::OutOfRange)? as u128;
    abs %= modulus;

    // Render as `target_digits` zero-padded decimal digits into a DISPLAY temp, then cob_move.
    let mut digits = vec![0u8; target_digits];
    let mut v = abs;
    for slot in digits.iter_mut().rev() {
        *slot = (v % 10) as u8;
        v /= 10;
    }
    let mut temp: Vec<u8> = digits.iter().map(|&d| sign::i2d(d)).collect();
    let signed = attr.have_sign();
    let temp_attr = FieldAttr {
        field_type: COB_TYPE_NUMERIC_DISPLAY,
        digits: attr.digits,
        scale: attr.scale,
        flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
    };
    // The sign follows the value's sign *before* overflow truncation, so an overflowed negative
    // result stores negative zero (e.g. -40 into 1 digit -> -0 = 0x70), matching libcob.
    if signed && negative {
        if let Some(last) = temp.last_mut() {
            *last = sign::put_sign_ascii(*last);
        }
    }

    let mut out = vec![0u8; field_storage_size(attr)];
    cob_move(&temp, &temp_attr, &mut out, attr).map_err(|_| ArithError::InvalidAttr)?;
    Ok(out)
}

/// Storage byte size implied by an attribute (DISPLAY = digits[+sep]; PACKED = digits/2+1).
fn field_storage_size(attr: &FieldAttr) -> usize {
    match attr.field_type {
        crate::attr::COB_TYPE_NUMERIC_PACKED => attr.digits as usize / 2 + 1,
        _ => attr.digits as usize + if attr.sign_separate() { 1 } else { 0 },
    }
}

/// `f1 := f1 (op) f2`, with rounding `round`, returning the bytes of `f1`'s field — matching
/// libcob `cob_add`/`cob_sub`/`cob_mul`.
pub fn cob_arith(
    op: Op,
    a: &[u8],
    a_attr: &FieldAttr,
    b: &[u8],
    b_attr: &FieldAttr,
    round: Round,
) -> Result<Vec<u8>, ArithError> {
    // ADD/SUBTRACT into a PACKED receiving field take libcob's separate cob_add_bcd path
    // (`GNURUST.13`). It computes the exact integer-decimal sum, aligns to the receiver scale,
    // rounds (nearest-away when requested) or truncates, truncates overflow, and stores — which is
    // exactly the integer-decimal path here, so the same `compute`/`store` produces identical bytes.
    let da = decode(a, a_attr)?;
    let db = decode(b, b_attr)?;
    // ADD/SUBTRACT into a PACKED receiver take libcob's cob_add_bcd path: it keeps a negative sign on
    // a zero-magnitude result AND rounds with the BCD nibble rules (NEAREST-EVEN away, GNURUST.ROUND.2).
    let bcd_path = matches!(op, Op::Add | Op::Subtract)
        && a_attr.field_type == crate::attr::COB_TYPE_NUMERIC_PACKED;
    match compute(da, db, op) {
        // ADD/SUBTRACT/MULTIPLY: the exact result lives in `r` (any dropped digits are already in its
        // magnitude), so there is no division-style sticky bit — pass `inexact: false`.
        Ok(r) => store(r, a_attr, round, bcd_path, false),
        // A MULTIPLY whose i128 product overflows: carry the exact 256-bit product (GNURUST.BIGNUM.1)
        // instead of failing closed, matching libcob's GMP product + low-digit truncating store.
        Err(ArithError::OutOfRange) if matches!(op, Op::Multiply) => {
            mul_store_big(da, db, a_attr, round, bcd_path)
        }
        // An ADD/SUBTRACT whose i128 operand alignment overflows (a >38-digit aligned intermediate, as
        // GnuCOBOL's GMP align_decimal produces): fall back to the arbitrary-precision Mpz cob_decimal
        // path (cob_add/cob_sub, the verified structural-1:1 layer == this i128 path for in-range), so
        // we never fail closed. The result still truncates to the receiver at store.
        Err(ArithError::OutOfRange) if matches!(op, Op::Add | Op::Subtract) => {
            let res = if matches!(op, Op::Add) {
                crate::cob_decimal::cob_add(a, a_attr, b, b_attr, round)
            } else {
                crate::cob_decimal::cob_sub(a, a_attr, b, b_attr, round)
            };
            res.map_err(|_| ArithError::OutOfRange)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::COB_TYPE_NUMERIC_PACKED;

    fn disp(d: u16, s: i16, signed: bool) -> FieldAttr {
        FieldAttr {
            field_type: COB_TYPE_NUMERIC_DISPLAY,
            digits: d,
            scale: s,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        }
    }
    fn packed(d: u16, s: i16, signed: bool) -> FieldAttr {
        FieldAttr {
            field_type: COB_TYPE_NUMERIC_PACKED,
            digits: d,
            scale: s,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        }
    }

    /// Store `mag * 10^-scale` into a signed S9(6) DISPLAY field with `mode`, then decode the stored
    /// integer back. Exercises the full do_round + scale-adjust + cob_move path (GNURUST.ROUND.1).
    fn round_int(mag: i128, scale: i32, mode: Round) -> Result<i128, ArithError> {
        let attr = disp(6, 0, true);
        let bytes = store(Dec { mag, scale }, &attr, mode, false, false)?;
        Ok(decode(&bytes, &attr)?.mag)
    }

    #[test]
    fn rounding_modes_exact_half() {
        use Round::*;
        // 2.5: the modes diverge exactly at the tie.
        assert_eq!(round_int(25, 1, NearAwayFromZero).unwrap(), 3);
        assert_eq!(round_int(25, 1, NearTowardZero).unwrap(), 2);
        assert_eq!(round_int(25, 1, NearEven).unwrap(), 2); // 2 is even
        assert_eq!(round_int(25, 1, AwayFromZero).unwrap(), 3);
        assert_eq!(round_int(25, 1, TowardGreater).unwrap(), 3);
        assert_eq!(round_int(25, 1, TowardLesser).unwrap(), 2);
        assert_eq!(round_int(25, 1, Truncate).unwrap(), 2);
        // 3.5: nearest-even rounds UP to the even 4.
        assert_eq!(round_int(35, 1, NearEven).unwrap(), 4);
        assert_eq!(round_int(35, 1, NearAwayFromZero).unwrap(), 4);
        assert_eq!(round_int(35, 1, NearTowardZero).unwrap(), 3);
    }

    #[test]
    fn rounding_modes_signed_half() {
        use Round::*;
        // -2.5: ceiling/floor follow +/-infinity, not magnitude.
        assert_eq!(round_int(-25, 1, NearAwayFromZero).unwrap(), -3);
        assert_eq!(round_int(-25, 1, NearTowardZero).unwrap(), -2);
        assert_eq!(round_int(-25, 1, NearEven).unwrap(), -2);
        assert_eq!(round_int(-25, 1, TowardGreater).unwrap(), -2); // toward +inf
        assert_eq!(round_int(-25, 1, TowardLesser).unwrap(), -3); // toward -inf
        assert_eq!(round_int(-25, 1, AwayFromZero).unwrap(), -3);
    }

    #[test]
    fn rounding_modes_nonhalf_and_multidigit() {
        use Round::*;
        // below/above the half -> nearest is unambiguous.
        assert_eq!(round_int(24, 1, NearAwayFromZero).unwrap(), 2);
        assert_eq!(round_int(26, 1, NearAwayFromZero).unwrap(), 3);
        // any dropped fraction -> away/ceiling round the magnitude up; floor/truncate keep it.
        assert_eq!(round_int(24, 1, AwayFromZero).unwrap(), 3);
        assert_eq!(round_int(24, 1, TowardGreater).unwrap(), 3);
        assert_eq!(round_int(24, 1, TowardLesser).unwrap(), 2);
        assert_eq!(round_int(24, 1, Truncate).unwrap(), 2);
        // two dropped digits: 2.45 is below the half -> nearest 2, away 3.
        assert_eq!(round_int(245, 2, NearAwayFromZero).unwrap(), 2);
        assert_eq!(round_int(245, 2, AwayFromZero).unwrap(), 3);
        // an exact value never rounds, whatever the mode (and PROHIBITED accepts it).
        assert_eq!(round_int(20, 1, AwayFromZero).unwrap(), 2);
        assert_eq!(round_int(20, 1, Prohibited).unwrap(), 2);
    }

    #[test]
    fn bignum_multiply_overflow_matches_oracle() {
        // 9(20) * 9(20) = (10^20-1)^2 = 9999999999999999999800000000000000000001 (40 digits, overflows
        // i128); stored into a 20-digit receiver keeps the low 20 = 00000000000000000001 (oracle-verified).
        let nines = b"99999999999999999999";
        let attr = disp(20, 0, false);
        let r = cob_arith(Op::Multiply, nines, &attr, nines, &attr, Round::Truncate).unwrap();
        assert_eq!(&r, b"00000000000000000001");
    }

    #[test]
    fn bignum_multiply_signed_and_scaled() {
        // -9(18)V9 * 9(18)V9 style: ensure sign + scale narrowing work through the bignum path.
        // 12 nines . 1 digit each: magnitudes ~1.0e19, product ~1.2e38 (may overflow), narrowed.
        let a = b"9999999999999999999"; // 19 digits
        let aattr = disp(19, 0, true);
        let r = cob_arith(Op::Multiply, a, &aattr, a, &aattr, Round::Truncate);
        // 19-nine squared = 39 digits, overflows i128 -> bignum path; low 19 digits stored.
        assert!(r.is_ok());
        // (10^19-1)^2 = ...80000000000000000001 ; low 19 = 0000000000000000001
        assert_eq!(r.unwrap().as_slice(), b"0000000000000000001");
    }

    #[test]
    fn mul_u256_full_range_incl_carry() {
        // 5 * 7 = 35
        assert_eq!(mul_u256(5, 7), (0, 35));
        // u128::MAX^2 = 2^256 - 2^129 + 1 -> hi = 2^128 - 2 = u128::MAX-1, lo = 1. Exercises the
        // bit-192 carry (carry1), which never fires for <=38-digit COBOL operands but must be correct.
        assert_eq!(mul_u256(u128::MAX, u128::MAX), (u128::MAX - 1, 1));
        // exact decimal of a known overflow product
        let (hi, lo) = mul_u256(10u128.pow(20) - 1, 10u128.pow(20) - 1);
        let d: String = u256_to_decimal(hi, lo).iter().map(|&x| (x + b'0') as char).collect();
        assert_eq!(d, "9999999999999999999800000000000000000001");
    }

    #[test]
    fn bignum_multiply_high_combined_scale_no_defer() {
        // 9(20) * V9(20) into 9(20): K = digits + drop = 20 + 20 = 40 (> 38, the case an early
        // mod-10^K reduction would defer). The full-product path handles it: (10^20-1)^2 / 10^20
        // truncated = 99999999999999999998 (oracle-verified).
        let nines = b"99999999999999999999";
        let recv = disp(20, 0, false);
        let frac = disp(20, 20, false); // V9(20): all 20 digits fractional
        let r = cob_arith(Op::Multiply, nines, &recv, nines, &frac, Round::Truncate).unwrap();
        assert_eq!(&r, b"99999999999999999998");
    }

    #[test]
    fn prohibited_inexact_is_size_error() {
        assert_eq!(round_int(25, 1, Round::Prohibited), Err(ArithError::RoundingProhibited));
    }

    #[test]
    fn add_display() {
        // 012.34 + 001.11 = 013.45  (S9(3)V99 display)
        let a = b"01234"; // 012.34
        let bb = b"00111"; // 001.11
        let r = cob_arith(
            Op::Add,
            a,
            &disp(5, 2, true),
            bb,
            &disp(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&r, b"01345");
    }

    #[test]
    fn multiply_truncates() {
        // 1.50 * 1.50 = 2.2500 -> stored in V99 truncates to 2.25
        let a = b"150"; // 1.50
        let bb = b"150";
        let r = cob_arith(
            Op::Multiply,
            a,
            &disp(3, 2, false),
            bb,
            &disp(3, 2, false),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&r, b"225"); // 2.25
    }

    #[test]
    fn rounded_half_away() {
        // 1.50 * 1.55 = 2.3250 ; V99 rounded -> 2.33 (half away)
        let r = cob_arith(
            Op::Multiply,
            b"150",
            &disp(3, 2, false),
            b"155",
            &disp(3, 2, false),
            Round::NearAwayFromZero,
        )
        .unwrap();
        assert_eq!(&r, b"233");
    }

    #[test]
    fn packed_add_sub_via_bcd() {
        // GNURUST.13: ADD/SUBTRACT into a PACKED field (libcob cob_add_bcd path).
        // -012.34 + -001.00 = -013.34 in S9(3)V99 COMP-3.
        let a = [0x01, 0x23, 0x4d];
        let b = [0x00, 0x10, 0x0d];
        let r = cob_arith(
            Op::Add,
            &a,
            &packed(5, 2, true),
            &b,
            &packed(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(r, vec![0x01, 0x33, 0x4d]); // -013.34

        // 012.34 + 001.11 = 013.45 (unsigned-ish positive), truncation.
        let r2 = cob_arith(
            Op::Add,
            &[0x01, 0x23, 0x4c],
            &packed(5, 2, true),
            &[0x00, 0x11, 0x1c],
            &packed(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(r2, vec![0x01, 0x34, 0x5c]); // 013.45
    }

    #[test]
    fn packed_multiply_ok() {
        // PACKED multiply uses the cob_decimal path (sealed): 1.50 * 2.00 = 3.00 in S9(3)V99 COMP-3
        let a = [0x00, 0x15, 0x0c]; // 001.50
        let b = [0x00, 0x20, 0x0c]; // 002.00
        let r = cob_arith(
            Op::Multiply,
            &a,
            &packed(5, 2, true),
            &b,
            &packed(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(r, vec![0x00, 0x30, 0x0c]); // 003.00
    }

    #[test]
    fn divide_truncates() {
        // 10.00 / 3.00 = 3.333... -> S9(5)V99 truncate = 3.33
        let r = cob_divide(
            b"0001000",
            &disp(7, 2, false),
            b"0000300",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&r, b"0000333");
    }

    #[test]
    fn add_intermediate_over_i128_vs_cobc() {
        // ADD 1.5 TO A where A = 2e20 (PIC 9(21)): aligning A to the addend's scale 18 needs
        // 2e20 * 10^18 = 2e38, which overflows i128 (max ~1.7e38). The old fast path failed closed; the
        // Mpz fallback computes it. Receiver is scale 0, so the .5 truncates. Oracle (built GnuCOBOL):
        // 200000000000000000001.
        let a = disp(21, 0, false);
        let b = disp(19, 18, false); // PIC 9V9(18) holding 1.5 -> "1" + "5" + 17 zeros
        let res = cob_arith(
            Op::Add,
            b"200000000000000000000",
            &a,
            b"1500000000000000000",
            &b,
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&res, b"200000000000000000001", "ADD with >38-digit aligned intermediate must not fail closed");
    }

    #[test]
    fn divide_intermediate_over_i128_vs_cobc() {
        // DIVIDE 2 BY 3 into a scale-37 receiver. The i128-scaled dividend (2 * 10^38) overflows i128,
        // so the old fast path failed closed (OutOfRange); the Mpz fallback computes it. The quotient
        // (6.6e37) still fits i128. Ground truth from the built GnuCOBOL oracle, PIC 9V9(37):
        // trunc = 0.(37 sixes), ROUNDED = 0.(36 sixes)7.
        let small = disp(1, 0, false);
        let recv = disp(38, 37, false);
        let mut want_t = vec![b'0'];
        want_t.extend(std::iter::repeat(b'6').take(37));
        let trunc = cob_divide(b"2", &small, b"3", &small, &recv, Round::Truncate).unwrap();
        assert_eq!(trunc, want_t, "2/3 scale-37 truncate must not fail closed");
        let mut want_r = vec![b'0'];
        want_r.extend(std::iter::repeat(b'6').take(36));
        want_r.push(b'7');
        let round = cob_divide(b"2", &small, b"3", &small, &recv, Round::NearAwayFromZero).unwrap();
        assert_eq!(round, want_r, "2/3 scale-37 rounded");
    }

    #[test]
    fn packed_invalid_nibble_arith_vs_cobc() {
        // A COMP-3 field with an INVALID digit nibble (>9). The arithmetic decode must fold the byte
        // through PACK_TO_BIN (cob_decimal_set_packed, numeric.c:1144), NOT split nibbles. Ground truth
        // from the built GnuCOBOL oracle: PIC S9(3) COMP-3 = X"2F6C", `COMPUTE acc = P3 + 0` -> 256
        // (PACK_TO_BIN[0x2F]=25 -> 25*10+6=256), where a per-nibble split would give 356.
        let p3 = packed(3, 0, true);
        let r = cob_arith(Op::Add, b"0000000", &disp(7, 0, false), &[0x2F, 0x6C], &p3, Round::Truncate).unwrap();
        assert_eq!(&r, b"0000256", "0x2F6C COMP-3 arith must fold via PACK_TO_BIN -> 256 (cobc), not nibble-split 356");
    }

    #[test]
    fn divide_rounded_all_modes_vs_cobc() {
        // Ground truth captured from the built GnuCOBOL 3.2 oracle: `DIVIDE x BY y GIVING z ROUNDED
        // MODE IS <mode>` for tie/inexact quotients into a scale-0 receiver. Before the fix, the
        // non-NEAREST-AWAY modes got no guard digit and the divide's sticky remainder was discarded,
        // so ev_35/aw_35/tg_35/ev_2501/nt_2501 were wrong (3 instead of 4, 2 instead of 3).
        let n4 = disp(4, 0, false);
        let n6 = disp(6, 0, false);
        let div = |a: &[u8], aa: &FieldAttr, b: &[u8], ba: &FieldAttr, m: Round| {
            cob_divide(a, aa, b, ba, &n6, m).unwrap()
        };
        // 35 / 10 = 3.5 (exact tie) -> scale-0
        assert_eq!(&div(b"0035", &n4, b"0010", &n4, Round::NearEven), b"000004", "ev_35: 3.5 -> even 4");
        assert_eq!(&div(b"0035", &n4, b"0010", &n4, Round::NearAwayFromZero), b"000004", "na_35");
        assert_eq!(&div(b"0035", &n4, b"0010", &n4, Round::NearTowardZero), b"000003", "nt_35: tie toward zero");
        assert_eq!(&div(b"0035", &n4, b"0010", &n4, Round::AwayFromZero), b"000004", "aw_35: inexact -> away");
        assert_eq!(&div(b"0035", &n4, b"0010", &n4, Round::TowardGreater), b"000004", "tg_35: ceiling");
        assert_eq!(&div(b"0035", &n4, b"0010", &n4, Round::TowardLesser), b"000003", "tl_35: floor");
        // 2501 / 1000 = 2.501 (above the half, sticky) -> scale-0
        assert_eq!(&div(b"002501", &n6, b"001000", &n6, Round::NearEven), b"000003", "ev_2501: >2.5 -> 3");
        assert_eq!(&div(b"002501", &n6, b"001000", &n6, Round::NearTowardZero), b"000003", "nt_2501: >2.5 -> 3");
    }

    #[test]
    fn divide_rounded_half_away() {
        // 20.00 / 3.00 = 6.666... -> trunc 6.66 ; ROUNDED 6.67
        let t = cob_divide(
            b"0002000",
            &disp(7, 2, false),
            b"0000300",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&t, b"0000666");
        let rr = cob_divide(
            b"0002000",
            &disp(7, 2, false),
            b"0000300",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::NearAwayFromZero,
        )
        .unwrap();
        assert_eq!(&rr, b"0000667");
    }

    #[test]
    fn divide_negative_truncates_toward_zero() {
        // -10.00 / 3.00 = -3.33 (truncate toward zero, not -3.34)
        let mut a = b"0001000".to_vec();
        *a.last_mut().unwrap() = 0x70; // -10.00 (last '0' -> negative overpunch)
        let r = cob_divide(
            &a,
            &disp(7, 2, true),
            b"0000300",
            &disp(7, 2, false),
            &disp(7, 2, true),
            Round::Truncate,
        )
        .unwrap();
        let mut exp = b"0000333".to_vec();
        *exp.last_mut().unwrap() = 0x73; // -3.33 ('3' negative overpunch)
        assert_eq!(r, exp);
    }

    #[test]
    fn divide_half_scale() {
        // 1.00 / 2.00 = 0.50
        let r = cob_divide(
            b"0000100",
            &disp(7, 2, false),
            b"0000200",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(&r, b"0000050");
    }

    #[test]
    fn divide_into_packed_receiver() {
        // DIVIDE into a COMP-3 receiver: 9.00 / 2.00 = 4.50 in S9(3)V99 COMP-3.
        let r = cob_divide(
            b"0000900",
            &disp(7, 2, false),
            b"0000200",
            &disp(7, 2, false),
            &packed(5, 2, true),
            Round::Truncate,
        )
        .unwrap();
        assert_eq!(r, vec![0x00, 0x45, 0x0c]); // 004.50
    }

    #[test]
    fn divide_by_zero_fails_closed() {
        let r = cob_divide(
            b"0001000",
            &disp(7, 2, false),
            b"0000000",
            &disp(7, 2, false),
            &disp(7, 2, false),
            Round::Truncate,
        );
        assert_eq!(r, Err(ArithError::DivideByZero));
    }

    // ---- GNURUST.REMAINDER.1: DIVIDE ... GIVING q REMAINDER r ----
    #[test]
    fn remainder_integer_quotient() {
        // 10.00 / 3.00 GIVING q[S9(5)] REMAINDER r[S9(5)V99] -> q=3, r=1.00
        let (q, r) = cob_divide_remainder(
            b"0001000",
            &disp(7, 2, false),
            b"0000300",
            &disp(7, 2, false),
            &disp(5, 0, false),
            &disp(7, 2, false),
        )
        .unwrap();
        assert_eq!(&q, b"00003");
        assert_eq!(&r, b"0000100");
    }

    #[test]
    fn remainder_scaled_quotient() {
        // 10.00 / 3.00 GIVING q[S9(5)V99] REMAINDER r[S9(5)V99] -> q=3.33, r=0.01
        let (q, r) = cob_divide_remainder(
            b"0001000",
            &disp(7, 2, false),
            b"0000300",
            &disp(7, 2, false),
            &disp(7, 2, false),
            &disp(7, 2, false),
        )
        .unwrap();
        assert_eq!(&q, b"0000333");
        assert_eq!(&r, b"0000001");
    }

    #[test]
    fn remainder_exact_is_zero() {
        // 10.00 / 5.00 GIVING q[S9(5)] REMAINDER r[S9(5)V99] -> q=2, r=0.00
        let (q, r) = cob_divide_remainder(
            b"0001000",
            &disp(7, 2, false),
            b"0000500",
            &disp(7, 2, false),
            &disp(5, 0, false),
            &disp(7, 2, false),
        )
        .unwrap();
        assert_eq!(&q, b"00002");
        assert_eq!(&r, b"0000000");
    }

    #[test]
    fn remainder_sign_follows_dividend() {
        // -10.00 / 3.00 GIVING q[S9(5)] REMAINDER r[S9(5)V99] -> q=-3, r=-1.00 (truncate toward zero)
        let mut a = b"0001000".to_vec();
        *a.last_mut().unwrap() = 0x70; // -10.00
        let (q, r) = cob_divide_remainder(
            &a,
            &disp(7, 2, true),
            b"0000300",
            &disp(7, 2, false),
            &disp(5, 0, true),
            &disp(7, 2, true),
        )
        .unwrap();
        let mut eq = b"00003".to_vec();
        *eq.last_mut().unwrap() = 0x73; // -3
        let mut er = b"0000100".to_vec();
        *er.last_mut().unwrap() = 0x70; // -1.00
        assert_eq!(q, eq);
        assert_eq!(r, er);
    }

    #[test]
    fn remainder_into_packed_receiver() {
        // 10.00 / 3.00 GIVING q[S9(5)] REMAINDER r[S9(3)V99 COMP-3] -> q=3, r=001.00 packed
        let (q, r) = cob_divide_remainder(
            b"0001000",
            &disp(7, 2, false),
            b"0000300",
            &disp(7, 2, false),
            &disp(5, 0, false),
            &packed(5, 2, true),
        )
        .unwrap();
        assert_eq!(&q, b"00003");
        assert_eq!(r, vec![0x00, 0x10, 0x0c]); // 001.00
    }

    #[test]
    fn remainder_divide_by_zero_fails_closed() {
        let e = cob_divide_remainder(
            b"0001000",
            &disp(7, 2, false),
            b"0000000",
            &disp(7, 2, false),
            &disp(5, 0, false),
            &disp(7, 2, false),
        );
        assert_eq!(e, Err(ArithError::DivideByZero));
    }

    #[test]
    fn out_of_range_fails_closed() {
        // 18-digit * 18-digit may overflow i128 (36 digits ok, but 38*? -> guard). Force overflow.
        let big = b"999999999999999999"; // 18 nines
        let r = cob_arith(
            Op::Multiply,
            big,
            &disp(18, 0, false),
            big,
            &disp(18, 0, false),
            Round::Truncate,
        );
        // 36-digit product fits i128; widening for storage into 18 digits truncates. Should be Ok.
        assert!(r.is_ok());
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.7, GNURUST.13, GNURUST.19, GNURUST.REMAINDER.1, GNURUST.ROUND.1, GNURUST.BIGNUM.1
    /// The 256-bit product helper (GNURUST.BIGNUM.1) is total over any operand pair — no panic/overflow.
    #[kani::proof]
    fn mul_u256_total() {
        let a: u128 = kani::any();
        let b: u128 = kani::any();
        let (_hi, _lo) = mul_u256(a, b);
    }
    /// Every ROUNDED MODE IS setting (GNURUST.ROUND.1) is total over a symbolic value: `do_round`
    /// returns Ok or a typed ArithError, never a panic/overflow (checked arithmetic + pow10 bounds).
    #[kani::proof]
    #[kani::unwind(8)]
    fn round_is_total() {
        let mag: i64 = kani::any();
        let scale: u8 = kani::any();
        let tgt: u8 = kani::any();
        let mode = match kani::any::<u8>() % 8 {
            0 => Round::Truncate,
            1 => Round::NearAwayFromZero,
            2 => Round::AwayFromZero,
            3 => Round::NearEven,
            4 => Round::NearTowardZero,
            5 => Round::TowardGreater,
            6 => Round::TowardLesser,
            _ => Round::Prohibited,
        };
        // do_round's precondition (set by its caller): non-zero value actually narrowing. Small scales
        // bound the pow10 loop; larger scales only widen the (checked) pow10 overflow -> Err path, no
        // new panic surface.
        kani::assume(mag != 0 && tgt < scale && scale < 6);
        let _ = do_round(mag as i128, scale as i32, tgt as i32, mode, false);
    }

    /// The arithmetic ops are total over symbolic operand bytes for fixed field attrs: Ok(bytes) or a typed
    /// ArithError, never a panic (incl. divide-by-zero, which fails closed).
    #[kani::proof]
    #[kani::unwind(8)]
    fn arith_is_total() {
        let a: [u8; 3] = kani::any();
        let b: [u8; 3] = kani::any();
        let fa = crate::pic::build_field("9(3)", crate::Usage::Display, false, false);
        let fr = crate::pic::build_field("9(5)", crate::Usage::Display, false, false);
        if let (Ok(fa), Ok(fr)) = (fa, fr) {
            let _ = cob_arith(Op::Add, &a, &fa.attr, &b, &fa.attr, Round::Truncate);
            let _ = cob_arith(Op::Subtract, &a, &fa.attr, &b, &fa.attr, Round::Truncate);
            let _ = cob_arith(Op::Multiply, &a, &fa.attr, &b, &fa.attr, Round::Truncate);
            let _ = cob_divide(&a, &fa.attr, &b, &fa.attr, &fr.attr, Round::Truncate);
        }
    }
}
