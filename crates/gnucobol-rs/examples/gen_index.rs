//! Generate USAGE INDEX cases (`GNURUST.INDEX.1`): `label|start|op|k|stride`. Each case `SET ix TO start`
//! then optionally `SET ix UP/DOWN BY k`; the dumped value is the 4-byte native-endian index word.
//! op ∈ to (no further op) / up / down. `stride` selects the index item: 0 = a standalone `USAGE INDEX`;
//! >0 = an `INDEXED BY` index-name over a `PIC X(stride)` table — proving the stored value is the
//! occurrence number, ELEMENT-SIZE INDEPENDENT (the bytes must be identical across strides).
fn main() {
    let mut id = 0u32;
    let mut emit = |start: i32, op: &str, k: i32, stride: i32| {
        println!("i{id}|{start}|{op}|{k}|{stride}");
        id += 1;
    };
    // Standalone USAGE INDEX — SET TO: occurrence numbers across byte boundaries (1,2,3,4-byte ranges).
    for s in [0i32, 1, 2, 3, 5, 9, 10, 16, 42, 99, 128, 255, 256, 1000, 65535, 65536] {
        emit(s, "to", 0, 0);
    }
    // Standalone — SET UP BY: ordinary increment + carry across a byte boundary.
    for (s, k) in [(5i32, 3i32), (0, 1), (99, 1), (255, 1), (250, 10), (65535, 1)] {
        emit(s, "up", k, 0);
    }
    // Standalone — SET DOWN BY: decrement, incl. crossing zero into two's-complement negatives (no clamp).
    for (s, k) in [(5i32, 5i32), (10, 4), (2, 5), (1, 2), (0, 1), (5, 9), (256, 1)] {
        emit(s, "down", k, 0);
    }
    // INDEXED BY over PIC X(4) AND PIC X(17): the SAME occurrence must store the SAME 4 bytes for both
    // strides (and equal the standalone case above) — element-size independence, not an offset.
    for stride in [4i32, 17] {
        for s in [1i32, 5, 9, 100] {
            emit(s, "to", 0, stride);
        }
        emit(5, "up", 3, stride); // arithmetic is on the occurrence, not the address
        emit(2, "down", 5, stride); // DOWN past zero is still occurrence arithmetic
    }
}
