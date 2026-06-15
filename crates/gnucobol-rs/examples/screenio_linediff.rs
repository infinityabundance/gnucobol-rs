//! Emit the native byte stream for TWO same-row SCREEN `DISPLAY`s, for the line-diff sweep
//! (`GNURUST.SCREENIO.LINEDIFF.1`). Usage: `screenio_linediff <row> <c1> <d1> <c2> <d2>` -> raw
//! terminal bytes on stdout (no ncurses linked).

use gnucobol_rs::screenio::two_display_line_and_stop;
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let row: i32 = a[0].parse().unwrap();
    let c1: i32 = a[1].parse().unwrap();
    let d1 = a[2].clone();
    let c2: i32 = a[3].parse().unwrap();
    let d2 = a[4].clone();
    let out = two_display_line_and_stop(row, c1, d1.as_bytes(), c2, d2.as_bytes());
    std::io::stdout().write_all(&out).unwrap();
}
