//! Emit the native SCREEN SECTION DISPLAY byte stream for one or more positioned fields, for the grid
//! sweeps (`GNURUST.SCREENIO.DISPLAY.2` / `.3`). Usage: `screenio_emit <line> <col> <text> [<line> <col>
//! <text> ...]` -> raw bytes on stdout, to be `cmp`'d against the oracle pty capture.

use gnucobol_rs::screenio::{display_and_stop, ScreenItem};
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut items = Vec::new();
    for triple in args.chunks(3) {
        if triple.len() < 3 {
            break;
        }
        items.push(ScreenItem::plain(
            triple[0].parse().unwrap_or(1),
            triple[1].parse().unwrap_or(1),
            triple[2].clone().into_bytes(),
        ));
    }
    std::io::stdout().write_all(&display_and_stop(&items)).unwrap();
}
