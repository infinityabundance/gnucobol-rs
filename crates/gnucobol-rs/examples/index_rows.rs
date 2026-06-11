//! Rust mirror of the USAGE INDEX oracle (`GNURUST.INDEX.1`): reads `label|start|op|k`, runs
//! set_index_to / set_index_up_by / set_index_down_by, prints `label <hex of the 4 index bytes>`.
use gnucobol_rs::{set_index_down_by, set_index_to, set_index_up_by};
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
        if f.len() != 4 {
            continue;
        }
        let label = f[0];
        let start = f[1].parse::<i32>().unwrap_or(0);
        let op = f[2];
        let k = f[3].parse::<i32>().unwrap_or(0);
        let base = set_index_to(start);
        let bytes = match op {
            "to" => base,
            "up" => set_index_up_by(&base, k),
            "down" => set_index_down_by(&base, k),
            _ => base,
        };
        let _ = writeln!(out, "{label} {}", hex(&bytes));
    }
}
