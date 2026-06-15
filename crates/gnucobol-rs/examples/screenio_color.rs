//! Emit the native SCREEN SECTION DISPLAY byte stream for a single COLORED field, for the colour
//! sweep (`GNURUST.SCREENIO.COLOR.1`). Usage: `screenio_color <line> <col> <text> <fg> <bg>` where
//! `fg`/`bg` are COBOL colour numbers 0..=7 -> raw terminal bytes on stdout (no ncurses linked).

use gnucobol_rs::screenio::color_display_and_stop;
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let line: i32 = a[0].parse().unwrap();
    let column: i32 = a[1].parse().unwrap();
    let text = a[2].clone();
    let fg: u8 = a[3].parse().unwrap();
    let bg: u8 = a[4].parse().unwrap();
    let out = color_display_and_stop(line, column, text.as_bytes(), fg, bg);
    std::io::stdout().write_all(&out).unwrap();
}
