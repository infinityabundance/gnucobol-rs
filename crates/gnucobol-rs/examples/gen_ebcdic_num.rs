//! Generate cp500 EBCDIC zoned-decimal numeric-DISPLAY decode cases (`GNURUST.17`). Emits
//! `label<TAB>pic<TAB>value<TAB>raw_ebcdic_hex<TAB>ascii_overpunch<TAB>out_pic`.
//!
//! `raw_ebcdic_hex` = the mainframe zoned bytes (Rust decodes these via `from_ebcdic_zoned`).
//! `ascii_overpunch` = the same field after cp500 translation (the `-fsign=EBCDIC` oracle program
//! `MOVE`s it into a signed field, then into the `out_pic` edited field whose bytes the Rust mirror
//! decodes via the sealed `decode_edited`). Test infra.

/// (pic, scale, signed, out_pic-for-this-scale)
fn cases() -> Vec<(String, i32, bool)> {
    let mut v = Vec::new();
    for &(n, m) in &[(3usize, 0usize), (5, 0), (1, 0), (4, 2), (6, 2), (7, 2)] {
        let body = if m == 0 {
            format!("9({n})")
        } else {
            format!("9({})V9({m})", n - m)
        };
        v.push((format!("S{body}"), m as i32, true));
        v.push((body, m as i32, false)); // unsigned
    }
    v
}

fn out_pic(scale: i32) -> String {
    if scale == 0 {
        "-(15)9".to_string()
    } else {
        format!("-(12)9.9({scale})")
    }
}

fn main() {
    let values: [i64; 7] = [0, 1, 5, 42, 1234, 90909, 7000099];
    let mut id = 0u32;
    for (pic, scale, signed) in cases() {
        let digits_total: usize = pic.chars().scan(0usize, |st, _| Some(*st)).count().max(1); // placeholder; recompute below
        let _ = digits_total;
        // count 9s in the pic (the digit positions)
        let ndig = count_nines(&pic);
        for &mv in &values {
            for &neg in &[false, true] {
                if neg && (!signed || mv == 0) {
                    continue;
                }
                // scale the integer value: `mv` is the unscaled integer at `scale`.
                let scaled = mv % 10i64.pow(ndig as u32);
                let (raw, ascii) = encode_zoned(scaled, ndig, signed, neg);
                // human value string at scale
                let val = fmt_value(scaled, scale, neg);
                println!(
                    "e{id}\t{pic}\t{val}\t{}\t{}\t{}",
                    hex(&raw),
                    String::from_utf8_lossy(&ascii),
                    out_pic(scale)
                );
                id += 1;
            }
        }
    }
}

fn count_nines(pic: &str) -> usize {
    // expand `9(n)` runs
    let chars: Vec<char> = pic.chars().collect();
    let mut total = 0usize;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '9' {
            i += 1;
            if i < chars.len() && chars[i] == '(' {
                let mut num = String::new();
                i += 1;
                while i < chars.len() && chars[i] != ')' {
                    num.push(chars[i]);
                    i += 1;
                }
                i += 1;
                total += num.parse::<usize>().unwrap_or(1);
            } else {
                total += 1;
            }
        } else {
            i += 1;
        }
    }
    total
}

/// Build the raw EBCDIC zoned bytes + the cp500-translated ASCII-overpunch bytes for `value` (>=0
/// magnitude) over `ndig` digit positions, signed/negative per flags.
fn encode_zoned(value: i64, ndig: usize, signed: bool, neg: bool) -> (Vec<u8>, Vec<u8>) {
    let mut d = vec![0u8; ndig];
    let mut v = value.unsigned_abs();
    for slot in d.iter_mut().rev() {
        *slot = (v % 10) as u8;
        v /= 10;
    }
    let mut raw = Vec::with_capacity(ndig);
    let mut ascii = Vec::with_capacity(ndig);
    for (i, &digit) in d.iter().enumerate() {
        let last = i + 1 == ndig;
        if last {
            if signed && neg {
                raw.push(0xD0 | digit); // negative zone
                ascii.push(if digit == 0 { b'}' } else { b'J' + digit - 1 });
            } else if signed {
                raw.push(0xC0 | digit); // positive zone
                ascii.push(if digit == 0 { b'{' } else { b'A' + digit - 1 });
            } else {
                raw.push(0xF0 | digit); // unsigned
                ascii.push(b'0' + digit);
            }
        } else {
            raw.push(0xF0 | digit);
            ascii.push(b'0' + digit);
        }
    }
    (raw, ascii)
}

fn fmt_value(value: i64, scale: i32, neg: bool) -> String {
    let s = format!("{:0w$}", value.unsigned_abs(), w = scale as usize + 1);
    let sign = if neg && value != 0 { "-" } else { "" };
    if scale == 0 {
        format!("{sign}{s}")
    } else {
        let (i, f) = s.split_at(s.len() - scale as usize);
        format!("{sign}{i}.{f}")
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
