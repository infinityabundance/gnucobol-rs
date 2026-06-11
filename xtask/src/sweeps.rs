//! Oracle-sweep glue replacing the inline-python in the byte-court *_sweep.sh scripts. These parse cobc's
//! marker-delimited output and append the extracted bytes; the byte COMPARISON stays in the Rust *_rows
//! examples. No court semantics change -- this is faithful binary parsing.
use std::path::Path;

fn hexd(b: &[u8]) -> String { let mut s = String::new(); for x in b { s.push_str(&format!("{x:02x}")); } s }
fn read(p: &str) -> String { std::fs::read_to_string(p).unwrap_or_default() }
fn readb(p: &str) -> Vec<u8> { std::fs::read(p).unwrap_or_default() }

fn find_from(buf: &[u8], marker: &[u8], cur: usize) -> Option<usize> {
    if marker.is_empty() || cur > buf.len() { return None; }
    buf[cur..].windows(marker.len()).position(|w| w == marker).map(|p| cur + p)
}

/// find `label[`, extract N bytes (N fixed or = cases column `col:K`) as hex, append to the full row.
pub fn extract(cases: &str, out: &str, bytes_spec: &str) -> i32 {
    let buf = readb(out);
    let mut cur = 0usize;
    for line in read(cases).lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        let label = cols.first().copied().unwrap_or("");
        let n: usize = if let Some(k) = bytes_spec.strip_prefix("col:") {
            cols.get(k.parse::<usize>().unwrap_or(0)).and_then(|s| s.parse().ok()).unwrap_or(0)
        } else { bytes_spec.parse().unwrap_or(0) };
        let marker = format!("{label}[").into_bytes();
        let hex = match find_from(&buf, &marker, cur) {
            Some(m) => { let s = m + marker.len(); cur = s + n; hexd(&buf[s..(s + n).min(buf.len())]) }
            None => String::new(),
        };
        println!("{line}\t{hex}");
    }
    0
}

/// remainder: dual markers `labelQ[` (csz=col10) and `labelR[` (dsz=col13) -> full row + qhex + rhex.
pub fn remainder(cases: &str, out: &str) -> i32 {
    let buf = readb(out);
    let mut cur = 0usize;
    for line in read(cases).lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        let label = cols.first().copied().unwrap_or("");
        let csz: usize = cols.get(10).and_then(|s| s.parse().ok()).unwrap_or(0);
        let dsz: usize = cols.get(13).and_then(|s| s.parse().ok()).unwrap_or(0);
        let mq = format!("{label}Q[").into_bytes();
        let qhex = match find_from(&buf, &mq, cur) { Some(m) => { let s = m + mq.len(); cur = s + csz; hexd(&buf[s..(s + csz).min(buf.len())]) } None => String::new() };
        let mr = format!("{label}R[").into_bytes();
        let rhex = match find_from(&buf, &mr, cur) { Some(m) => { let s = m + mr.len(); cur = s + dsz; hexd(&buf[s..(s + dsz).min(buf.len())]) } None => String::new() };
        println!("{line}\t{qhex}\t{rhex}");
    }
    0
}

/// ordchar: per-row op. ORD -> marker `label=`, 4 bytes latin1; else -> marker `label[`, 1 byte hex.
pub fn ordchar(cases: &str, out: &str) -> i32 {
    let buf = readb(out);
    let mut cur = 0usize;
    for line in read(cases).lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        let (label, op, arg) = (cols.first().copied().unwrap_or(""), cols.get(1).copied().unwrap_or(""), cols.get(2).copied().unwrap_or(""));
        let val = if op == "ORD" {
            let m = format!("{label}=").into_bytes();
            match find_from(&buf, &m, cur) { Some(p) => { let s = p + m.len(); cur = s + 4; buf[s..(s + 4).min(buf.len())].iter().map(|&b| b as char).collect::<String>() } None => String::new() }
        } else {
            let m = format!("{label}[").into_bytes();
            match find_from(&buf, &m, cur) { Some(p) => { let s = p + m.len(); cur = s + 1; hexd(&buf[s..(s + 1).min(buf.len())]) } None => String::new() }
        };
        println!("{label}\t{op}\t{arg}\t{val}");
    }
    0
}

pub fn _unused(_: &Path) {}
