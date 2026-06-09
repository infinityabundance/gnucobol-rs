//! `SEARCH` / `SEARCH ALL` over an `OCCURS` table (`GNURUST.SEARCH.TABLE.1`): the index a table lookup lands
//! on, proven against GnuCOBOL 3.2. A real byte court (not an atlas) — it composes the sealed 1-based
//! subscript/decode model with the two COBOL search algorithms. The #2 surface gap by frequency in the
//! admitted testsuite (`SEARCH` 74×, per `GNURUST.PUBLIC.GAP.1`).
//!
//! **Witnessed rules (from the oracle):**
//! - **`SEARCH`** (serial) scans **forward only from the current index** for the first element whose key
//!   equals the target, returning its **1-based** index; a target before the start index is **not found**
//!   (`SET IX TO 3` then searching for a key at index 1 yields AT END).
//! - **`SEARCH ALL`** binary-searches the table's **ascending key**, returning the 1-based index of a match
//!   anywhere (independent of the start index); a missing key is not found.
//!
//! **Non-claims:** multi-key / `DESCENDING` keys, alphanumeric / signed / `V`-scaled keys (unsigned `9(n)`
//! here), the `VARYING`/`AT END`/`WHEN`-imperative control flow (only the landing index), `SEARCH ALL` on an
//! unsorted table (undefined), `OCCURS DEPENDING ON`, and all dialects.

use std::cmp::Ordering;

/// A keyed `OCCURS` table: where it starts, each element's width, and the key field within an element.
pub struct SearchTable {
    pub base_offset: usize,
    pub elem_size: usize,
    pub key_offset: usize,
    pub key_size: usize,
    pub occurs: usize,
}

/// The decoded unsigned key of the **1-based** element `i` (`0` if out of `1..=occurs`).
pub fn key_at(record: &[u8], t: &SearchTable, i: usize) -> i64 {
    if i < 1 || i > t.occurs {
        return 0;
    }
    let off = t.base_offset + (i - 1) * t.elem_size + t.key_offset;
    record
        .get(off..off + t.key_size)
        .map(|b| b.iter().fold(0i64, |a, &c| if c.is_ascii_digit() { a * 10 + (c - b'0') as i64 } else { a }))
        .unwrap_or(0)
}

/// `SEARCH <table>` (serial): scan **forward from `from`** (1-based) for the first element whose key equals
/// `target`; returns its 1-based index, or `None` (AT END).
pub fn search_serial(record: &[u8], t: &SearchTable, from: usize, target: i64) -> Option<usize> {
    (from.max(1)..=t.occurs).find(|&i| key_at(record, t, i) == target)
}

/// `SEARCH ALL <table>` (binary): binary-search the **ascending** key for `target`; returns the 1-based index
/// of a match, or `None`. Requires the table sorted ascending by key (an unsorted table is a non-claim).
pub fn search_all(record: &[u8], t: &SearchTable, target: i64) -> Option<usize> {
    let mut lo = 1usize;
    let mut hi = t.occurs;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        match key_at(record, t, mid).cmp(&target) {
            Ordering::Equal => return Some(mid),
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => {
                if mid == 1 {
                    break;
                }
                hi = mid - 1;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // E-KEY 9(3) OCCURS 5 = "010020050080099" (ascending)
    fn table() -> SearchTable {
        SearchTable { base_offset: 0, elem_size: 3, key_offset: 0, key_size: 3, occurs: 5 }
    }

    #[test]
    fn serial_search_is_forward_from_index() {
        let r = b"010020050080099";
        let t = table();
        assert_eq!(search_serial(r, &t, 1, 50), Some(3)); // found at 3
        assert_eq!(search_serial(r, &t, 1, 77), None); // not present
        assert_eq!(search_serial(r, &t, 3, 10), None); // 10 is at index 1, BEFORE the start -> not found
        assert_eq!(search_serial(r, &t, 1, 10), Some(1));
    }

    #[test]
    fn search_all_is_binary_on_ascending_key() {
        let r = b"010020050080099";
        let t = table();
        assert_eq!(search_all(r, &t, 80), Some(4));
        assert_eq!(search_all(r, &t, 10), Some(1));
        assert_eq!(search_all(r, &t, 99), Some(5));
        assert_eq!(search_all(r, &t, 55), None);
        assert_eq!(search_all(r, &t, 5), None);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.SEARCH.TABLE.1
    /// Soundness: any index either search returns is in 1..=occurs AND its key equals the target (no false
    /// positives, no out-of-bounds landing).
    #[kani::proof]
    #[kani::unwind(7)]
    fn search_results_are_sound() {
        let rec: [u8; 15] = kani::any();
        let t = SearchTable { base_offset: 0, elem_size: 3, key_offset: 0, key_size: 3, occurs: 5 };
        let target: i64 = kani::any();
        if let Some(i) = search_serial(&rec, &t, 1, target) {
            assert!((1..=5).contains(&i));
            assert_eq!(key_at(&rec, &t, i), target);
        }
        if let Some(i) = search_all(&rec, &t, target) {
            assert!((1..=5).contains(&i));
            assert_eq!(key_at(&rec, &t, i), target);
        }
    }
}
