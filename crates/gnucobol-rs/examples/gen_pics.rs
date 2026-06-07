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
}
