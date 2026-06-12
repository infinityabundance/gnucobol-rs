//! Deterministic generator for the bignum MULTIPLY sweep (`GNURUST.BIGNUM.1`). Emits `f1 := f1 * f2`
//! rows with large (15-20 digit) operands whose exact product exceeds i128, across signs, scales,
//! and every ROUNDED mode, so the libcob oracle (cob_mul, GMP product) and the Rust port (the
//! u256 bignum fallback) produce identical receiver bytes. Reuses the arith harness + `arith_rows`.
//!
//! Row: `label op a_type a_dig a_scale a_flags a_size a_hex b_type b_dig b_scale b_flags b_size b_hex opt`
//! op = 3 (MULTIPLY). Receiver is f1 (operand a).

const DISPLAY: u32 = 16;
const HAVE_SIGN: u32 = 1;
const OPTS: &[u32] = &[0, 33, 17, 65, 129, 513, 1025]; // truncate + the six ROUNDED byte-modes

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn enc_display(digits: &[u8], neg: bool) -> Vec<u8> {
    let mut o: Vec<u8> = digits.iter().map(|d| b'0' + d).collect();
    if neg {
        if let Some(l) = o.last_mut() {
            *l |= 0x40;
        }
    }
    o
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
}

/// `nd` digits in one of a few patterns (all-9s, trailing-5 tie driver, leading-1, pseudo-random).
fn pattern(rng: &mut Lcg, nd: usize, kind: u8) -> Vec<u8> {
    let mut v: Vec<u8> = match kind {
        0 => vec![9u8; nd],
        1 => {
            let mut d = vec![0u8; nd];
            d[0] = 1;
            *d.last_mut().unwrap() = 5;
            d
        }
        2 => {
            let mut d: Vec<u8> = (0..nd).map(|_| (rng.next() % 10) as u8).collect();
            *d.last_mut().unwrap() = 5; // trailing 5 drives rounding ties
            d
        }
        _ => (0..nd).map(|_| (rng.next() % 10) as u8).collect(),
    };
    if v[0] == 0 {
        v[0] = 1; // keep the magnitude wide (avoid a short value)
    }
    v
}

fn main() {
    let mut rng = Lcg(0xB16_0DEC_1234_5678);
    let mut id = 0u64;
    for nd in [16usize, 18, 19, 20] {
        for asc in 0..=2i32 {
            for bsc in 0..=2i32 {
                for (sa, sb) in [(false, false), (true, false), (false, true), (true, true)] {
                    for ka in 0u8..=3 {
                        for kb in 0u8..=3 {
                            let ad = pattern(&mut rng, nd, ka);
                            let bd = pattern(&mut rng, nd, kb);
                            let ab = enc_display(&ad, sa);
                            let bb = enc_display(&bd, sb);
                            for &opt in OPTS {
                                println!(
                                    "g{id} 3 {DISPLAY} {nd} {asc} {HAVE_SIGN} {nd} {} {DISPLAY} {nd} {bsc} {HAVE_SIGN} {nd} {} {opt}",
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
