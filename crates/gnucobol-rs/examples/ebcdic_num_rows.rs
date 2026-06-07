//! Mirror for the cp500 EBCDIC zoned-numeric sweep (`GNURUST.17`). Reads
//! `label<TAB>pic<TAB>value<TAB>raw_hex<TAB>out_pic<TAB>oracle_out_hex` and checks that
//! `Decimal::from_ebcdic_zoned(raw)` recovers the expected value AND equals GnuCOBOL's own decode
//! (the `-fsign=EBCDIC` program's edited output, decoded via the sealed `decode_edited`). PASS=n FAIL=n.

use gnucobol_rs::{
    decode_edited, Decimal, FieldAttr, COB_FLAG_HAVE_SIGN, COB_TYPE_NUMERIC_DISPLAY,
};
use std::io::BufRead;

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|k| u8::from_str_radix(&s[k * 2..k * 2 + 2], 16).unwrap_or(0))
        .collect()
}

/// (digit count, scale, signed) from a PIC like `S9(5)V9(2)`.
fn pic_meta(pic: &str) -> (u16, i16, bool) {
    let signed = pic.starts_with('S');
    let chars: Vec<char> = pic.chars().collect();
    let (mut total, mut frac, mut after_v, mut i) = (0u16, 0i16, false, 0usize);
    while i < chars.len() {
        match chars[i] {
            'V' => after_v = true,
            '9' => {
                let mut count = 1u16;
                if i + 1 < chars.len() && chars[i + 1] == '(' {
                    let mut num = String::new();
                    i += 2;
                    while i < chars.len() && chars[i] != ')' {
                        num.push(chars[i]);
                        i += 1;
                    }
                    count = num.parse().unwrap_or(1);
                }
                total += count;
                if after_v {
                    frac += count as i16;
                }
            }
            _ => {}
        }
        i += 1;
    }
    (total, frac, signed)
}

/// Canonical signed-int-at-scale of a Decimal.
fn canon(d: &Decimal) -> i128 {
    let mut s: i128 = 0;
    for &x in &d.digits {
        s = s * 10 + x as i128;
    }
    if d.negative {
        -s
    } else {
        s
    }
}

fn parse_expected(value: &str, scale: i16) -> i128 {
    let neg = value.starts_with('-');
    let t = value.trim_start_matches(['-', '+']);
    let (i, f) = t.split_once('.').unwrap_or((t, ""));
    let mut v: i128 = 0;
    for b in i.bytes().chain(f.bytes()) {
        v = v * 10 + (b - b'0') as i128;
    }
    let _ = scale;
    if neg {
        -v
    } else {
        v
    }
}

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 6 {
            continue;
        }
        let (label, pic, value, raw_hex, out_pic, oracle_hex) =
            (f[0], f[1], f[2], f[3], f[4], f[5]);
        let (digits, scale, signed) = pic_meta(pic);
        let attr = FieldAttr {
            field_type: COB_TYPE_NUMERIC_DISPLAY,
            digits,
            scale,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        };
        let mine = canon(&Decimal::from_ebcdic_zoned(&unhex(raw_hex), &attr));
        let expected = parse_expected(value, scale);
        // GnuCOBOL's own decode, via the edited output it produced (sealed decode_edited):
        let oracle = decode_edited(out_pic, &unhex(oracle_hex))
            .ok()
            .and_then(|d| d.numeric_value)
            .map(|n| canon(&n));
        if mine == expected && oracle == Some(expected) {
            pass += 1;
        } else {
            println!(
                "{label} FAIL pic={pic} mine={mine} expected={expected} oracle={oracle:?} out='{}'",
                String::from_utf8_lossy(&unhex(oracle_hex))
            );
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
