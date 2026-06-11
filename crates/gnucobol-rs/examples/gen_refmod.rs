//! Generate reference-modification cases (`GNURUST.REFMOD.1`): `label|field|op|start|length|src`.
//! op ∈ src (`F(start:length)`), end (`F(start:)`), recv (`MOVE src TO F(start:length)`).
fn main() {
    let mut id = 0u32;
    let mut emit = |field: &str, op: &str, start: usize, length: usize, src: &str| {
        println!("r{id}|{field}|{op}|{start}|{length}|{src}");
        id += 1;
    };
    for (f, s, l) in [("ABCDEF", 1, 3), ("ABCDEF", 2, 3), ("ABCDEF", 4, 3), ("ABCDEF", 1, 6), ("ABCDEF", 6, 1), ("HELLO", 2, 3), ("12345", 3, 2)] {
        emit(f, "src", s, l, "");
    }
    for (f, s) in [("ABCDEF", 3), ("ABCDEF", 1), ("HELLO", 4), ("12345", 5)] {
        emit(f, "end", s, 0, "");
    }
    for (f, src, s, l) in [("ABCDEF", "XY", 2, 2), ("ABCDEF", "Z", 5, 2), ("ABCDEF", "123", 1, 3), ("HELLO", "Q", 3, 1), ("ABCDEF", "PQRS", 2, 3)] {
        emit(f, "recv", s, l, src);
    }
}
