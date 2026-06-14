//! Oracle mirror for the SCREEN SECTION DISPLAY framing court (`GNURUST.SCREENIO.INIT.1`).
//!
//! Reads the raw terminal byte stream of the canonical screen program (`DISPLAY "X" LINE 2 COLUMN 3.` then
//! `STOP RUN.`) captured from the oracle under a pty on stdin, builds the same `ScreenItem` list, emits the
//! native reproduction ([`gnucobol_rs::screenio::display_and_stop`]), and checks byte-identity. Prints
//! `PASS=n FAIL=n`.

use gnucobol_rs::screenio::{display_and_stop, ScreenItem};
use std::io::Read;

fn main() {
    let mut oracle = Vec::new();
    std::io::stdin().read_to_end(&mut oracle).unwrap();

    // The canonical program's single positioned item.
    let items = vec![ScreenItem { line: 2, column: 3, data: b"X".to_vec() }];
    let mine = display_and_stop(&items);

    if mine == oracle {
        println!("PASS screenio framing+display ({} bytes)", mine.len());
        println!("PASS=1 FAIL=0");
    } else {
        println!("FAIL screenio bytes differ");
        println!("  mine  ({}): {}", mine.len(), escape(&mine));
        println!("  oracle({}): {}", oracle.len(), escape(&oracle));
        println!("PASS=0 FAIL=1");
        std::process::exit(1);
    }
}

fn escape(b: &[u8]) -> String {
    let mut s = String::new();
    for &c in b {
        if c == 0x1b {
            s.push_str("<ESC>");
        } else if (0x20..0x7f).contains(&c) {
            s.push(c as char);
        } else {
            s.push_str(&format!("<{c:02x}>"));
        }
    }
    s
}
