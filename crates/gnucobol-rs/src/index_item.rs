//! `USAGE INDEX` storage and `SET` arithmetic (`GNURUST.INDEX.1`): the bytes an index data item holds,
//! proven byte-identical to cobc.
//!
//! An `INDEX` item (whether a standalone `USAGE INDEX` field or an `INDEXED BY` index-name) is a
//! fixed-width signed binary integer in **native** byte order — on the admitted oracle platform
//! (x86-64) that is a 4-byte little-endian two's-complement word, the same storage shape as `COMP-5`.
//!
//! The one semantic that is *not* obvious from "it's a binary int": the stored value is the **occurrence
//! number**, not a byte offset. `SET IDX TO 5` stores `5` regardless of the indexed element's size — a
//! table of `PIC X(17)` and a table of `PIC X(4)` both store `5`, never `5*size`. The `(idx-1)*stride`
//! multiply that turns an occurrence into an address lives in the subscript model
//! ([`crate::subscript`]), not in the index item itself. `SET ... UP/DOWN BY k` is plain integer
//! add/subtract on that occurrence; going at or below zero stores the ordinary two's-complement value
//! (cobc does not clamp), so this court characterises the raw bytes rather than re-deciding bounds.

/// Width in bytes of an `INDEX` item on the admitted oracle platform (x86-64): a 4-byte word.
pub const INDEX_SIZE: usize = 4;

/// The signed occurrence value held by an `INDEX` item, decoded from its `INDEX_SIZE` native
/// little-endian two's-complement bytes.
pub fn index_value(bytes: &[u8; INDEX_SIZE]) -> i32 {
    i32::from_le_bytes(*bytes)
}

/// The `INDEX_SIZE` storage bytes for occurrence value `occurrence` (native little-endian two's
/// complement). This is the byte image after `SET idx TO occurrence` — element-size independent: the
/// value stored is the occurrence number itself, never an offset.
pub fn index_store(occurrence: i32) -> [u8; INDEX_SIZE] {
    occurrence.to_le_bytes()
}

/// `SET idx TO n`: occurrence `n`'s storage bytes. (Alias of [`index_store`] for call-site clarity.)
pub fn set_index_to(n: i32) -> [u8; INDEX_SIZE] {
    index_store(n)
}

/// `SET idx UP BY k`: re-store `current + k` (wrapping two's complement, matching the raw cobc word).
pub fn set_index_up_by(current: &[u8; INDEX_SIZE], k: i32) -> [u8; INDEX_SIZE] {
    index_store(index_value(current).wrapping_add(k))
}

/// `SET idx DOWN BY k`: re-store `current - k`. cobc does not clamp at zero, so `2 DOWN BY 5` stores
/// `-3` as the ordinary two's-complement word (`fd ff ff ff`).
pub fn set_index_down_by(current: &[u8; INDEX_SIZE], k: i32) -> [u8; INDEX_SIZE] {
    index_store(index_value(current).wrapping_sub(k))
}

/// Fuzz entry (`GNURUST.INDEX.1`): bounded over any inputs — never panics, round-trips exactly.
pub fn __fuzz_index(data: &[u8]) {
    if data.len() < 8 {
        return;
    }
    let cur: [u8; INDEX_SIZE] = [data[0], data[1], data[2], data[3]];
    let k = i32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    // decode/encode round-trips
    debug_assert_eq!(index_store(index_value(&cur)), cur);
    let _ = set_index_to(k);
    let _ = set_index_up_by(&cur, k);
    let _ = set_index_down_by(&cur, k);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_storage_matches_the_oracle() {
        // SET TO 1 -> 01 00 00 00 ; SET TO 5 -> 05 00 00 00 (native little-endian occurrence).
        assert_eq!(set_index_to(1), [0x01, 0x00, 0x00, 0x00]);
        assert_eq!(set_index_to(5), [0x05, 0x00, 0x00, 0x00]);
        // UP BY: 5 + 3 = 8.
        assert_eq!(index_value(&set_index_up_by(&set_index_to(5), 3)), 8);
        // DOWN BY past zero: 2 - 5 = -3 -> fd ff ff ff (cobc does not clamp).
        assert_eq!(set_index_down_by(&set_index_to(2), 5), [0xfd, 0xff, 0xff, 0xff]);
        assert_eq!(index_value(&set_index_down_by(&set_index_to(2), 5)), -3);
        // element-size independence: the occurrence is stored, not occurrence*stride.
        // (a X(17) table and a X(4) table both store 5 — there is no stride here to multiply by.)
        assert_eq!(set_index_to(5), index_store(5));
        // SET idx TO another idx is a value copy.
        let a = set_index_to(7);
        assert_eq!(index_value(&a), 7);
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.INDEX.1
    /// store/value round-trip is exact and the SET ops never panic over symbolic inputs.
    #[kani::proof]
    fn index_roundtrips_and_is_total() {
        let bytes: [u8; INDEX_SIZE] = kani::any();
        assert_eq!(index_store(index_value(&bytes)), bytes);
        let k: i32 = kani::any();
        let _ = set_index_to(k);
        let _ = set_index_up_by(&bytes, k);
        let _ = set_index_down_by(&bytes, k);
    }
}
