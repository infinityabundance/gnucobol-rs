//! Deterministic generator for the integer-power sweep (GNURUST.INTPOW.1): (width, base, power)
//! rows fed to the libcob oracle (cob_s32_pow/cob_s64_pow) and the Rust port. Row: `label width base power`.
fn main() {
    let bases: &[i64] = &[0, 1, -1, 2, -2, 3, -3, 7, 10, -10, 99, 1000, -1000, 46341, 2147483647, -2147483648, 3037000500];
    let powers: &[i64] = &[0, 1, 2, 3, 4, 5, 9, 18, 19, 31, 32, 40, 62, 63, 64, -1, -2, -5];
    let mut id = 0u64;
    for &w in &[32u32, 64] {
        for &b in bases {
            if w == 32 && (b > i32::MAX as i64 || b < i32::MIN as i64) { continue; }
            for &p in powers {
                if b == 0 && p < 0 { continue; } // oracle SIGFPE: generator never emits it
                println!("p{id} {w} {b} {p}");
                id += 1;
            }
        }
    }
}
