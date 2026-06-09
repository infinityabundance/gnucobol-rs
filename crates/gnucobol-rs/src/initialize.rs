//! `INITIALIZE` receiver byte effects (`GNURUST.INITIALIZE.1`): the bytes a plain `INITIALIZE <record>`
//! statement changes or preserves, proven against GnuCOBOL 3.2. This is the first Procedure-Division byte
//! court — it touches receiver storage without needing full statement-flow execution.
//!
//! **Witnessed rules (from the oracle, not assumed):** a plain `INITIALIZE` sets each *elementary* item to
//! its **category default** — `X(n)` → **spaces** (0x20), numeric `DISPLAY` (`9`/`S9`) → **`'0'` digits**
//! (0x30, no sign overpunch on +0), `COMP-3` → **packed zero with a sign nibble** (`00…0C` signed, `00…0F` unsigned),
//! binary (`COMP`/`COMP-5`/`COMP-X`) → **zero bytes** — while **`FILLER` is preserved**, a **`REDEFINES`
//! redefiner is skipped** (only the base definition is initialized), **every `OCCURS` element is
//! initialized**, and a **`VALUE` clause is NOT restored** (the category default wins, not the value).
//!
//! **Non-claims:** full Procedure Division execution, `INITIALIZE ... REPLACING` / `TO VALUE` / `WITH
//! FILLER`, numeric-edited and `JUSTIFIED` / `BLANK WHEN ZERO` receivers, `OCCURS DEPENDING ON` runtime
//! active count, the active `REDEFINES` view, and all-dialect behavior.

/// The category default `INITIALIZE` applies to one elementary item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InitCategory {
    /// `PIC X(n)` → spaces.
    Alphanumeric,
    /// `PIC 9(n)` / `S9(n)` DISPLAY → `'0'` digits (no sign overpunch on zero).
    NumericDisplay,
    /// `COMP-3` / packed → zero nibbles + a sign nibble (`C` signed / `F` unsigned) in the low nibble of the last byte.
    Packed,
    /// `COMP` / `COMP-5` / `COMP-X` → zero bytes.
    Binary,
}

/// One elementary item for `INITIALIZE`: where it lives, its category, and whether it is skipped.
#[derive(Debug, Clone)]
pub struct InitField {
    pub offset: usize,
    pub size: usize,
    pub category: InitCategory,
    /// Signed (`S9`)? Only affects `Packed`: signed → sign nibble `C`, unsigned → `F`.
    pub signed: bool,
    /// `FILLER` — preserved by `INITIALIZE`.
    pub is_filler: bool,
    /// a `REDEFINES` redefiner — skipped (only the base definition is initialized).
    pub is_redefiner: bool,
}

/// Apply a plain `INITIALIZE <record>` to `prefill` (the record's bytes before the statement), returning the
/// bytes after — byte-for-byte as GnuCOBOL. Skipped items (`FILLER`, redefiners) keep their prior bytes.
pub fn initialize_record(fields: &[InitField], prefill: &[u8]) -> Vec<u8> {
    let mut out = prefill.to_vec();
    for f in fields {
        if f.is_filler || f.is_redefiner {
            continue;
        }
        let end = f.offset + f.size;
        if end > out.len() {
            continue;
        }
        match f.category {
            InitCategory::Alphanumeric => out[f.offset..end].fill(b' '),
            InitCategory::NumericDisplay => out[f.offset..end].fill(b'0'),
            InitCategory::Binary => out[f.offset..end].fill(0x00),
            InitCategory::Packed => {
                out[f.offset..end].fill(0x00);
                // zero magnitude + sign nibble: signed -> C (positive), unsigned -> F.
                out[end - 1] = if f.signed { 0x0c } else { 0x0f };
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::InitCategory::*;
    use super::*;
    fn f(offset: usize, size: usize, c: InitCategory) -> InitField {
        InitField { offset, size, category: c, signed: true, is_filler: false, is_redefiner: false }
    }

    #[test]
    fn initialize_matches_oracle_byte_effects() {
        // the probed record: A X(4) · N 9(3) · SN S9(3) · P S9(3)COMP-3 · B 9(4)COMP · FILLER X(2) ·
        // T1/T2 X(2) (OCCURS 2) · BASE X(4) · RED REDEFINES BASE 9(4) · V X(3) VALUE "XYZ"  (27 bytes)
        let fields = vec![
            f(0, 4, Alphanumeric),               // A
            f(4, 3, NumericDisplay),             // N
            f(7, 3, NumericDisplay),             // SN (signed -> still '0's, no overpunch)
            f(10, 2, Packed),                    // P
            f(12, 2, Binary),                    // B
            InitField { offset: 14, size: 2, category: Alphanumeric, signed: false, is_filler: true, is_redefiner: false }, // FILLER preserved
            f(16, 2, Alphanumeric),              // T1
            f(18, 2, Alphanumeric),              // T2 (every OCCURS element)
            f(20, 4, Alphanumeric),              // BASE
            InitField { offset: 20, size: 4, category: NumericDisplay, signed: false, is_filler: false, is_redefiner: true }, // RED skipped
            f(24, 3, Alphanumeric),              // V (VALUE "XYZ" NOT restored -> spaces)
        ];
        let prefill = vec![0x5au8; 27]; // MOVE ALL "Z"
        let out = initialize_record(&fields, &prefill);
        let mut expect = Vec::new();
        expect.extend_from_slice(b"    ");        // A spaces
        expect.extend_from_slice(b"000");         // N
        expect.extend_from_slice(b"000");         // SN
        expect.extend_from_slice(&[0x00, 0x0c]);  // P packed +0
        expect.extend_from_slice(&[0x00, 0x00]);  // B binary 0
        expect.extend_from_slice(&[0x5a, 0x5a]);  // FILLER preserved
        expect.extend_from_slice(b"  ");          // T1
        expect.extend_from_slice(b"  ");          // T2
        expect.extend_from_slice(b"    ");        // BASE
        expect.extend_from_slice(b"   ");         // V spaces (not "XYZ")
        assert_eq!(out, expect);
    }

    #[test]
    fn unsigned_packed_uses_f_sign_nibble() {
        let signed = InitField { offset: 0, size: 2, category: Packed, signed: true, is_filler: false, is_redefiner: false };
        let unsigned = InitField { offset: 0, size: 2, category: Packed, signed: false, is_filler: false, is_redefiner: false };
        assert_eq!(initialize_record(&[signed], &[0x7e; 2]), vec![0x00, 0x0c]);
        assert_eq!(initialize_record(&[unsigned], &[0x7e; 2]), vec![0x00, 0x0f]);
    }

    #[test]
    fn filler_and_redefiner_are_preserved() {
        let fields = vec![
            InitField { offset: 0, size: 3, category: Alphanumeric, signed: false, is_filler: true, is_redefiner: false },
            InitField { offset: 3, size: 3, category: NumericDisplay, signed: false, is_filler: false, is_redefiner: true },
            f(6, 2, Alphanumeric),
        ];
        let out = initialize_record(&fields, &[0xaa; 8]);
        assert_eq!(&out[0..3], &[0xaa, 0xaa, 0xaa]); // FILLER kept
        assert_eq!(&out[3..6], &[0xaa, 0xaa, 0xaa]); // redefiner kept
        assert_eq!(&out[6..8], b"  "); // the real field -> spaces
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.INITIALIZE.1
    /// INITIALIZE never changes the record length; skipped (FILLER/redefiner) and out-of-range fields are safe.
    #[kani::proof]
    #[kani::unwind(6)]
    fn initialize_preserves_record_length() {
        let prefill: [u8; 8] = kani::any();
        let cat: u8 = kani::any();
        let category = match cat % 4 { 0 => InitCategory::Alphanumeric, 1 => InitCategory::NumericDisplay, 2 => InitCategory::Packed, _ => InitCategory::Binary };
        let f = InitField { offset: 0, size: 4, category, signed: cat & 4 == 0, is_filler: cat & 8 == 0, is_redefiner: false };
        let out = initialize_record(&[f], &prefill);
        assert_eq!(out.len(), prefill.len());
    }
}
