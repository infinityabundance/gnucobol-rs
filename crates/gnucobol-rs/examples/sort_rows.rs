//! Rust mirror for the SORT-comparison sweep (`GNURUST.FILEIO.SORT.1`). Reads `sorted=<hex>` (the bytes
//! of the oracle's GIVING file after a `SORT ON ASCENDING KEY K1 ON DESCENDING KEY K2`) and compares to
//! [`gnucobol_rs::fileio::sort_records`] over the same records and keys. PASS=n FAIL=n.
use gnucobol_rs::fileio::{cob_file_sort_init_key, sort_records};
use std::io::BufRead;

fn recs() -> Vec<&'static [u8]> {
    vec![b"BBB10xyz", b"AAA20xyz", b"BBB05xyz", b"AAA20abc", b"CCC00xyz"]
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
        // K1 = X(3) at offset 0 ASCENDING; K2 = X(2) at offset 3 DESCENDING
        let mut keys = Vec::new();
        cob_file_sort_init_key(&mut keys, 0, 3, true);
        cob_file_sort_init_key(&mut keys, 3, 2, false);
        let r = recs();
        let order = sort_records(&r, &keys, None);
        let mut out = Vec::new();
        for i in order {
            out.extend_from_slice(r[i]);
        }
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
