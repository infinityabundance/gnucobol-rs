//! Rust mirror for the INDEXED sweep (`GNURUST.FILEIO.INDEXED.1`). Replays a fixed INDEXED script on
//! [`gnucobol_rs::fileio::IndexedStore`] and checks each result against the admitted libcob oracle's
//! DISPLAY for the same operation. The oracle emits one `tag rest` line per step; this mirror computes the
//! same `tag rest` from the in-memory keyed store and compares pairwise. PASS=n FAIL=n.
//!
//! Script (primary RECORD KEY = K, 3 bytes; record = K(3)+V(5), 8 bytes):
//!   WRITE BBB/DDD/FFF (00,00,00); WRITE BBB dup (22); READ DDD (00 ddddd); READ CCC (23);
//!   READ NEXT x3 in key order (BBB,DDD,FFF); START >=CCC (00) + READ NEXT (DDD); START >FFF (23);
//!   REWRITE DDD->DDD22 (00) + READ DDD (00 DDD22); REWRITE GGG (23);
//!   DELETE BBB (00) + READ BBB (23); DELETE BBB again (23).
use gnucobol_rs::fileio::{AccessMode, IndexedStore, OpenMode, StartCond};
use std::io::BufRead;

fn rec(k: &str, v: &str) -> Vec<u8> {
    let mut r = Vec::new();
    r.extend_from_slice(k.as_bytes());
    r.extend_from_slice(v.as_bytes());
    r
}
fn txt(b: &[u8]) -> String {
    String::from_utf8_lossy(b).to_string()
}

/// Run the fixed script and return the ordered `(tag, result)` lines this implementation produces.
fn script() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut s = IndexedStore::indexed_open(0, 3, AccessMode::Dynamic, OpenMode::Output);
    out.push(("w1".into(), s.indexed_write(&rec("BBB", "bbbbb")).to_string()));
    out.push(("w2".into(), s.indexed_write(&rec("DDD", "ddddd")).to_string()));
    out.push(("w3".into(), s.indexed_write(&rec("FFF", "fffff")).to_string()));
    out.push(("wdup".into(), s.indexed_write(&rec("BBB", "dupbb")).to_string()));
    s.indexed_close();
    // OPEN I-O: random read, read-next sweep, start, rewrite, delete
    s.open_mode = OpenMode::Io;
    let (st, r) = s.indexed_read(b"DDD");
    out.push(("rDDD".into(), format!("{st} {}", r.map(|x| txt(&x[3..])).unwrap_or_default())));
    let (st, _) = s.indexed_read(b"CCC");
    out.push(("rCCC".into(), st.to_string()));
    // READ NEXT from the beginning (fresh cursor)
    s.indexed_start(StartCond::Ge, b"\x00\x00\x00");
    for tag in ["n1", "n2", "n3"] {
        let (st, r) = s.indexed_read_next();
        let r = r.unwrap_or_default();
        out.push((tag.into(), format!("{st} {} {}", txt(&r[..3]), txt(&r[3..]))));
    }
    // START >= CCC then READ NEXT -> DDD
    out.push(("ge".into(), s.indexed_start(StartCond::Ge, b"CCC").to_string()));
    let (st, r) = s.indexed_read_next();
    out.push(("gen".into(), format!("{st} {}", r.map(|x| txt(&x[..3])).unwrap_or_default())));
    // START > FFF -> 23
    out.push(("gtf".into(), s.indexed_start(StartCond::Gt, b"FFF").to_string()));
    // REWRITE DDD -> DDD22, read back
    out.push(("rwD".into(), s.indexed_rewrite(&rec("DDD", "DD222")).to_string()));
    let (st, r) = s.indexed_read(b"DDD");
    out.push(("rrD".into(), format!("{st} {}", r.map(|x| txt(&x[3..])).unwrap_or_default())));
    out.push(("rwG".into(), s.indexed_rewrite(&rec("GGG", "ggggg")).to_string()));
    // DELETE BBB, read back, delete again
    out.push(("delB".into(), s.indexed_delete(b"BBB").to_string()));
    out.push(("rB".into(), s.indexed_read(b"BBB").0.to_string()));
    out.push(("delB2".into(), s.indexed_delete(b"BBB").to_string()));
    out
}

fn main() {
    let mine = script();
    let mut expect = std::collections::HashMap::new();
    for (tag, res) in &mine {
        expect.insert(tag.clone(), res.trim().to_string());
    }
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (tag, rest) = match line.split_once(' ') {
            Some((t, r)) => (t.to_string(), r.trim().to_string()),
            None => (line.to_string(), String::new()),
        };
        match expect.get(&tag) {
            Some(mine_res) if *mine_res == rest => pass += 1,
            Some(mine_res) => {
                println!("{tag} FAIL mine='{mine_res}' oracle='{rest}'");
                fail += 1;
            }
            None => {}
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
