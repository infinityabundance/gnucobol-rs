//! EBCDIC code-page boundary for DISPLAY-class fields (`GNURUST.15`).
//!
//! **Doctrine.** GNURUST.15 admits EBCDIC only as an explicit code-page boundary for DISPLAY-class
//! field decoding: text bytes are decoded under a *named* table, binary and packed fields remain raw
//! storage domains, and no mixed-encoding, auto-detection, collation, national/DBCS, or
//! business-meaning claim is made.
//!
//! **Oracle-faithful code page.** The admitted GnuCOBOL 3.2 oracle ships an `ebcdic500_ascii8bit`
//! translation table (`share/gnucobol/config/`), loadable via `cob_load_collation` — i.e. **cp500**.
//! (It does **not** ship cp037, so cp037 is *not* admitted: a code page is admitted only when its
//! table comes from the oracle, never from a spec we looked up.) The 256-byte table below is the one
//! the oracle's `cob_load_collation` produces, verified byte-for-byte (`lab/oracle/ebcdic_sweep.sh`).
//!
//! **Scope.** Alphanumeric DISPLAY bytes → text. **Fails closed / not claimed:** any other code page,
//! numeric EBCDIC zoned sign processing (the EBCDIC-machine sign mode — a separate court), national /
//! DBCS, collation, mixed/auto-detected encodings, and binary/packed conversion (those bytes are raw
//! storage and must pass through untouched — EBCDIC is not a record-wide "convert everything").

/// A named EBCDIC code page admitted by the oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CodePage {
    /// `ebcdic500_ascii8bit` as shipped by the admitted GnuCOBOL 3.2 oracle (cp500).
    Cp500,
}

/// Why a byte/string could not be handled under the code page (fail closed).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EbcdicError {
    /// The requested code page is not admitted (only [`CodePage::Cp500`] is).
    UnknownCodePage,
}

impl core::fmt::Display for EbcdicError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EbcdicError::UnknownCodePage => {
                write!(
                    f,
                    "code page not admitted by the GnuCOBOL 3.2 oracle (only cp500)"
                )
            }
        }
    }
}
impl std::error::Error for EbcdicError {}

/// cp500 EBCDIC→ASCII (8-bit) table — the bytes the admitted oracle's `cob_load_collation` yields for
/// `ebcdic500_ascii8bit`. Index by the raw EBCDIC byte; the result is the Latin-1/ASCII byte.
pub(crate) const CP500_TO_ASCII: [u8; 256] = [
    0x00, 0x01, 0x02, 0x03, 0x9c, 0x09, 0x86, 0x7f, 0x97, 0x8d, 0x8e, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x9d, 0x85, 0x08, 0x87, 0x18, 0x19, 0x92, 0x8f, 0x1c, 0x1d, 0x1e, 0x1f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x0a, 0x17, 0x1b, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x05, 0x06, 0x07,
    0x90, 0x91, 0x16, 0x93, 0x94, 0x95, 0x96, 0x04, 0x98, 0x99, 0x9a, 0x9b, 0x14, 0x15, 0x9e, 0x1a,
    0x20, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0x5b, 0x2e, 0x3c, 0x28, 0x2b, 0x21,
    0x26, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb0, 0xb1, 0x5d, 0x24, 0x2a, 0x29, 0x3b, 0x5e,
    0x2d, 0x2f, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0x7c, 0x2c, 0x25, 0x5f, 0x3e, 0x3f,
    0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xc0, 0xc1, 0xc2, 0x60, 0x3a, 0x23, 0x40, 0x27, 0x3d, 0x22,
    0xc3, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9,
    0xca, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0xcb, 0xcc, 0xcd, 0xce, 0xcf, 0xd0,
    0xd1, 0x7e, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
    0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf, 0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7,
    0x7b, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed,
    0x7d, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0xee, 0xef, 0xf0, 0xf1, 0xf2, 0xf3,
    0x5c, 0x9f, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

/// Translate one EBCDIC byte to its ASCII/Latin-1 byte under `cp`.
pub fn translate_byte(cp: CodePage, b: u8) -> Result<u8, EbcdicError> {
    match cp {
        CodePage::Cp500 => Ok(CP500_TO_ASCII[b as usize]),
    }
}

/// Decode raw EBCDIC bytes of an **alphanumeric DISPLAY** field into the **byte-length-preserving**
/// ASCII/Latin-1 storage image — cconv.c's `translate` maps each EBCDIC byte to exactly one result
/// byte (`out.len() == bytes.len()`). This is the field-storage-faithful form: use it wherever the
/// result is stored back into a COBOL field. Numeric EBCDIC zoned sign processing, binary, and packed
/// fields are **not** handled here (text decoding only).
pub fn decode_display_bytes(cp: CodePage, bytes: &[u8]) -> Result<Vec<u8>, EbcdicError> {
    let mut v = Vec::with_capacity(bytes.len());
    for &b in bytes {
        v.push(translate_byte(cp, b)?);
    }
    Ok(v)
}

/// As [`decode_display_bytes`], but as a `String` for **text/diagnostic** use (each result byte taken
/// as a Latin-1 code point). NOTE: a Latin-1 high byte (>=0x80) UTF-8-encodes to two bytes in the
/// returned `String`, so this is 1:1 in *chars* but not in *bytes* — for byte-length field-storage
/// fidelity use [`decode_display_bytes`].
pub fn decode_display(cp: CodePage, bytes: &[u8]) -> Result<String, EbcdicError> {
    Ok(decode_display_bytes(cp, bytes)?.into_iter().map(|b| b as char).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp500_decodes_text() {
        // EBCDIC: 'A'=0xC1, '0'=0xF0, space=0x40, '.'=0x4B, '$'=0x5B, '-'=0x60.
        assert_eq!(
            decode_display(CodePage::Cp500, &[0xC1, 0xC2, 0xC3]).unwrap(),
            "ABC"
        );
        assert_eq!(
            decode_display(CodePage::Cp500, &[0xF1, 0xF2, 0xF3]).unwrap(),
            "123"
        );
        assert_eq!(
            decode_display(CodePage::Cp500, &[0xC1, 0x40, 0xC9]).unwrap(),
            "A I"
        );
        // "POL." in EBCDIC: P=0xD7, O=0xD6, L=0xD3, .=0x4B
        assert_eq!(
            decode_display(CodePage::Cp500, &[0xD7, 0xD6, 0xD3, 0x4B]).unwrap(),
            "POL."
        );
    }

    #[test]
    fn table_anchors() {
        assert_eq!(translate_byte(CodePage::Cp500, 0x40).unwrap(), b' ');
        assert_eq!(translate_byte(CodePage::Cp500, 0xC1).unwrap(), b'A');
        assert_eq!(translate_byte(CodePage::Cp500, 0xF0).unwrap(), b'0');
        assert_eq!(translate_byte(CodePage::Cp500, 0x5B).unwrap(), b'$');
    }

    #[test]
    fn decode_bytes_is_byte_length_preserving() {
        // cconv.c translate is 1:1 in BYTES: 256 input bytes -> 256 output bytes, even for the high
        // cp500 images (>=0x80) that UTF-8-expand to two bytes in the String form. This is the
        // field-storage-faithful property the String API cannot offer.
        let all: Vec<u8> = (0u16..256).map(|b| b as u8).collect();
        let v = decode_display_bytes(CodePage::Cp500, &all).unwrap();
        assert_eq!(v.len(), 256, "byte-length preserved 1:1");
        // The String form is 1:1 in CHARS but its UTF-8 byte length exceeds 256 (high bytes expand).
        let s = decode_display(CodePage::Cp500, &all).unwrap();
        assert_eq!(s.chars().count(), 256);
        assert!(s.len() > 256, "String UTF-8 length expands for >=0x80 cp500 images");
        // Each output byte equals its translate_byte image.
        assert!(v.iter().zip(all.iter()).all(|(&o, &i)| o == translate_byte(CodePage::Cp500, i).unwrap()));
    }

    #[test]
    fn decode_is_total_and_length_preserving() {
        // Every one of the 256 byte values decodes (no panic, no error), one char per byte.
        let all: Vec<u8> = (0u16..256).map(|b| b as u8).collect();
        let s = decode_display(CodePage::Cp500, &all).unwrap();
        assert_eq!(s.chars().count(), 256);
        // The table is a bijection over 0..256 (a true code page), so no two bytes collide.
        let mut seen = [false; 256];
        for b in 0u16..256 {
            let a = translate_byte(CodePage::Cp500, b as u8).unwrap();
            assert!(!seen[a as usize], "cp500 table not a bijection at {a:#x}");
            seen[a as usize] = true;
        }
    }
}

/// Fuzz entry: EBCDIC decode is total over arbitrary bytes (no panic, length-preserving).
#[cfg(feature = "fuzzing")]
pub fn __fuzz_ebcdic(data: &[u8]) {
    if let Ok(s) = decode_display(CodePage::Cp500, data) {
        debug_assert_eq!(s.chars().count(), data.len());
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.15
    /// EBCDIC cp500 translation/decoding is total over any byte: Ok or typed error, never a panic.
    #[kani::proof]
    #[kani::unwind(6)]
    fn ebcdic_decode_is_total() {
        let b: u8 = kani::any();
        let _ = translate_byte(CodePage::Cp500, b);
        let bytes: [u8; 4] = kani::any();
        let _ = decode_display(CodePage::Cp500, &bytes);
    }
}
