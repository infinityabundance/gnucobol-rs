//! Deterministic generator of binary MOVE sweep rows (`GNURUST.14`). Emits DISPLAY↔binary moves
//! (COMP/BINARY, COMP-5, COMP-X) in the shared `decimal_harness`/`rows` row format, so identical
//! bytes feed the libcob oracle and the Rust `cob_move`. Test infrastructure, not API.
//!
//! Row: `label s_type s_digits s_scale s_flags s_size s_hex d_type d_digits d_scale d_flags d_size`

const DISPLAY: u32 = 16; // 0x10
const BINARY: u32 = 17; // 0x11
const HAVE_SIGN: u32 = 1;
const SWAP: u32 = 0x20;
const REAL_BINARY: u32 = 0x40;
const TRUNC: u32 = 0x800;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn binary_size(d: u32) -> usize {
    match d {
        1..=2 => 1,
        3..=4 => 2,
        5..=9 => 4,
        10..=18 => 8,
        _ => 16,
    }
}
fn comp_x_size(d: u32) -> usize {
    let mut ten: u128 = 1;
    for _ in 0..d {
        ten = ten.saturating_mul(10);
    }
    let (mut k, mut cap) = (0usize, 1u128);
    while cap < ten {
        cap = cap.saturating_mul(256);
        k += 1;
    }
    k.max(1)
}

/// usage -> (flags-without-sign, size)
fn usage_meta(usage: &str, digits: u32) -> (u32, usize) {
    match usage {
        "COMP" => (SWAP | TRUNC, binary_size(digits)),
        "COMP-5" => (REAL_BINARY, binary_size(digits)),
        _ => (SWAP, comp_x_size(digits)), // COMP-X
    }
}

/// Encode `value` (signed integer at the field scale) into binary bytes for the sweep's *source*
/// side. Mirrors the oracle's storage exactly so the harness reads valid bytes.
fn enc_binary(value: i128, usage: &str, signed: bool, digits: u32, size: usize) -> Vec<u8> {
    let mut v = value;
    if usage == "COMP" {
        let mut m: i128 = 1;
        for _ in 0..digits {
            m = m.saturating_mul(10);
        }
        v %= m;
    }
    let bits = size * 8;
    let mask: u128 = if bits >= 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    let uv = (v as u128) & mask;
    let swap = usage != "COMP-5";
    let _ = signed;
    let mut out = vec![0u8; size];
    for (i, b) in out.iter_mut().enumerate() {
        *b = (uv >> (8 * if swap { size - 1 - i } else { i })) as u8;
    }
    out
}

fn enc_display(value: i128, digits: u32, scale: u32, signed: bool) -> Vec<u8> {
    let neg = value < 0;
    let mut abs = value.unsigned_abs();
    let mut m: u128 = 1;
    for _ in 0..digits {
        m = m.saturating_mul(10);
    }
    abs %= m;
    let mut d = vec![0u8; digits as usize];
    for slot in d.iter_mut().rev() {
        *slot = (abs % 10) as u8;
        abs /= 10;
    }
    let _ = scale;
    let mut o: Vec<u8> = d.iter().map(|x| b'0' + x).collect();
    if signed && neg {
        if let Some(l) = o.last_mut() {
            *l |= 0x40;
        }
    }
    o
}

fn main() {
    let mut id = 0u64;
    let values: [i128; 8] = [0, 1, -1, 42, -42, 1234, -1234, 9999];
    for usage in ["COMP", "COMP-5", "COMP-X"] {
        for digits in [2u32, 4, 6, 9] {
            for scale in [0u32, 2] {
                if scale >= digits {
                    continue;
                }
                for signed in [false, true] {
                    let (bflags, bsize) = usage_meta(usage, digits);
                    let bflags = bflags | if signed { HAVE_SIGN } else { 0 };
                    let dsign = if signed { HAVE_SIGN } else { 0 };
                    let dsize = digits as usize;
                    for &v in values.iter() {
                        if !signed && v < 0 {
                            continue;
                        }
                        // DISPLAY -> binary
                        let dhex = hex(&enc_display(v, digits, scale, signed));
                        println!(
                            "d{id} {DISPLAY} {digits} {scale} {dsign} {dsize} {dhex} {BINARY} {digits} {scale} {bflags} {bsize}"
                        );
                        id += 1;
                        // binary -> DISPLAY
                        let bhex = hex(&enc_binary(v, usage, signed, digits, bsize));
                        println!(
                            "b{id} {BINARY} {digits} {scale} {bflags} {bsize} {bhex} {DISPLAY} {digits} {scale} {dsign} {dsize}"
                        );
                        id += 1;
                    }
                }
            }
        }
    }
}
