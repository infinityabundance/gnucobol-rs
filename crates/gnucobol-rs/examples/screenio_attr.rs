//! Emit the native SCREEN SECTION DISPLAY byte stream for a single attributed field, for the attribute
//! sweep (`GNURUST.SCREENIO.ATTR.1`). Usage: `screenio_attr <line> <col> <text> <attr>` where attr is one
//! of highlight|lowlight|underline|blink|reverse -> raw bytes on stdout.

use gnucobol_rs::screenio::{display_and_stop, ScreenAttr, ScreenItem};
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let line: i32 = a[0].parse().unwrap();
    let column: i32 = a[1].parse().unwrap();
    let text = a[2].clone();
    let attr = match a[3].as_str() {
        "highlight" => ScreenAttr::Highlight,
        "lowlight" => ScreenAttr::Lowlight,
        "underline" => ScreenAttr::Underline,
        "blink" => ScreenAttr::Blink,
        "reverse" => ScreenAttr::Reverse,
        other => panic!("unknown attr {other}"),
    };
    let items = vec![ScreenItem::with_attr(line, column, text.into_bytes(), attr)];
    std::io::stdout().write_all(&display_and_stop(&items)).unwrap();
}
