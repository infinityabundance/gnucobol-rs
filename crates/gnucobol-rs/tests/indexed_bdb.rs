//! End-to-end: the port reads an INDEXED file written by the *genuine* GnuCOBOL compiler. The fixture
//! `indexed_cobc.db` is a Berkeley DB B-tree `.dat` produced by real `cobc` (libcob over Berkeley DB)
//! for an `ORGANIZATION IS INDEXED` file with `RECORD KEY` PIC X(4) at offset 0 and a 14-byte record
//! (records 0001/0002/0003 written out of order). `IndexedStore::indexed_load_bdb` parses it via the
//! pure-safe `gnucobol-rs-bdb-format` crate and populates the store, so READ by key and READ NEXT work.

use gnucobol_rs::fileio::{AccessMode, IndexedStore, OpenMode};

const COBC_INDEXED: &[u8] = include_bytes!("fixtures/indexed_cobc.db");

#[test]
fn reads_a_cobc_written_indexed_file() {
    let mut store = IndexedStore::indexed_open(0, 4, AccessMode::Dynamic, OpenMode::Input);
    let n = store.indexed_load_bdb(COBC_INDEXED).expect("a cobc-written BDB B-tree .dat parses");
    assert_eq!(n, 3, "all three records loaded from the genuine GnuCOBOL file");

    // random READ by key returns the full record image cobc stored
    let (st, rec) = store.indexed_read(b"0002");
    assert_eq!(st, "00");
    assert_eq!(rec.as_deref(), Some(&b"0002two       "[..]));

    let (st, rec) = store.indexed_read(b"0003");
    assert_eq!(st, "00");
    assert_eq!(rec.as_deref(), Some(&b"0003three     "[..]));

    // a missing key fails closed (no such record)
    let (st, _) = store.indexed_read(b"9999");
    assert_ne!(st, "00", "absent key is not status 00");

    // sequential READ NEXT walks the records in ascending key order
    let mut next = IndexedStore::indexed_open(0, 4, AccessMode::Dynamic, OpenMode::Input);
    next.indexed_load_bdb(COBC_INDEXED).unwrap();
    let mut keys = Vec::new();
    loop {
        let (st, rec) = next.indexed_read_next();
        if st != "00" {
            break;
        }
        keys.push(rec.unwrap()[..4].to_vec());
    }
    assert_eq!(
        keys,
        vec![b"0001".to_vec(), b"0002".to_vec(), b"0003".to_vec()],
        "READ NEXT yields keys in ascending order"
    );
}

#[test]
fn writes_a_dat_then_reads_it_back() {
    // Write the INDEXED file via the port, then read it back -- the write side of interchange. (The
    // separate gnucobol-rs-bdb-format crate proves the genuine cobc OPENs + READs a port-written file;
    // here we prove the gnucobol-rs store round-trips through its own writer + reader.)
    let mut store = IndexedStore::indexed_open(0, 4, AccessMode::Dynamic, OpenMode::Io);
    // load the cobc fixture as the source records, then re-serialise + re-read
    store.indexed_load_bdb(COBC_INDEXED).unwrap();
    let bytes = store.indexed_to_bdb().expect("serialise to a BDB .dat");

    let mut back = IndexedStore::indexed_open(0, 4, AccessMode::Dynamic, OpenMode::Input);
    assert_eq!(back.indexed_load_bdb(&bytes).unwrap(), 3, "all records survive the write+read");
    assert_eq!(back.indexed_read(b"0001").1.as_deref(), Some(&b"0001one       "[..]));
    assert_eq!(back.indexed_read(b"0003").1.as_deref(), Some(&b"0003three     "[..]));
}

#[test]
fn rejects_a_non_indexed_buffer() {
    let mut store = IndexedStore::indexed_open(0, 4, AccessMode::Dynamic, OpenMode::Input);
    // an empty / never-written file is not a B-tree DB: a typed error, not a panic or silent success.
    assert!(store.indexed_load_bdb(&[0u8; 16]).is_err());
}
