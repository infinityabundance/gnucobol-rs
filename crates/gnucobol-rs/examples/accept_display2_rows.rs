//! Rust mirror for the DISPLAY-numeric sweep (`GNURUST.ACCEPT.DISPLAY.2`). Reads `label=[content]` from the
//! oracle and recomputes each via display_numeric. PASS=n FAIL=n.
use gnucobol_rs::accept_display::display_numeric;
use std::io::BufRead;
fn compute(label: &str) -> Vec<u8> {
    // (digits, scale, signed, negative) per probed field/value
    let (d, sc, sg, ng): (&[u8], usize, bool, bool) = match label {
        "sn"  => (b"042", 0, true, true),       // S9(3) = -42
        "sp"  => (b"042", 0, true, false),      // S9(3) = +42
        "sz"  => (b"000", 0, true, false),      // S9(3) = 0
        "uv"  => (b"01234", 2, false, false),   // 9(3)V99 = 12.34
        "snv" => (b"01234", 2, true, true),     // S9(3)V99 = -12.34
        "spv" => (b"01234", 2, true, false),    // S9(3)V99 = +12.34
        "big" => (b"00123456", 3, true, true),  // S9(5)V9(3) = -123.456
        "uz"  => (b"000", 1, false, false),     // 9(2)V9 = 0.0
        _ => return Vec::new(),
    };
    display_numeric(d, sc, sg, ng)
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((label, content)) = line.split_once('=') else { continue };
        let oracle = content.trim_start_matches('[').trim_end_matches(']');
        if compute(label) == oracle.as_bytes() { pass += 1; }
        else { println!("{label} FAIL mine={:?} oracle={:?}", String::from_utf8_lossy(&compute(label)), oracle); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
