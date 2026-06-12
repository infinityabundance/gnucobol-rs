//! Generator for the bit-logical sweep (GNURUST.LOGICAL.1). Row: `label op v0 v1`.
fn main() {
    let vals: &[i64] = &[0, 1, 2, 3, 255, 256, 0xFF00, -1, -2, -255, 1023, 1000000,
                         0x0123456789ABCDEFu64 as i64, i64::MAX, i64::MIN, 64, 65, 63, 7, 100];
    let mut id = 0u64;
    for op in ["and", "or", "xor", "not", "shl", "shr"] {
        for &a in vals {
            for &b in vals {
                println!("l{id} {op} {a} {b}");
                id += 1;
            }
        }
    }
}
