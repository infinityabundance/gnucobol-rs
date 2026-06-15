//! Emit the native SCREEN SECTION DISPLAY byte stream for a single NUMERIC-EDITED field, for the
//! numeric-edited sweep (`GNURUST.SCREENIO.NUMEDIT.1`). Usage:
//! `screenio_numedit <line> <col> <pic> <value>` where `<value>` is a signed decimal literal (e.g.
//! `1234.56`, `-88.10`). The edited field image is produced by the sealed move/edit engine
//! (`edited::encode_edited`), then positioned by `screenio::display_edited_and_stop`. Raw bytes ->
//! stdout (no ncurses linked).

use gnucobol_rs::edited::encode_edited;
use gnucobol_rs::screenio::display_edited_and_stop;
use gnucobol_rs::value::Decimal;
use std::io::Write;

/// Parse a signed decimal literal like `-88.10` into a [`Decimal`] (digits 0..=9 + scale + sign).
fn parse_decimal(s: &str) -> Decimal {
    let negative = s.starts_with('-');
    let body = s.trim_start_matches(['+', '-']);
    let (int_part, frac_part) = match body.split_once('.') {
        Some((i, f)) => (i, f),
        None => (body, ""),
    };
    let mut digits: Vec<u8> = Vec::new();
    for c in int_part.chars().chain(frac_part.chars()) {
        if c.is_ascii_digit() {
            digits.push((c as u8) - b'0');
        }
    }
    let scale = frac_part.chars().filter(|c| c.is_ascii_digit()).count() as i16;
    Decimal { negative, digits, scale }
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let line: i32 = a[0].parse().unwrap();
    let column: i32 = a[1].parse().unwrap();
    let pic = a[2].clone();
    let value = parse_decimal(&a[3]);
    let edited = encode_edited(&pic, &value).expect("encode_edited");
    let out = display_edited_and_stop(line, column, &edited);
    std::io::stdout().write_all(&out).unwrap();
}
