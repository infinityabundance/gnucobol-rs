//! Rust mirror for the SIZE ERROR sweep (`GNURUST.SIZE.ERROR.1`). For each case: `label_t=[trunc]` checks the
//! no-ON-SIZE-ERROR truncated store; `label_e=flag` checks the size-error condition. PASS=n FAIL=n.
use gnucobol_rs::size_error::arith_size_error;
use std::io::BufRead;
fn spec(label: &str) -> (&'static [u8], &'static [u8], usize, usize) {
    match label {
        "a" => (b"1998", b"", 3, 0),   // 999+999 into 9(3)
        "b" => (b"1234", b"567", 3, 2), // 1234.567 into 9(3)V99
        "c" => (b"46", b"", 3, 0),      // 12+34 into 9(3)
        "d" => (b"50000", b"", 3, 0),   // 50000 into 9(3)
        "e" => (b"12", b"5", 1, 1),     // 12.5 into 9(1)V9
        "f" => (b"7", b"89", 1, 1),     // 7.89 into 9(1)V9
        _ => (b"", b"", 0, 0),
    }
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, val)) = line.split_once('=') else { continue };
        let (label, kind) = (&tag[..tag.len() - 2], &tag[tag.len() - 1..]);
        let (i, f, ri, rs) = spec(label);
        let r = arith_size_error(i, f, ri, rs);
        let ok = if kind == "t" {
            let oracle: Vec<u8> = val.bytes().filter(|&b| b != b'[' && b != b']' && b != b'.').collect();
            r.truncated == oracle
        } else {
            r.size_error == (val.trim() == "1")
        };
        if ok { pass += 1; } else { println!("{tag} FAIL mine_t={:?} se={} val={val}", String::from_utf8_lossy(&r.truncated), r.size_error); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
