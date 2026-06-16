//! Reference modification (`GNURUST.REFMOD.1`): `field(start:length)` substring extraction, `field(start:)`
//! to-end, and the receiver form `MOVE src TO field(start:length)` — proven byte-identical to cobc **in
//! bounds**, and **fail-closed out of bounds** (where cobc's behavior is bounds-check-flag-dependent and may
//! read/write adjacent storage — gnucobol-rs deliberately refuses rather than touch neighbouring bytes).
//!
//! `start` is **1-based** (COBOL convention). Pure byte slicing/overwrite; no numeric or edited semantics.

/// Why a reference-modification was refused (fail closed — never an out-of-bounds read/write).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RefModError {
    /// `length` was zero (COBOL requires `length >= 1`).
    ZeroLength,
    /// `start == 0`, or the window `start..start+length` runs past the field.
    OutOfBounds {
        /// 1-based start position requested.
        start: usize,
        /// length requested (`0` for the to-end form).
        length: usize,
        /// the field's byte length.
        field_len: usize,
    },
}

impl core::fmt::Display for RefModError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RefModError::ZeroLength => write!(f, "reference modification length must be >= 1"),
            RefModError::OutOfBounds { start, length, field_len } => write!(
                f,
                "reference modification ({start}:{length}) out of bounds for a {field_len}-byte field"
            ),
        }
    }
}
impl std::error::Error for RefModError {}

/// `field(start:length)` — the `length` bytes of `field` starting at 1-based `start`. A runtime
/// `length` of 0 is a **valid empty result** under the default dialect (`ref-mod-zero-length: yes`):
/// GnuCOBOL yields an empty string (e.g. `A(2:0)` → ``), so we return an empty slice rather than
/// failing closed. (Strict dialects — cobol85/ibm/mf/mvs — reject zero length; the default-dialect
/// oracle is the pinned target.) Still fail closed when the window runs past the field or `start` is 0.
pub fn ref_mod(field: &[u8], start: usize, length: usize) -> Result<&[u8], RefModError> {
    if start == 0 || start - 1 + length > field.len() {
        return Err(RefModError::OutOfBounds { start, length, field_len: field.len() });
    }
    Ok(&field[start - 1..start - 1 + length])
}

/// `field(start:)` — from 1-based `start` to the end of `field`. Fail closed if `start` is 0 or past the end.
pub fn ref_mod_to_end(field: &[u8], start: usize) -> Result<&[u8], RefModError> {
    if start == 0 || start - 1 > field.len() {
        return Err(RefModError::OutOfBounds { start, length: 0, field_len: field.len() });
    }
    Ok(&field[start - 1..])
}

/// `MOVE src TO field(start:length)` — overwrite the `length`-byte window at 1-based `start` with `src`
/// under alphanumeric MOVE rules (left-justified, space-padded, truncated to the window). A runtime
/// `length` of 0 is a valid no-op (an empty receiver window; default dialect), like the source form.
/// Fail closed only when the window runs past the field or `start` is 0; on error `field` is untouched.
pub fn apply_ref_mod(
    field: &mut [u8],
    start: usize,
    length: usize,
    src: &[u8],
) -> Result<(), RefModError> {
    if start == 0 || start - 1 + length > field.len() {
        return Err(RefModError::OutOfBounds { start, length, field_len: field.len() });
    }
    for (i, slot) in field[start - 1..start - 1 + length].iter_mut().enumerate() {
        *slot = src.get(i).copied().unwrap_or(b' ');
    }
    Ok(())
}

/// Fuzz entry (`GNURUST.REFMOD.1`): bounded slicing/overwrite over any inputs — never panics.
pub fn __fuzz_refmod(data: &[u8]) {
    if data.len() < 2 {
        return;
    }
    let start = data[0] as usize;
    let length = data[1] as usize;
    let field = &data[2..];
    let _ = ref_mod(field, start, length);
    let _ = ref_mod_to_end(field, start);
    let mut buf = field.to_vec();
    let _ = apply_ref_mod(&mut buf, start, length, field);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ref_mod_matches_the_oracle() {
        assert_eq!(ref_mod(b"ABCDEF", 1, 3), Ok(&b"ABC"[..]));
        assert_eq!(ref_mod(b"ABCDEF", 2, 3), Ok(&b"BCD"[..]));
        assert_eq!(ref_mod(b"ABCDEF", 4, 3), Ok(&b"DEF"[..]));
        assert_eq!(ref_mod(b"ABCDEF", 1, 6), Ok(&b"ABCDEF"[..]));
        assert_eq!(ref_mod(b"ABCDEF", 6, 1), Ok(&b"F"[..]));
        assert_eq!(ref_mod_to_end(b"ABCDEF", 3), Ok(&b"CDEF"[..]));
        // fail-closed out of bounds / start-0
        assert!(matches!(ref_mod(b"ABCDEF", 5, 3), Err(RefModError::OutOfBounds { .. })));
        assert!(matches!(ref_mod(b"ABCDEF", 0, 3), Err(RefModError::OutOfBounds { .. })));
        // runtime zero length is a VALID empty result in the default dialect (matches cobc: A(s:0) -> "")
        assert_eq!(ref_mod(b"ABCDEF", 1, 0), Ok(&b""[..]));
        assert_eq!(ref_mod(b"ABCDEF", 2, 0), Ok(&b""[..]));
        assert_eq!(ref_mod(b"ABCDEF", 7, 0), Ok(&b""[..])); // start = size+1, empty at end
        assert!(matches!(ref_mod(b"ABCDEF", 8, 0), Err(RefModError::OutOfBounds { .. }))); // start past end+1
        // receiver zero length is a no-op (field untouched)
        let mut f = *b"ABCDEF";
        assert_eq!(apply_ref_mod(&mut f, 2, 0, b"ZZ"), Ok(()));
        assert_eq!(&f, b"ABCDEF");
    }

    #[test]
    fn receiver_overwrites_with_space_pad() {
        let mut g = *b"ABCDEF";
        apply_ref_mod(&mut g, 2, 2, b"XY").unwrap();
        assert_eq!(&g, b"AXYDEF");
        let mut g = *b"ABCDEF";
        apply_ref_mod(&mut g, 5, 2, b"Z").unwrap(); // 1-byte src into a 2-byte window -> "Z "
        assert_eq!(&g, b"ABCDZ ");
        let mut g = *b"ABCDEF";
        assert!(apply_ref_mod(&mut g, 5, 3, b"Z").is_err()); // 5+3-1=7 > 6
        assert_eq!(&g, b"ABCDEF"); // untouched on error
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.REFMOD.1
    /// ref_mod / apply_ref_mod are total over symbolic start/length/bytes (never panic, never out-of-bounds).
    #[kani::proof]
    #[kani::unwind(7)]
    fn refmod_is_total() {
        let field: [u8; 6] = kani::any();
        let start: usize = kani::any();
        let length: usize = kani::any();
        let _ = ref_mod(&field, start, length);
        let _ = ref_mod_to_end(&field, start);
        let mut buf = field;
        let _ = apply_ref_mod(&mut buf, start, length, &field);
    }
}
