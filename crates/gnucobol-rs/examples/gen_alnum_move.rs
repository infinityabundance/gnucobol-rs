//! Generator for the alphanumeric MOVE differential (move.c cob_move_alphanum_to_alphanum /
//! display_to_alphanum / alphanum_to_display): ALPHANUMERIC<->ALPHANUMERIC (truncation/padding +
//! JUSTIFIED), DISPLAY->ALPHANUMERIC (P-scaling + JUSTIFIED), and ALPHANUMERIC->DISPLAY (free-form
//! parse). Identical bytes feed the libcob oracle (cob_move) and the Rust port.
//!
//! Row: `label s_type s_dig s_scale s_flags s_size s_hex d_type d_dig d_scale d_flags d_size`

const DISPLAY: u32 = 0x10;
const ALNUM: u32 = 0x21;
const HAVE_SIGN: u32 = 1;
const JUST: u32 = 0x10;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// zoned DISPLAY image of a signed integer value at the given digit count (trailing overpunch sign)
fn disp(digits: usize, val: i64, signed: bool) -> Vec<u8> {
    let neg = val < 0;
    let mut s: Vec<u8> = format!("{:0width$}", val.unsigned_abs(), width = digits).into_bytes();
    s.truncate(digits);
    while s.len() < digits {
        s.insert(0, b'0');
    }
    if signed && neg {
        let l = s.len() - 1;
        s[l] |= 0x40; // ASCII trailing overpunch for negative
    }
    s
}

fn main() {
    let mut id = 0u64;
    let row = |s_type: u32, s_dig: u32, s_sc: i32, s_fl: u32, s: &[u8], d_type: u32, d_dig: u32, d_sc: i32, d_fl: u32, d_size: u32, id: &mut u64| {
        println!(
            "a{} {s_type} {s_dig} {s_sc} {s_fl} {} {} {d_type} {d_dig} {d_sc} {d_fl} {d_size}",
            *id,
            s.len(),
            hex(s),
        );
        *id += 1;
    };

    // 1) ALPHANUMERIC -> ALPHANUMERIC : truncation / padding, left + JUSTIFIED RIGHT
    let texts: &[&[u8]] = &[b"ABCDEFGH", b"XY", b"hello world", b"   ", b"A", b"12345", b"  pad  "];
    for t in texts {
        for &dsize in &[1u32, 2, 4, 8, 11, 16] {
            for &just in &[0u32, JUST] {
                row(ALNUM, 0, 0, 0, t, ALNUM, 0, 0, just, dsize, &mut id);
            }
        }
    }

    // 2) DISPLAY -> ALPHANUMERIC : numeric to text, P-scaling, JUSTIFIED, padding
    for &(dig, sc, signed) in &[(5u32, 0i32, true), (5, 2, true), (4, 0, false), (6, 3, true), (3, -2, true), (3, 5, true)] {
        for &v in &[0i64, 1, 42, -42, 12345, -12345] {
            if !signed && v < 0 {
                continue;
            }
            if (v.unsigned_abs()) >= 10u64.pow(dig) {
                continue;
            }
            let img = disp(dig as usize, v, signed);
            let sfl = if signed { HAVE_SIGN } else { 0 };
            for &dsize in &[3u32, 5, 8, 12] {
                for &just in &[0u32, JUST] {
                    row(DISPLAY, dig, sc, sfl, &img, ALNUM, 0, 0, just, dsize, &mut id);
                }
            }
        }
    }

    // 3) ALPHANUMERIC -> DISPLAY : free-form parse (sign, spaces, separators, decimal point, errors)
    let parses: &[&[u8]] = &[
        b"123", b"  45 ", b"+678", b"-99", b"1,234", b"12.34", b"  -7.5 ", b"0", b"abc", b"1.2.3", b"   ",
        b"999999", b"-0",
    ];
    for t in parses {
        for &(dig, sc, signed) in &[(5u32, 0i32, true), (6, 2, true), (4, 0, false), (7, 3, true)] {
            let sfl = if signed { HAVE_SIGN } else { 0 };
            row(ALNUM, 0, 0, 0, t, DISPLAY, dig, sc, sfl, dig, &mut id);
        }
    }
}
