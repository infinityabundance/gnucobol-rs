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

    /// Decode a **cp500 EBCDIC zoned-decimal** numeric DISPLAY field (`GNURUST.17`) into a [`Decimal`].
    ///
    /// Each raw EBCDIC byte is translated through the sealed `GNURUST.15` cp500 table; the resulting
    /// ASCII bytes are an EBCDIC-derived **overpunch** zoned form, decoded per GnuCOBOL's
    /// `cob_get_sign_ebcdic` (`libcob/common.c`): in the **last** byte `'A'..'I'` → positive `1..9`,
    /// `'{'` → positive `0`, `'J'..'R'` → negative `1..9`, `'}'` → negative `0`, `'0'..'9'` → unsigned;
    /// every other byte contributes its decimal digit. The value's scale is `attr.scale`.
    pub fn from_ebcdic_zoned(data: &[u8], attr: &FieldAttr) -> Self {
        let ascii: Vec<u8> = data
            .iter()
            .map(|&b| crate::ebcdic::translate_byte(crate::ebcdic::CodePage::Cp500, b).unwrap_or(b))
            .collect();
        let n = ascii.len();
        let mut negative = false;
        let mut digits: Vec<u8> = Vec::with_capacity(n);
        for (i, &c) in ascii.iter().enumerate() {
            if i + 1 == n {
                // The sign-bearing final position (cob_get_sign_ebcdic, ASCII-host #else branch).
                let (neg, d) = match c {
                    b'{' => (false, 0),
                    b'A'..=b'I' => (false, c - b'A' + 1),
                    b'}' => (true, 0),
                    b'J'..=b'R' => (true, c - b'J' + 1),
                    b'0'..=b'9' => (false, c - b'0'), // unsigned
                    _ => (false, 0), // foreign byte cleaned to 0 (faithful to upstream)
                };
                negative = neg;
                digits.push(d);
            } else {
                digits.push(if c.is_ascii_digit() { c - b'0' } else { 0 });
            }
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

#[cfg(test)]
mod ebcdic_zoned_tests {
    use super::*;
    use crate::attr::{COB_FLAG_HAVE_SIGN, COB_TYPE_NUMERIC_DISPLAY};

    fn attr(digits: u16, scale: i16, signed: bool) -> FieldAttr {
        FieldAttr {
            field_type: COB_TYPE_NUMERIC_DISPLAY,
            digits,
            scale,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        }
    }

    #[test]
    fn cp500_zoned_sign() {
        // EBCDIC S9(3): F1 F2 C3 = +123 (0xC zone = positive), F1 F2 D3 = -123 (0xD = negative),
        // F1 F2 F3 = 123 unsigned.
        let pos = Decimal::from_ebcdic_zoned(&[0xF1, 0xF2, 0xC3], &attr(3, 0, true));
        assert_eq!((pos.negative, pos.unscaled_i128()), (false, Some(123)));
        let neg = Decimal::from_ebcdic_zoned(&[0xF1, 0xF2, 0xD3], &attr(3, 0, true));
        assert_eq!((neg.negative, neg.unscaled_i128()), (true, Some(-123)));
        let uns = Decimal::from_ebcdic_zoned(&[0xF1, 0xF2, 0xF3], &attr(3, 0, false));
        assert_eq!((uns.negative, uns.unscaled_i128()), (false, Some(123)));
        // negative zero: F0 D0 = -00 (zone D, digit 0). Magnitude 0.
        let nz = Decimal::from_ebcdic_zoned(&[0xF0, 0xD0], &attr(2, 0, true));
        assert_eq!(nz.unscaled_i128(), Some(0));
        // scale: S9(3)V99 from F1 F2 F3 F4 D5 = -123.45 (scale 2).
        let sc = Decimal::from_ebcdic_zoned(&[0xF1, 0xF2, 0xF3, 0xF4, 0xD5], &attr(5, 2, true));
        assert_eq!(
            (sc.negative, sc.scale, sc.unscaled_i128()),
            (true, 2, Some(-12345))
        );
    }
}
