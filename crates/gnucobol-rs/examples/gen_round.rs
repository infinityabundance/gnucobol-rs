//! Deterministic generator for the ROUNDED-MODE sweep (`GNURUST.ROUND.1`). Each row is an
//! `ADD 0 + value` whose store narrows `value` to a smaller scale under one `ROUNDED MODE IS`
//! setting, so the libcob oracle (`cob_add` with the `COB_STORE_<mode>` opt) and the Rust port
//! (`cob_arith`) produce identical receiver bytes. Reuses the arith harness + `arith_rows` mirror.
//!
//! Row: `label op a_type a_dig a_scale a_flags a_size a_hex b_type b_dig b_scale b_flags b_size b_hex opt`
//! op = 1 (ADD). opt = COB_STORE_ROUND(1) | mode-bit; opt 0 = no-round truncation.

const DISPLAY: u32 = 16;
const PACKED: u32 = 18;
const HAVE_SIGN: u32 = 1;

/// The eight `ROUNDED MODE IS` settings as their libcob opt words (numeric.c:949-960).
const OPTS: &[u32] = &[
    0,    // (no ROUNDED) TRUNCATION
    33,   // ROUND | NEAR_AWAY_FROM_ZERO  (default ROUNDED)
    17,   // ROUND | AWAY_FROM_ZERO
    65,   // ROUND | NEAR_EVEN            (banker's)
    129,  // ROUND | NEAR_TOWARD_ZERO
    513,  // ROUND | TOWARD_GREATER       (ceiling)
    1025, // ROUND | TOWARD_LESSER        (floor)
];

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn enc_display(digits: &[u8], neg: bool) -> Vec<u8> {
    let mut o: Vec<u8> = digits.iter().map(|d| b'0' + d).collect();
    if neg {
        if let Some(l) = o.last_mut() {
            *l |= 0x40; // ASCII overpunch negative (0x3x -> 0x7x)
        }
    }
    o
}

fn enc_packed(digits: &[u8], neg: bool) -> (Vec<u8>, usize) {
    let mut nibs: Vec<u8> = digits.to_vec();
    nibs.push(if neg { 0x0D } else { 0x0C });
    if nibs.len() % 2 == 1 {
        nibs.insert(0, 0);
    }
    (
        nibs.chunks(2).map(|c| (c[0] << 4) | c[1]).collect(),
        digits.len() / 2 + 1,
    )
}

/// `m` rendered as `nd` zero-padded decimal digits, DISPLAY-encoded.
fn render(m: u64, nd: usize, neg: bool) -> Vec<u8> {
    let mut d = vec![0u8; nd];
    let mut v = m;
    for slot in d.iter_mut().rev() {
        *slot = (v % 10) as u8;
        v /= 10;
    }
    enc_display(&d, neg)
}

fn main() {
    let mut id = 0u64;
    // Receiver `a` = 0, 9 digits, scale = tgt. Value `b` = 7-digit mantissa at scale s = tgt+drop.
    // ROUND.1 covers the cob_decimal store path (cob_decimal_do_round): COMPUTE / MOVE / a DISPLAY
    // receiver. ROUND.2 adds the packed receiver: ADD/SUBTRACT *into a packed field* take the separate
    // cob_add_bcd nibble-rounding (numeric.c:2826+), which matches cob_decimal except NEAREST-EVEN
    // (resolved away-from-zero). Both receivers are swept here.
    for &a_ut in &[DISPLAY, PACKED] {
        let (a_bytes, a_size) = if a_ut == DISPLAY {
            (enc_display(&vec![0u8; 9], false), 9usize)
        } else {
            enc_packed(&vec![0u8; 9], false)
        };
        for tgt in 0..=2i64 {
            for drop in 1..=2i64 {
                let s = tgt + drop;
                let dpow = 10i64.pow(drop as u32);
                let half = 5 * 10i64.pow((drop - 1) as u32);
                // Dropped-part cases: below half, exactly half, above half, and exact (zero).
                let dropped: Vec<i64> = vec![half - 5, half, (half + 5).min(dpow - 1), 0];
                // Kept units digit 0-9: exercises NEAR_EVEN at the tie for every even/odd kept digit
                // (the packed cob_add_bcd path diverges here -- GNURUST.ROUND.2).
                for kept in 0..=9i64 {
                    for d in &dropped {
                        let m = (kept * dpow + d) as u64; // mantissa < 1000
                        for neg in [false, true] {
                            let b_bytes = render(m, 7, neg);
                            for &opt in OPTS {
                                println!(
                                    "r{id} 1 {a_ut} 9 {tgt} {HAVE_SIGN} {a_size} {} {DISPLAY} 7 {s} {HAVE_SIGN} 7 {} {opt}",
                                    hex(&a_bytes),
                                    hex(&b_bytes),
                                );
                                id += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}
