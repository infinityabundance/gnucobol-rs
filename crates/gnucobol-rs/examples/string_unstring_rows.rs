//! Rust mirror for the STRING/UNSTRING sweep (`GNURUST.STRING.UNSTRING.1`). Reads `label=<hex>` result
//! records from the oracle, recomputes each via string_into/unstring, and compares the result-record bytes.
use gnucobol_rs::string_ops::{string_into, unstring, StringSource, UnstringResult};
use std::io::BufRead;
fn unhex(s: &str) -> Vec<u8> { (0..s.len()/2).map(|k| u8::from_str_radix(&s[k*2..k*2+2],16).unwrap_or(0)).collect() }
fn d2(n: usize) -> Vec<u8> { format!("{:02}", n).into_bytes() }
fn dbyte(d: &[u8]) -> Vec<u8> { if d.is_empty() { vec![b' '] } else { vec![d[0]] } }

// each case -> the result-record bytes (must match the oracle's RES dump exactly)
fn compute(label: &str) -> Vec<u8> {
    let mut o = Vec::new();
    match label {
        "s_size" => { let r = string_into(b"~~~~~~", &[StringSource::Size(b"AB"), StringSource::Size(b"CDE")], 1); o.extend(r.target); }
        "s_ptr"  => { let r = string_into(b"~~~~~~", &[StringSource::Size(b"XY")], 2); o.extend(r.target); o.extend(d2(r.pointer)); }
        "s_ovf"  => { let r = string_into(b"~~~~~~", &[StringSource::Size(b"ABCDEF"), StringSource::Size(b"GH")], 1); o.extend(r.target); o.push(if r.overflow {b'1'} else {b'0'}); }
        "s_delim"=> { let r = string_into(b"~~~~~~", &[StringSource::Delimited(b"HELLO,WORLD", b",")], 1); o.extend(r.target); }
        "u_base" => { let r: UnstringResult = unstring(b"AB,CDE,F  ", Some(b","), &[4,4,4], 1);
                      for f in &r.fields { o.extend(&f.data); o.extend(d2(f.count)); o.extend(dbyte(&f.delimiter)); } o.extend(d2(r.tally)); }
        "u_empty"=> { let r = unstring(b"A,,B      ", Some(b","), &[4,4,4], 1);
                      for f in &r.fields { o.extend(&f.data); o.extend(d2(f.count)); o.push(b' '); } o.extend(b"  "); }
        "u_ptr"  => { let r = unstring(b"ABCDEFGH  ", None, &[4], 3); o.extend(&r.fields[0].data); o.extend(d2(r.pointer)); }
        _ => {}
    }
    o
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((label, hex)) = line.split_once('=') else { continue };
        let oracle = unhex(hex.trim());
        let mine = compute(label.trim());
        if mine == oracle { pass += 1; } else {
            println!("{label} FAIL mine={} oracle={}", mine.iter().map(|b| format!("{b:02x}")).collect::<String>(), hex.trim());
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
