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
///
/// The default (non-`COB_EXPERIMENTAL`) GnuCOBOL build converts a binary field to the decimal core via
/// `cob_binary_get_sint64`/`cob_binary_get_uint64` (numeric.c) -- which read **only the low 64 bits**.
/// A binary field wider than 8 bytes (a 16-byte `COMP-X`; `COMP`/`COMP-5` are capped at 18 digits = 8
/// bytes by the compiler) is therefore taken **mod 2^64**. We reproduce that: cap at 8 bytes, taking the
/// least-significant 8 per endianness. (Oracle: `PIC 9(38) COMP-X <- 1234567890123456789012345` reads
/// back `1096246371337559929 == that mod 2^64`.)
pub(crate) fn binary_decode(bytes: &[u8], attr: &FieldAttr) -> i128 {
    let full = bytes.len().min(16);
    let swap = attr.flags & COB_FLAG_BINARY_SWAP != 0;
    let n = full.min(8); // the low 64 bits, like cob_binary_get_*int64
    let mut u: u128 = 0;
    if swap {
        // big-endian: the least-significant 8 bytes are at the END of the field.
        for &b in &bytes[full - n..full] {
            u = (u << 8) | b as u128;
        }
    } else {
        // native little-endian: the least-significant 8 bytes are at the START.
        for &b in bytes[..n].iter().rev() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::COB_TYPE_NUMERIC_BINARY;

    #[test]
    fn binary_decode_le_and_be_round_8_bytes() {
        // A normal <=8-byte field is unchanged: 2-byte COMP (big-endian/SWAP) 0x0410 -> 1040.
        let be = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 4, scale: 0, flags: COB_FLAG_BINARY_SWAP };
        assert_eq!(binary_decode(&[0x04, 0x10], &be), 1040);
        // 2-byte COMP-5 (native little-endian) 0x10 0x04 -> 1040.
        let le = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 4, scale: 0, flags: 0 };
        assert_eq!(binary_decode(&[0x10, 0x04], &le), 1040);
    }

    #[test]
    fn binary_decode_caps_wide_field_at_low_64_bits_vs_c_default() {
        // The default C build reads a binary field through cob_binary_get_*int64 (low 64 bits only), so a
        // 16-byte COMP-X with non-zero high bytes is taken mod 2^64. Oracle: PIC 9(38) COMP-X <-
        // 1234567890123456789012345 reads back 1096246371337559929 (== that value mod 2^64).
        let v: u128 = 1234567890123456789012345;
        // COMP-X is big-endian (SWAP), unsigned, 16 bytes.
        let attr = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 38, scale: 0, flags: COB_FLAG_BINARY_SWAP };
        let mut be = [0u8; 16];
        for (i, b) in be.iter_mut().enumerate() {
            *b = (v >> (8 * (15 - i))) as u8;
        }
        assert_eq!(binary_decode(&be, &attr), 1096246371337559929);
        assert_eq!(binary_decode(&be, &attr), (v & u64::MAX as u128) as i128);

        // Same value stored little-endian (a hypothetical 16-byte COMP-5): the low 8 bytes are at the start.
        let le_attr = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 38, scale: 0, flags: 0 };
        let mut le = [0u8; 16];
        for (i, b) in le.iter_mut().enumerate() {
            *b = (v >> (8 * i)) as u8;
        }
        assert_eq!(binary_decode(&le, &le_attr), 1096246371337559929);
    }
}
