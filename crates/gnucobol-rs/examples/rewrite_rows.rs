//! Rust mirror for the REWRITE sweep (`GNURUST.FILE.REWRITE.1`). Reads `rw=<hex>` (the oracle file after
//! OPEN I-O REWRITE) and compares to rewrite_records over the known initial file. PASS=n FAIL=n.
use gnucobol_rs::file_seq::rewrite_records;
use std::io::BufRead;
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        if tag != "rw" { continue; }
        // initial RECORD SEQUENTIAL file "AAAABBBBCCCC" (record_len 4); rewrite records 0 and 2
        let mine: String = rewrite_records(b"AAAABBBBCCCC", 4, &[(0, b"X1X1"), (2, b"Z3Z3")])
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        if mine == oracle.trim() { pass += 1; } else { println!("rw FAIL mine={mine} oracle={}", oracle.trim()); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
