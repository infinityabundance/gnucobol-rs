//! Rust mirror for the read-loop sweep (`GNURUST.FILE.FLOW.SLICE.1`). Reads `file=<hex>`, `count=<n>`,
//! `sum=<n>` and recomputes COUNT/SUM via eval_read_loop over the same file bytes. PASS=n FAIL=n.
use gnucobol_rs::file_flow_slice::*;
use gnucobol_rs::file_seq::FileOrg;
use gnucobol_rs::if_eval::SliceField;
use std::collections::HashMap;
use std::io::BufRead;
fn unhex(s: &str) -> Vec<u8> { (0..s.len() / 2).map(|k| u8::from_str_radix(&s[k * 2..k * 2 + 2], 16).unwrap_or(0)).collect() }
fn parse(b: &[u8]) -> i64 { b.iter().fold(0i64, |a, &c| if c.is_ascii_digit() { a * 10 + (c - b'0') as i64 } else { a }) }
fn main() {
    let mut m: HashMap<String, String> = HashMap::new();
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        if let Some((k, v)) = line.split_once('=') { m.insert(k.to_string(), v.trim().to_string()); }
    }
    let file = unhex(m.get("file").map(|s| s.as_str()).unwrap_or(""));
    let record_fields = [SliceField { name: "R-ID", offset: 0, size: 2 }, SliceField { name: "R-AMT", offset: 2, size: 3 }];
    let ws_fields = [SliceField { name: "CNT", offset: 0, size: 3 }, SliceField { name: "SM", offset: 3, size: 5 }];
    let body = [LoopOp::Count("CNT"), LoopOp::SumField { field: "R-AMT", into: "SM" }];
    let r = eval_read_loop(&file, FileOrg::RecordSequential, 5, &record_fields, b"00000000", &ws_fields, &body);
    let (mut pass, mut fail) = (0u32, 0u32);
    let mine_cnt = parse(&r.ws[0..3]);
    let mine_sum = parse(&r.ws[3..8]);
    let oc: i64 = m.get("count").and_then(|s| s.trim_start_matches('0').parse().ok()).unwrap_or(if m.get("count").map(|s| s.chars().all(|c| c == '0')).unwrap_or(false) { 0 } else { -1 });
    let os: i64 = m.get("sum").and_then(|s| s.trim_start_matches('0').parse().ok()).unwrap_or(if m.get("sum").map(|s| s.chars().all(|c| c == '0')).unwrap_or(false) { 0 } else { -1 });
    if mine_cnt == oc { pass += 1; } else { println!("count FAIL mine={mine_cnt} oracle={oc}"); fail += 1; }
    if mine_sum == os { pass += 1; } else { println!("sum FAIL mine={mine_sum} oracle={os}"); fail += 1; }
    println!("PASS={pass} FAIL={fail}");
}
