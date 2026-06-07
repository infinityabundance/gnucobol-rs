//! Deterministic generator of arithmetic sweep rows (`GNURUST.7`). Emits ADD/SUBTRACT/MULTIPLY
//! over DISPLAY and COMP-3 operands, signed/unsigned, varied digits/scales, truncation and ROUNDED,
//! with sign combinations, cross-scale, rounding-tie digits, and overflow. Operand bytes are
//! encoded here so identical bytes feed the libcob oracle and the Rust mirror.
//!
//! Row: `label op a_type a_dig a_scale a_flags a_size a_hex b_type b_dig b_scale b_flags b_size b_hex opt`
//! op: 1=ADD 2=SUB 3=MUL ; opt: 0=truncate 1=ROUNDED(near-away). Test infrastructure, not API.

const DISPLAY: u32 = 16;
const PACKED: u32 = 18;
const HAVE_SIGN: u32 = 1;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn enc(ut: u32, digits: &[u8], signed: bool, neg: bool) -> (Vec<u8>, usize) {
    if ut == DISPLAY {
        let mut o: Vec<u8> = digits.iter().map(|d| b'0' + d).collect();
        if signed && neg {
            if let Some(l) = o.last_mut() {
                *l |= 0x40;
            }
        }
        (o, digits.len())
    } else {
        let mut nibs: Vec<u8> = digits.to_vec();
        nibs.push(if !signed {
            0x0F
        } else if neg {
            0x0D
        } else {
            0x0C
        });
        if nibs.len() % 2 == 1 {
            nibs.insert(0, 0);
        }
        (
            nibs.chunks(2).map(|c| (c[0] << 4) | c[1]).collect(),
            digits.len() / 2 + 1,
        )
    }
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

fn digits_of(rng: &mut Lcg, nd: usize, kind: u8) -> Vec<u8> {
    match kind {
        0 => vec![0u8; nd],
        1 => {
            let mut v: Vec<u8> = (0..nd).map(|_| rng.below(10) as u8).collect();
            if let Some(l) = v.last_mut() {
                *l = 5; // trailing 5 to drive rounding ties
            }
            v
        }
        2 => vec![9u8; nd],
        _ => (0..nd).map(|_| rng.below(10) as u8).collect(),
    }
}

fn main() {
    let mut rng = Lcg(0xA5A5_1234_5678_9ABC);
    let mut id = 0u64;
    // Receiver (a) usage × operand (b) usage covers GNURUST.13's mixed cases — e.g. a=PACKED,
    // b=DISPLAY is "ADD DISPLAY TO COMP-3" (the cob_add_bcd path). opt 0 = truncate, 33 =
    // COB_STORE_ROUND|NEAR_AWAY_FROM_ZERO (the real `ADD ROUNDED` opt).
    for a_ut in [DISPLAY, PACKED] {
        for b_ut in [DISPLAY, PACKED] {
            for op in [1u32, 2, 3] {
                for nd in [1usize, 2, 3, 4, 6, 9] {
                    for sc in 0..=2i32 {
                        if sc as usize >= nd {
                            continue;
                        }
                        for signed in [false, true] {
                            for (sa, sb) in
                                [(false, false), (true, false), (false, true), (true, true)]
                            {
                                if !signed && (sa || sb) {
                                    continue;
                                }
                                for opt in [0u32, 33] {
                                    for kind in 1u8..=3 {
                                        let ad = digits_of(&mut rng, nd, kind);
                                        let bsc = if rng.below(2) == 0 {
                                            sc
                                        } else {
                                            (sc + 1).min(nd as i32 - 1).max(0)
                                        };
                                        let bd = digits_of(&mut rng, nd, 3);
                                        let (ab, asz) = enc(a_ut, &ad, signed, sa);
                                        let (bb, bsz) = enc(b_ut, &bd, signed, sb);
                                        let asf = if signed { HAVE_SIGN } else { 0 };
                                        println!(
                                        "x{id} {op} {a_ut} {nd} {sc} {asf} {asz} {} {b_ut} {nd} {bsc} {asf} {bsz} {} {opt}",
                                        hex(&ab),
                                        hex(&bb),
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
    }
}
