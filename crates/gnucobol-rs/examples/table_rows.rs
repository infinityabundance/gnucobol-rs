//! Rust mirror for the table PERFORM slice sweep (`GNURUST.TABLE.PERFORM.SLICE.1`).
use gnucobol_rs::if_eval::Relop;
use gnucobol_rs::table_slice::*;
use std::collections::HashMap;
use std::io::BufRead;
fn parse(b: &[u8]) -> i64 { b.iter().fold(0i64, |a, &c| if c.is_ascii_digit() { a * 10 + (c - b'0') as i64 } else { a }) }
fn main() {
    let mut m: HashMap<String, String> = HashMap::new();
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        if let Some((k, v)) = line.split_once('=') { m.insert(k.to_string(), v.trim().trim_start_matches('[').trim_end_matches(']').to_string()); }
    }
    let tbl_bytes = m.get("table").cloned().unwrap_or_default().into_bytes();
    let t = Table { base_offset: 0, elem_size: 3, occurs: 5 };
    let checks = [
        ("sum", eval_table_loop(&tbl_bytes, &t, 1, 1, 5, None).sum),
        ("ge50_count", eval_table_loop(&tbl_bytes, &t, 1, 1, 5, Some((Relop::Ge, 50))).count as i64),
        ("sum_by2", eval_table_loop(&tbl_bytes, &t, 1, 2, 5, None).sum),
    ];
    let (mut pass, mut fail) = (0u32, 0u32);
    for (k, mine) in checks {
        let o = m.get(k).map(|s| parse(s.as_bytes())).unwrap_or(-1);
        if mine == o { pass += 1; } else { println!("{k} FAIL mine={mine} oracle={o}"); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
