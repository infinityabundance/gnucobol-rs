//! Rust mirror of the INSPECT sweep (`GNURUST.INSPECT.1`). Reads cases + the oracle's count/target bytes,
//! runs inspect_tallying/replacing/converting, and compares. PASS=n FAIL=n.
use gnucobol_rs::inspect::{inspect_converting, inspect_replacing, inspect_tallying, Region, ReplaceMode, TallyMode};
use std::io::BufRead;
fn region<'a>(spec: &'a str, buf: &'a mut Vec<u8>) -> Region<'a> {
    if let Some(x) = spec.strip_prefix("before:") { buf.extend_from_slice(x.as_bytes()); Region::Before(buf) }
    else if let Some(x) = spec.strip_prefix("after:") { buf.extend_from_slice(x.as_bytes()); Region::After(buf) }
    else { Region::All }
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 8 { continue; }
        let (label, op, target, mode, a1, a2, rspec, oracle_hex) = (f[0], f[1], f[2].as_bytes(), f[3], f[4], f[5], f[6], f[7]);
        let oracle: Vec<u8> = (0..oracle_hex.len()/2).map(|k| u8::from_str_radix(&oracle_hex[k*2..k*2+2],16).unwrap_or(0)).collect();
        let mut rbuf = Vec::new();
        let r = region(rspec, &mut rbuf);
        let mine: Vec<u8> = match op {
            "TALLY" => {
                let tm = match mode { "leading" => TallyMode::Leading(a1.as_bytes()), "chars" => TallyMode::Characters, _ => TallyMode::All(a1.as_bytes()) };
                format!("{:03}", inspect_tallying(target, tm, r)).into_bytes()
            }
            "REPL" => {
                let rm = match mode { "leading" => ReplaceMode::Leading(a1.as_bytes(), a2.as_bytes()), "first" => ReplaceMode::First(a1.as_bytes(), a2.as_bytes()), _ => ReplaceMode::All(a1.as_bytes(), a2.as_bytes()) };
                inspect_replacing(target, rm, r)
            }
            _ => inspect_converting(target, a1.as_bytes(), a2.as_bytes(), r),
        };
        if mine == oracle { pass += 1; } else {
            println!("{label} FAIL op={op} mine={} oracle={}", String::from_utf8_lossy(&mine), oracle_hex);
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
