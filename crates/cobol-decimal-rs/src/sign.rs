//! Sign handling, ported faithfully from `libcob`. **ASCII host only** (`COB_EBCDIC_MACHINE`
//! off, `ebcdic_sign = 0`): the EBCDIC-host sign path is a classified non-claim.
//!
//! Citations: `cob_get_sign_ascii` / `cob_put_sign_ascii` (`common.c:1450` / `:1516`, non-EBCDIC
//! branches), `cob_real_get_sign` / `cob_real_put_sign` (`common.c:3712` / `:3763`),
//! `locate_sign` (`common.c:3693`), `cob_packed_get_sign` (`numeric.c:967`), and the
//! `COB_D2I`/`COB_I2D`/`IS_VALID_DIGIT_DATA` macros (`coblocal.h:243-244`, `common.c:428`).
//! Evidence kind: **source_port**.

use crate::attr::FieldAttr;

/// `COB_D2I(x) = (x) & 0x0F` (`coblocal.h:243`).
#[inline]
pub const fn d2i(x: u8) -> u8 {
    x & 0x0F
}

/// `COB_I2D(x) = '0' + x` (`coblocal.h:244`).
#[inline]
pub const fn i2d(x: u8) -> u8 {
    b'0'.wrapping_add(x)
}

/// `IS_VALID_DIGIT_DATA(c) = (unsigned char)(c - '0') <= 9` (`common.c:428`).
#[inline]
pub const fn is_valid_digit_data(c: u8) -> bool {
    c.wrapping_sub(b'0') <= 9
}

/// Index of the sign byte within a DISPLAY field's storage — `locate_sign` (`common.c:3693`):
/// leading → byte 0, otherwise the last byte.
#[inline]
pub fn locate_sign_index(attr: &FieldAttr, full_size: usize) -> usize {
    if attr.sign_leading() {
        0
    } else {
        full_size.saturating_sub(1)
    }
}

/// `cob_get_sign_ascii` (`common.c:1450`, non-EBCDIC branch). Returns the sign and the byte with
/// any negative overpunch removed: `'p'..'y'` (0x70..0x79) → `&= ~0x40` and sign `-1`; a plain
/// digit → unchanged, sign `+1`; anything else → forced to `'0'`, sign `+1`.
#[inline]
pub fn get_sign_ascii(b: u8) -> (i32, u8) {
    if (b'p'..=b'y').contains(&b) {
        (-1, b & !0x40)
    } else if is_valid_digit_data(b) {
        (1, b)
    } else {
        (1, b'0')
    }
}

/// `cob_put_sign_ascii` (`common.c:1516`, non-EBCDIC branch): overpunch a negative sign by
/// setting bit `0x40` on the sign byte (`'0'`→`'p'`, …).
#[inline]
pub const fn put_sign_ascii(b: u8) -> u8 {
    b | 0x40
}

/// `cob_real_get_sign(f, adjust_ebcdic=0)` for a DISPLAY field on an ASCII host
/// (`common.c:3712-3750`), the variant used by `cob_move_display_to_display`. Operates on a
/// mutable copy of the field data: when the sign byte is overpunched it is **stripped in place**
/// (mirroring upstream, which mutates then restores `f1`). Returns the sign (`0` when unsigned).
pub fn display_get_sign_strip(data: &mut [u8], attr: &FieldAttr) -> i32 {
    if !attr.have_sign() {
        return 0;
    }
    let idx = locate_sign_index(attr, data.len());
    let p = match data.get(idx) {
        Some(&b) => b,
        None => return 1,
    };
    if attr.sign_separate() {
        return if p == b'-' { -1 } else { 1 };
    }
    if is_valid_digit_data(p) {
        return 1;
    }
    if p == b' ' {
        return 1;
    }
    let (sign, cleaned) = get_sign_ascii(p);
    data[idx] = cleaned;
    sign
}

/// `cob_real_get_sign(f, adjust_ebcdic=1)` for a DISPLAY field on an ASCII host
/// (`common.c:3712`, the `adjust_ebcdic` branch at `:3743`), used by `COB_GET_SIGN_ADJUST` inside
/// `cob_move_display_to_packed`. This variant does **not** modify the data: it reads the sign by
/// testing the high nibble (`(*p & 0xF0) == 0x70`), and the digit itself is later recovered via
/// the low-nibble `COB_D2I`. Returns the sign (`0` when unsigned).
pub fn display_get_sign_adjust_readonly(data: &[u8], attr: &FieldAttr) -> i32 {
    if !attr.have_sign() {
        return 0;
    }
    let idx = locate_sign_index(attr, data.len());
    let p = match data.get(idx) {
        Some(&b) => b,
        None => return 1,
    };
    if attr.sign_separate() {
        return if p == b'-' { -1 } else { 1 };
    }
    if is_valid_digit_data(p) {
        return 1;
    }
    if p == b' ' {
        return 1;
    }
    if (p & 0xF0) == 0x70 {
        -1
    } else {
        1
    }
}

/// `cob_real_put_sign(f, sign)` for a DISPLAY field on an ASCII host (`common.c:3763-3784`):
/// `SIGN SEPARATE` writes `'+'`/`'-'`; otherwise a negative sign overpunches the sign byte. A
/// non-negative sign on a non-separate field is a no-op. Unsigned fields (`!have_sign`) do nothing.
pub fn display_put_sign(data: &mut [u8], attr: &FieldAttr, sign: i32) {
    if !attr.have_sign() {
        return;
    }
    let idx = locate_sign_index(attr, data.len());
    if attr.sign_separate() {
        let c = if sign == -1 { b'-' } else { b'+' };
        if let Some(slot) = data.get_mut(idx) {
            *slot = c;
        }
    } else if sign == -1 {
        if let Some(slot) = data.get_mut(idx) {
            *slot = put_sign_ascii(*slot);
        }
    }
}

/// `cob_packed_get_sign` (`numeric.c:967`): for a signed packed field, the low nibble of the last
/// byte is `0x0D` → `-1`, else `+1`; unsigned → `0`.
#[inline]
pub fn packed_get_sign(data: &[u8], attr: &FieldAttr) -> i32 {
    if !attr.have_sign() {
        return 0;
    }
    match data.last() {
        Some(&b) if (b & 0x0F) == 0x0D => -1,
        _ => 1,
    }
}
