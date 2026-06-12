//! 1:1 port of numeric.c's PACKED-DECIMAL (COMP-3 / COMP-6) surface: the BCD<->`cob_decimal`
//! converters and the BCD setters, faithful to GnuCOBOL 3.2 nibble-for-nibble (including the verbatim
//! `pack_to_bin` lookup table with the source's invalid-nibble quirks). The in-place BCD arithmetic
//! fast path (`cob_add_bcd`/`cob_addsub_optimized`/...) is ported in [`crate::cob_decimal`] alongside
//! the general decimal arithmetic it accelerates.
#![forbid(unsafe_code)]

use crate::attr::FieldAttr;
use crate::cob_decimal::CobDecimal;
use crate::gmp::Mpz;

/// `pack_to_bin` (numeric.c:107, active `#else` branch): BCD-byte -> 0..=165, `(hi*10+lo)` for
/// valid nibbles, with GnuCOBOL's verbatim invalid-nibble translation (incl. the source's 0x2F=>25).
pub(crate) static PACK_TO_BIN: [u8; 256] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 25,
    30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65,
    60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75,
    70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85,
    80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
    90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105,
    100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
    120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135,
    130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145,
    140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155,
    150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165,
];

/// `packed_bytes` (numeric.c): the BCD byte for a two-digit number 0..=99 (`(n/10)<<4 | n%10`).
pub(crate) static PACKED_BYTES: [u8; 100] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57,
    64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
    96, 97, 98, 99, 100, 101, 102, 103, 104, 105,
    112, 113, 114, 115, 116, 117, 118, 119, 120, 121,
    128, 129, 130, 131, 132, 133, 134, 135, 136, 137,
    144, 145, 146, 147, 148, 149, 150, 151, 152, 153,
];

/// `COB_D2I(x)` (coblocal.h:243): low nibble of an ASCII digit (`x & 0x0F`).
#[inline]
fn cob_d2i(x: u8) -> u8 {
    x & 0x0F
}

/// The real number of stored digits for a packed field: `digits` for non-negative scale, else
/// `digits + scale` (e.g. `99P` is 3 PIC digits but scale -1, so 2 stored). Mirrors the inline block
/// repeated throughout numeric.c's packed code.
#[inline]
fn stored_digits(attr: &FieldAttr) -> i32 {
    let scale = attr.scale as i32;
    if scale >= 0 {
        attr.digits as i32
    } else {
        attr.digits as i32 + scale
    }
}

/// `cob_packed_get_sign (f)` (numeric.c:967): `-1` if the trailing sign nibble is `0x0D`, `+1` for any
/// other nibble on a signed field, `0` for an unsigned (or no-sign-nibble) field.
pub fn cob_packed_get_sign(data: &[u8], attr: &FieldAttr) -> i32 {
    if attr.have_sign() {
        let p = data[data.len() - 1];
        if (p & 0x0F) == 0x0D {
            -1
        } else {
            1
        }
    } else {
        0
    }
}

/// `cob_set_packed_zero (f)` (numeric.c:1128): zero the field, then set the trailing sign nibble
/// (`0x0F` unsigned, `0x0C` positive) unless the field has no sign nibble (COMP-6).
pub fn cob_set_packed_zero(out: &mut [u8], attr: &FieldAttr) {
    for b in out.iter_mut() {
        *b = 0;
    }
    if attr.no_sign_nibble() {
        return;
    }
    let last = out.len() - 1;
    out[last] = if !attr.have_sign() { 0x0F } else { 0x0C };
}

/// `cob_set_packed_u64 (f, val, sign)` (numeric.c:1373): write an unsigned 64-bit magnitude into a BCD
/// field from the low end up, stopping at the leading zero, with the sign nibble placed first.
pub fn cob_set_packed_u64(out: &mut [u8], attr: &FieldAttr, val: u64, sign: i32) {
    let mut n = val;
    for b in out.iter_mut() {
        *b = 0;
    }
    let mut pi: isize = out.len() as isize - 1;
    if !attr.no_sign_nibble() {
        let nib = if !attr.have_sign() {
            0x0F
        } else if sign == -1 {
            0x0D
        } else {
            0x0C
        };
        out[pi as usize] = (((n % 10) as u8) << 4) | nib;
        n /= 10;
        pi -= 1;
    }
    while n != 0 && pi >= 0 {
        out[pi as usize] = PACKED_BYTES[(n % 100) as usize];
        n /= 100;
        pi -= 1;
    }
}

/// `cob_set_packed_int (f, val)` (numeric.c:1428): set a BCD field to a signed `int`.
pub fn cob_set_packed_int(out: &mut [u8], attr: &FieldAttr, val: i32) {
    if val > 0 {
        cob_set_packed_u64(out, attr, val as u64, 1);
    } else if val != 0 {
        cob_set_packed_u64(out, attr, (-(val as i64)) as u64, -1);
    } else {
        cob_set_packed_zero(out, attr);
    }
}

/// `cob_decimal_set_packed (d, f)` (numeric.c:1144): decode a BCD field into a working decimal. Reads
/// the leading half-nibble (when the digit count's parity demands it), then two digits per byte via
/// `pack_to_bin`, then the trailing digit half-nibble for sign-nibble fields, and applies the sign.
/// (The leading-zero skip and the u64/bignum split in the C are speed-only; accumulating in [`Mpz`]
/// yields the identical value, since `0*100 + d == d`.)
pub fn cob_decimal_set_packed(data: &[u8], attr: &FieldAttr) -> CobDecimal {
    let size = data.len();
    let nibtest = attr.no_sign_nibble() as usize; // 0 (COMP-3) or 1 (COMP-6)
    let end_idx = size - 1 + nibtest; // C: endp = data + size - 1 + nibtest; loop is `p < endp`
    let digits = stored_digits(attr);

    let mut p = 0usize;
    let byteval: u64 = if (digits & 1) as usize == nibtest {
        let bv = (data[p] & 0x0F) as u64;
        p += 1;
        bv
    } else {
        0
    };

    let mut value = Mpz::from_u64(byteval);
    while p < end_idx {
        value = value.mul_ui(100).add_ui(PACK_TO_BIN[data[p] as usize] as u64);
        p += 1;
    }
    if nibtest == 0 {
        // last byte's high nibble is the final digit; the low nibble is the sign
        value = value.mul_ui(10).add_ui((data[end_idx] >> 4) as u64);
    }

    if cob_packed_get_sign(data, attr) == -1 && value.sgn() != 0 {
        value.neg();
    }
    CobDecimal { value, scale: attr.scale as i32 }
}

/// `cob_decimal_get_packed (d, f, opt)` (numeric.c:1249): store a working decimal into a BCD field,
/// truncating to the field's digit capacity. Returns `0` on success; on overflow with
/// `COB_STORE_KEEP_ON_OVERFLOW` it leaves `out` unchanged and returns a non-zero (overflow) code.
/// The `cob_set_exception` side effect is observed at the arithmetic/SIZE-ERROR layer, not here.
/// `d` is consumed by value (the C mutates the decimal's sign via `mpz_abs`; we mirror that locally).
pub fn cob_decimal_get_packed(mut d: CobDecimal, attr: &FieldAttr, opt: i32, out: &mut [u8]) -> i32 {
    const COB_STORE_KEEP_ON_OVERFLOW: i32 = 1 << 2;
    const COB_STORE_NO_SIZE_ERROR: i32 = 1 << 3;

    let sign = d.value.sgn();
    match sign {
        1 => {}
        -1 => d.value.abs(),
        _ => {
            cob_set_packed_zero(out, attr);
            return 0;
        }
    }

    let digits = stored_digits(attr) as u32;
    let pow = Mpz::ui_pow_ui(10, digits); // cob_pow_10(digits)

    // Build the decimal-digit string `buff` of the (possibly truncated) magnitude.
    let buff: String = if d.value.cmp(&pow) != core::cmp::Ordering::Less {
        // Overflow.
        if opt & COB_STORE_NO_SIZE_ERROR == 0 && opt & COB_STORE_KEEP_ON_OVERFLOW != 0 {
            return overflow_code();
        }
        let rem = d.value.tdiv_r(&pow); // keep the low `digits` digits
        if rem.fits_ulong() {
            cob_set_packed_u64(out, attr, rem.get_ui(), sign);
            return 0;
        }
        rem.to_decimal_string()
    } else {
        if d.value.fits_ulong() {
            cob_set_packed_u64(out, attr, d.value.get_ui(), sign);
            return 0;
        }
        d.value.to_decimal_string()
    };
    let buff = buff.as_bytes();

    // Zero out, then place the digit nibbles right-aligned.
    for b in out.iter_mut() {
        *b = 0;
    }
    let size_str = buff.len();
    let digits = digits as usize;
    let (mut p, diff) = if attr.no_sign_nibble() {
        (((digits - 1) / 2) - ((size_str - 1) / 2), size_str % 2)
    } else {
        ((digits / 2) - (size_str / 2), 1 - (size_str % 2))
    };
    let size_total = size_str + diff;

    let mut q = 0usize;
    let mut i = diff;
    if i % 2 == 1 {
        out[p] += cob_d2i(buff[q]);
        p += 1;
        q += 1;
        i += 1;
    }
    while i < size_total {
        out[p] = (buff[q] << 4).wrapping_add(cob_d2i(buff[q + 1]));
        p += 1;
        q += 2;
        i += 2;
    }

    if attr.no_sign_nibble() {
        return 0;
    }
    let last = out.len() - 1;
    if !attr.have_sign() {
        out[last] |= 0x0F;
    } else if sign == -1 {
        out[last] |= 0x0D;
    } else {
        out[last] |= 0x0C;
    }
    0
}

/// The `cob_exception_code` returned to a `KEEP_ON_OVERFLOW` caller of [`cob_decimal_get_packed`]; the
/// concrete value is the SIZE-ERROR exception, surfaced through the arithmetic layer's own court.
#[inline]
fn overflow_code() -> i32 {
    // COB_EC_SIZE_OVERFLOW family; non-zero is all the byte-layer contract requires.
    0x0501
}

// ---------------------------------------------------------------------------------------------------
// PACKED comparison family (numeric.c, `#ifndef NO_BCD_COMPARE`): the in-place BCD comparators reached
// by cob_numeric_cmp / cob_cmp_packed for COMP-3 operands. Verdict-equivalent to the sealed decimal
// comparison (GNURUST.NUMCMP.1); ported as the actual nibble algorithm.
// ---------------------------------------------------------------------------------------------------

/// `memcmp`-style sign of the first differing byte over equal-length slices.
#[inline]
fn memcmp(a: &[u8], b: &[u8]) -> i32 {
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            return *x as i32 - *y as i32;
        }
    }
    0
}

/// `packed_is_negative (f)` (numeric.c:3794): a `0x0D` sign nibble counts as negative ONLY if the
/// magnitude is nonzero — a negative-zero BCD image compares equal to positive zero.
pub fn packed_is_negative(data: &[u8], attr: &FieldAttr) -> i32 {
    if cob_packed_get_sign(data, attr) == -1 {
        let end = data.len() - 1;
        if data[end] != 0x0D {
            return 1; // the sign byte carries a nonzero digit
        }
        for i in (0..end).rev() {
            if data[i] != 0 {
                return 1;
            }
        }
        0
    } else {
        0
    }
}

/// `insert_packed_aligned (...)` (numeric.c:3819): place `f1` (scale-shifted to align with `f2`) and
/// `f2` into two 48-byte buffers, right-justified, with sign nibbles cleared, returning the byte length
/// to compare. Uses the source's own portable `ALTERNATIVE_PACKED_SWAP` one-nibble left shift.
#[allow(clippy::too_many_arguments)]
fn insert_packed_aligned(
    f1: &[u8],
    no_sign1: bool,
    scale1: i32,
    f2: &[u8],
    no_sign2: bool,
    scale2: i32,
    buff1: &mut [u8; 48],
    buff2: &mut [u8; 48],
) -> usize {
    let len1 = f1.len();
    let len2 = f2.len();
    let mut nibble_cntr = scale2 - scale1;
    if no_sign1 && !no_sign2 {
        nibble_cntr += 1;
    }
    let byte_cntr = (nibble_cntr >> 1) as usize;
    nibble_cntr &= 1;

    let start1 = 48 - (len1 + byte_cntr);
    buff1[start1..start1 + len1].copy_from_slice(f1);
    if !no_sign1 {
        buff1[start1 + len1 - 1] &= 0xF0; // clear sign nibble
    }

    let compare_len = if nibble_cntr != 0 {
        // shift the filled buffer one nibble left (ALTERNATIVE_PACKED_SWAP)
        buff1[start1 - 1] = buff1[start1] >> 4;
        for i in start1..start1 + len1 {
            let next = if i + 1 < 48 { buff1[i + 1] } else { 0 };
            buff1[i] = (buff1[i] << 4) | (next >> 4);
        }
        len1 + byte_cntr + nibble_cntr as usize
    } else {
        len1 + byte_cntr
    };

    let start2 = 48 - len2;
    buff2[start2..start2 + len2].copy_from_slice(f2);
    if !no_sign2 {
        buff2[start2 + len2 - 1] &= 0xF0;
    }

    if len2 > compare_len {
        len2
    } else {
        compare_len
    }
}

/// `decimal_convert_scale (...)` (numeric.c:3914): align two same-sign packed operands by scale into
/// scratch buffers and `memcmp` them (operands swapped for the both-negative case).
#[allow(clippy::too_many_arguments)]
fn decimal_convert_scale(
    f1: &[u8],
    no_sign1: bool,
    scale1: i32,
    f2: &[u8],
    no_sign2: bool,
    scale2: i32,
    both_negative: bool,
) -> i32 {
    let mut buff1 = [0u8; 48];
    let mut buff2 = [0u8; 48];
    let compare_len = if scale1 < scale2 || (scale1 == scale2 && no_sign1) {
        insert_packed_aligned(f1, no_sign1, scale1, f2, no_sign2, scale2, &mut buff1, &mut buff2)
    } else {
        insert_packed_aligned(f2, no_sign2, scale2, f1, no_sign1, scale1, &mut buff2, &mut buff1)
    };
    let a = 48 - compare_len;
    if both_negative {
        memcmp(&buff2[a..a + compare_len], &buff1[a..a + compare_len])
    } else {
        memcmp(&buff1[a..a + compare_len], &buff2[a..a + compare_len])
    }
}

/// `cob_bcd_cmp (f1, f2)` (numeric.c:3962): compare two PACKED fields, returning the sign of the
/// ordering. Sign-disjoint cases short-circuit; equal layout/scale compares directly (sign nibble
/// last); otherwise scales are aligned via [`decimal_convert_scale`].
pub fn cob_bcd_cmp(d1: &[u8], a1: &FieldAttr, d2: &[u8], a2: &FieldAttr) -> i32 {
    let f1_neg = packed_is_negative(d1, a1) != 0;
    let f2_neg = packed_is_negative(d2, a2) != 0;
    if f1_neg && !f2_neg {
        return -1;
    }
    if !f1_neg && f2_neg {
        return 1;
    }
    let no_sign1 = a1.no_sign_nibble();
    let no_sign2 = a2.no_sign_nibble();
    let scale1 = a1.scale as i32;
    let scale2 = a2.scale as i32;
    if d1.len() == d2.len() && no_sign1 == no_sign2 && scale1 == scale2 {
        let ret = if no_sign1 {
            memcmp(d1, d2)
        } else {
            let len = d1.len() - 1;
            let mut r = memcmp(&d1[..len], &d2[..len]);
            if r == 0 {
                r = (d1[len] & 0xF0) as i32 - (d2[len] & 0xF0) as i32;
            }
            r
        };
        let ret = if !no_sign1 && f1_neg { -ret } else { ret };
        return ret.signum();
    }
    decimal_convert_scale(d1, no_sign1, scale1, d2, no_sign2, scale2, f1_neg).signum()
}

/// `cmp_packed_intern (f, n, both_are_negative)` (numeric.c:4089): re-pack the field and the unsigned
/// magnitude `n` into 20-byte buffers (sign nibble dropped) and compare byte-wise.
fn cmp_packed_intern(data: &[u8], attr: &FieldAttr, n: u64, both_negative: bool) -> i32 {
    let mut val1 = [0u8; 20];
    let first_pos = 20 - data.len();
    val1[first_pos..].copy_from_slice(data);
    if !attr.no_sign_nibble() {
        val1[19] &= 0xF0;
    }
    let mut packed_value = [0u8; 20];
    let mut nn = n;
    if nn != 0 {
        let mut p: isize = 19;
        if !attr.no_sign_nibble() {
            packed_value[19] = ((nn % 10) as u8) << 4;
            p -= 1;
            nn /= 10;
        }
        while nn != 0 && p >= 0 {
            let s = (nn % 100) as u8;
            packed_value[p as usize] = (s % 10) | ((s / 10) << 4);
            nn /= 100;
            p -= 1;
        }
    }
    for i in 0..20 {
        let ret = val1[i] as i32 - packed_value[i] as i32;
        if ret != 0 {
            return if both_negative { (-ret).signum() } else { ret.signum() };
        }
    }
    0
}

/// `cob_cmp_packed (f, val)` (numeric.c:4156): compare a PACKED field with a signed integer. For
/// >=19-digit fields the sign-aware `cmp_packed_intern` is used; smaller fields decode to an integer.
pub fn cob_cmp_packed(data: &[u8], attr: &FieldAttr, val: i64) -> i32 {
    if attr.digits >= 19 {
        let is_negative = packed_is_negative(data, attr) != 0;
        if is_negative && val >= 0 {
            return -1;
        }
        if !is_negative && val < 0 {
            return 1;
        }
        if val < 0 {
            cmp_packed_intern(data, attr, (-(val as i128)) as u64, true)
        } else {
            cmp_packed_intern(data, attr, val as u64, false)
        }
    } else {
        let n = cob_decimal_set_packed(data, attr).value.to_i128().unwrap_or(0) as i64;
        (n > val) as i32 - (n < val) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::{COB_FLAG_HAVE_SIGN, COB_FLAG_NO_SIGN_NIBBLE, COB_TYPE_NUMERIC_PACKED};

    fn packed(digits: u16, scale: i16, signed: bool, comp6: bool) -> FieldAttr {
        let mut flags = 0;
        if signed {
            flags |= COB_FLAG_HAVE_SIGN;
        }
        if comp6 {
            flags |= COB_FLAG_NO_SIGN_NIBBLE;
        }
        FieldAttr { field_type: COB_TYPE_NUMERIC_PACKED, digits, scale, flags }
    }

    fn bytes_for(attr: &FieldAttr, val: i64) -> Vec<u8> {
        // COMP-3: ceil((digits+1)/2) bytes; COMP-6: ceil(digits/2). Use a generous buffer + set.
        let n = if attr.no_sign_nibble() {
            (attr.digits as usize).div_ceil(2)
        } else {
            (attr.digits as usize / 2) + 1
        };
        let mut out = vec![0u8; n];
        cob_set_packed_int(&mut out, attr, val as i32);
        out
    }

    #[test]
    fn set_then_decode_round_trips() {
        for &(digits, scale, signed, comp6) in
            &[(5u16, 0i16, true, false), (9, 2, true, false), (4, 0, false, false), (6, 0, false, true)]
        {
            let attr = packed(digits, scale, signed, comp6);
            for &v in &[0i64, 1, 42, 12345, -7, -12345, 99, -1] {
                if !signed && v < 0 {
                    continue;
                }
                let limit = 10i64.pow(digits as u32 - scale.max(0) as u32);
                if v.abs() >= limit {
                    continue;
                }
                let img = bytes_for(&attr, v);
                let d = cob_decimal_set_packed(&img, &attr);
                let got = d.value.to_i128().unwrap();
                assert_eq!(got, v as i128, "decode {v} digits={digits} comp6={comp6} img={img:02x?}");
            }
        }
    }

    #[test]
    fn set_packed_agrees_with_sealed_from_packed_decoder() {
        // Transitive oracle proof: value::Decimal::from_packed is sealed against the GnuCOBOL 3.2
        // oracle (COMP-3 MOVE / COMP-6 sweeps). If the new nibble-faithful cob_decimal_set_packed
        // produces the same value on every valid image, it is byte-faithful to the oracle too.
        use crate::value::Decimal;
        fn dec_to_i128(d: &Decimal) -> i128 {
            let mut v: i128 = 0;
            for &dig in &d.digits {
                v = v * 10 + dig as i128;
            }
            if d.negative {
                -v
            } else {
                v
            }
        }
        for &(digits, scale, signed, comp6) in &[
            (5u16, 0i16, true, false),
            (9, 2, true, false),
            (4, 0, false, false),
            (6, 0, false, true),
            (8, 3, true, false),
            (1, 0, true, false),
            (18, 0, true, false),
        ] {
            let attr = packed(digits, scale, signed, comp6);
            let limit = 10i128.pow((digits as i32 - scale.max(0) as i32).max(0) as u32);
            for &v in &[0i64, 1, 7, 42, 99, 1234, 12345, 999999, -1, -7, -42, -12345, -999999] {
                if (!signed && v < 0) || (v.abs() as i128) >= limit {
                    continue;
                }
                let img = bytes_for(&attr, v);
                let mine = cob_decimal_set_packed(&img, &attr).value.to_i128().unwrap();
                let sealed = dec_to_i128(&Decimal::from_packed(&img, &attr));
                assert_eq!(mine, sealed, "v={v} digits={digits} scale={scale} comp6={comp6} img={img:02x?}");
                assert_eq!(mine, v as i128);
            }
        }
    }

    #[test]
    fn bcd_cmp_agrees_with_sealed_numeric_cmp() {
        // cob_bcd_cmp is the COMP-3 fast path; its verdict must match the sealed cob_numeric_cmp
        // (GNURUST.NUMCMP.1, oracle 1024/0) for every packed-vs-packed pair.
        use crate::cob_decimal::cob_numeric_cmp;
        let cases: &[(u16, i16)] = &[(5, 0), (7, 2), (9, 0), (4, 1), (6, 0)];
        for &(dg1, sc1) in cases {
            for &(dg2, sc2) in cases {
                let a1 = packed(dg1, sc1, true, false);
                let a2 = packed(dg2, sc2, true, false);
                for &v1 in &[0i64, 1, -1, 42, -42, 999, -999, 12345, -12345] {
                    for &v2 in &[0i64, 1, -1, 42, -42, 999, -999] {
                        let lim1 = 10i64.pow((dg1 as i32 - sc1.max(0) as i32).max(0) as u32);
                        let lim2 = 10i64.pow((dg2 as i32 - sc2.max(0) as i32).max(0) as u32);
                        if v1.abs() >= lim1 || v2.abs() >= lim2 {
                            continue;
                        }
                        let b1 = bytes_for(&a1, v1);
                        let b2 = bytes_for(&a2, v2);
                        let bcd = cob_bcd_cmp(&b1, &a1, &b2, &a2);
                        let sealed = cob_numeric_cmp(&b1, &a1, &b2, &a2);
                        assert_eq!(
                            bcd, sealed,
                            "bcd_cmp({v1}@{dg1}.{sc1}, {v2}@{dg2}.{sc2}) = {bcd} but numeric_cmp = {sealed}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn cmp_packed_agrees_with_value() {
        // cob_cmp_packed (field vs int) over both the >=19-digit cmp_packed_intern path and the small path.
        for &dg in &[5u16, 9, 18, 19, 20] {
            let attr = packed(dg, 0, true, false);
            for &fv in &[0i64, 1, -1, 42, -42, 9999, -9999] {
                let lim = 10i128.pow(dg as u32);
                if (fv.abs() as i128) >= lim {
                    continue;
                }
                let img = bytes_for(&attr, fv);
                for &val in &[0i64, 1, -1, 42, -42, 9999, -9999] {
                    let want = (fv > val) as i32 - (fv < val) as i32;
                    assert_eq!(cob_cmp_packed(&img, &attr, val), want, "cmp_packed fld={fv} val={val} dg={dg}");
                }
            }
        }
    }

    #[test]
    fn get_packed_matches_set_packed() {
        // cob_decimal_get_packed (decimal -> BCD) must produce the same image as cob_set_packed_int.
        let attr = packed(7, 2, true, false);
        for &v in &[0i64, 5, -5, 1234, -1234, 99999] {
            let want = bytes_for(&attr, v);
            let d = CobDecimal { value: Mpz::from_i128(v as i128), scale: 2 };
            let mut got = vec![0u8; want.len()];
            cob_decimal_get_packed(d, &attr, 0, &mut got);
            assert_eq!(got, want, "get_packed {v}");
        }
    }
}
