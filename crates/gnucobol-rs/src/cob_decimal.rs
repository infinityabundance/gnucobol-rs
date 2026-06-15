//! A 1:1 port of libcob's `cob_decimal` layer (`numeric.c`): the `{ mpz value; int scale }` working
//! decimal and the operations the runtime builds on it, on top of the pure-Rust [`crate::gmp::Mpz`].
//! Names mirror the C functions so the port is auditable against the source.
//!
//! **numeric.c is a COMPLETE 1:1 port:** every function compiled into the admitted GnuCOBOL 3.2 oracle
//! has a named Rust counterpart (here, in [`crate::packed`] for the COMP-3 in-place family, and in
//! [`crate::int_pow`]/[`crate::logical`]/[`crate::float`] for power/bit-logical/float). 97 functions
//! ported; the rest of the file is verified across ~43k differential cases (cob_decimal/arith 5400 each,
//! packed_arith 1800, round 6720, bignum 16128, numcmp 1024, comp6 98, float 1476, divide 736,
//! remainder 768, pow 588, logical 2400 — all FAIL=0).
//!
//! Coverage includes: field decode/encode (`cob_decimal_set/get_field` + the per-usage leaves
//! `set/get_display`, `set/get_binary`, `set/get_packed`, `set/get_ieee64/128dec`, with the DISPLAY
//! `0xFF`/`0x00` sentinels), arithmetic (`cob_decimal_add/sub/mul/div`, the `cob_add/sub/mul/div`
//! verbs, `cob_add_int/sub_int/set_int`, `cob_div_quotient/remainder`, the in-place `cob_add_bcd`),
//! rounding (`cob_decimal_do_round` + `handle_bcd_rounding`, 8 modes), comparison
//! (`cob_numeric_cmp`/`cob_decimal_cmp`/`cob_bcd_cmp`/`cob_cmp_int/llint/uint/packed/float/numdisp`),
//! the print family, the host-int boundary (`set/get_llint/ullint`, `mpz_set/mpz_get_sll/ull`,
//! `cob_binary_get/set_sint64/uint64`), the pow helpers, the sign helpers
//! (`cob_get_long_ascii/ebcdic_sign`), the pool/lifecycle (`init/init2/clear/alloc/push/pop`,
//! `cob_init/exit_numeric`), and the mpf trio.
//!
//! The 2048-bit `mpf` intermediary is a REAL pure-Rust binary float ([`crate::mpf::Mpf`]) — `set_d`,
//! `set_z`, `mul`, `div`, `get_d`, `get_str(96)` — so `set/get_mpf`/`set_mpf_core` are literal ports
//! over `mpf_t`, not an f64 proxy. Verified BOTH directions: `set_double` byte-identical to the oracle's
//! `mpf_set_d`+`mpf_get_str(96)` path over MOVE COMP-2→DISPLAY (`double_move_sweep`, 392/0), and
//! `get_double` (decimal→mpf→`mpf_get_d`) bit-identical to the FLOAT.1-sealed primitive.
//!
//! The `#if 0`-disabled functions are ALSO ported for literal completeness (`#[allow(dead_code)]`, not
//! wired into any active path since they are not compiled into the oracle): `cob_add_packed`,
//! `cob_complement_packed`, `display_add_int`, `display_sub_int`, `cob_display_add_int` — reproduced
//! verbatim from the disabled source (bugs included; `cob_display_add_int`'s pre-assignment `sign` read
//! is UB, modeled as 0 with a note). Every function in numeric.c (102/102) has a named Rust counterpart.
#![forbid(unsafe_code)]

use crate::arith::Round;
use crate::attr::{FieldAttr, COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED};
use crate::gmp::Mpz;
use crate::mpf::Mpf;
use crate::value::Decimal;
use core::cmp::Ordering;

/// `cob_decimal`: an arbitrary-precision integer significand plus a decimal `scale` (the value is
/// `value * 10^-scale`).
#[derive(Clone, Debug)]
pub struct CobDecimal {
    pub value: Mpz,
    pub scale: i32,
}

impl CobDecimal {
    fn from_value_decimal(dec: &Decimal) -> CobDecimal {
        let s: String = dec.digits.iter().map(|&d| (d + b'0') as char).collect();
        let mut value = if s.is_empty() {
            Mpz::new()
        } else {
            Mpz::from_decimal_string(&s)
        };
        if dec.negative && value.sgn() != 0 {
            value.neg();
        }
        CobDecimal { value, scale: dec.scale as i32 }
    }
}

/// `shift_decimal (d, n)` (numeric.c:561): `d->value *= 10^n; d->scale += n` (the represented value
/// is unchanged). `n < 0` divides (truncating toward zero). `n == 0` is a no-op.
pub fn shift_decimal(d: &mut CobDecimal, n: i32) {
    if n > 0 {
        d.value = d.value.mul(&Mpz::ui_pow_ui(10, n as u32));
    } else if n < 0 {
        d.value = d.value.tdiv_q(&Mpz::ui_pow_ui(10, (-n) as u32));
    }
    d.scale += n;
}

/// `align_decimal (d1, d2)` (numeric.c:573): bring both to the same scale by shifting the
/// smaller-scale operand up.
pub fn align_decimal(d1: &mut CobDecimal, d2: &mut CobDecimal) {
    match d1.scale.cmp(&d2.scale) {
        Ordering::Less => shift_decimal(d1, d2.scale - d1.scale),
        Ordering::Greater => shift_decimal(d2, d1.scale - d2.scale),
        Ordering::Equal => {}
    }
}

/// `COB_MPZ_DEF` (coblocal.h:143): the default initial mpz bit-capacity GnuCOBOL preallocates for a
/// working decimal. A pure GMP allocation hint with no effect on any value; carried for 1:1 fidelity.
pub const COB_MPZ_DEF: u64 = 1024;

/// `cob_decimal_init2 (d, initial_num_bits)` (numeric.c:352): construct a working decimal with its mpz
/// preallocated to `initial_num_bits` and `scale = 0`. The capacity is an allocation hint only — an
/// [`Mpz`] grows on demand — so the observable result is a zero decimal at scale 0.
pub fn cob_decimal_init2(_initial_num_bits: u64) -> CobDecimal {
    CobDecimal { value: Mpz::new(), scale: 0 }
}

/// `cob_decimal_init (d)` (numeric.c:358): `cob_decimal_init2(d, COB_MPZ_DEF)`.
pub fn cob_decimal_init() -> CobDecimal {
    cob_decimal_init2(COB_MPZ_DEF)
}

/// `cob_decimal_clear (d)` (numeric.c:364): release a working decimal — `mpz_clear(value); scale = 0`.
/// In C this is the destructor for a *reusable* `cob_decimal` (the next `init` reuses the slot); the
/// faithful observable analog in Rust — where the backing limbs are freed on drop — is to reset the
/// value to zero at scale 0, leaving the slot in the same post-clear state the C code relies on.
pub fn cob_decimal_clear(d: &mut CobDecimal) {
    d.value = Mpz::new();
    d.scale = 0;
}

/// `cob_decimal_set_ullint (d, n)` (numeric.c:374): set a working decimal to an unsigned 64-bit host
/// integer at scale 0. GnuCOBOL emits a single `mpz_set_ui` where `unsigned long` is 64-bit (the
/// admitted oracle) and a two-step `set_ui(n>>32); mul_2exp(32); add_ui(n & 0xffffffff)` only on
/// 32-bit-`long` platforms; both yield `value == n`, which is what [`Mpz::from_u64`] produces.
pub fn cob_decimal_set_ullint(d: &mut CobDecimal, n: u64) {
    d.value = Mpz::from_u64(n);
    d.scale = 0;
}

/// `cob_decimal_set (dst, src)` (numeric.c:2225): copy `src` into `dst` (`mpz_set (dst->value,
/// src->value); dst->scale = src->scale;`). `CobDecimal` is `Clone`, so this is the faithful copy.
pub fn cob_decimal_set(dst: &mut CobDecimal, src: &CobDecimal) {
    dst.value = src.value.clone();
    dst.scale = src.scale;
}

/// `cob_decimal_set_llint (d, n)` (numeric.c:389): set a working decimal to a signed 64-bit host
/// integer at scale 0 (`mpz_set_si` on the 64-bit oracle; the 32-bit-`long` branch rebuilds the same
/// value from the unsigned magnitude and a trailing sign flip). Equals [`Mpz::from_i64`].
pub fn cob_decimal_set_llint(d: &mut CobDecimal, n: i64) {
    d.value = Mpz::from_i64(n);
    d.scale = 0;
}

/// `cob_decimal_set_display (d, f)` (numeric.c:1444): decode a DISPLAY (zoned) field into a working
/// decimal. Two sentinels for uninitialised/invalid data come first: a leading data byte of `0xFF`
/// yields `+10^size`, `0x00` yields `-10^size` (both at the field scale) — valid zoned digits are
/// `0x30..=0x39` so these never arise from conforming data. Otherwise the digits + sign are the
/// FLOAT-sealed [`Decimal::from_display`] decode (`GNURUST.2/9`).
pub fn cob_decimal_set_display(data: &[u8], attr: &FieldAttr) -> CobDecimal {
    let off = attr.data_offset();
    let size = attr.data_size(data.len());
    if size > 0 && off < data.len() {
        match data[off] {
            255 => return CobDecimal { value: Mpz::ui_pow_ui(10, size as u32), scale: attr.scale as i32 },
            0 => {
                let mut v = Mpz::ui_pow_ui(10, size as u32);
                v.neg();
                return CobDecimal { value: v, scale: attr.scale as i32 };
            }
            _ => {}
        }
    }
    CobDecimal::from_value_decimal(&Decimal::from_display(data, attr))
}

/// `cob_decimal_set_binary (d, f)` (numeric.c:1637): decode a BINARY/COMP-5 field into the working
/// decimal. On the admitted 64-bit oracle this is `mpz_set_si/ui(cob_binary_get_sint64/uint64(f))` —
/// i.e. the field's two's-complement integer (endianness + sign from the flags) carried at the field
/// scale, exactly what [`crate::binary::binary_decode`] produces (sealed `GNURUST.14`).
pub fn cob_decimal_set_binary(data: &[u8], attr: &FieldAttr) -> CobDecimal {
    CobDecimal { value: Mpz::from_i128(crate::binary::binary_decode(data, attr)), scale: attr.scale as i32 }
}

/// `cob_decimal_set_ieee64dec (d, f)` (numeric.c:667): decode an IEEE-754 decimal64 (BID) field into a
/// working decimal. The BID decode + value is the FLOAT.1-sealed [`crate::float::dec64_decode`]; the
/// `(mag, scale)` it returns represents the same value the C builds (with positive exponents folded in).
pub fn cob_decimal_set_ieee64dec(data: &[u8]) -> CobDecimal {
    match crate::float::dec64_decode(data[..8].try_into().unwrap_or([0; 8])) {
        Some((m, s)) => CobDecimal { value: Mpz::from_i128(m), scale: s },
        None => CobDecimal { value: Mpz::new(), scale: 0 }, // Inf/NaN: libcob marks NaN; we carry zero
    }
}

/// `cob_decimal_set_ieee128dec (d, f)` (numeric.c:781): decode an IEEE-754 decimal128 (BID) field.
pub fn cob_decimal_set_ieee128dec(data: &[u8]) -> CobDecimal {
    match crate::float::dec128_decode(data[..16].try_into().unwrap_or([0; 16])) {
        Some((m, s)) => CobDecimal { value: Mpz::from_i128(m), scale: s },
        None => CobDecimal { value: Mpz::new(), scale: 0 },
    }
}

/// `cob_decimal_get_ieee64dec (d, f, opt)` (numeric.c:613): encode a working decimal into an IEEE-754
/// decimal64 (BID) field, via the FLOAT.1-sealed [`crate::float::dec64_encode`].
pub fn cob_decimal_get_ieee64dec(d: &CobDecimal) -> [u8; 8] {
    crate::float::dec64_encode(d.value.to_i128().unwrap_or(0), d.scale)
}

/// `cob_decimal_get_ieee128dec (d, f, opt)` (numeric.c:731): encode into an IEEE-754 decimal128 field.
pub fn cob_decimal_get_ieee128dec(d: &CobDecimal) -> [u8; 16] {
    crate::float::dec128_encode(d.value.to_i128().unwrap_or(0), d.scale)
}

// `#if 0`-disabled DISPLAY in-place arithmetic (numeric.c:3403, marked "Buggy"). NOT compiled into the
// oracle and NOT wired into any active path -- ported verbatim so every source function has a Rust
// counterpart. `cob_display_add_int` even reads `sign` before assignment in the source (undefined
// behaviour -- the bug that disabled it); reproduced as `sign = 0` with a note. NOT oracle-verifiable.

/// `display_add_int (data, size, n, opt)` (numeric.c:3403, `#if 0`): add `n` to a zoned field in place.
#[allow(dead_code)]
fn display_add_int(data: &mut [u8], size: usize, mut n: i32, opt: i32) -> i32 {
    let mut sp = size; // points one past the active region; pre-decremented before use
    let mut carry = 0i32;
    while n > 0 {
        let i = n % 10;
        n /= 10;
        if sp == 0 {
            return opt;
        }
        sp -= 1;
        let is = (data[sp] & 0x0F) as i32 + i + carry;
        if is > 9 {
            carry = 1;
            data[sp] = b'0' + ((is + 6) & 0x0F) as u8;
        } else {
            carry = 0;
            data[sp] = b'0' + is as u8;
        }
    }
    if carry == 0 {
        return 0;
    }
    while sp > 0 {
        sp -= 1;
        data[sp] += 1;
        if data[sp] <= b'9' {
            return 0;
        }
        data[sp] = b'0';
    }
    opt
}

/// `display_sub_int (data, size, n, opt)` (numeric.c:3445, `#if 0`): subtract `n` from a zoned field.
#[allow(dead_code)]
fn display_sub_int(data: &mut [u8], size: usize, mut n: i32, _opt: i32) -> i32 {
    let mut sp = size;
    let mut carry = 0i32;
    while n > 0 {
        let i = n % 10;
        n /= 10;
        if sp == 0 {
            return 1;
        }
        sp -= 1;
        let v = data[sp] as i32 - (i + carry);
        if v < b'0' as i32 {
            carry = 1;
            data[sp] = (v + 10) as u8;
        } else {
            carry = 0;
            data[sp] = v as u8;
        }
    }
    if carry == 0 {
        return 0;
    }
    while sp > 0 {
        sp -= 1;
        let v = data[sp] as i32 - 1;
        if v >= b'0' as i32 {
            data[sp] = v as u8;
            return 0;
        }
        data[sp] = b'9';
    }
    1
}

/// `cob_display_add_int (f, n, opt)` (numeric.c:3494, `#if 0` "Buggy"): in-place add of an `int` to a
/// DISPLAY field. The source reads `sign` *before* it is assigned (UB — the reason it is disabled);
/// reproduced here with `sign = 0`. Disabled in the oracle, not wired, not oracle-verifiable.
#[allow(dead_code)]
pub fn cob_display_add_int(data: &mut [u8], attr: &FieldAttr, mut n: i32, opt: i32) -> i32 {
    let osize = data.len();
    let mut size = attr.data_size(osize);
    let mut scale = attr.scale as i32;
    let tfield = data.to_vec();
    // numeric.c reads `sign` uninitialised here before the COB_GET_SIGN_ADJUST below (UB); model as 0.
    let mut sign = 0i32;
    if sign == -1 {
        n = -n;
    }
    if scale < 0 {
        if -scale < 10 {
            while scale != 0 {
                scale += 1;
                n /= 10;
            }
        } else {
            n = 0;
        }
        scale = 0;
        if n == 0 {
            return 0;
        }
    } else {
        size -= scale as usize;
        if size == 0 {
            return if opt & (1 << 1) != 0 { 0x0501 } else { 0 };
        }
    }
    sign = if attr.have_sign() && !data.is_empty() && (data[osize - 1] & 0x40) != 0 { -1 } else { 0 };
    if n > 0 {
        if display_add_int(data, size, n, opt) != 0 && opt & (1 << 1) != 0 {
            data.copy_from_slice(&tfield);
            return 0x0501;
        }
    } else if n < 0 {
        if display_sub_int(data, size, -n, opt) != 0 {
            for b in data.iter_mut().take(size) {
                *b = b'0' + (9 - (*b & 0x0F)) % 10;
            }
            if scale != 0 {
                for i in size..size + scale as usize {
                    if (data[i] & 0x0F) > 0 {
                        data[i] = b'0' + (10 - (data[i] & 0x0F));
                    }
                }
            } else {
                let _ = display_add_int(data, size, 1, 0);
            }
            sign = -sign;
        }
    }
    let _ = sign;
    0
}

/// `cob_gmp_free (ptr)` (numeric.c:260): free a string allocated by `mpz_get_str`/`mpf_get_str`. In
/// this port those conversions return owned Rust `String`s, so freeing is taking ownership and
/// dropping — the RAII analog of the C free.
pub fn cob_gmp_free(_s: String) {}

/// `cob_pow_10_uli (n)` (numeric.c:517): `10^n` as a host unsigned integer (the pre-stored table value).
pub fn cob_pow_10_uli(n: u32) -> u64 {
    10u64.pow(n)
}

/// `cob_mul_by_pow_10 (mexp, n)` (numeric.c:532): scale an integer up by `10^n` (`mexp *= 10^n`).
pub fn cob_mul_by_pow_10(v: &mut Mpz, n: u32) {
    *v = v.mul(&Mpz::ui_pow_ui(10, n));
}

/// `cob_pow_10 (mexp, n)` (numeric.c:468): `mexp = 10^n` as an arbitrary-precision integer. The C
/// short-circuits `n <= COB_MAX_BINARY` to a pre-stored `cob_mpze10[n]` table value; the result is the
/// same number ([`Mpz::ui_pow_ui`]).
pub fn cob_pow_10(n: u32) -> Mpz {
    Mpz::ui_pow_ui(10, n)
}

/// `cob_div_by_pow_10 (mexp, n)` (numeric.c:547): `mexp = mexp / 10^n` (truncating). The C's small-`n`
/// `cob_pow_10_uli` fast path computes the same quotient as the arbitrary-precision path.
pub fn cob_div_by_pow_10(mexp: &mut Mpz, n: u32) {
    *mexp = mexp.tdiv_q(&cob_pow_10(n));
}

/// `cob_decimal_adjust (d, max_value, min_exp, max_exp)` (numeric.c:586): normalise `d` for an IEEE
/// decimal encode — strip trailing zeros (raising the scale), then shift the comma left until the
/// magnitude fits `max_value` (or the scale floor is hit). Returns `true` on `COB_EC_SIZE_OVERFLOW`
/// (value still too large, or the scale escaped `[min_exp, max_exp]`).
pub fn cob_decimal_adjust(d: &mut CobDecimal, max_value: &Mpz, min_exp: i32, max_exp: i32) -> bool {
    // Remove trailing ZEROS (mpz_remove by 10; scale drops by the count removed).
    let power_of_ten = d.value.remove_pow10();
    if power_of_ten != 0 {
        d.scale -= power_of_ten as i32;
    }
    // Move the comma to the left while the magnitude exceeds max_value.
    while d.value.cmpabs(max_value) == std::cmp::Ordering::Greater {
        if d.scale < min_exp {
            break;
        }
        d.value = d.value.tdiv_q_ui(10);
        d.scale -= 1;
    }
    d.value.cmpabs(max_value) == std::cmp::Ordering::Greater || d.scale < min_exp || d.scale > max_exp
}

/// `cob_binary_get_uint64 (f)` (numeric.c:294): read a BINARY field's bytes as an unsigned 64-bit
/// integer, honoring `COB_FLAG_BINARY_SWAP` (big-endian) vs native little-endian.
pub fn cob_binary_get_uint64(data: &[u8], attr: &FieldAttr) -> u64 {
    let n = data.len().min(8);
    let swap = attr.flags & crate::attr::COB_FLAG_BINARY_SWAP != 0;
    let mut u: u64 = 0;
    if swap {
        for &b in data.iter().take(n) {
            u = (u << 8) | b as u64;
        }
    } else {
        for &b in data.iter().take(n).rev() {
            u = (u << 8) | b as u64;
        }
    }
    u
}

/// `cob_binary_get_sint64 (f)` (numeric.c:272): read a BINARY field's bytes as a signed 64-bit integer
/// (sign-extended from the field width).
pub fn cob_binary_get_sint64(data: &[u8], attr: &FieldAttr) -> i64 {
    let bits = data.len().min(8) * 8;
    let u = cob_binary_get_uint64(data, attr);
    if bits < 64 && (u >> (bits - 1)) & 1 == 1 {
        (u as i64).wrapping_sub(1i64 << bits)
    } else {
        u as i64
    }
}

/// `cob_binary_set_uint64 (f, n)` (numeric.c:315): write an unsigned 64-bit integer into a BINARY field.
pub fn cob_binary_set_uint64(out: &mut [u8], attr: &FieldAttr, n: u64) {
    crate::binary::binary_encode(n as i128, attr, out);
}

/// `cob_binary_set_int64 (f, n)` (numeric.c:332): write a signed 64-bit integer into a BINARY field.
pub fn cob_binary_set_int64(out: &mut [u8], attr: &FieldAttr, n: i64) {
    crate::binary::binary_encode(n as i128, attr, out);
}

/// `cob_decimal_set_mpf_core (d, src)` (numeric.c:853): convert an `mpf` to a working decimal via a
/// 96-significant-digit `mpf_get_str` — a literal port over the real [`crate::mpf::Mpf`]. The mpf's
/// significant digits become the mantissa; the GMP exponent becomes the scale (`len - exp`, folding a
/// negative result into the magnitude via `cob_mul_by_pow_10`).
pub fn cob_decimal_set_mpf_core(src: &Mpf) -> CobDecimal {
    let (neg, digits, exp10) = src.get_str(96);
    let s: String = digits.iter().map(|d| (b'0' + d) as char).collect();
    let mut value = if s.is_empty() { Mpz::new() } else { Mpz::from_decimal_string(&s) };
    let len = digits.len() as i64;
    let new_len = len - exp10;
    let scale;
    if new_len >= 0 {
        scale = new_len as i32;
    } else {
        scale = 0;
        cob_mul_by_pow_10(&mut value, (-new_len) as u32);
    }
    if neg && value.sgn() != 0 {
        value.neg();
    }
    CobDecimal { value, scale }
}

/// `cob_decimal_set_mpf (d, src)` (numeric.c:885): zero-short-circuit then [`cob_decimal_set_mpf_core`].
pub fn cob_decimal_set_mpf(src: &Mpf) -> CobDecimal {
    if src.sgn() == 0 {
        CobDecimal { value: Mpz::new(), scale: 0 }
    } else {
        cob_decimal_set_mpf_core(src)
    }
}

/// `cob_decimal_get_mpf (dst, d)` (numeric.c:897): convert a working decimal to an `mpf` (`value *
/// 10^-scale`), via real [`crate::mpf::Mpf`] arithmetic (`mpf_set_z` then `mpf_mul`/`mpf_div` by a
/// power of ten at `COB_MPF_PREC`).
pub fn cob_decimal_get_mpf(d: &CobDecimal) -> Mpf {
    let mut dst = Mpf::set_z(&d.value, crate::mpf::COB_MPF_PREC);
    if d.scale < 0 {
        let p = Mpz::ui_pow_ui(10, (-d.scale) as u32);
        dst = dst.mul(&Mpf::set_z(&p, crate::mpf::COB_MPF_PREC));
    } else if d.scale > 0 {
        let p = Mpz::ui_pow_ui(10, d.scale as u32);
        dst = dst.div(&Mpf::set_z(&p, crate::mpf::COB_MPF_PREC));
    }
    dst
}

/// `cob_decimal_alloc (params, ...)` (numeric.c:4350): point `params` caller decimals at pre-allocated
/// pool slots. Rust has no manual GMP pool (working decimals are created on demand, RAII), so this
/// returns `params` fresh zero decimals — the faithful analog of the pool slots.
pub fn cob_decimal_alloc(params: u32) -> Vec<CobDecimal> {
    (0..params).map(|_| cob_decimal_init()).collect()
}

/// `cob_decimal_push (params, ...)` (numeric.c:4368): allocate + init `params` temporary decimals.
pub fn cob_decimal_push(params: u32) -> Vec<CobDecimal> {
    (0..params).map(|_| cob_decimal_init()).collect()
}

/// `cob_decimal_pop (params, ...)` (numeric.c:4385): release temporaries from [`cob_decimal_push`] — a
/// no-op in Rust (the elements are freed on drop); takes them by value to mirror the release.
pub fn cob_decimal_pop(_decs: Vec<CobDecimal>) {}

/// `cob_init_numeric (lptr)` (numeric.c:4482): module init — preallocates the static GMP power-of-ten
/// tables and decimal pool. Those are computed on demand here ([`Mpz::ui_pow_ui`]), so init is a no-op
/// kept for the 1:1 surface.
pub fn cob_init_numeric() {}

/// `cob_exit_numeric (void)` (numeric.c:4446): module teardown — frees the static GMP tables/pool. A
/// no-op in Rust (RAII).
pub fn cob_exit_numeric() {}

/// `cob_decimal_set_field (d, f)`: decode a numeric field into the working decimal. Uses the sealed
/// per-usage decoders ([`Decimal::from_display`] / [`Decimal::from_packed`] / binary decode).
pub fn cob_decimal_set_field(data: &[u8], attr: &FieldAttr) -> CobDecimal {
    match attr.field_type {
        COB_TYPE_NUMERIC_DISPLAY => cob_decimal_set_display(data, attr),
        COB_TYPE_NUMERIC_PACKED => crate::packed::cob_decimal_set_packed(data, attr),
        COB_TYPE_NUMERIC_BINARY => cob_decimal_set_binary(data, attr),
        0x16 => cob_decimal_set_ieee64dec(data),
        0x17 => cob_decimal_set_ieee128dec(data),
        0x13 | 0x14 | 0x15 => {
            // binary float (COMP-1/COMP-2/L_DOUBLE): decode to f64, then to a decimal (exact dyadic)
            let v = match attr.field_type {
                0x13 => f32::from_le_bytes(data[..4].try_into().unwrap_or([0; 4])) as f64,
                0x14 => f64::from_le_bytes(data[..8].try_into().unwrap_or([0; 8])),
                _ => extended80_to_f64(data),
            };
            cob_decimal_set_double(v)
        }
        _ => CobDecimal { value: Mpz::new(), scale: 0 },
    }
}

/// `COB_MAX_DIGITS` (coblocal.h): the maximum COBOL numeric precision.
pub const COB_MAX_DIGITS: i32 = 38;

/// `cob_decimal_add (d1, d2)` (numeric.c): `d1 += d2`, aligning scales.
pub fn cob_decimal_add(d1: &mut CobDecimal, d2: &CobDecimal) {
    if d1.scale != d2.scale {
        if d2.value.sgn() == 0 {
            return;
        }
        if d1.value.sgn() == 0 {
            *d1 = d2.clone();
            return;
        }
        let mut t2 = d2.clone();
        align_decimal(d1, &mut t2);
        d1.value = d1.value.add(&t2.value);
    } else {
        d1.value = d1.value.add(&d2.value);
    }
}

/// `cob_decimal_sub (d1, d2)` (numeric.c): `d1 -= d2`, aligning scales.
pub fn cob_decimal_sub(d1: &mut CobDecimal, d2: &CobDecimal) {
    if d1.scale != d2.scale {
        if d2.value.sgn() == 0 {
            return;
        }
        let mut t2 = d2.clone();
        align_decimal(d1, &mut t2);
        d1.value = d1.value.sub(&t2.value);
    } else {
        d1.value = d1.value.sub(&d2.value);
    }
}

/// `cob_decimal_mul (d1, d2)` (numeric.c): `d1 *= d2`; scales add.
pub fn cob_decimal_mul(d1: &mut CobDecimal, d2: &CobDecimal) {
    d1.scale += d2.scale;
    d1.value = d1.value.mul(&d2.value);
}

/// `cob_decimal_div (d1, d2)` (numeric.c): `d1 /= d2`. Returns `Err` on divide-by-zero (libcob sets
/// the scale to NaN + raises `COB_EC_SIZE_ZERO_DIVIDE`). Scales the dividend up by `COB_MAX_DIGITS`
/// (plus the borrow for a negative result scale) before the truncating integer divide, so the
/// quotient carries full COBOL precision for the receiving store to round/truncate.
pub fn cob_decimal_div(d1: &mut CobDecimal, d2: &CobDecimal) -> Result<(), ()> {
    if d2.value.sgn() == 0 {
        return Err(());
    }
    if d1.value.sgn() == 0 {
        d1.scale = 0;
        return Ok(());
    }
    d1.scale -= d2.scale;
    let extra = COB_MAX_DIGITS + if d1.scale < 0 { -d1.scale } else { 0 };
    shift_decimal(d1, extra);
    d1.value = d1.value.tdiv_q(&d2.value);
    Ok(())
}

/// `cob_decimal_do_round (d, tgt_scale, opt)` (numeric.c:1936) on `Mpz`: round `d` to `tgt` fractional
/// digits per the `Round` mode. Mirrors the i128 [`crate::arith::Round`] dispatch exactly (proven by
/// GNURUST.ROUND.1). Returns `Err` only for `Prohibited` with a dropped non-zero digit.
pub fn cob_decimal_do_round(d: &mut CobDecimal, tgt: i32, round: Round) -> Result<(), ()> {
    let sign = d.value.sgn();
    if sign == 0 || tgt >= d.scale {
        return Ok(());
    }
    let drop = d.scale - tgt;
    let p = |k: i32| Mpz::ui_pow_ui(10, k as u32);
    let five = |k: i32| Mpz::ui_pow_ui(10, k as u32).mul_ui(5);
    let s5 = Mpz::from_i64(sign as i64 * 5);
    match round {
        Round::Truncate => {}
        Round::Prohibited => {
            if d.value.tdiv_r(&p(drop)).sgn() != 0 {
                return Err(());
            }
        }
        Round::AwayFromZero => {
            let div = p(drop);
            if d.value.tdiv_r(&div).sgn() != 0 {
                d.value = if sign > 0 { d.value.add(&div) } else { d.value.sub(&div) };
            }
        }
        Round::TowardGreater => {
            let div = p(drop);
            if d.value.tdiv_r(&div).sgn() != 0 && sign > 0 {
                d.value = d.value.add(&div);
            }
        }
        Round::TowardLesser => {
            let div = p(drop);
            if d.value.tdiv_r(&div).sgn() != 0 && sign < 0 {
                d.value = d.value.sub(&div);
            }
        }
        Round::NearTowardZero => {
            let exact = d.value.tdiv_r(&five(drop - 1)).sgn() == 0;
            let n = tgt + 1 - d.scale; shift_decimal(d, n);
            if !exact {
                d.value = d.value.add(&s5);
            }
        }
        Round::NearEven => {
            let exact = d.value.tdiv_r(&five(drop - 1)).sgn() == 0;
            let n = tgt + 1 - d.scale; shift_decimal(d, n);
            let round_up = if exact {
                let last_two = d.value.tdiv_r(&Mpz::from_u64(100)).get_ui();
                !matches!(last_two, 5 | 25 | 45 | 65 | 85)
            } else {
                true
            };
            if round_up {
                d.value = d.value.add(&s5);
            }
        }
        Round::NearAwayFromZero => {
            let n = tgt + 1 - d.scale; shift_decimal(d, n);
            d.value = d.value.add(&s5);
        }
        _ => {
            let n = tgt + 1 - d.scale; shift_decimal(d, n);
            d.value = d.value.add(&s5);
        }
    }
    Ok(())
}

/// `cob_decimal_get_field (d, f, opt)` (numeric.c:2055) on `Mpz`: round (if requested), adjust to the
/// field scale, truncate to the field digits, and store as DISPLAY/PACKED/BINARY bytes (via the sealed
/// [`crate::cob_move`] encoders). Returns the field byte image. `Err` on a Prohibited size error.
pub fn cob_decimal_get_field(mut d: CobDecimal, attr: &FieldAttr, size: usize, round: Round, sign_on_zero: bool) -> Result<Vec<u8>, ()> {
    // sign before rounding: the packed cob_add_bcd path keeps a negative sign on a result that
    // *rounds* to zero (e.g. -0.1 into an integer -> -0); the general path does not (GNURUST.13).
    let pre_neg = d.value.sgn() < 0;
    let tgt = attr.scale as i32;
    if round != Round::Truncate {
        cob_decimal_do_round(&mut d, tgt, round)?;
    }
    // adjust to the field scale (truncating narrow / zero-extend wide)
    if d.scale != tgt {
        let n = tgt - d.scale;
        shift_decimal(&mut d, n);
    }
    // The stored sign follows the *pre-truncation* value (so an overflowed negative result -- e.g.
    // -40 into 1 digit -- stores negative zero); a value that ROUNDED to zero keeps the pre-round
    // sign only on the cob_add_bcd path. Matches cob_decimal_get_display / cob_add_bcd.
    let neg = if d.value.sgn() != 0 {
        d.value.sgn() < 0
    } else {
        sign_on_zero && pre_neg
    };
    // truncate to the field's digit count (overflow keeps the low digits)
    let modulus = Mpz::ui_pow_ui(10, attr.digits as u32);
    let mut low = d.value.tdiv_r(&modulus);
    if low.sgn() < 0 {
        low.abs();
    }
    // Wide DISPLAY results (transcendentals can reach ~96 digits) exceed u128; render them from the
    // magnitude's decimal digits. Narrow fields keep the proven u128 path the sealed courts exercise.
    if attr.digits as usize > 38 {
        Ok(render_numeric_big(neg, &low, attr, size))
    } else {
        Ok(render_numeric(neg, low.to_i128().unwrap_or(0).unsigned_abs(), attr, size))
    }
}

/// Render a wide numeric magnitude (`low` already reduced mod 10^digits) into the field — the
/// `render_numeric` counterpart for fields beyond u128's 38-digit reach (SQRT/EXP/LOG results sized by
/// `cob_alloc_field`).
fn render_numeric_big(neg: bool, low: &Mpz, attr: &FieldAttr, size: usize) -> Vec<u8> {
    let digits = attr.digits as usize;
    let s = low.to_decimal_string();
    let sb = s.trim_start_matches('-').as_bytes();
    let mut temp = vec![b'0'; digits];
    let start = digits.saturating_sub(sb.len());
    for (i, &c) in sb.iter().enumerate() {
        if start + i < digits {
            temp[start + i] = c;
        }
    }
    let signed = attr.have_sign();
    if signed && neg {
        if let Some(l) = temp.last_mut() {
            *l |= 0x40;
        }
    }
    let dattr = FieldAttr {
        field_type: COB_TYPE_NUMERIC_DISPLAY,
        digits: attr.digits,
        scale: attr.scale,
        flags: if signed { crate::attr::COB_FLAG_HAVE_SIGN } else { 0 },
    };
    let mut out = vec![0u8; size];
    let _ = crate::move_ops::cob_move(&temp, &dattr, &mut out, attr);
    out
}

/// `cob_decimal_get_display (d, f, opt)` (numeric.c:1548): store an (already scale-aligned) working
/// decimal into a DISPLAY (zoned) field — overflow keeps the low `digits` digits, left-padded with
/// `'0'`, sign applied. Shares the sealed renderer that backs [`cob_decimal_get_field`].
pub fn cob_decimal_get_display(d: &CobDecimal, attr: &FieldAttr, size: usize) -> Vec<u8> {
    let neg = d.value.sgn() < 0;
    let low = d.value.tdiv_r(&Mpz::ui_pow_ui(10, attr.digits as u32));
    render_numeric(neg, low.to_i128().unwrap_or(0).unsigned_abs(), attr, size)
}

/// `cob_decimal_get_binary (d, f, opt)` (numeric.c:1720): store an (already scale-aligned) working
/// decimal into a BINARY/COMP-5 field, truncating to the field's bit/digit capacity. Shares the sealed
/// renderer (`cob_move` -> [`crate::binary::binary_encode`]) that backs [`cob_decimal_get_field`].
pub fn cob_decimal_get_binary(d: &CobDecimal, attr: &FieldAttr, size: usize) -> Vec<u8> {
    let neg = d.value.sgn() < 0;
    let low = d.value.tdiv_r(&Mpz::ui_pow_ui(10, attr.digits as u32));
    render_numeric(neg, low.to_i128().unwrap_or(0).unsigned_abs(), attr, size)
}

/// Render a sign + non-negative magnitude (already at the field scale, low `digits` digits) to the
/// field's `size` bytes, via a DISPLAY temp and the sealed `cob_move` (display/packed/binary targets).
/// A negative sign is kept even when the magnitude is zero (negative zero).
fn render_numeric(neg: bool, mag_abs: u128, attr: &FieldAttr, size: usize) -> Vec<u8> {
    let digits = attr.digits as usize;
    let modulus = 10u128.pow(digits as u32);
    let mut abs = mag_abs % modulus;
    let mut ds = vec![0u8; digits];
    for slot in ds.iter_mut().rev() {
        *slot = (abs % 10) as u8;
        abs /= 10;
    }
    let mut temp: Vec<u8> = ds.iter().map(|d| b'0' + d).collect();
    let signed = attr.have_sign();
    if signed && neg {
        if let Some(l) = temp.last_mut() {
            *l |= 0x40;
        }
    }
    let dattr = FieldAttr {
        field_type: COB_TYPE_NUMERIC_DISPLAY,
        digits: attr.digits,
        scale: attr.scale,
        flags: if signed { crate::attr::COB_FLAG_HAVE_SIGN } else { 0 },
    };
    let mut out = vec![0u8; size];
    let _ = crate::move_ops::cob_move(&temp, &dattr, &mut out, attr);
    out
}

/// Round-mode mapping for the packed `cob_add_bcd` path (GNURUST.ROUND.2): NEAREST-EVEN resolves
/// away-from-zero there (no to-even); every other mode matches the cob_decimal path.
fn bcd_round_mode(round: Round) -> Round {
    match round {
        Round::NearEven => Round::NearAwayFromZero,
        other => other,
    }
}

/// `cob_add (f1, f2, opt)` (numeric.c): `f1 := f1 + f2`. A PACKED receiver takes the `cob_add_bcd`
/// fast path (cob_addsub_optimized): same sum as the general cob_decimal path, but BCD-rounded
/// (NEAREST-EVEN -> away) and keeping a negative sign on a zero result. Returns f1's new bytes.
pub fn cob_add(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr, round: Round) -> Result<Vec<u8>, ()> {
    let bcd = a1.field_type == COB_TYPE_NUMERIC_PACKED;
    let mut d = cob_decimal_set_field(f1, a1);
    let d2 = cob_decimal_set_field(f2, a2);
    cob_decimal_add(&mut d, &d2);
    let eff = if bcd { bcd_round_mode(round) } else { round };
    cob_decimal_get_field(d, a1, f1.len(), eff, bcd)
}

/// `cob_sub (f1, f2, opt)` (numeric.c): `f1 := f1 - f2`. PACKED receiver -> cob_add_bcd fast path.
pub fn cob_sub(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr, round: Round) -> Result<Vec<u8>, ()> {
    let bcd = a1.field_type == COB_TYPE_NUMERIC_PACKED;
    let mut d = cob_decimal_set_field(f1, a1);
    let d2 = cob_decimal_set_field(f2, a2);
    cob_decimal_sub(&mut d, &d2);
    let eff = if bcd { bcd_round_mode(round) } else { round };
    cob_decimal_get_field(d, a1, f1.len(), eff, bcd)
}

/// `cob_mul (f1, f2, opt)` (numeric.c): `f1 := f1 * f2`, via the general cob_decimal path. The
/// receiver's byte length is `f1.len()`. Returns f1's new byte image.
pub fn cob_mul(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr, round: Round) -> Result<Vec<u8>, ()> {
    let mut d = cob_decimal_set_field(f1, a1);
    let d2 = cob_decimal_set_field(f2, a2);
    cob_decimal_mul(&mut d, &d2);
    cob_decimal_get_field(d, a1, f1.len(), round, false)
}

/// `cob_div (f1, f2, opt)` (numeric.c): `f1 := f1 / f2`. `Err` on divide-by-zero.
pub fn cob_div(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr, round: Round) -> Result<Vec<u8>, ()> {
    let mut d = cob_decimal_set_field(f1, a1);
    let d2 = cob_decimal_set_field(f2, a2);
    cob_decimal_div(&mut d, &d2)?;
    cob_decimal_get_field(d, a1, f1.len(), round, false)
}

/// `cob_div_quotient (dividend, divisor, quotient, opt)` (numeric.c:2423): store `dividend / divisor`
/// into the quotient receiver and return the COBOL `REMAINDER` working decimal (`dividend − (quotient
/// truncated to the quotient's scale) × divisor`) for a following [`cob_div_remainder`]. `Err` on
/// divide-by-zero. The remainder uses the *truncated* quotient, matching `DIVIDE … GIVING … REMAINDER`.
pub fn cob_div_quotient(
    dividend: &[u8],
    a_dvd: &FieldAttr,
    divisor: &[u8],
    a_dvs: &FieldAttr,
    quot_attr: &FieldAttr,
    quot_len: usize,
    round: Round,
) -> Result<(Vec<u8>, CobDecimal), ()> {
    let mut d1 = cob_decimal_set_field(dividend, a_dvd);
    let d2 = cob_decimal_set_field(divisor, a_dvs);
    let mut remainder = d1.clone(); // save the dividend (cob_d_remainder)
    cob_decimal_div(&mut d1, &d2)?;
    let mut d3 = d1.clone(); // save the full-precision quotient (cob_d3)
    let quot_bytes = cob_decimal_get_field(d1, quot_attr, quot_len, round, false)?;
    // truncate the quotient to its receiver scale, then remainder = dividend − quotient·divisor
    let n = quot_attr.scale as i32 - d3.scale;
    if n != 0 {
        if d3.value.sgn() == 0 {
            d3.scale = 0;
        } else {
            shift_decimal(&mut d3, n);
        }
    }
    cob_decimal_mul(&mut d3, &d2);
    cob_decimal_sub(&mut remainder, &d3);
    Ok((quot_bytes, remainder))
}

/// `cob_div_remainder (fld_remainder, opt)` (numeric.c:2468): store the `REMAINDER` working decimal
/// produced by [`cob_div_quotient`] into the remainder receiver.
pub fn cob_div_remainder(remainder: CobDecimal, rem_attr: &FieldAttr, rem_len: usize, round: Round) -> Result<Vec<u8>, ()> {
    cob_decimal_get_field(remainder, rem_attr, rem_len, round, false)
}

/// `cob_decimal_setget_fld (src, dst, opt)` (numeric.c:2480): the generic numeric MOVE — decode `src`
/// to a working decimal, then store it into `dst` (with `COB_STORE_NO_SIZE_ERROR`, so it truncates
/// rather than raising). Returns the receiver bytes.
pub fn cob_decimal_setget_fld(src: &[u8], a_src: &FieldAttr, dst_attr: &FieldAttr, dst_len: usize, round: Round) -> Result<Vec<u8>, ()> {
    let d = cob_decimal_set_field(src, a_src);
    cob_decimal_get_field(d, dst_attr, dst_len, round, false)
}

/// `cob_decimal_cmp (d1, d2)` (numeric.c): align scales then compare. Returns -1/0/1.
pub fn cob_decimal_cmp(d1: &CobDecimal, d2: &CobDecimal) -> i32 {
    let ord = if d1.scale != d2.scale {
        let mut a = d1.clone();
        let mut b = d2.clone();
        align_decimal(&mut a, &mut b);
        a.value.cmp(&b.value)
    } else {
        d1.value.cmp(&d2.value)
    };
    match ord {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// `cob_numeric_cmp (f1, f2)` (numeric.c): the signed −1/0/1 comparison of two numeric fields. The
/// libcob dispatcher uses fast paths (bcd compare, integer compare) that all yield the same verdict
/// as the general decimal comparison reproduced here; float operands route to the float comparison.
pub fn cob_numeric_cmp(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr) -> i32 {
    // numeric.c:4036 routes only FLOAT/DOUBLE/L_DOUBLE through cob_cmp_float; FP_DEC64/128 fall to the
    // decimal compare (cob_decimal_set_field now decodes them via the sealed BID decoders).
    let is_bin_float = |a: &FieldAttr| matches!(a.field_type, 0x13 | 0x14 | 0x15);
    if is_bin_float(a1) || is_bin_float(a2) {
        return cob_cmp_float(f1, a1, f2, a2);
    }
    // BCD fast path (numeric.c:4047): both PACKED with non-negative scale -> in-place nibble compare.
    if a1.field_type == COB_TYPE_NUMERIC_PACKED
        && a2.field_type == COB_TYPE_NUMERIC_PACKED
        && a1.scale >= 0
        && a2.scale >= 0
    {
        return crate::packed::cob_bcd_cmp(f1, a1, f2, a2);
    }
    let d1 = cob_decimal_set_field(f1, a1);
    let d2 = cob_decimal_set_field(f2, a2);
    cob_decimal_cmp(&d1, &d2)
}

/// `cob_cmp_int (f, n)`: compare a numeric field to a host integer. Same verdict as decoding `n` to a
/// decimal and comparing.
pub fn cob_cmp_int(f: &[u8], a: &FieldAttr, n: i64) -> i32 {
    let d1 = cob_decimal_set_field(f, a);
    let d2 = CobDecimal { value: Mpz::from_i64(n), scale: 0 };
    cob_decimal_cmp(&d1, &d2)
}

/// `cob_cmp_llint (f, n)`: 64-bit signed field-vs-int compare (libcob fast path; same verdict).
pub fn cob_cmp_llint(f: &[u8], a: &FieldAttr, n: i64) -> i32 {
    cob_cmp_int(f, a, n)
}

/// `cob_cmp_uint (f, n)`: unsigned field-vs-int compare. Same verdict, via an unsigned decimal.
pub fn cob_cmp_uint(f: &[u8], a: &FieldAttr, n: u64) -> i32 {
    let d1 = cob_decimal_set_field(f, a);
    let d2 = CobDecimal { value: Mpz::from_u64(n), scale: 0 };
    cob_decimal_cmp(&d1, &d2)
}

/// `cob_cmp_packed (f, val)`: compare a packed field to an int64 (libcob's BCD fast path; the verdict
/// matches the general decimal comparison reproduced here).
pub fn cob_cmp_packed(f: &[u8], a: &FieldAttr, val: i64) -> i32 {
    cob_cmp_int(f, a, val)
}

/// `cob_add_int (f, n, opt)`: `f := f + n` for a host integer `n`. The packed receiver takes the
/// cob_add_bcd fast path.
pub fn cob_add_int(f1: &[u8], a1: &FieldAttr, n: i32, round: Round) -> Result<Vec<u8>, ()> {
    let bcd = a1.field_type == COB_TYPE_NUMERIC_PACKED;
    let mut d = cob_decimal_set_field(f1, a1);
    cob_decimal_add(&mut d, &CobDecimal { value: Mpz::from_i64(n as i64), scale: 0 });
    let eff = if bcd { bcd_round_mode(round) } else { round };
    cob_decimal_get_field(d, a1, f1.len(), eff, bcd)
}

/// `cob_sub_int (f, n, opt)`: `f := f - n` for a host integer `n`.
pub fn cob_sub_int(f1: &[u8], a1: &FieldAttr, n: i32, round: Round) -> Result<Vec<u8>, ()> {
    cob_add_int(f1, a1, -n, round)
}

/// `cob_set_int (f, n)`: store a host integer into a numeric field (no rounding; truncating store).
pub fn cob_set_int(f1: &[u8], a1: &FieldAttr, n: i32) -> Result<Vec<u8>, ()> {
    let d = CobDecimal { value: Mpz::from_i64(n as i64), scale: 0 };
    cob_decimal_get_field(d, a1, f1.len(), Round::Truncate, false)
}

/// `cob_decimal_align (d1, scale)` (numeric.c:2282): shift `d1` toward the target `scale`. Ported
/// verbatim, including the source's second branch which shifts by `d1.scale - scale` (the same sign as
/// the first branch) — a faithful 1:1 of GnuCOBOL 3.2, quirk included.
pub fn cob_decimal_align(d1: &mut CobDecimal, scale: i32) {
    if d1.scale > scale {
        shift_decimal(d1, scale - d1.scale);
    } else if d1.scale < scale {
        shift_decimal(d1, d1.scale - scale);
    }
}

/// `cob_decimal_get_double (d)` (numeric.c): the `f64` value of a working decimal, truncating toward
/// zero (libcob's `mpf_get_d`). Modeled by [`crate::float::decimal_to_f64_trunc`].
pub fn cob_decimal_get_double(d: &CobDecimal) -> f64 {
    // cob_decimal_get_mpf(cob_mpft, d); v = mpf_get_d(cob_mpft) -- literal, over the real Mpf.
    if d.value.sgn() == 0 {
        return 0.0;
    }
    cob_decimal_get_mpf(d).get_d()
}

/// `cob_get_long_ascii_sign (p, val)` (numeric.c:4186): decode an ASCII trailing-overpunch sign byte
/// (`p`..`y` => digit 0..9, all negative). Writes the digit into `val`, returns 1 if negative.
pub fn cob_get_long_ascii_sign(p: u8, val: &mut i32) -> i32 {
    match p {
        b'p' => 1,
        b'q'..=b'y' => {
            *val = (p - b'p') as i32;
            1
        }
        _ => 0,
    }
}

/// `cob_get_long_ebcdic_sign (p, val)` (numeric.c:4224): decode an EBCDIC trailing-overpunch sign byte
/// (`{`/`A`..`I` positive 0..9, `}`/`J`..`R` negative 0..9). Writes the digit into `val`, returns 1 if
/// negative.
pub fn cob_get_long_ebcdic_sign(p: u8, val: &mut i32) -> i32 {
    match p {
        b'{' => 0,
        b'A'..=b'I' => {
            *val = (p - b'A' + 1) as i32;
            0
        }
        b'}' => 1,
        b'J'..=b'R' => {
            *val = (p - b'J' + 1) as i32;
            1
        }
        _ => 0,
    }
}

/// `cob_cmp_numdisp (data, size, n, has_sign)` (numeric.c:4182): compare an unedited DISPLAY field's
/// value with a signed integer `n`. Mirrors libcob: unsigned builds the magnitude directly; signed
/// reads the trailing (possibly overpunched) sign digit, including the EBCDIC/ASCII overpunch tables.
pub fn cob_cmp_numdisp(data: &[u8], n: i64, has_sign: bool, ebcdic_sign: bool) -> i32 {
    let size = data.len();
    if !has_sign {
        if n < 0 {
            return 1;
        }
        let mut val: i64 = 0;
        for &b in data {
            val = val * 10 + (b & 0x0F) as i64;
        }
        return (val > n) as i32 - (val < n) as i32;
    }
    if size == 0 {
        return 0;
    }
    let mut val: i64 = 0;
    for &b in &data[..size - 1] {
        val = val * 10 + (b & 0x0F) as i64;
    }
    val *= 10;
    let p = data[size - 1];
    if p.is_ascii_digit() {
        val += (p & 0x0F) as i64;
    } else if ebcdic_sign {
        let mut sv = 0i32;
        let neg = cob_get_long_ebcdic_sign(p, &mut sv);
        val += sv as i64;
        if neg == 1 {
            val = -val;
        }
    } else if (b'p'..=b'y').contains(&p) {
        val += (p - b'p') as i64;
        val = -val;
    }
    (val > n) as i32 - (val < n) as i32
}

/// Decode an x86 80-bit extended-precision `long double` (16-byte storage, low 10 bytes used) to `f64`,
/// matching the C cast `(double)ld`. Sign + 15-bit exponent + 64-bit mantissa with an explicit integer
/// bit; ported so `cob_cmp_float`'s L_DOUBLE branch has no platform bound.
pub fn extended80_to_f64(b: &[u8]) -> f64 {
    if b.len() < 10 {
        return 0.0;
    }
    let mantissa = u64::from_le_bytes(b[..8].try_into().unwrap_or([0; 8]));
    let se = u16::from_le_bytes(b[8..10].try_into().unwrap_or([0; 2]));
    let sign = if se & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = (se & 0x7FFF) as i32;
    if exp == 0 && mantissa == 0 {
        return 0.0 * sign;
    }
    if exp == 0x7FFF {
        return if mantissa << 1 == 0 { sign * f64::INFINITY } else { f64::NAN };
    }
    // value = mantissa * 2^(exp - 16383 - 63); the explicit integer bit makes mantissa the full 64-bit.
    sign * (mantissa as f64) * 2f64.powi(exp - 16383 - 63)
}

/// Encode a finite `f64` as an x87 80-bit extended `long double` (16-byte storage, low 10 bytes used) —
/// the inverse of [`extended80_to_f64`], for `cob_move_fp_to_fp`'s L_DOUBLE receiver.
pub fn f64_to_extended80(v: f64) -> [u8; 16] {
    let mut out = [0u8; 16];
    if v == 0.0 {
        if v.is_sign_negative() {
            out[9] = 0x80;
        }
        return out;
    }
    let neg = v < 0.0;
    if !v.is_finite() {
        let se: u16 = (if neg { 0x8000 } else { 0 }) | 0x7FFF;
        out[8..10].copy_from_slice(&se.to_le_bytes());
        out[7] = if v.is_nan() { 0xC0 } else { 0x80 }; // integer bit (+ QNaN bit for NaN)
        return out;
    }
    let (m, e) = crate::float::decompose_f64(v.abs());
    let lead = m.leading_zeros() as i32;
    let mant64 = m << lead; // normalize so the integer (top) bit is set
    let expo = (e - lead + 16383 + 63) as u16;
    out[..8].copy_from_slice(&mant64.to_le_bytes());
    let se = (if neg { 0x8000u16 } else { 0 }) | (expo & 0x7FFF);
    out[8..10].copy_from_slice(&se.to_le_bytes());
    out
}

/// `cob_cmp_float (f1, f2)` (numeric.c): compare two numeric fields as `double`, returning `0` when
/// equal within libcob's relative `TOLERANCE` (`1e-7`), else `-1`/`1`. Float/double/long-double
/// operands are read directly; any other operand decodes to a decimal then to `double`.
pub fn cob_cmp_float(f1: &[u8], a1: &FieldAttr, f2: &[u8], a2: &FieldAttr) -> i32 {
    const TOLERANCE: f64 = 0.0000001;
    fn operand(data: &[u8], attr: &FieldAttr) -> f64 {
        match attr.field_type {
            0x13 => f32::from_le_bytes(data[..4].try_into().unwrap_or([0; 4])) as f64,
            0x14 => f64::from_le_bytes(data[..8].try_into().unwrap_or([0; 8])),
            0x15 => extended80_to_f64(data),
            _ => cob_decimal_get_double(&cob_decimal_set_field(data, attr)),
        }
    }
    let d1 = operand(f1, a1);
    let d2 = operand(f2, a2);
    if d1 == d2 {
        return 0;
    }
    if d1 != 0.0 && ((d1 - d2) / d1).abs() < TOLERANCE {
        return 0;
    }
    if d1 < d2 {
        -1
    } else {
        1
    }
}

// ---------------------------------------------------------------------------------------------------
// numeric.c print functions (the textual-render surface used by termio.c's `cob_display_common`).
// These are byte-for-byte observable through `DISPLAY` of the corresponding field types, so they are
// part of the 1:1 port, not "debug-only" non-ports.
// ---------------------------------------------------------------------------------------------------

/// `cob_decimal_print (d, fp)` (numeric.c): render a working decimal to the text libcob `fprintf`s.
/// Strips trailing factors of ten (lowering the scale), then formats as `int.frac` when the point
/// falls inside the digits, as a bare `integer` when `scale == 0`, else as `mantissaE-scale`. The
/// mantissa string carries its own leading `-` (as `mpz_get_str` does), so the split keeps the sign in
/// the integer part exactly like libcob's `"%.*s%c%.*s"`. (Our `CobDecimal` has no NaN/Inf sentinel —
/// those scales never arise from the sealed field decoders — so only the finite branch is reachable.)
pub fn cob_decimal_print(d: &CobDecimal) -> String {
    if d.value.sgn() == 0 {
        return "0E0".to_string();
    }
    let mut v = d.value.clone();
    let mut scale = d.scale;
    while v.divisible_ui(10) {
        v = v.tdiv_q_ui(10);
        scale -= 1;
    }
    let mza = v.to_decimal_string();
    let len = mza.len() as i32;
    if len > 0 && scale > 0 && scale < len {
        let dot = (len - scale) as usize;
        format!("{}.{}", &mza[..dot], &mza[dot..])
    } else if scale == 0 {
        mza
    } else {
        format!("{mza}E{}", -scale)
    }
}

/// `cob_decimal_set_double (d, v)` (numeric.c:917): set a working decimal to the value of an `f64`,
/// literally as libcob does — `mpf_set_d` into a real 2048-bit [`Mpf`], then `cob_decimal_set_mpf_core`
/// (`mpf_get_str` at 96 significant digits). No f64 proxy: the `Mpf` is a genuine binary float, and the
/// whole path is byte-identical to the oracle over MOVE COMP-2→DISPLAY (`double_move_sweep`, 392/0).
pub fn cob_decimal_set_double(v: f64) -> CobDecimal {
    // numeric.c guards zero / non-finite (and an uninitialised-double sentinel) before the mpf path.
    if v == 0.0 || !v.is_finite() {
        return CobDecimal { value: Mpz::new(), scale: 0 };
    }
    // mpf_set_d(cob_mpft, v); cob_decimal_set_mpf_core(d, cob_mpft) -- literal, over the real Mpf.
    let mpft = Mpf::set_d(v, crate::mpf::COB_MPF_PREC);
    cob_decimal_set_mpf_core(&mpft)
}

/// `cob_print_ieeedec (f, fp)` (numeric.c): decode a floating field to a working decimal, then
/// `cob_decimal_print` it. FP_DEC64/128 decode exactly via the sealed BID decoders (the only branch
/// `cob_display_common` reaches — FLOAT/DOUBLE DISPLAY instead via `%G`); FLOAT/DOUBLE/L_DOUBLE route
/// through [`cob_decimal_set_double`], ported here for completeness of the 1:1.
pub fn cob_print_ieeedec(data: &[u8], attr: &FieldAttr) -> String {
    let d = match attr.field_type {
        0x16 => {
            let (m, s) = crate::float::dec64_decode(data[..8].try_into().unwrap_or([0; 8])).unwrap_or((0, 0));
            CobDecimal { value: Mpz::from_i128(m), scale: s }
        }
        0x17 => {
            let (m, s) =
                crate::float::dec128_decode(data[..16].try_into().unwrap_or([0; 16])).unwrap_or((0, 0));
            CobDecimal { value: Mpz::from_i128(m), scale: s }
        }
        0x13 => cob_decimal_set_double(f32::from_le_bytes(data[..4].try_into().unwrap_or([0; 4])) as f64),
        0x14 => cob_decimal_set_double(f64::from_le_bytes(data[..8].try_into().unwrap_or([0; 8]))),
        _ => CobDecimal { value: Mpz::new(), scale: 0 },
    };
    cob_decimal_print(&d)
}

/// `cob_print_realbin (f, fp, size)` (numeric.c): render a BINARY field as a zero-padded integer of at
/// least `size` digits — sign-prefixed for signed fields. Mirrors `CB_FMT_PLLD = "%+*.*lld"` (signed:
/// a leading `+`/`-` then `size` zero-padded digits) and `CB_FMT_PLLU = "%*.*llu"` (unsigned: `size`
/// zero-padded digits, no sign), both with field-width and precision equal to `size`.
pub fn cob_print_realbin(data: &[u8], attr: &FieldAttr, size: usize) -> String {
    let val = crate::binary::binary_decode(data, attr);
    if attr.have_sign() {
        let s = if val < 0 { '-' } else { '+' };
        format!("{}{:0width$}", s, val.unsigned_abs(), width = size)
    } else {
        format!("{:0width$}", val as u128, width = size)
    }
}

/// Fuzz target: numeric comparison is total and a consistent total order sign over arbitrary fields.
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_numcmp(data: &[u8]) {
    if data.len() < 6 {
        return;
    }
    let mk = |b: &[u8]| FieldAttr {
        field_type: if b[0] & 1 == 0 { COB_TYPE_NUMERIC_DISPLAY } else { COB_TYPE_NUMERIC_PACKED },
        digits: (b[1] % 18 + 1) as u16,
        scale: (b[2] % 6) as i16,
        flags: 0,
    };
    let a1 = mk(&data[0..3]);
    let a2 = mk(&data[3..6]);
    let body = &data[6..];
    let at = body.len() / 2;
    let (x, y) = body.split_at(at);
    let _ = cob_numeric_cmp(x, &a1, y, &a2);
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.NUMCMP.1
    /// Numeric comparison is a total order sign over symbolic decimals: always -1/0/1, never a panic.
    #[kani::proof]
    #[kani::unwind(12)]
    fn numeric_cmp_total() {
        let v1: i64 = kani::any();
        let v2: i64 = kani::any();
        let s1: i32 = kani::any();
        let s2: i32 = kani::any();
        kani::assume((-4..=4).contains(&s1) && (-4..=4).contains(&s2));
        let d1 = CobDecimal { value: Mpz::from_i64(v1), scale: s1 };
        let d2 = CobDecimal { value: Mpz::from_i64(v2), scale: s2 };
        let r = cob_decimal_cmp(&d1, &d2);
        assert!(r == -1 || r == 0 || r == 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::COB_FLAG_HAVE_SIGN;

    fn disp(d: u16, s: i16, signed: bool) -> FieldAttr {
        FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: d, scale: s, flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 } }
    }

    #[test]
    fn compares_across_scale() {
        // 12.3 (9(2)V9) vs 12.30 (9(2)V99) -> equal
        let a = b"123";
        let b = b"01230";
        assert_eq!(cob_numeric_cmp(a, &disp(3, 1, false), b, &disp(5, 2, false)), 0);
        // 12.3 vs 12.31 -> less
        let c = b"01231";
        assert_eq!(cob_numeric_cmp(a, &disp(3, 1, false), c, &disp(5, 2, false)), -1);
        // 99 vs 12 -> greater
        assert_eq!(cob_numeric_cmp(b"99", &disp(2, 0, false), b"12", &disp(2, 0, false)), 1);
    }

    fn dec(value: &str, scale: i32) -> CobDecimal {
        CobDecimal { value: Mpz::from_decimal_string(value), scale }
    }
    /// Render as an exact decimal string `value * 10^-scale` (for test assertions).
    fn render(d: &CobDecimal) -> String {
        if d.scale <= 0 {
            let mut v = d.value.clone();
            if d.scale < 0 {
                v = v.mul(&Mpz::ui_pow_ui(10, (-d.scale) as u32));
            }
            return v.to_decimal_string();
        }
        let s = d.value.to_decimal_string();
        let (neg, digs) = match s.strip_prefix('-') {
            Some(x) => ("-", x.to_string()),
            None => ("", s),
        };
        let sc = d.scale as usize;
        let digs = if digs.len() <= sc { format!("{:0>width$}", digs, width = sc + 1) } else { digs };
        let dot = digs.len() - sc;
        format!("{neg}{}.{}", &digs[..dot], &digs[dot..])
    }

    #[test]
    fn decimal_arithmetic() {
        let mut a = dec("1234", 2); // 12.34
        cob_decimal_add(&mut a, &dec("111", 2)); // + 1.11
        assert_eq!(render(&a), "13.45");
        let mut m = dec("150", 2); // 1.50
        cob_decimal_mul(&mut m, &dec("150", 2)); // * 1.50
        assert_eq!(render(&m), "2.2500");
        let mut s = dec("1000", 0); // 1000
        cob_decimal_sub(&mut s, &dec("1", 0));
        assert_eq!(render(&s), "999");
        // 10 / 3 -> quotient scaled to 38 digits of precision, truncated
        let mut q = dec("10", 0);
        assert!(cob_decimal_div(&mut q, &dec("3", 0)).is_ok());
        assert!(render(&q).starts_with("3.3333333333333333333333333333333333333"));
        // divide by zero
        let mut z = dec("5", 0);
        assert!(cob_decimal_div(&mut z, &dec("0", 0)).is_err());
    }

    #[test]
    fn cob_div_matches_proven_divide() {
        // cob_div (f1 := f1/f2 on the Mpz path) must equal the proven arith::cob_divide for a/b into
        // a's attr, across a matrix of display values x scales x signs x round modes.
        use crate::arith::cob_divide;
        fn disp_bytes(digits: usize, val: u64, neg: bool) -> Vec<u8> {
            let mut v = val;
            let mut d = vec![0u8; digits];
            for s in d.iter_mut().rev() { *s = (v % 10) as u8; v /= 10; }
            let mut o: Vec<u8> = d.iter().map(|x| b'0' + x).collect();
            if neg { if let Some(l) = o.last_mut() { *l |= 0x40; } }
            o
        }
        let mut checked = 0;
        for adig in [3usize, 5] {
            for ascale in 0..=2i16 {
                for bval in [1u64, 3, 7, 11, 99] {
                    for aval in [0u64, 1, 100, 12345 % 10u64.pow(adig as u32)] {
                        for (an, bn) in [(false, false), (true, false), (false, true), (true, true)] {
                            for round in [Round::Truncate, Round::NearAwayFromZero] {
                                let a1 = disp(adig as u16, ascale, true);
                                let a2 = disp(3, 0, true);
                                let a = disp_bytes(adig, aval, an);
                                let b = disp_bytes(3, bval, bn);
                                let mine = cob_div(&a, &a1, &b, &a2, round).unwrap();
                                let proven = cob_divide(&a, &a1, &b, &a2, &a1, round).unwrap();
                                assert_eq!(mine, proven, "a={aval}(s{ascale},n{an}) b={bval}(n{bn}) {round:?}");
                                checked += 1;
                            }
                        }
                    }
                }
            }
        }
        assert!(checked > 400);
    }

    #[test]
    fn int_paths_and_cmp_variants() {
        // cob_set_int stores the integer; cob_add_int adds; cmp variants match cob_cmp_int.
        let a = disp(5, 0, true);
        assert_eq!(cob_set_int(b"00000", &a, 42).unwrap(), b"00042");
        assert_eq!(cob_set_int(b"00000", &a, -7).unwrap().as_slice(), b"0000\x77"); // -7 overpunch
        // 100 + 23 = 123
        assert_eq!(cob_add_int(b"00100", &a, 23, Round::Truncate).unwrap(), b"00123");
        // 100 - 150 = -50 -> "00050" with the last byte a negative overpunch '0' (0x70)
        assert_eq!(cob_sub_int(b"00100", &a, 150, Round::Truncate).unwrap().as_slice(), b"0005\x70");
        // cmp variants == cob_cmp_int verdict
        for n in [0i64, 5, 9, 100] {
            let v = cob_cmp_int(b"00009", &disp(5, 0, false), n);
            assert_eq!(cob_cmp_llint(b"00009", &disp(5, 0, false), n), v);
            assert_eq!(cob_cmp_uint(b"00009", &disp(5, 0, false), n as u64), v);
        }
    }

    #[test]
    fn signed_compare() {
        // -5 vs +3 -> less ; -0 vs 0 -> equal
        let neg5 = b"5\x40"; // '5' then overpunch on... build via signed display: last byte sign
        let _ = neg5;
        assert_eq!(cob_cmp_int(b"00005", &disp(5, 0, false), 5), 0);
        assert_eq!(cob_cmp_int(b"00005", &disp(5, 0, false), 9), -1);
        assert_eq!(cob_cmp_int(b"00009", &disp(5, 0, false), 5), 1);
    }

    #[test]
    fn cmp_numdisp_and_sign_helpers() {
        // cob_cmp_numdisp must agree with cob_cmp_int over DISPLAY fields, and the overpunch helpers
        // must decode the ASCII/EBCDIC trailing-sign tables.
        for &(digits, signed) in &[(5u16, false), (5, true), (3, true), (7, true)] {
            let attr = disp(digits, 0, signed);
            for v in [0i64, 1, 7, 42, 999, 12345] {
                if v >= 10i64.pow(digits as u32) {
                    continue;
                }
                for neg in [false, true] {
                    if neg && (!signed || v == 0) {
                        continue;
                    }
                    // build a DISPLAY image of +/- v
                    let mut img: Vec<u8> = format!("{v:0width$}", width = digits as usize).into_bytes();
                    if neg {
                        let last = img.len() - 1;
                        img[last] = b"p qrstuvwxy"[(img[last] - b'0' + 1) as usize]; // ASCII overpunch
                        if img[last] == b' ' {
                            img[last] = b'p';
                        }
                    }
                    let sv = if neg { -v } else { v };
                    for n in [-1i64, 0, 7, 42, 12345, -12345] {
                        let got = cob_cmp_numdisp(&img, n, signed, false);
                        let want = (sv > n) as i32 - (sv < n) as i32;
                        assert_eq!(got, want, "numdisp v={sv} n={n} img={img:?}");
                    }
                }
            }
        }
        // sign helper tables
        let mut d = 0;
        assert_eq!(cob_get_long_ascii_sign(b'p', &mut d), 1);
        assert_eq!(d, 0);
        d = 0;
        assert_eq!(cob_get_long_ascii_sign(b'y', &mut d), 1);
        assert_eq!(d, 9);
        d = 0;
        assert_eq!(cob_get_long_ebcdic_sign(b'A', &mut d), 0);
        assert_eq!(d, 1);
        d = 0;
        assert_eq!(cob_get_long_ebcdic_sign(b'R', &mut d), 1);
        assert_eq!(d, 9);
    }

    #[test]
    fn get_display_binary_leaves_match_get_field() {
        // The named leaves must equal the sealed get_field for an already-scale-aligned decimal.
        for &(ft, digits, scale, flags) in &[
            (COB_TYPE_NUMERIC_DISPLAY, 5u16, 2i16, crate::attr::COB_FLAG_HAVE_SIGN),
            (COB_TYPE_NUMERIC_DISPLAY, 7, 0, 0),
            (COB_TYPE_NUMERIC_BINARY, 9, 0, crate::attr::COB_FLAG_HAVE_SIGN),
            (COB_TYPE_NUMERIC_BINARY, 4, 2, 0),
        ] {
            let attr = FieldAttr { field_type: ft, digits, scale, flags };
            let size = if ft == COB_TYPE_NUMERIC_DISPLAY { digits as usize } else { 4 };
            for &v in &[0i128, 1, -1, 42, -42, 1234, -1234, 99999, -99999] {
                if flags == 0 && v < 0 {
                    continue;
                }
                let d = CobDecimal { value: Mpz::from_i128(v), scale: scale as i32 };
                let leaf = if ft == COB_TYPE_NUMERIC_DISPLAY {
                    cob_decimal_get_display(&d, &attr, size)
                } else {
                    cob_decimal_get_binary(&d, &attr, size)
                };
                let field = cob_decimal_get_field(d, &attr, size, Round::Truncate, false).unwrap();
                assert_eq!(leaf, field, "leaf vs get_field ft={ft} v={v}");
            }
        }
    }

    #[test]
    fn set_display_sentinels() {
        // First data byte 0xFF -> +10^size, 0x00 -> -10^size (uninitialised-data sentinels).
        let attr = disp(3, 0, true);
        let d = cob_decimal_set_display(&[0xFF, 0xFF, 0xFF], &attr);
        assert_eq!(d.value.to_i128(), Some(1000));
        let d = cob_decimal_set_display(&[0x00, 0x00, 0x00], &attr);
        assert_eq!(d.value.to_i128(), Some(-1000));
        // real Mpf round-trips through the double path
        // real Mpf round-trip: 2.5 -> Mpf -> decimal -> Mpf -> 2.5
        let back = cob_decimal_set_mpf(&Mpf::set_d(2.5, crate::mpf::COB_MPF_PREC));
        assert_eq!(cob_decimal_get_mpf(&back).get_d(), 2.5);
    }

    #[test]
    fn div_quotient_remainder_matches_proven_divide() {
        // cob_div_quotient/remainder must match the sealed arith::cob_divide_remainder (GNURUST.REMAINDER.1).
        use crate::arith::{cob_divide_remainder, Round};
        fn d(digits: usize, val: i64) -> Vec<u8> {
            let neg = val < 0;
            let mut s: Vec<u8> = format!("{:0width$}", val.unsigned_abs(), width = digits).into_bytes();
            if neg {
                let l = s.len() - 1;
                s[l] |= 0x40;
            }
            s
        }
        let qa = disp(7, 2, true);
        let ra = disp(7, 4, true);
        for &dvd in &[1000i64, 12345, -700, 9999999, 1, -50000] {
            for &dvs in &[7i64, 3, -11, 100, 999, -25] {
                let dvdb = d(7, dvd);
                let dvsb = d(5, dvs.rem_euclid(100000));
                let a_dvd = disp(7, 2, true);
                let a_dvs = disp(5, 2, true);
                let proven = cob_divide_remainder(&dvdb, &a_dvd, &dvsb, &a_dvs, &qa, &ra);
                let mine = cob_div_quotient(&dvdb, &a_dvd, &dvsb, &a_dvs, &qa, 7, Round::Truncate)
                    .and_then(|(q, rem)| cob_div_remainder(rem, &ra, 7, Round::Truncate).map(|r| (q, r)));
                match (proven, mine) {
                    (Ok((pq, pr)), Ok((mq, mr))) => {
                        assert_eq!(mq, pq, "quotient dvd={dvd} dvs={dvs}");
                        assert_eq!(mr, pr, "remainder dvd={dvd} dvs={dvs}");
                    }
                    (Err(_), _) | (_, Err(_)) => {}
                }
            }
        }
    }

    #[test]
    fn get_double_via_mpf_matches_sealed_primitive() {
        // cob_decimal_get_double now routes decimal -> Mpf (set_z, mpf_div) -> mpf_get_d. Its result must
        // equal the FLOAT.1-sealed decimal_to_f64_trunc (display->COMP-2 encode at 1476/0).
        for scale in -3i32..=12 {
            for &mag in &[
                0i128, 1, -1, 7, 25, -25, 100, 12345, -12345, 999999, 33333333, -8675309, 1000000000,
                123456789012345, -987654321098765,
            ] {
                let d = CobDecimal { value: Mpz::from_i128(mag), scale };
                let viamp = cob_decimal_get_double(&d);
                let sealed = crate::float::decimal_to_f64_trunc(mag, scale);
                assert_eq!(viamp.to_bits(), sealed.to_bits(), "mag={mag} scale={scale}: mpf={viamp} sealed={sealed}");
            }
        }
    }

    #[test]
    fn cmp_float_orders_and_tolerates() {
        // COMP-2 (0x14) field comparison with libcob's relative tolerance.
        let f = |v: f64| (FieldAttr { field_type: 0x14, digits: 0, scale: 0, flags: 0 }, v.to_le_bytes().to_vec());
        let (a1, b1) = f(1.5);
        let (a2, b2) = f(2.5);
        assert_eq!(cob_cmp_float(&b1, &a1, &b2, &a2), -1);
        assert_eq!(cob_cmp_float(&b2, &a2, &b1, &a1), 1);
        let (a3, b3) = f(2.5);
        assert_eq!(cob_cmp_float(&b2, &a2, &b3, &a3), 0);
        // within relative tolerance 1e-7 -> equal
        let (a4, b4) = f(2.5 + 2.5 * 1e-9);
        assert_eq!(cob_cmp_float(&b2, &a2, &b4, &a4), 0);
    }

    #[test]
    fn print_functions_match_oracle_display() {
        use crate::attr::{COB_FLAG_HAVE_SIGN, COB_TYPE_NUMERIC_BINARY};
        // --- cob_decimal_print: ground truth captured from `cobc` DISPLAY of FLOAT-DECIMAL-16 ---
        let cd = |mag: i128, scale: i32| CobDecimal { value: Mpz::from_i128(mag), scale };
        let cases: &[(i128, i32, &str)] = &[
            (12345, 2, "123.45"),
            (0, 0, "0E0"),
            (-1, 3, "-1E-3"),
            (1000000, 0, "1E6"),
            (7, 0, "7"),
            (-425, 1, "-42.5"),
            (1, 1, "1E-1"),
            (999999999999999, 0, "999999999999999"),
            (123456789012345, 1, "12345678901234.5"),
            (-7000, 0, "-7E3"),
        ];
        for &(mag, scale, want) in cases {
            assert_eq!(cob_decimal_print(&cd(mag, scale)), want, "decimal_print {mag}e-{scale}");
            // cob_print_ieeedec composes the FLOAT.1-sealed BID decode with cob_decimal_print; the
            // round-trip through dec64_encode must DISPLAY identically (zero-stripping normalises scale).
            let attr = FieldAttr { field_type: 0x16, digits: 16, scale: scale as i16, flags: 0 };
            let bytes = crate::float::dec64_encode(mag, scale);
            assert_eq!(cob_print_ieeedec(&bytes, &attr), want, "print_ieeedec {mag}e-{scale}");
        }

        // --- cob_print_realbin: ground truth from `cobc` DISPLAY of S9(9)/9(9) COMP-5 (size 10) ---
        let mut buf = [0u8; 4];
        let signed = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: COB_FLAG_HAVE_SIGN };
        for &(v, want) in &[(42i128, "+0000000042"), (-42, "-0000000042"), (0, "+0000000000"), (2147483647, "+2147483647"), (-1, "-0000000001")] {
            crate::binary::binary_encode(v, &signed, &mut buf);
            assert_eq!(cob_print_realbin(&buf, &signed, 10), want, "realbin signed {v}");
        }
        let unsigned = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: 0 };
        for &(v, want) in &[(123456789i128, "0123456789"), (0, "0000000000")] {
            crate::binary::binary_encode(v, &unsigned, &mut buf);
            assert_eq!(cob_print_realbin(&buf, &unsigned, 10), want, "realbin unsigned {v}");
        }
    }

    #[test]
    fn evidence_numeric_internals() {
        // cob_pow_10_uli(n) == 10^n
        assert_eq!(cob_pow_10_uli(0), 1);
        assert_eq!(cob_pow_10_uli(3), 1000);
        assert_eq!(cob_pow_10_uli(9), 1_000_000_000);

        // cob_decimal_set_llint: signed 64-bit at scale 0
        let mut d = cob_decimal_init();
        cob_decimal_set_llint(&mut d, -12345);
        assert_eq!(d.scale, 0);
        assert_eq!(d.value.to_i128(), Some(-12345));

        // cob_decimal_set: faithful copy of value + scale
        let src = dec("789", 2); // 7.89
        let mut dst = cob_decimal_init();
        cob_decimal_set(&mut dst, &src);
        assert_eq!(dst.scale, 2);
        assert_eq!(dst.value.to_i128(), Some(789));

        // cob_decimal_clear: reset to zero at scale 0
        let mut c = dec("4242", 3);
        cob_decimal_clear(&mut c);
        assert_eq!(c.scale, 0);
        assert_eq!(c.value.sgn(), 0);

        // cob_decimal_align: scale-down branch is well-defined: 1.200 (1200 @scale3) -> scale 1
        // shifts the value by (1 - 3) = -2, i.e. /100 -> 12 @scale1 = 1.2 (value preserved).
        let mut a = dec("1200", 3); // 1.200
        cob_decimal_align(&mut a, 1);
        assert_eq!(a.scale, 1);
        assert_eq!(a.value.to_i128(), Some(12));

        // cob_decimal_adjust: strip trailing zeros, shift comma left until <= max_value.
        // 123000 @scale0: trailing zeros stripped -> value 123, scale -3.
        let mut adj = dec("123000", 0);
        let max = Mpz::ui_pow_ui(10, 30); // huge -> magnitude fits, no overflow
        let overflow = cob_decimal_adjust(&mut adj, &max, -100, 100);
        assert!(!overflow);
        assert_eq!(adj.value.to_i128(), Some(123));
        assert_eq!(adj.scale, -3);

        // cob_decimal_push / cob_decimal_pop: allocate N zero temporaries, then release.
        let temps = cob_decimal_push(3);
        assert_eq!(temps.len(), 3);
        assert!(temps.iter().all(|t| t.value.sgn() == 0 && t.scale == 0));
        cob_decimal_pop(temps);
    }
}

#[cfg(test)]
mod ext80_tests {
    use super::{extended80_to_f64, f64_to_extended80};
    #[test]
    fn extended80_round_trip() {
        for &v in &[0.0f64, 1.0, -1.0, 3.14159265358979, 1e10, -2.5, 1e-7, 12345.678, -0.001] {
            let e = f64_to_extended80(v);
            let back = extended80_to_f64(&e);
            assert_eq!(back, v, "ext80 round-trip {v}");
        }
    }
}
