//! Rust mirror for the ACCEPT/DISPLAY sweep (`GNURUST.ACCEPT.DISPLAY.1`). Reads `label=<content>` lines from
//! the oracle and recomputes each via display_line/accept_field. PASS=n FAIL=n.
use gnucobol_rs::accept_display::{accept_field, display_line};
use std::io::BufRead;
fn nolf(mut v: Vec<u8>) -> Vec<u8> { if v.last() == Some(&b'\n') { v.pop(); } v }
fn compute(label: &str) -> Vec<u8> {
    match label {
        "d_lit"   => nolf(display_line(&[b"ABC"])),
        "d_alnum" => nolf(display_line(&[b"HEL  "])),
        "d_unum"  => nolf(display_line(&[b"042"])),
        "d_multi" => nolf(display_line(&[b"X", b"Y", b"Z"])),
        "d_cat"   => nolf(display_line(&[b"HEL  ", b"042"])),
        "a_short" => accept_field(b"HI", 6),
        "a_long"  => accept_field(b"ABCDEFGH", 6),
        _ => Vec::new(),
    }
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((label, content)) = line.split_once('=') else { continue };
        if compute(label) == content.as_bytes() { pass += 1; }
        else { println!("{label} FAIL mine={:?} oracle={:?}", String::from_utf8_lossy(&compute(label)), content); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
