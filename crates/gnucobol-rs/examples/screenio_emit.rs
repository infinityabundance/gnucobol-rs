//! Emit the native SCREEN SECTION DISPLAY byte stream for a single positioned field, for the grid sweep
//! (`GNURUST.SCREENIO.DISPLAY.2`). Usage: `screenio_emit <line> <column> <text>` -> raw bytes on stdout,
//! to be `cmp`'d against the oracle pty capture of `DISPLAY <text> LINE <line> COLUMN <column>`.

use gnucobol_rs::screenio::{display_and_stop, ScreenItem};
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let line: i32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let column: i32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let text = args.get(3).cloned().unwrap_or_else(|| "Z".to_string());
    let items = vec![ScreenItem { line, column, data: text.into_bytes() }];
    std::io::stdout().write_all(&display_and_stop(&items)).unwrap();
}
