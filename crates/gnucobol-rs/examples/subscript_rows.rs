//! Rust mirror of the table-subscript oracle (`GNURUST.SUBSCRIPT.1`): reads `label|shape|fieldhex|i|j`,
//! runs element_1d / element_2d, prints `label hex`.
use gnucobol_rs::{element_1d, element_2d};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('|').collect();
        if f.len() != 5 {
            continue;
        }
        let (label, shape, hex) = (f[0], f[1], f[2]);
        let (i, j) = (f[3].parse::<usize>().unwrap_or(0), f[4].parse::<usize>().unwrap_or(0));
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|k| u8::from_str_radix(&hex[k * 2..k * 2 + 2], 16).unwrap_or(0))
            .collect();
        let r = match shape {
            "1d" => element_1d(&bytes, 3, i),
            "2d" => element_2d(&bytes, 2, 4, i, j),
            _ => Ok(&bytes[..0]),
        };
        let h = match r {
            Ok(b) => b.iter().map(|x| format!("{x:02x}")).collect::<String>(),
            Err(_) => "ERR".into(),
        };
        let _ = writeln!(out, "{label} {h}");
    }
}
