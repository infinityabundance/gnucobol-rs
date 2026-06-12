//! 1:1 port of move.c's "Convenience functions" — the `cob_get_*` / `cob_set_*` accessor API that C
//! application code (and the COBOL runtime) uses to read/write a `cob_field` as a host integer. Each
//! decodes/encodes via the sealed field codecs; `cob_get_int`/`cob_get_llint` truncate the fractional
//! part ("removes decimal part — per design", move.c:2063).
#![forbid(unsafe_code)]

use crate::attr::{FieldAttr, COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED};
use crate::sign;

fn pow10_i64(n: u32) -> i64 {
    10i64.checked_pow(n).unwrap_or(i64::MAX)
}

/// `cob_packed_get_int (field)` (move.c:1871): decode a PACKED/COMP-6 field to a host `int`, ignoring
/// the field scale (integer part of the stored nibbles), sign from the trailing nibble.
pub fn cob_packed_get_int(data: &[u8], attr: &FieldAttr) -> i32 {
    packed_get_ll_raw(data, attr) as i32
}

/// `packed_get_long_long (field)` (move.c:1923): like [`cob_packed_get_int`] as `i64`, then scaled by
/// the field scale (`/10^scale`, or `*10^-scale` for `P`-scaled).
pub fn packed_get_long_long(data: &[u8], attr: &FieldAttr) -> i64 {
    let val = packed_get_ll_raw(data, attr);
    let scale = attr.scale as i32;
    if scale < 0 {
        val.wrapping_mul(pow10_i64((-scale) as u32))
    } else {
        val / pow10_i64(scale as u32)
    }
}

/// The shared nibble-decode of move.c's packed accessors (the integer value of the stored digits,
/// signed; scale NOT applied).
fn packed_get_ll_raw(data: &[u8], attr: &FieldAttr) -> i64 {
    let p2b = &crate::packed::PACK_TO_BIN;
    let size = data.len();
    let d_end = size - 1; // index of the last byte
    let mut di = 0usize;
    if attr.no_sign_nibble() {
        // COMP-6
        let offset = attr.digits as usize % 2;
        let mut val: i64 = if offset == 1 {
            let v = (data[di] & 0x0F) as i64;
            di += 1;
            v
        } else {
            0
        };
        if val == 0 {
            while di <= d_end && data[di] == 0x00 {
                di += 1;
            }
        }
        while di <= d_end {
            val = val * 100 + p2b[data[di] as usize] as i64;
            di += 1;
        }
        val
    } else {
        // PACKED-DECIMAL / COMP-3
        let offset = 1 - attr.digits as usize % 2;
        let mut val: i64 = if offset == 1 {
            let v = (data[di] & 0x0F) as i64;
            di += 1;
            v
        } else {
            0
        };
        if val == 0 {
            while di < d_end && data[di] == 0x00 {
                di += 1;
            }
        }
        while di < d_end {
            val = val * 100 + p2b[data[di] as usize] as i64;
            di += 1;
        }
        val = val * 10 + (data[d_end] >> 4) as i64;
        if (data[d_end] & 0x0F) == 0x0D {
            val = -val;
        }
        val
    }
}

/// `cob_display_get_int (f)` (move.c:1981): decode a DISPLAY field to a host `int`, skipping leading
/// zero-like bytes, dropping the fractional digits, and applying the (de-overpunched) sign.
pub fn cob_display_get_int(data: &[u8], attr: &FieldAttr) -> i32 {
    display_get_ll(data, attr) as i32
}

/// `display_get_long_long (f)` (move.c:2018): the `i64` form, with the field scale applied (`P`-scaled
/// multiplies, otherwise the fractional digits are simply not consumed).
pub fn display_get_long_long(data: &[u8], attr: &FieldAttr) -> i64 {
    display_get_ll(data, attr)
}

fn display_get_ll(data: &[u8], attr: &FieldAttr) -> i64 {
    let scale = attr.scale as i32;
    let off = attr.data_offset();
    let full = attr.data_size(data.len());
    let mut tmp = data.to_vec();
    let sign = sign::display_get_sign_strip(&mut tmp, attr); // COB_GET_SIGN_ADJUST
    let d = tmp.get(off..off + full).unwrap_or(&[]);
    let size0 = d.len();
    let mut i = 0usize;
    while i < size0 && sign::d2i(d[i]) == 0 {
        i += 1;
    }
    let mut val: i64 = 0;
    if scale < 0 {
        while i < size0 {
            val = val * 10 + sign::d2i(d[i]) as i64;
            i += 1;
        }
        val = val.wrapping_mul(pow10_i64((-scale) as u32));
    } else {
        let size = size0.saturating_sub(scale as usize);
        while i < size {
            val = val * 10 + sign::d2i(d[i]) as i64;
            i += 1;
        }
    }
    if sign < 0 {
        -val
    } else {
        val
    }
}

/// `cob_get_int (f)` (move.c:2064): read any numeric field as a host `int`, fractional part removed.
pub fn cob_get_int(data: &[u8], attr: &FieldAttr) -> i32 {
    match attr.field_type {
        COB_TYPE_NUMERIC_DISPLAY => cob_display_get_int(data, attr),
        COB_TYPE_NUMERIC_PACKED => cob_packed_get_int(data, attr),
        COB_TYPE_NUMERIC_BINARY => {
            let val = crate::binary::binary_decode(data, attr) as i64;
            let scale = attr.scale as i32;
            if scale == 0 {
                val as i32
            } else if scale > 0 {
                (val / pow10_i64(scale as u32)) as i32
            } else {
                val.wrapping_mul(pow10_i64((-scale) as u32)) as i32
            }
        }
        // default (float/edited/alphanumeric): cob_move into a binary int receiver
        _ => cob_get_llint(data, attr) as i32,
    }
}

/// `cob_get_llint (f)` (move.c:2106): read any numeric field as a host `cob_s64_t`, fractional removed.
pub fn cob_get_llint(data: &[u8], attr: &FieldAttr) -> i64 {
    match attr.field_type {
        COB_TYPE_NUMERIC_DISPLAY => display_get_long_long(data, attr),
        COB_TYPE_NUMERIC_PACKED => packed_get_long_long(data, attr),
        COB_TYPE_NUMERIC_BINARY => {
            let val = crate::binary::binary_decode(data, attr) as i64;
            let scale = attr.scale as i32;
            if scale == 0 {
                val
            } else if scale > 0 {
                val / pow10_i64(scale as u32)
            } else {
                val.wrapping_mul(pow10_i64((-scale) as u32))
            }
        }
        // default: move through the decimal layer, then truncate to integer.
        _ => {
            let d = crate::cob_decimal::cob_decimal_set_field(data, attr);
            let mut v = d.clone();
            crate::cob_decimal::shift_decimal(&mut v, -(d.scale));
            v.value.to_i128().unwrap_or(0) as i64
        }
    }
}

/// `cob_set_int (f, n)` (move.c:2055): store a host `int` into a field — `cob_move` from a temp COMP-5
/// receiver holding `n` at scale 0.
pub fn cob_set_int(out: &mut [u8], attr: &FieldAttr, n: i32) -> Result<(), crate::error::DecimalError> {
    let bin = n.to_le_bytes();
    let battr = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: crate::attr::COB_FLAG_HAVE_SIGN | crate::attr::COB_FLAG_REAL_BINARY };
    crate::move_ops::cob_move(&bin, &battr, out, attr)
}

/// `cob_set_llint (f, n)` (move.c:2096): store a host `cob_s64_t` into a field via `cob_move` from a
/// temp 8-byte COMP-5 receiver.
pub fn cob_set_llint(out: &mut [u8], attr: &FieldAttr, n: i64) -> Result<(), crate::error::DecimalError> {
    let bin = n.to_le_bytes();
    let battr = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 18, scale: 0, flags: crate::attr::COB_FLAG_HAVE_SIGN | crate::attr::COB_FLAG_REAL_BINARY };
    crate::move_ops::cob_move(&bin, &battr, out, attr)
}

/// `cob_init_move (lptr, sptr)` (move.c:2138): module init binding the runtime globals. A no-op in this
/// port (no mutable runtime globals; module config is passed explicitly).
pub fn cob_init_move() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::{COB_FLAG_HAVE_SIGN, COB_TYPE_NUMERIC_PACKED};

    fn disp(digits: u16, scale: i16, signed: bool) -> FieldAttr {
        FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits, scale, flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 } }
    }

    #[test]
    fn get_int_display_and_packed_match_decoders() {
        // cob_get_int/llint must equal the sealed decode -> integer truncation.
        for &(v, dg, sc) in &[(12345i64, 5, 0), (12345, 7, 2), (700, 5, 0), (-4200, 6, 2), (0, 4, 0), (-1, 5, 0)] {
            let a = disp(dg, sc, true);
            // build a zoned image of v at scale sc
            let intpart = v / 10i64.pow(sc.max(0) as u32);
            let neg = v < 0;
            let mut img: Vec<u8> = format!("{:0width$}", v.unsigned_abs(), width = dg as usize).into_bytes();
            img.truncate(dg as usize);
            while img.len() < dg as usize {
                img.insert(0, b'0');
            }
            if neg {
                let l = img.len() - 1;
                img[l] |= 0x40;
            }
            assert_eq!(cob_get_int(&img, &a) as i64, intpart, "display get_int v={v} dg={dg} sc={sc}");
            assert_eq!(cob_get_llint(&img, &a), intpart, "display get_llint v={v}");
        }
        // packed: cross-check against the sealed packed decoder
        let pa = FieldAttr { field_type: COB_TYPE_NUMERIC_PACKED, digits: 7, scale: 2, flags: COB_FLAG_HAVE_SIGN };
        for &v in &[0i64, 123, -123, 99999, -99999] {
            let mut img = vec![0u8; 4];
            crate::packed::cob_set_packed_int(&mut img, &pa, v as i32);
            let want = v / 100; // scale 2 truncation
            assert_eq!(cob_get_llint(&img, &pa), want, "packed get_llint v={v}");
        }
    }
}
