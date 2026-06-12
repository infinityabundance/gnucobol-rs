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

// ---------------------------------------------------------------------------------------------------
// Typed raw-memory accessors (move.c:2148+): the `cob_get/put_<u64|s64>_<compx|comp5|comp3|comp6|pic9>`
// + comp1/comp2/picx/pointer routines C application code uses to read/write a value at a byte address.
// COMP-X is big-endian, COMP-5 native little-endian (this is a little-endian host). `ebcdic_sign` (a
// module global in C) is an explicit parameter here.
// ---------------------------------------------------------------------------------------------------

/// Sign-extend the low `len*8` bits of `u` to `i64`.
fn sign_extend(u: u64, len: usize) -> i64 {
    let bits = len * 8;
    if bits < 64 && (u >> (bits - 1)) & 1 == 1 {
        (u | (!0u64 << bits)) as i64
    } else {
        u as i64
    }
}

/// `cob_put_u64_compx` (move.c:2149): write `val`'s low `len` bytes big-endian.
pub fn cob_put_u64_compx(val: u64, mem: &mut [u8], len: usize) {
    let be = val.to_be_bytes();
    mem[..len].copy_from_slice(&be[8 - len..]);
}
/// `cob_put_u64_comp5` (move.c:2209): write `val`'s low `len` bytes native little-endian.
pub fn cob_put_u64_comp5(val: u64, mem: &mut [u8], len: usize) {
    let le = val.to_le_bytes();
    mem[..len].copy_from_slice(&le[..len]);
}
/// `cob_put_s64_compx` (move.c:2243): signed value, big-endian (same two's-complement bytes).
pub fn cob_put_s64_compx(val: i64, mem: &mut [u8], len: usize) {
    cob_put_u64_compx(val as u64, mem, len);
}
/// `cob_put_s64_comp5` (move.c:2303): signed value, native little-endian.
pub fn cob_put_s64_comp5(val: i64, mem: &mut [u8], len: usize) {
    cob_put_u64_comp5(val as u64, mem, len);
}
/// `cob_get_u64_compx` (move.c:2337): read `len` big-endian bytes as unsigned.
pub fn cob_get_u64_compx(mem: &[u8], len: usize) -> u64 {
    let mut b = [0u8; 8];
    b[8 - len..].copy_from_slice(&mem[..len]);
    u64::from_be_bytes(b)
}
/// `cob_get_u64_comp5` (move.c:2392): read `len` little-endian bytes as unsigned.
pub fn cob_get_u64_comp5(mem: &[u8], len: usize) -> u64 {
    let mut b = [0u8; 8];
    b[..len].copy_from_slice(&mem[..len]);
    u64::from_le_bytes(b)
}
/// `cob_get_s64_compx` (move.c:2466): read `len` big-endian bytes, sign-extended.
pub fn cob_get_s64_compx(mem: &[u8], len: usize) -> i64 {
    sign_extend(cob_get_u64_compx(mem, len), len)
}
/// `cob_get_s64_comp5` (move.c:2426): read `len` little-endian bytes, sign-extended.
pub fn cob_get_s64_comp5(mem: &[u8], len: usize) -> i64 {
    sign_extend(cob_get_u64_comp5(mem, len), len)
}

/// `cob_put_s64_comp3` (move.c:2531): pack a signed value into `len` BCD bytes (trailing sign nibble
/// `0x0C`/`0x0D`).
pub fn cob_put_s64_comp3(val: i64, mem: &mut [u8], len: usize) {
    let (mut num, sign) = if val < 0 { ((val as i128).unsigned_abs() as u64, 0x0Du8) } else { (val as u64, 0x0C) };
    for b in mem[..len].iter_mut() {
        *b = 0;
    }
    let mut l = len;
    l -= 1;
    mem[l] = (((num % 10) << 4) as u8) | sign;
    num /= 10;
    while num > 0 && l > 0 {
        l -= 1;
        let dig1 = (num % 10) as u8;
        num /= 10;
        let dig2 = (num % 10) as u8;
        num /= 10;
        mem[l] = (dig2 << 4) | dig1;
    }
}
/// `cob_put_u64_comp3` (move.c:2558): pack an unsigned value into `len` BCD bytes (sign nibble `0x0F`).
pub fn cob_put_u64_comp3(val: u64, mem: &mut [u8], len: usize) {
    let mut num = val;
    for b in mem[..len].iter_mut() {
        *b = 0;
    }
    let mut l = len - 1;
    mem[l] = (((num % 10) << 4) as u8) | 0x0F;
    num /= 10;
    while num > 0 && l > 0 {
        l -= 1;
        let dig1 = (num % 10) as u8;
        num /= 10;
        let dig2 = (num % 10) as u8;
        num /= 10;
        mem[l] = (dig2 << 4) | dig1;
    }
}
/// `cob_get_s64_comp3` (move.c:2578): unpack `len` BCD bytes to a signed value.
pub fn cob_get_s64_comp3(mem: &[u8], len: usize) -> i64 {
    let sign: i64 = if (mem[len - 1] & 0x0F) == 0x0D { -1 } else { 1 };
    let mut val: i64 = 0;
    for &b in &mem[..len - 1] {
        val = val * 10 + (b >> 4) as i64;
        val = val * 10 + (b & 0x0F) as i64;
    }
    val = val * 10 + (mem[len - 1] >> 4) as i64;
    val * sign
}
/// `cob_get_u64_comp3` (move.c:2599): unpack `len` BCD bytes to an unsigned value.
pub fn cob_get_u64_comp3(mem: &[u8], len: usize) -> u64 {
    let mut val: u64 = 0;
    for &b in &mem[..len - 1] {
        val = val * 10 + (b >> 4) as u64;
        val = val * 10 + (b & 0x0F) as u64;
    }
    val * 10 + (mem[len - 1] >> 4) as u64
}

/// `cob_put_u64_comp6` (move.c:2615): pack into `len` COMP-6 BCD bytes (no sign nibble).
pub fn cob_put_u64_comp6(val: u64, mem: &mut [u8], len: usize) {
    let mut num = val;
    for b in mem[..len].iter_mut() {
        *b = 0;
    }
    let mut l = len;
    while num > 0 && l > 0 {
        l -= 1;
        let dig1 = (num % 10) as u8;
        num /= 10;
        let dig2 = (num % 10) as u8;
        num /= 10;
        mem[l] = (dig2 << 4) | dig1;
    }
}
/// `cob_get_u64_comp6` (move.c:2632): unpack `len` COMP-6 BCD bytes.
pub fn cob_get_u64_comp6(mem: &[u8], len: usize) -> u64 {
    let mut val: u64 = 0;
    for &b in &mem[..len] {
        val = val * 10 + (b >> 4) as u64;
        val = val * 10 + (b & 0x0F) as u64;
    }
    val
}

const EBCDIC_POS: &[u8; 10] = b"{ABCDEFGHI";
const EBCDIC_NEG: &[u8; 10] = b"}JKLMNOPQR";

/// `cob_put_s64_pic9` (move.c:2652): store a signed value as `len` zoned ASCII digits (trailing
/// overpunch for the sign, or EBCDIC sign glyph when `ebcdic`).
pub fn cob_put_s64_pic9(val: i64, mem: &mut [u8], len: usize, ebcdic: bool) {
    for b in mem[..len].iter_mut() {
        *b = b'0';
    }
    let mut num;
    let mut l = len;
    if val < 0 {
        num = (val as i128).unsigned_abs() as u64;
        l -= 1;
        mem[l] = if ebcdic { EBCDIC_NEG[(num % 10) as usize] } else { (b'0' + (num % 10) as u8) | 0x40 };
    } else {
        num = val as u64;
        l -= 1;
        mem[l] = if ebcdic { EBCDIC_POS[(num % 10) as usize] } else { b'0' + (num % 10) as u8 };
    }
    num /= 10;
    while num > 0 && l > 0 {
        l -= 1;
        mem[l] = b'0' + (num % 10) as u8;
        num /= 10;
    }
}
/// `cob_get_s64_pic9` (move.c:2685): read `len` zoned ASCII digits to a signed value (`-`/`+`,
/// overpunch, or EBCDIC sign in the last byte).
pub fn cob_get_s64_pic9(mem: &[u8], len: usize, ebcdic: bool) -> i64 {
    let mut val: i64 = 0;
    let mut sign: i64 = 1;
    for &p in &mem[..len - 1] {
        if p.is_ascii_digit() {
            val = val * 10 + (p & 0x0F) as i64;
        } else if p == b'-' {
            sign = -1;
        }
    }
    let p = mem[len - 1];
    if p.is_ascii_digit() {
        val = val * 10 + (p & 0x0F) as i64;
    } else if p == b'-' {
        sign = -1;
    } else if p == b'+' {
        sign = 1;
    } else if ebcdic {
        if let Some(i) = EBCDIC_POS.iter().position(|&c| c == p) {
            val = val * 10 + i as i64;
            sign = 1;
        } else if let Some(i) = EBCDIC_NEG.iter().position(|&c| c == p) {
            val = val * 10 + i as i64;
            sign = -1;
        }
    } else {
        let dig = p & 0x3F;
        if dig.is_ascii_digit() {
            val = val * 10 + (dig & 0x0F) as i64;
        }
        if p & 0x40 != 0 {
            sign = -1;
        }
    }
    val * sign
}
/// `cob_put_u64_pic9` (move.c:2754): store an unsigned value as `len` zoned ASCII digits.
pub fn cob_put_u64_pic9(val: u64, mem: &mut [u8], len: usize) {
    let mut num = val;
    for b in mem[..len].iter_mut() {
        *b = b'0';
    }
    let mut l = len;
    while num > 0 && l > 0 {
        l -= 1;
        mem[l] = b'0' + (num % 10) as u8;
        num /= 10;
    }
}
/// `cob_get_u64_pic9` (move.c:2767): read `len` zoned ASCII digits to an unsigned value.
pub fn cob_get_u64_pic9(mem: &[u8], len: usize) -> u64 {
    let mut val: u64 = 0;
    for &p in &mem[..len] {
        val = val * 10 + (p & 0x0F) as u64;
    }
    val
}

/// `cob_put_comp1` (move.c:2782): store a `float` as its 4 native bytes.
pub fn cob_put_comp1(val: f32, mem: &mut [u8]) {
    mem[..4].copy_from_slice(&val.to_ne_bytes());
}
/// `cob_put_comp2` (move.c:2787): store a `double` as its 8 native bytes.
pub fn cob_put_comp2(val: f64, mem: &mut [u8]) {
    mem[..8].copy_from_slice(&val.to_ne_bytes());
}
/// `cob_get_comp1` (move.c:2791): read 4 native bytes as a `float`.
pub fn cob_get_comp1(mem: &[u8]) -> f32 {
    f32::from_ne_bytes(mem[..4].try_into().unwrap_or([0; 4]))
}
/// `cob_get_comp2` (move.c:2798): read 8 native bytes as a `double`.
pub fn cob_get_comp2(mem: &[u8]) -> f64 {
    f64::from_ne_bytes(mem[..8].try_into().unwrap_or([0; 8]))
}
/// `cob_put_pointer` (move.c:2806): store a pointer value as 8 native bytes.
pub fn cob_put_pointer(val: u64, mem: &mut [u8]) {
    mem[..8].copy_from_slice(&val.to_ne_bytes());
}

/// `cob_get_picx (cbl_data, len, char_field, num_chars)` (move.c:2812): copy the alphanumeric field's
/// content (trailing spaces/NULs trimmed) into a NUL-terminated buffer. Returns the bytes (without the
/// terminator), the Rust-idiomatic form of the C out-pointer/allocation contract.
pub fn cob_get_picx(cbl_data: &[u8], len: usize, num_chars: Option<usize>) -> Vec<u8> {
    let mut i = len;
    while i != 0 && (cbl_data[i - 1] == b' ' || cbl_data[i - 1] == 0) {
        i -= 1;
    }
    let cap = num_chars.unwrap_or(i + 1);
    if i > cap - 1 {
        i = cap - 1;
    }
    cbl_data[..i].to_vec()
}
/// `cob_put_picx (cbl_data, len, string)` (move.c:2834): copy `string` into the `len`-byte alphanumeric
/// field, space-padding the remainder (truncating if longer).
pub fn cob_put_picx(cbl_data: &mut [u8], len: usize, string: &[u8]) {
    let j = string.len().min(len);
    cbl_data[..j].copy_from_slice(&string[..j]);
    for b in cbl_data[j..len].iter_mut() {
        *b = b' ';
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::{COB_FLAG_HAVE_SIGN, COB_TYPE_NUMERIC_PACKED};

    #[test]
    fn typed_accessor_round_trips() {
        for &len in &[1usize, 2, 3, 4, 5, 6, 7, 8] {
            let cap = if len >= 8 { i64::MAX } else { 1i64 << (len * 8 - 1) };
            for &v in &[0i64, 1, 127, -1, -128, 1000, -1000, 65535] {
                if v.unsigned_abs() as i128 >= cap as i128 {
                    continue;
                }
                let mut m = vec![0u8; 8];
                cob_put_s64_compx(v, &mut m, len);
                assert_eq!(cob_get_s64_compx(&m, len), v, "compx len={len} v={v}");
                cob_put_s64_comp5(v, &mut m, len);
                assert_eq!(cob_get_s64_comp5(&m, len), v, "comp5 len={len} v={v}");
            }
        }
        // BCD comp3 round-trip
        for &len in &[2usize, 3, 4, 5] {
            for &v in &[0i64, 1, 42, -42, 9999, -9999] {
                if v.unsigned_abs() >= 10u64.pow((len * 2 - 1) as u32) {
                    continue;
                }
                let mut m = vec![0u8; len];
                cob_put_s64_comp3(v, &mut m, len);
                assert_eq!(cob_get_s64_comp3(&m, len), v, "comp3 len={len} v={v}");
            }
        }
        // pic9 round-trip (ASCII overpunch)
        for &v in &[0i64, 5, -5, 12345, -12345] {
            let mut m = vec![0u8; 6];
            cob_put_s64_pic9(v, &mut m, 6, false);
            assert_eq!(cob_get_s64_pic9(&m, 6, false), v, "pic9 v={v}");
        }
        // comp1/comp2 + picx
        let mut m = vec![0u8; 8];
        cob_put_comp2(3.5, &mut m);
        assert_eq!(cob_get_comp2(&m), 3.5);
        let mut x = vec![0u8; 8];
        cob_put_picx(&mut x, 8, b"HI");
        assert_eq!(&x, b"HI      ");
        assert_eq!(cob_get_picx(&x, 8, None), b"HI");
    }

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
