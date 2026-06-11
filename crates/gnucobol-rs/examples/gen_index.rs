//! Generate USAGE INDEX cases (`GNURUST.INDEX.1`): `label|start|op|k`. Each case `SET IXS TO start`
//! then optionally `SET IXS UP/DOWN BY k`; the dumped value is the 4-byte native-endian index word.
//! op ∈ to (no further op) / up / down.
fn main() {
    let mut id = 0u32;
    let mut emit = |start: i32, op: &str, k: i32| {
        println!("i{id}|{start}|{op}|{k}");
        id += 1;
    };
    // SET TO: occurrence numbers across byte boundaries (1,2,3,4-byte ranges).
    for s in [0i32, 1, 2, 3, 5, 9, 10, 16, 42, 99, 128, 255, 256, 1000, 65535, 65536] {
        emit(s, "to", 0);
    }
    // SET UP BY: ordinary increment + carry across a byte boundary.
    for (s, k) in [(5i32, 3i32), (0, 1), (99, 1), (255, 1), (250, 10), (65535, 1)] {
        emit(s, "up", k);
    }
    // SET DOWN BY: decrement, incl. crossing zero into two's-complement negatives (cobc does not clamp).
    for (s, k) in [(5i32, 5i32), (10, 4), (2, 5), (1, 2), (0, 1), (5, 9), (256, 1)] {
        emit(s, "down", k);
    }
}
