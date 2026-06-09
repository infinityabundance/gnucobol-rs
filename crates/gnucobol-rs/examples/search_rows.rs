//! Rust mirror for the SEARCH sweep (`GNURUST.SEARCH.TABLE.1`). Reads `label=<IX|notfound>` and recomputes the
//! landing index via search_serial/search_all over the same keyed table. PASS=n FAIL=n.
use gnucobol_rs::search::*;
use std::io::BufRead;
fn table() -> SearchTable { SearchTable { base_offset: 0, elem_size: 3, key_offset: 0, key_size: 3, occurs: 5 } }
fn compute(label: &str) -> Option<usize> {
    let r = b"010020050080099";
    let t = table();
    match label {
        "serial_50" => search_serial(r, &t, 1, 50),
        "serial_77" => search_serial(r, &t, 1, 77),
        "serial_from3_10" => search_serial(r, &t, 3, 10),
        "binary_80" => search_all(r, &t, 80),
        "binary_55" => search_all(r, &t, 55),
        "binary_10" => search_all(r, &t, 10),
        _ => None,
    }
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((label, oracle)) = line.split_once('=') else { continue };
        let oracle = oracle.trim();
        let mine = compute(label);
        let ok = if oracle == "notfound" {
            mine.is_none()
        } else {
            let o: usize = oracle.trim_start_matches('+').trim_start_matches('0').parse().unwrap_or(0);
            mine == Some(o)
        };
        if ok { pass += 1; } else { println!("{label} FAIL mine={mine:?} oracle={oracle}"); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
