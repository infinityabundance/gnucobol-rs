//! Rust mirror for the filter read-loop sweep (`GNURUST.FILE.FILTER.SLICE.1`).
use gnucobol_rs::file_flow_slice::*;
use gnucobol_rs::file_seq::FileOrg;
use gnucobol_rs::if_eval::{Relop, SliceField};
use std::collections::HashMap;
use std::io::BufRead;
fn unhex(s: &str) -> Vec<u8> { (0..s.len() / 2).map(|k| u8::from_str_radix(&s[k * 2..k * 2 + 2], 16).unwrap_or(0)).collect() }
fn parse(b: &[u8]) -> i64 { b.iter().fold(0i64, |a, &c| if c.is_ascii_digit() { a * 10 + (c - b'0') as i64 } else { a }) }
fn oracle(m: &HashMap<String, String>, k: &str) -> i64 {
    m.get(k).map(|s| parse(s.as_bytes())).unwrap_or(-1)
}
fn main() {
    let mut m: HashMap<String, String> = HashMap::new();
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        if let Some((k, v)) = line.split_once('=') { m.insert(k.to_string(), v.trim().to_string()); }
    }
    let file = unhex(m.get("file").map(|s| s.as_str()).unwrap_or(""));
    let rf = [SliceField { name: "R-ST", offset: 0, size: 1 }, SliceField { name: "R-AMT", offset: 1, size: 3 }];
    let wf = [SliceField { name: "CNT", offset: 0, size: 3 }, SliceField { name: "SM", offset: 3, size: 5 }];
    let body = [LoopOp::Count("CNT"), LoopOp::SumField { field: "R-AMT", into: "SM" }];
    let num = eval_filter_loop(&file, FileOrg::RecordSequential, 4, &rf, b"00000000", &wf, &FilterCond::Numeric { field: "R-AMT", op: Relop::Ge, value: 50 }, &body);
    let alpha = eval_filter_loop(&file, FileOrg::RecordSequential, 4, &rf, b"00000000", &wf, &FilterCond::Alpha { field: "R-ST", op: Relop::Eq, value: b"A" }, &body);
    let checks = [
        ("amt_ge50_count", parse(&num.ws[0..3])),
        ("amt_ge50_sum", parse(&num.ws[3..8])),
        ("st_A_count", parse(&alpha.ws[0..3])),
        ("st_A_sum", parse(&alpha.ws[3..8])),
    ];
    let (mut pass, mut fail) = (0u32, 0u32);
    for (k, mine) in checks {
        if mine == oracle(&m, k) { pass += 1; } else { println!("{k} FAIL mine={mine} oracle={}", oracle(&m, k)); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
