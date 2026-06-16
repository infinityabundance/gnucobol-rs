//! ALTERNATE RECORD KEY + WITH DUPLICATES + READ NEXT/PREVIOUS over an INDEXED file, modelled on the
//! built-cobc oracle. Record layout `F-ID X(3) | F-CAT X(2) | F-VAL X(5)`, primary key F-ID, alternate
//! key F-CAT WITH DUPLICATES; records 001AAhello / 002BBworld / 003AAagain (AA is a duplicate alt value).
//!
//! cobc oracle (FILE STATUS): all WRITEs `00` (the C sets 02 for a duplicate then clobbers it with the
//! DB_PUT result); `READ KEY IS F-CAT` of "AA" -> `00`, F-ID 001; `READ NEXT` -> `00`, F-ID 003 (AA dup 2);
//! `READ NEXT` -> `00`, F-ID 002 (BB). So a read-by-alt then READ NEXT walks the alternate-key index order
//! (alt-value asc, then dupno asc), two-hopping to the primary record.

use gnucobol_rs::fileio::{AccessMode, IndexedStore, OpenMode};

fn store_with_records() -> IndexedStore {
    let mut s = IndexedStore::indexed_open(0, 3, AccessMode::Dynamic, OpenMode::Io);
    let alt = s.indexed_add_alt_key(3, 2, true); // F-CAT at offset 3, len 2, WITH DUPLICATES
    assert_eq!(alt, 0);
    for r in [b"001AAhello", b"002BBworld", b"003AAagain"] {
        assert_eq!(s.indexed_write(r), "00"); // duplicate alt value still writes 00
    }
    s
}

#[test]
fn read_by_alt_then_read_next_walks_alt_order() {
    let mut s = store_with_records();
    // read by the alternate key value "AA" -> the first (lowest-dupno) primary, 001.
    let (st, rec) = s.indexed_read_alt(0, b"AA");
    assert_eq!(st, "00");
    assert_eq!(rec.as_deref(), Some(&b"001AAhello"[..]));
    // READ NEXT -> AA's second duplicate, 003.
    let (st, rec) = s.indexed_read_next();
    assert_eq!((st, rec.as_deref()), ("00", Some(&b"003AAagain"[..])));
    // READ NEXT -> the next alt value BB, 002.
    let (st, rec) = s.indexed_read_next();
    assert_eq!((st, rec.as_deref()), ("00", Some(&b"002BBworld"[..])));
    // READ NEXT -> end of the alternate-key index.
    assert_eq!(s.indexed_read_next().0, "10");
}

#[test]
fn read_previous_walks_alt_order_backwards() {
    let mut s = store_with_records();
    // position on BB, then walk back through AA's duplicates (highest dupno first).
    assert_eq!(s.indexed_read_alt(0, b"BB").1.as_deref(), Some(&b"002BBworld"[..]));
    assert_eq!(s.indexed_read_previous().1.as_deref(), Some(&b"003AAagain"[..])); // AA dup 2
    assert_eq!(s.indexed_read_previous().1.as_deref(), Some(&b"001AAhello"[..])); // AA dup 1
    assert_eq!(s.indexed_read_previous().0, "10"); // start of the index
}

#[test]
fn a_primary_read_reverts_read_next_to_primary_order() {
    let mut s = store_with_records();
    s.indexed_read_alt(0, b"AA"); // enter alt mode
    let (st, rec) = s.indexed_read(b"002"); // a primary read clears the alt cursor
    assert_eq!((st, rec.as_deref()), ("00", Some(&b"002BBworld"[..])));
    // READ NEXT now follows primary key order (after 002 -> 003), not alt order.
    assert_eq!(s.indexed_read_next().1.as_deref(), Some(&b"003AAagain"[..]));
}

#[test]
fn alt_index_serialises_with_le_dupno_trailer() {
    // The on-disk alt file encodes each entry as alt-value -> primary || dupno (native-LE u32, slot+1).
    let s = store_with_records();
    let bytes = s.indexed_alt_to_bdb(0).expect("serialise the alternate-key B-tree");
    let db = gnucobol_rs_bdb_format::BdbFile::parse(&bytes).unwrap();
    let pairs = db.records().unwrap();
    // AA appears twice (dupno 1 then 2), BB once (dupno 1); ascending alt-value, then dupno.
    assert_eq!(
        pairs,
        vec![
            (b"AA".to_vec(), b"001\x01\x00\x00\x00".to_vec()),
            (b"AA".to_vec(), b"003\x02\x00\x00\x00".to_vec()),
            (b"BB".to_vec(), b"002\x01\x00\x00\x00".to_vec()),
        ]
    );
}

#[test]
fn reads_a_genuine_cobc_alternate_key_file() {
    // The fixtures are the actual `data` (primary) and `data.1` (alternate, F-CAT WITH DUPLICATES) files
    // written by the genuine GnuCOBOL compiler for the program in the module doc -- proving the port reads
    // a real cobc alternate-key index (the shared duplicate-key item + the LE dupno trailer) end to end.
    const PRIMARY: &[u8] = include_bytes!("fixtures/indexed_altk_primary.db");
    const ALT1: &[u8] = include_bytes!("fixtures/indexed_altk_alt1.db");

    let mut s = IndexedStore::indexed_open(0, 3, AccessMode::Dynamic, OpenMode::Input);
    s.indexed_add_alt_key(3, 2, true);
    assert_eq!(s.indexed_load_bdb(PRIMARY).unwrap(), 3);
    assert_eq!(s.indexed_load_alt(0, ALT1).unwrap(), 3, "three alt entries (AA x2, BB)");

    // read by the genuine alternate index: AA -> 001, then its duplicate 003, then BB -> 002.
    assert_eq!(s.indexed_read_alt(0, b"AA").1.as_deref(), Some(&b"001AAhello"[..]));
    assert_eq!(s.indexed_read_next().1.as_deref(), Some(&b"003AAagain"[..]));
    assert_eq!(s.indexed_read_next().1.as_deref(), Some(&b"002BBworld"[..]));
    assert_eq!(s.indexed_read_next().0, "10");
    // an absent alternate value is not found.
    assert_eq!(s.indexed_read_alt(0, b"ZZ").0, "23");
}

#[test]
fn alt_index_round_trips_through_the_on_disk_file() {
    // Serialise the primary + alternate files, then reload both into a fresh store and read by alt.
    let src = store_with_records();
    let prim = src.indexed_to_bdb().unwrap();
    let alt = src.indexed_alt_to_bdb(0).unwrap();

    let mut back = IndexedStore::indexed_open(0, 3, AccessMode::Dynamic, OpenMode::Input);
    back.indexed_add_alt_key(3, 2, true);
    back.indexed_load_bdb(&prim).unwrap();
    back.indexed_load_alt(0, &alt).unwrap(); // replace the alt index with the genuine on-disk image

    assert_eq!(back.indexed_read_alt(0, b"AA").1.as_deref(), Some(&b"001AAhello"[..]));
    assert_eq!(back.indexed_read_next().1.as_deref(), Some(&b"003AAagain"[..]));
    assert_eq!(back.indexed_read_next().1.as_deref(), Some(&b"002BBworld"[..]));
}
