//! `OCCURS DEPENDING ON` (`GNURUST.ODO.1`): the **used length** of a variable-length table group and
//! **bounded element access**, proven byte-identical to cobc.
//!
//! A `05 E OCCURS 1 TO max DEPENDING ON N` table is physically `max` elements wide, but its *used* length
//! — what `LENGTH OF` reports and what a group `MOVE`/record write copies — is `fixed_prefix + N *
//! element_size` (the controlling value `N` chooses how many elements are live). Element access `E(i)` is
//! admitted only for `1 <= i <= N`; gnucobol-rs **fails closed** beyond the active count rather than read a
//! physically-present-but-inactive slot. Composes the sealed offset model (`GNURUST.SUBSCRIPT.1`).

/// Why an `OCCURS DEPENDING ON` access was refused (fail closed).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OdoError {
    /// `i == 0` or `i > controlling` — outside the active `1..=N` range.
    BeyondActiveRange {
        /// the 1-based index requested.
        index: usize,
        /// the controlling `DEPENDING ON` value (active element count).
        controlling: usize,
    },
    /// The element window runs past the physical table bytes.
    OutOfBounds {
        /// the computed byte offset.
        offset: usize,
        /// element size.
        element_size: usize,
        /// the table's byte length.
        table_len: usize,
    },
}

impl core::fmt::Display for OdoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            OdoError::BeyondActiveRange { index, controlling } => {
                write!(f, "ODO element {index} beyond the active count {controlling} (1..={controlling})")
            }
            OdoError::OutOfBounds { offset, element_size, table_len } => {
                write!(f, "ODO element at offset {offset} (size {element_size}) past the {table_len}-byte table")
            }
        }
    }
}
impl std::error::Error for OdoError {}

/// The **used** byte length of an `OCCURS 1 TO max DEPENDING ON N` group: `fixed_prefix_size + controlling *
/// element_size` (matches cobc `LENGTH OF` of the group).
pub fn odo_used_length(controlling: usize, fixed_prefix_size: usize, element_size: usize) -> usize {
    fixed_prefix_size + controlling * element_size
}

/// `E(i)` of an `OCCURS DEPENDING ON N` table: the `element_size` bytes at `fixed_prefix_size +
/// (i-1)*element_size`, admitted only for `1 <= i <= controlling`. Fail closed otherwise.
pub fn odo_element(
    table: &[u8],
    fixed_prefix_size: usize,
    element_size: usize,
    controlling: usize,
    i: usize,
) -> Result<&[u8], OdoError> {
    if i == 0 || i > controlling {
        return Err(OdoError::BeyondActiveRange { index: i, controlling });
    }
    let offset = fixed_prefix_size + (i - 1) * element_size;
    if offset + element_size > table.len() {
        return Err(OdoError::OutOfBounds { offset, element_size, table_len: table.len() });
    }
    Ok(&table[offset..offset + element_size])
}

/// Fuzz entry (`GNURUST.ODO.1`): bounded over any inputs — never panics.
pub fn __fuzz_odo(data: &[u8]) {
    if data.len() < 4 {
        return;
    }
    let controlling = (data[0] % 8) as usize;
    let prefix = (data[1] % 4) as usize;
    let es = (data[2] % 6) as usize + 1;
    let i = data[3] as usize;
    let _ = odo_used_length(controlling, prefix, es);
    let _ = odo_element(&data[4..], prefix, es, controlling, i);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odo_length_and_access_match_the_oracle() {
        // REC = 05 N PIC 9 (prefix 1) + E OCCURS 1 TO 5 DEPENDING ON N PIC X(3): len = 1 + N*3.
        assert_eq!(odo_used_length(2, 1, 3), 7);
        assert_eq!(odo_used_length(5, 1, 3), 16);
        assert_eq!(odo_used_length(1, 1, 3), 4);
        // element access at N=3 over "?PPPQQQRRR..." (prefix byte then 3 elems)
        let rec = b"3PPPQQQRRR......";
        assert_eq!(odo_element(rec, 1, 3, 3, 1), Ok(&b"PPP"[..]));
        assert_eq!(odo_element(rec, 1, 3, 3, 3), Ok(&b"RRR"[..]));
        // fail-closed: beyond the active count, and 0
        assert!(matches!(odo_element(rec, 1, 3, 3, 4), Err(OdoError::BeyondActiveRange { .. })));
        assert!(matches!(odo_element(rec, 1, 3, 3, 0), Err(OdoError::BeyondActiveRange { .. })));
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.ODO.1
    /// odo_used_length / odo_element are total over symbolic inputs (never panic / out-of-bounds).
    #[kani::proof]
    #[kani::unwind(5)]
    fn odo_is_total() {
        let table: [u8; 16] = kani::any();
        let n: usize = kani::any();
        let i: usize = kani::any();
        let _ = odo_used_length(n, 1, 3);
        let _ = odo_element(&table, 1, 3, n, i);
    }
}
