//! Rust mirror for the WRITE sweep (`GNURUST.FILE.WRITE.1`). Reads `rs=<hex>` / `ls=<hex>` (the oracle output
//! file bytes) and compares to write_sequential for the same hardcoded records. PASS=n FAIL=n.
use gnucobol_rs::file_seq::{write_sequential, FileOrg};
use std::io::BufRead;
fn recs() -> Vec<&'static [u8]> {
    vec![b"AB", b"HELLO123", b"", b"XY", b"12345678"]
}
fn hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        let mine = match tag {
            "rs" => hex(&write_sequential(&recs(), FileOrg::RecordSequential, 8)),
            "ls" => hex(&write_sequential(&recs(), FileOrg::LineSequential, 8)),
            _ => continue,
        };
        if mine == oracle.trim() { pass += 1; } else { println!("{tag} FAIL mine={mine} oracle={}", oracle.trim()); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
