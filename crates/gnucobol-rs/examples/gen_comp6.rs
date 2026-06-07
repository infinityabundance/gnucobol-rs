//! Deterministic generator of DISPLAY<->COMP-6 MOVE sweep rows (`GNURUST.18`). COMP-6 is PACKED
//! (0x12) + NO_SIGN_NIBBLE (0x100), size ceil(digits/2); unsigned. Emits the shared decimal_harness/
//! rows format so identical bytes feed libcob and the Rust cob_move. Test infra.
//! Row: `label s_type s_digits s_scale s_flags s_size s_hex d_type d_digits d_scale d_flags d_size`

const DISPLAY: u32 = 16;
const PACKED: u32 = 18;
const NO_SIGN: u32 = 256; // COB_FLAG_NO_SIGN_NIBBLE

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn c6size(n: usize) -> usize {
    (n + 1) / 2
}

/// zoned display bytes of an unsigned value over `ndig` digits at `scale` (scale only affects meaning).
fn zoned(value: u64, ndig: usize) -> Vec<u8> {
    let mut d = vec![0u8; ndig];
    let mut v = value;
    for slot in d.iter_mut().rev() {
        *slot = (v % 10) as u8;
        v /= 10;
    }
    d.iter().map(|x| b'0' + x).collect()
}
/// COMP-6 packed bytes: two digits/byte, ceil(ndig/2) bytes, no sign nibble (leading 0 nibble if odd).
fn comp6(value: u64, ndig: usize) -> Vec<u8> {
    let mut d = vec![0u8; ndig];
    let mut v = value;
    for slot in d.iter_mut().rev() {
        *slot = (v % 10) as u8;
        v /= 10;
    }
    if ndig % 2 == 1 {
        d.insert(0, 0);
    } // pad to even with a leading 0 nibble
    d.chunks(2).map(|c| (c[0] << 4) | c[1]).collect()
}

fn main() {
    let mut id = 0u64;
    let values: [u64; 7] = [0, 1, 9, 42, 1234, 99999, 7000099];
    for (ndig, scale) in [
        (1usize, 0usize),
        (3, 0),
        (4, 0),
        (5, 0),
        (8, 0),
        (4, 2),
        (6, 2),
    ] {
        let dsize = ndig; // display size (unsigned)
        let csize = c6size(ndig);
        for &v in values.iter() {
            let vv = v % 10u64.pow(ndig as u32);
            // DISPLAY -> COMP-6
            println!("d{id} {DISPLAY} {ndig} {scale} 0 {dsize} {} {PACKED} {ndig} {scale} {NO_SIGN} {csize}", hex(&zoned(vv, ndig)));
            id += 1;
            // COMP-6 -> DISPLAY
            println!("c{id} {PACKED} {ndig} {scale} {NO_SIGN} {csize} {} {DISPLAY} {ndig} {scale} 0 {dsize}", hex(&comp6(vv, ndig)));
            id += 1;
        }
    }
}
