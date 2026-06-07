//! Rust mirror of the layout oracle (`lab/oracle/layout_harness.sh`): reads one record's items
//! (the same `name<TAB>decl` lines), lays them out, and emits `name offset size` per item, so the
//! layout sweep can compare to the compiler's offsets. Test infrastructure, not API.

use gnucobol_rs::layout::{lay_out, Item};
use gnucobol_rs::Usage;
use std::io::{self, BufRead, Write};

/// Parse a COBOL data-item declaration like `05 C PIC S9(4)V99 COMP-3 OCCURS 3 TIMES` into an Item.
fn parse_item(decl: &str) -> Option<Item> {
    let toks: Vec<String> = decl
        .split_whitespace()
        .map(|s| s.to_ascii_uppercase())
        .collect();
    if toks.len() < 2 {
        return None;
    }
    let level: u16 = toks[0].parse().ok()?;
    let name = toks[1].clone();

    let mut pic: Option<String> = None;
    let mut usage = Usage::Display;
    let mut occurs: Option<u32> = None;
    let mut redefines: Option<String> = None;
    let mut sep = false;
    let mut lead = false;

    let mut i = 2;
    while i < toks.len() {
        match toks[i].as_str() {
            "PIC" | "PICTURE" => {
                if i + 1 < toks.len() {
                    pic = Some(toks[i + 1].clone());
                    i += 2;
                    continue;
                }
            }
            "USAGE" => {
                i += 1;
                continue; // the next token is the usage keyword, handled below
            }
            "COMP-3" | "PACKED-DECIMAL" | "COMPUTATIONAL-3" => usage = Usage::Comp3,
            "DISPLAY" => usage = Usage::Display,
            "OCCURS" => {
                if i + 1 < toks.len() {
                    occurs = toks[i + 1].parse().ok();
                    i += 2;
                    continue;
                }
            }
            "TIMES" => {}
            "REDEFINES" => {
                if i + 1 < toks.len() {
                    redefines = Some(toks[i + 1].clone());
                    i += 2;
                    continue;
                }
            }
            "SEPARATE" => sep = true,
            "LEADING" => lead = true,
            _ => {}
        }
        i += 1;
    }

    Some(Item {
        level,
        name,
        pic: pic.map(|p| (p, usage, sep, lead)),
        occurs,
        redefines,
    })
}

fn main() {
    let stdin = io::stdin();
    let mut items = Vec::new();
    for line in stdin.lock().lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, '\t');
        let _name = parts.next().unwrap_or("");
        let decl = parts.next().unwrap_or("");
        if let Some(it) = parse_item(decl) {
            items.push(it);
        }
    }
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match lay_out(&items) {
        Ok(laid) => {
            for l in laid {
                let _ = writeln!(out, "{} {} {}", l.name, l.offset, l.size);
            }
        }
        Err(e) => {
            let _ = writeln!(out, "LAYOUT_ERROR {e}");
        }
    }
}
