//! Deterministic generator of differential-sweep rows (`GNURUST.DECEDGE.0` / `GNURUST.SWEEPCORPUS.0`).
//! Emits harness rows (the shared `decimal_harness` / `rows` format) covering structured boundary
//! families plus a seeded pseudo-random corpus, so the same bytes feed the C oracle and the Rust
//! port. Optional arg: a u64 seed (default fixed). Test infrastructure, not API.
//!
//! Each row exercises one of the sealed pairs: DISPLAY->PACKED, PACKED->DISPLAY, DISPLAY->DISPLAY.

#![allow(clippy::too_many_arguments)] // a row generator; explicit fields are clearer than a struct

const DISPLAY: u32 = 16; // 0x10
const PACKED: u32 = 18; // 0x12
const FLAG_HAVE_SIGN: u32 = 1;

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// DISPLAY (zoned) bytes: one ASCII byte per digit; trailing overpunch for a negative signed value.
fn encode_display(digits: &[u8], signed: bool, negative: bool) -> Vec<u8> {
    let mut out: Vec<u8> = digits.iter().map(|d| b'0' + d).collect();
    if signed && negative {
        if let Some(last) = out.last_mut() {
            *last |= 0x40; // ASCII overpunch
        }
    }
    out
}

/// PACKED (COMP-3) bytes for the given stored digits: trailing sign nibble (C/D/F), leading pad
/// nibble when the total nibble count is odd.
fn encode_packed(digits: &[u8], signed: bool, negative: bool) -> Vec<u8> {
    let mut nibs: Vec<u8> = digits.to_vec();
    let sign_nib = if !signed {
        0x0F
    } else if negative {
        0x0D
    } else {
        0x0C
    };
    nibs.push(sign_nib);
    if nibs.len() % 2 == 1 {
        nibs.insert(0, 0);
    }
    nibs.chunks(2).map(|c| (c[0] << 4) | c[1]).collect()
}

fn packed_size(digits: usize) -> usize {
    digits / 2 + 1
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 16
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

fn emit_pair(
    label: &str,
    src_type: u32,
    src_digits: &[u8],
    src_scale: i32,
    src_signed: bool,
    src_neg: bool,
    dst_type: u32,
    dst_digits_count: usize,
    dst_scale: i32,
    dst_signed: bool,
) {
    let (src_bytes, src_size) = match src_type {
        DISPLAY => {
            let b = encode_display(src_digits, src_signed, src_neg);
            (b.clone(), b.len())
        }
        _ => {
            let b = encode_packed(src_digits, src_signed, src_neg);
            (b.clone(), b.len())
        }
    };
    let dst_size = match dst_type {
        DISPLAY => dst_digits_count,
        _ => packed_size(dst_digits_count),
    };
    let sf = if src_signed { FLAG_HAVE_SIGN } else { 0 };
    let df = if dst_signed { FLAG_HAVE_SIGN } else { 0 };
    println!(
        "{label} {src_type} {} {src_scale} {sf} {src_size} {} {dst_type} {} {dst_scale} {df} {dst_size}",
        src_digits.len(),
        hex(&src_bytes),
        dst_digits_count,
    );
}

fn digits_for(value_kind: u8, n: usize, rng: &mut Lcg) -> Vec<u8> {
    match value_kind {
        0 => vec![0u8; n], // zero
        1 => {
            let mut v = vec![0u8; n]; // one
            if let Some(l) = v.last_mut() {
                *l = 1;
            }
            v
        }
        2 => vec![9u8; n], // all nines
        3 => {
            let mut v = vec![0u8; n]; // leading zeros, low digit set
            if n >= 2 {
                v[n - 2] = 7;
            }
            v
        }
        _ => (0..n).map(|_| rng.below(10) as u8).collect(), // random
    }
}

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x9E3779B97F4A7C15);
    let mut rng = Lcg(seed);
    let mut id = 0u64;

    // --- Structured boundary families (DECEDGE) ---
    let digit_counts = [1usize, 2, 3, 4, 5, 7, 17, 18];
    for &nd in &digit_counts {
        let scales = [0i32, 1, (nd as i32 - 1).max(0), nd as i32];
        for &sc in &scales {
            for vk in 0u8..=3 {
                for &signed in &[false, true] {
                    for &neg in &[false, true] {
                        if !signed && neg {
                            continue;
                        }
                        let d = digits_for(vk, nd, &mut rng);
                        // DISPLAY -> PACKED (COMP-3 encode), same scale
                        emit_pair(
                            &format!("e{id}_d2p"),
                            DISPLAY,
                            &d,
                            sc,
                            signed,
                            neg,
                            PACKED,
                            nd,
                            sc,
                            signed,
                        );
                        id += 1;
                        // PACKED -> DISPLAY (COMP-3 decode), same scale
                        emit_pair(
                            &format!("e{id}_p2d"),
                            PACKED,
                            &d,
                            sc,
                            signed,
                            neg,
                            DISPLAY,
                            nd,
                            sc,
                            signed,
                        );
                        id += 1;
                        // DISPLAY -> DISPLAY, same scale
                        emit_pair(
                            &format!("e{id}_d2d"),
                            DISPLAY,
                            &d,
                            sc,
                            signed,
                            neg,
                            DISPLAY,
                            nd,
                            sc,
                            signed,
                        );
                        id += 1;
                    }
                }
            }
        }
    }

    // --- Seeded random corpus (SWEEPCORPUS), including cross-scale store_common_region exercise ---
    for _ in 0..4000 {
        let nd = 1 + rng.below(18) as usize;
        let sc = rng.below(nd as u64 + 1) as i32;
        let signed = rng.below(2) == 1;
        let neg = signed && rng.below(2) == 1;
        let d = digits_for(4, nd, &mut rng);
        // destination may have a different digit count / scale to drive alignment paths
        let nd2 = 1 + rng.below(18) as usize;
        let sc2 = rng.below(nd2 as u64 + 1) as i32;

        emit_pair(
            &format!("r{id}_d2p"),
            DISPLAY,
            &d,
            sc,
            signed,
            neg,
            PACKED,
            nd,
            sc,
            signed,
        );
        id += 1;
        emit_pair(
            &format!("r{id}_p2d"),
            PACKED,
            &d,
            sc,
            signed,
            neg,
            DISPLAY,
            nd,
            sc,
            signed,
        );
        id += 1;
        emit_pair(
            &format!("r{id}_d2d"),
            DISPLAY,
            &d,
            sc,
            signed,
            neg,
            DISPLAY,
            nd2,
            sc2,
            signed,
        );
        id += 1;
    }
}
