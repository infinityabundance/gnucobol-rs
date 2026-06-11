//! Rust mirror of the OCCURS DEPENDING ON oracle (`GNURUST.ODO.1`): reads `label|type|N|i|contenthex`,
//! runs odo_used_length / odo_element, prints `label hex`.
use gnucobol_rs::{odo_element, odo_used_length};
use std::io::{self, BufRead, Write};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

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
        let (label, ty, n) = (f[0], f[1], f[2].parse::<usize>().unwrap_or(0));
        let i = f[3].parse::<usize>().unwrap_or(0);
        let content: Vec<u8> = (0..f[4].len() / 2)
            .map(|k| u8::from_str_radix(&f[4][k * 2..k * 2 + 2], 16).unwrap_or(0))
            .collect();
        let h = match ty {
            "len" => hex(format!("{:03}", odo_used_length(n, 1, 3)).as_bytes()),
            "elem" => match odo_element(&content, 1, 3, n, i) {
                Ok(b) => hex(b),
                Err(_) => "ERR".into(),
            },
            _ => String::new(),
        };
        let _ = writeln!(out, "{label} {h}");
    }
}
