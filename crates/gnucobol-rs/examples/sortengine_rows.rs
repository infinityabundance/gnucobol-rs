//! Rust mirror for the SORT-engine sweep (`GNURUST.FILEIO.SORTENGINE.1`). Reads `sorted=<hex>` (the bytes
//! of the oracle's GIVING file after a `SORT ON ASCENDING KEY K1 ON DESCENDING KEY K2` over a larger,
//! duplicate-laden record set) and compares to the in-memory 4-queue natural-merge engine
//! [`gnucobol_rs::fileio::CobSort`] — submit every record, then drain in sorted order. PASS=n FAIL=n.
//!
//! The dataset is wide enough (16 records) to force several `cob_sort_queues` merge rounds, and carries
//! full-key duplicates (same K1+K2, different REST) so the `unique` insertion-order tiebreak (stability)
//! is exercised: equal-key records must come out in the order they were written.
use gnucobol_rs::fileio::CobSort;
use std::io::BufRead;

/// The records WRITEd by the oracle, in insertion order (8 bytes each: K1 X(3), K2 X(2), REST X(3)).
fn recs() -> Vec<&'static [u8]> {
    vec![
        b"BBB10p01", b"AAA20p02", b"BBB05p03", b"AAA20p04", b"CCC00p05", b"BBB10p06",
        b"AAA20p07", b"DDD99p08", b"BBB05p09", b"CCC00p10", b"AAA10p11", b"BBB10p12",
        b"AAA20p13", b"CCC50p14", b"DDD99p15", b"AAA10p16",
    ]
}
fn hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        if tag != "sorted" {
            continue;
        }
        // K1 = X(3) at offset 0 ASCENDING; K2 = X(2) at offset 3 DESCENDING; REST X(3) ignored.
        let mut s = CobSort::cob_file_sort_init(8, None);
        s.cob_file_sort_init_key(0, 3, true);
        s.cob_file_sort_init_key(3, 2, false);
        s.cob_file_sort_using(&recs());
        let out: Vec<u8> = s.cob_file_sort_giving().into_iter().flatten().collect();
        let mine = hex(&out);
        if mine == oracle.trim() {
            pass += 1;
        } else {
            println!("{tag} FAIL mine={mine} oracle={}", oracle.trim());
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
