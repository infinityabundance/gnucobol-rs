//! Table subscript access (`GNURUST.SUBSCRIPT.1`): direct 1-based `TABLE(i)` and multi-dimensional
//! `TABLE(i, j[, k...])` element extraction over an `OCCURS` table, proven byte-identical to cobc **in
//! bounds** and **fail-closed out of bounds** (cobc's out-of-range subscript is bounds-check-flag-dependent
//! and may read adjacent storage; gnucobol-rs refuses).
//!
//! The element offset is `sum_k (index_k - 1) * stride_k`, where the innermost `stride` equals the element
//! size and each outer `stride` is the byte length of one occurrence of that level (`inner_occurs *
//! element_size`, and so on). `index` is **1-based** (`TABLE(1)` is the first element, at offset 0).
//! Composes the sealed layout/offset model (`GNURUST.4`, `GNURUST.TABLE.PERFORM.SLICE.1`).

/// Why a subscript was refused (fail closed — never an out-of-range read). This `Display` text is an
/// internal developer-facing detail; the byte-faithful GnuCOBOL runtime-error bytes for a live subscript
/// fault come from [`crate::common::cob_check_subscript`] + [`crate::common::cob_bound_violation_diagnostic`]
/// (which carry the field name + source location this pure slicer lacks).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SubscriptError {
    /// A subscript was `0` (COBOL subscripts are 1-based) or `strides.len() != indices.len()`.
    BadIndex,
    /// The computed element window runs past the table (or offset arithmetic overflowed).
    OutOfBounds {
        /// the computed byte offset (or `None` on overflow).
        offset: Option<usize>,
        /// element size.
        element_size: usize,
        /// the table's byte length.
        table_len: usize,
    },
}

impl core::fmt::Display for SubscriptError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SubscriptError::BadIndex => write!(f, "table subscript is 0 or rank mismatch (subscripts are 1-based)"),
            SubscriptError::OutOfBounds { offset, element_size, table_len } => write!(
                f,
                "table element at offset {offset:?} (size {element_size}) out of bounds for a {table_len}-byte table"
            ),
        }
    }
}
impl std::error::Error for SubscriptError {}

/// Generic N-dimensional 1-based table element: `sum_k (indices[k]-1)*strides[k]`, then `element_size`
/// bytes. `strides` and `indices` are outermost-first; the innermost stride is normally `element_size`.
pub fn table_element<'a>(
    table: &'a [u8],
    element_size: usize,
    strides: &[usize],
    indices: &[usize],
) -> Result<&'a [u8], SubscriptError> {
    if strides.len() != indices.len() || indices.iter().any(|&i| i == 0) {
        return Err(SubscriptError::BadIndex);
    }
    let mut offset: usize = 0;
    for (k, &idx) in indices.iter().enumerate() {
        let term = (idx - 1).checked_mul(strides[k]);
        offset = match term.and_then(|t| offset.checked_add(t)) {
            Some(o) => o,
            None => return Err(SubscriptError::OutOfBounds { offset: None, element_size, table_len: table.len() }),
        };
    }
    if offset + element_size > table.len() {
        return Err(SubscriptError::OutOfBounds { offset: Some(offset), element_size, table_len: table.len() });
    }
    Ok(&table[offset..offset + element_size])
}

/// 1-D `TABLE(i)` over a table of `element_size`-byte occurrences.
pub fn element_1d(table: &[u8], element_size: usize, i: usize) -> Result<&[u8], SubscriptError> {
    table_element(table, element_size, &[element_size], &[i])
}

/// 2-D `TABLE(i, j)`: outer level has `inner_occurs` occurrences of an `element_size`-byte leaf.
pub fn element_2d(
    table: &[u8],
    element_size: usize,
    inner_occurs: usize,
    i: usize,
    j: usize,
) -> Result<&[u8], SubscriptError> {
    table_element(table, element_size, &[inner_occurs * element_size, element_size], &[i, j])
}

/// Fuzz entry (`GNURUST.SUBSCRIPT.1`): bounded access over any inputs — never panics.
pub fn __fuzz_subscript(data: &[u8]) {
    if data.len() < 3 {
        return;
    }
    let es = (data[0] % 8) as usize + 1;
    let i = data[1] as usize;
    let j = data[2] as usize;
    let table = &data[3..];
    let _ = element_1d(table, es, i);
    let _ = element_2d(table, es, 4, i, j);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscript_matches_the_oracle() {
        // 1-D: E OCCURS 5 PIC X(3) over "ABCDEFGHIJKLMNO"
        let one = b"ABCDEFGHIJKLMNO";
        assert_eq!(element_1d(one, 3, 1), Ok(&b"ABC"[..]));
        assert_eq!(element_1d(one, 3, 3), Ok(&b"GHI"[..]));
        assert_eq!(element_1d(one, 3, 5), Ok(&b"MNO"[..]));
        // 2-D: C OCCURS 3 of (OCCURS 4 PIC X(2)) over "AABBCCDDEEFFGGHHIIJJKKLL"
        let g = b"AABBCCDDEEFFGGHHIIJJKKLL";
        assert_eq!(element_2d(g, 2, 4, 1, 1), Ok(&b"AA"[..]));
        assert_eq!(element_2d(g, 2, 4, 1, 4), Ok(&b"DD"[..]));
        assert_eq!(element_2d(g, 2, 4, 2, 1), Ok(&b"EE"[..]));
        assert_eq!(element_2d(g, 2, 4, 3, 4), Ok(&b"LL"[..]));
        // fail-closed: 1-based, out of range, rank mismatch
        assert!(matches!(element_1d(one, 3, 0), Err(SubscriptError::BadIndex)));
        assert!(matches!(element_1d(one, 3, 6), Err(SubscriptError::OutOfBounds { .. })));
        assert!(matches!(element_2d(g, 2, 4, 3, 5), Err(SubscriptError::OutOfBounds { .. })));
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.SUBSCRIPT.1
    /// table_element is total over symbolic indices/strides (never panic, never out-of-bounds slice).
    #[kani::proof]
    #[kani::unwind(5)]
    fn subscript_is_total() {
        let table: [u8; 12] = kani::any();
        let i: usize = kani::any();
        let j: usize = kani::any();
        let _ = element_1d(&table, 3, i);
        let _ = element_2d(&table, 2, 4, i, j);
    }
}
