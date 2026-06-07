//! Binary numeric codec (`GNURUST.14`): `COMP`/`BINARY`/`COMP-5`/`COMP-X` storage bytes ↔ integer
//! value, under the admitted oracle config (`binary-byteorder: big-endian`, `binary-truncate: yes`).
//!
//! Endianness comes from the field flags: [`COB_FLAG_BINARY_SWAP`] means big-endian (the byte order
//! is swapped relative to a little-endian host) — `COMP`/`BINARY`/`COMP-X`; otherwise native
//! little-endian — `COMP-5`. Signed fields ([`COB_FLAG_HAVE_SIGN`]) are two's complement. `COMP`
//! truncates a stored value to the PIC digit range ([`COB_FLAG_BINARY_TRUNC`]); `COMP-X`/`COMP-5`
//! keep the full byte-width value.

use crate::attr::{FieldAttr, COB_FLAG_BINARY_SWAP, COB_FLAG_BINARY_TRUNC, COB_FLAG_HAVE_SIGN};

fn ten_pow_i128(n: u16) -> i128 {
    let mut r: i128 = 1;
    for _ in 0..n {
        r = r.saturating_mul(10);
    }
    r
}

/// Decode binary field `bytes` into a signed integer (the field's value is `int * 10^(-attr.scale)`).
pub(crate) fn binary_decode(bytes: &[u8], attr: &FieldAttr) -> i128 {
    let n = bytes.len().min(16);
    let swap = attr.flags & COB_FLAG_BINARY_SWAP != 0;
    let mut u: u128 = 0;
    if swap {
        // big-endian: most-significant byte first
        for &b in bytes.iter().take(n) {
            u = (u << 8) | b as u128;
        }
    } else {
        // native little-endian: least-significant byte first
        for &b in bytes.iter().take(n).rev() {
            u = (u << 8) | b as u128;
        }
    }
    let bits = n * 8;
    let signed = attr.flags & COB_FLAG_HAVE_SIGN != 0;
    if signed && bits > 0 && bits < 128 && (u >> (bits - 1)) & 1 == 1 {
        (u as i128).wrapping_sub(1i128 << bits)
    } else {
        u as i128
    }
}

/// Encode a signed integer (at the field scale) into binary `out`, applying `COMP` digit-truncation
/// or `COMP-X`/`COMP-5` byte-masking and the field's endianness.
pub(crate) fn binary_encode(value: i128, attr: &FieldAttr, out: &mut [u8]) {
    let size = out.len().min(16);
    let mut v = value;
    if attr.flags & COB_FLAG_BINARY_TRUNC != 0 && attr.digits <= 38 {
        let m = ten_pow_i128(attr.digits);
        if m > 0 {
            v %= m; // truncate to the PIC digit range (keeps sign)
        }
    }
    let bits = size * 8;
    let mask: u128 = if bits >= 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    let uv = (v as u128) & mask; // two's-complement low bits
    let swap = attr.flags & COB_FLAG_BINARY_SWAP != 0;
    for (i, b) in out.iter_mut().enumerate().take(size) {
        let byte = (uv >> (8 * if swap { size - 1 - i } else { i })) as u8;
        *b = byte;
    }
}
