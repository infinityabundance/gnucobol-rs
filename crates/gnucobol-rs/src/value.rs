//! A digit/sign/scale-first decimal value (`GNURUST.DECAPI.0`): the canonical representation of a
//! COBOL numeric field's *value*, never narrowed through `f32`/`f64` (`GNURUST.NO-FLOAT-DECIMAL.0`)
//! and only fallibly through host integers. This is a convenience decode/encode layer over the
//! byte semantics; the sealed parity claim is the byte-level [`crate::cob_move`], not this type.

use crate::attr::FieldAttr;
use crate::sign;

/// A decimal value as COBOL holds it: a sign, a big-endian vector of decimal digits (0..=9), and
/// a scale (number of those digits that are fractional). No floating point is ever involved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decimal {
    /// `true` if the value is negative. Negative zero is representable (`negative == true`,
    /// all-zero digits) and preserved — a representation fact, not necessarily an accounting fact.
    pub negative: bool,
    /// Significant digits, most-significant first, each in `0..=9`.
    pub digits: Vec<u8>,
    /// Number of trailing `digits` that are fractional (the implied decimal point position).
    pub scale: i16,
}

impl Decimal {
    /// Decode a PACKED (COMP-3) or COMP-6 field's value. Reads the nibbles directly; the sign is
    /// the low nibble of the last byte (`0x0D` → negative) unless the field is unsigned/no-nibble.
    pub fn from_packed(data: &[u8], attr: &FieldAttr) -> Self {
        let mut digits = Vec::new();
        if attr.no_sign_nibble() {
            for &b in data {
                digits.push(b >> 4);
                digits.push(b & 0x0F);
            }
        } else {
            // every nibble except the final (sign) nibble is a digit
            for (idx, &b) in data.iter().enumerate() {
                digits.push(b >> 4);
                if idx + 1 != data.len() {
                    digits.push(b & 0x0F);
                }
            }
        }
        let negative = sign::packed_get_sign(data, attr) < 0;
        Decimal {
            negative,
            digits,
            scale: attr.scale,
        }
    }

    /// Decode a DISPLAY (zoned) field's value on an ASCII host. The sign is taken from the
    /// (possibly overpunched) sign byte; each digit is the low nibble of its byte.
    pub fn from_display(data: &[u8], attr: &FieldAttr) -> Self {
        let mut tmp = data.to_vec();
        let negative = sign::display_get_sign_strip(&mut tmp, attr) < 0;
        let off = attr.data_offset();
        let size = attr.data_size(data.len());
        let digits = tmp
            .iter()
            .skip(off)
            .take(size)
            .map(|&b| sign::d2i(b))
            .collect();
        Decimal {
            negative,
            digits,
            scale: attr.scale,
        }
    }

    /// Decode a binary field (`COMP`/`BINARY`/`COMP-5`/`COMP-X`, `GNURUST.14`) into a [`Decimal`].
    /// Endianness and sign come from the field flags; the value is the two's-complement integer at
    /// the field scale, rendered as `digits` decimal digits.
    pub fn from_binary(data: &[u8], attr: &FieldAttr) -> Self {
        let int = crate::binary::binary_decode(data, attr);
        let negative = int < 0;
        let mut abs = int.unsigned_abs();
        let width = attr.digits.max(1) as usize;
        let mut digits = vec![0u8; width];
        for slot in digits.iter_mut().rev() {
            *slot = (abs % 10) as u8;
            abs /= 10;
        }
        Decimal {
            negative,
            digits,
            scale: attr.scale,
        }
    }

    /// Fallible, lossless conversion to `i128` of the *unscaled* integer formed by the digits
    /// (i.e. value × 10^scale). Returns `None` on overflow. This is convenience only — the digit
    /// vector is the authority (`GNURUST.DECAPI.0`).
    pub fn unscaled_i128(&self) -> Option<i128> {
        let mut acc: i128 = 0;
        for &d in &self.digits {
            acc = acc.checked_mul(10)?.checked_add(d as i128)?;
        }
        if self.negative {
            acc = acc.checked_neg()?;
        }
        Some(acc)
    }
}
