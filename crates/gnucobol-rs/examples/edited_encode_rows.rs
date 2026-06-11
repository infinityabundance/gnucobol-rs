//! Mirror for the numeric->edited ENCODE sweep (`GNURUST.16c`). Reads `label<TAB>pic<TAB>value<TAB>hex`
//! (hex = the oracle's edited field bytes), calls `gnucobol_rs::encode_edited(pic, value)`, and checks
//! the produced bytes equal the oracle bytes EXACTLY (byte-for-byte). Prints `PASS=n FAIL=n`.
use gnucobol_rs::{encode_edited, Decimal};
use std::io::BufRead;

fn to_decimal(value: &str) -> Decimal {
    let negative = value.starts_with('-');
    let t = value.trim_start_matches(['-', '+']);
    let (i, f) = t.split_once('.').unwrap_or((t, ""));
    let digits: Vec<u8> = i
        .bytes()
        .chain(f.bytes())
        .filter(u8::is_ascii_digit)
        .map(|b| b - b'0')
        .collect();
    Decimal { negative, digits, scale: f.len() as i16 }
}

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 {
            continue;
        }
        let (label, pic, value, hex) = (f[0], f[1], f[2], f[3]);
        let want: Vec<u8> = (0..hex.len() / 2)
            .map(|k| u8::from_str_radix(&hex[k * 2..k * 2 + 2], 16).unwrap_or(0))
            .collect();
        match encode_edited(pic, &to_decimal(value)) {
            Ok(got) if got == want => pass += 1,
            Ok(got) => {
                println!(
                    "{label} FAIL pic={pic} value={value} got='{}' want='{}'",
                    String::from_utf8_lossy(&got),
                    String::from_utf8_lossy(&want)
                );
                fail += 1;
            }
            Err(e) => {
                println!("{label} FAIL encode-error {e} (pic={pic} value={value})");
                fail += 1;
            }
        }
    }
    println!("PASS={pass} FAIL={fail}");
}
