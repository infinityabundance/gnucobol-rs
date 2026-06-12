//! Generator for the binary-float sweep (GNURUST.FLOAT.1), BOTH directions:
//! DISPLAY -> COMP-1/COMP-2 (encode) and COMP-1/COMP-2 -> DISPLAY (decode).
//! Row (decimal_harness): label s_type s_dig s_scale s_flags s_size s_hex d_type d_dig d_scale d_flags d_size
const DISPLAY: u32 = 16;
const FLOAT: u32 = 19;
const DOUBLE: u32 = 20;
const IS_FP: u32 = 512;
const HAVE_SIGN: u32 = 1;
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
struct Lcg(u64);
impl Lcg { fn next(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); self.0 >> 16 } }
fn main() {
    let mut rng = Lcg(0xF10A7);
    let mut id = 0u64;
    // ENCODE: DISPLAY value -> COMP-1/COMP-2
    for nd in [1usize, 3, 5, 9, 15, 18] {
        for scale in 0..=(nd.min(6) as i32) {
            for neg in [false, true] {
                for _ in 0..4 {
                    let digits: Vec<u8> = (0..nd).map(|_| (rng.next() % 10) as u8).collect();
                    let mut a: Vec<u8> = digits.iter().map(|d| b'0' + d).collect();
                    if neg { if let Some(l) = a.last_mut() { *l |= 0x40; } }
                    for (dt, dsz) in [(DOUBLE, 8u32), (FLOAT, 4)] {
                        println!("e{id} {DISPLAY} {nd} {scale} {HAVE_SIGN} {nd} {} {dt} 0 0 {IS_FP} {dsz}", hex(&a));
                        id += 1;
                    }
                }
            }
        }
    }
    // DECODE: COMP-1/COMP-2 value -> DISPLAY. Source f64 built from a decimal (round-nearest input).
    for nd in [3usize, 6, 9, 15] {
        for scale in 0..=(nd.min(6) as i32) {
            for neg in [false, true] {
                for _ in 0..5 {
                    // value = random integer / 10^randscale, as f64
                    let intdigits: Vec<u8> = (0..nd).map(|_| (rng.next() % 10) as u8).collect();
                    let istr: String = intdigits.iter().map(|d| (b'0'+d) as char).collect();
                    let vscale = (rng.next() % 7) as i32;
                    let s = format!("{}{}e-{}", if neg {"-"} else {""}, istr, vscale);
                    let v: f64 = s.parse().unwrap();
                    let f64b = v.to_le_bytes();
                    let f32b = (v as f32).to_le_bytes();
                    // COMP-2 -> DISPLAY(nd, scale) signed
                    println!("d{id} {DOUBLE} 0 0 {IS_FP} 8 {} {DISPLAY} {nd} {scale} {HAVE_SIGN} {nd}", hex(&f64b));
                    id += 1;
                    println!("d{id} {FLOAT} 0 0 {IS_FP} 4 {} {DISPLAY} {nd} {scale} {HAVE_SIGN} {nd}", hex(&f32b));
                    id += 1;
                }
            }
        }
    }
}
