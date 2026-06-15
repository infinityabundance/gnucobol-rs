//! Emit the native SCREEN SECTION ACCEPT byte stream for a single alphanumeric field, for the accept
//! sweep (`GNURUST.SCREENIO.ACCEPT.1`). Usage: `screenio_accept <line> <col> <width> <typed>` where
//! `<typed>` is the printable input entered before Enter (may be empty -- pass "") -> raw terminal
//! bytes on stdout (no ncurses linked).

use gnucobol_rs::screenio::accept_field_and_stop;
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let line: i32 = a[0].parse().unwrap();
    let col: i32 = a[1].parse().unwrap();
    let width: i32 = a[2].parse().unwrap();
    let typed = a.get(3).cloned().unwrap_or_default();
    let out = accept_field_and_stop(line, col, width, typed.as_bytes());
    std::io::stdout().write_all(&out).unwrap();
}
