//! Rust mirror of the INITIALIZE sweep (`GNURUST.INITIALIZE.1`). Reads gen_initialize cases + the oracle's
//! post-INITIALIZE bytes, lays out the record, builds InitFields, runs initialize_record from a sentinel
//! prefill, and compares. PASS=n FAIL=n.
use gnucobol_rs::initialize::{initialize_record, InitCategory, InitField};
use gnucobol_rs::layout::{lay_out, Item};
use gnucobol_rs::Usage;
use std::io::BufRead;

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|k| u8::from_str_radix(&s[k * 2..k * 2 + 2], 16).unwrap_or(0)).collect()
}
// parse one "05 NAME PIC X(4) ... [REDEFINES T] [COMP-3|COMP|...]" line into an Item (+ category hint).
fn parse_line(line: &str) -> (Item, Option<InitCategory>) {
    let toks: Vec<String> = line.trim().trim_end_matches('.').split_whitespace().map(|s| s.to_string()).collect();
    let level: u16 = toks[0].parse().unwrap_or(1);
    let name = toks.get(1).cloned().unwrap_or_default();
    let mut pic: Option<String> = None;
    let mut usage = Usage::Display;
    let mut redefines: Option<String> = None;
    let mut i = 2;
    while i < toks.len() {
        match toks[i].as_str() {
            "PIC" | "PICTURE" => { pic = toks.get(i + 1).cloned(); i += 2; continue; }
            "REDEFINES" => { redefines = toks.get(i + 1).cloned(); i += 2; continue; }
            "COMP-3" | "PACKED-DECIMAL" | "COMPUTATIONAL-3" => usage = Usage::Comp3,
            "COMP" | "COMPUTATIONAL" | "BINARY" => usage = Usage::Comp,
            "COMP-5" | "COMPUTATIONAL-5" => usage = Usage::Comp5,
            "COMP-X" | "COMPUTATIONAL-X" => usage = Usage::CompX,
            "VALUE" => break,
            _ => {}
        }
        i += 1;
    }
    let cat = pic.as_ref().map(|p| {
        let numeric = p.starts_with('9') || p.starts_with('S') || p.starts_with('s');
        match usage {
            Usage::Comp3 => InitCategory::Packed,
            Usage::Comp | Usage::Comp5 | Usage::CompX => InitCategory::Binary,
            _ if numeric => InitCategory::NumericDisplay,
            _ => InitCategory::Alphanumeric,
        }
    });
    (Item { level, name, pic: pic.map(|p| (p, usage, false, false)), occurs: None, redefines, odo: None }, cat)
}

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 { continue; }
        let (label, reclen, lines, posthex) = (f[0], f[1].parse::<usize>().unwrap_or(0), f[2], f[3]);
        // 01 record wraps the 05/10 lines
        let mut items = vec![Item { level: 1, name: "REC".into(), pic: None, occurs: None, redefines: None, odo: None }];
        let mut cats: Vec<(String, Option<InitCategory>, bool, bool, bool)> = Vec::new(); // name, cat, signed, is_filler, is_redefiner
        for l in lines.split('|') {
            let (it, cat) = parse_line(l);
            let (filler, redef) = (it.name == "FILLER", it.redefines.is_some());
            let signed = it.pic.as_ref().map(|(p,_,_,_)| p.starts_with('S')||p.starts_with('s')).unwrap_or(false);
            cats.push((it.name.clone(), cat, signed, filler, redef));
            items.push(it);
        }
        let laid = match lay_out(&items) { Ok(v) => v, Err(e) => { println!("{label} FAIL layout {e:?}"); fail += 1; continue; } };
        // map name -> (offset,size) from laid
        let mut fields: Vec<InitField> = Vec::new();
        for (name, cat, signed, filler, redef) in &cats {
            if let Some(c) = cat {
                if let Some(l) = laid.iter().find(|l| &l.name == name) {
                    fields.push(InitField { offset: l.offset, size: l.size, category: *c, signed: *signed, is_filler: *filler, is_redefiner: *redef });
                }
            }
        }
        let prefill = vec![0x7eu8; reclen]; // MOVE ALL "~"
        let mine = initialize_record(&fields, &prefill);
        let oracle = unhex(posthex);
        if mine == oracle { pass += 1; } else {
            println!("{label} FAIL mine={} oracle={}", mine.iter().map(|b| format!("{b:02x}")).collect::<String>(), posthex);
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
