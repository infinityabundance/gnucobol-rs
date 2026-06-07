//! Deterministic generator of PIC sweep cases for `GNURUST.3`. Emits the sealed subset only
//! (`9 X S V`, repeats, DISPLAY/COMP-3, SIGN clauses) — P and edited pictures are rejected by the
//! parser and proven in unit tests, so they are not part of the parity sweep.
//!
//! Output line: `label<TAB>pic<TAB>usage<TAB>sign`. Test infrastructure, not API.

fn main() {
    let int_parts = [0usize, 1, 2, 3, 5, 8, 9, 12];
    let frac_parts = [0usize, 1, 2, 4];
    let mut id = 0u64;
    let mut emit = |pic: &str, usage: &str, sign: &str| {
        println!("p{id}\t{pic}\t{usage}\t{sign}");
        id += 1;
    };

    for &ip in &int_parts {
        for &fp in &frac_parts {
            let total = ip + fp;
            if total == 0 || total > 18 {
                continue;
            }
            // Build the numeric picture body (without the leading S).
            let body = match (ip > 0, fp > 0) {
                (true, true) => format!("9({ip})V9({fp})"),
                (true, false) => format!("9({ip})"),
                (false, true) => format!("V9({fp})"),
                (false, false) => unreachable!(),
            };

            for usage in ["DISPLAY", "COMP-3"] {
                // unsigned
                emit(&body, usage, "");
                // signed overpunch / packed sign nibble
                emit(&format!("S{body}"), usage, "");
                // DISPLAY also: separate sign (leading / trailing)
                if usage == "DISPLAY" {
                    emit(&format!("S{body}"), usage, "SIGN LEADING SEPARATE");
                    emit(&format!("S{body}"), usage, "SIGN TRAILING SEPARATE");
                }
            }
        }
    }

    // Alphanumeric widths.
    for n in [1usize, 2, 5, 10, 20, 80] {
        println!("ax{id}\tX({n})\t\t");
        id += 1;
    }

    // P-scaling (`GNURUST.9`): trailing P (`9..P..`) and leading P (`P..9..`), DISPLAY and COMP-3,
    // signed and unsigned. Storage size = the stored 9s; attr digits/scale are the asymmetric rule.
    for nine in [1usize, 2, 3, 5] {
        for p in [1usize, 2, 3] {
            let nines = "9".repeat(nine);
            let ps = "P".repeat(p);
            let trailing = format!("{nines}{ps}");
            let leading = format!("{ps}{nines}");
            for body in [trailing, leading] {
                for usage in ["DISPLAY", "COMP-3"] {
                    println!("pp{id}\t{body}\t{usage}\t");
                    id += 1;
                    println!("pp{id}\tS{body}\t{usage}\t");
                    id += 1;
                }
            }
        }
    }

    // COMP-6 (`GNURUST.18`): unsigned packed, ceil(digits/2) bytes. Unsigned only (signed -> COMP-3).
    for nd in [1usize, 2, 3, 4, 5, 6, 8, 9, 18] {
        for sc in [0usize, 2] {
            if sc >= nd {
                continue;
            }
            let body = if sc == 0 {
                format!("9({nd})")
            } else {
                format!("9({})V9({sc})", nd - sc)
            };
            println!("c6{id}\t{body}\tCOMP-6\t");
            id += 1;
        }
    }

    // Binary families (`GNURUST.14`): COMP/BINARY, COMP-5, COMP-X over the 1-2-4-8 size boundaries,
    // signed and unsigned, integer and V-scaled. Proves type/digits/scale/flags/size vs cobc.
    for nd in [1usize, 2, 3, 4, 5, 6, 9, 10, 18] {
        for sc in [0usize, 2] {
            if sc >= nd {
                continue;
            }
            let body = if sc == 0 {
                format!("9({nd})")
            } else {
                format!("9({})V9({sc})", nd - sc)
            };
            for usage in ["COMP", "BINARY", "COMP-5", "COMP-X"] {
                println!("b{id}\t{body}\t{usage}\t");
                id += 1;
                println!("b{id}\tS{body}\t{usage}\t");
                id += 1;
            }
        }
    }
}
