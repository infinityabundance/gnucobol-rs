//! Generate class-condition cases (`GNURUST.CLASS.1`): `label|len|test|hex`. test ∈ num/alp/upr/lwr.
fn main() {
    let mut id = 0u32;
    let mut emit = |test: &str, s: &[u8]| {
        let hex: String = s.iter().map(|b| format!("{b:02x}")).collect();
        println!("c{id}|{}|{test}|{hex}", s.len());
        id += 1;
    };
    for s in [&b"0012"[..], b" 12 ", b"12AB", b"0000", b"    ", b"999", b"00", b"7"] {
        emit("num", s);
    }
    for s in [&b"ABCD"[..], b"AB12", b"AB  ", b"abcd", b"    ", b"a1", b"Hello", b"MiXeD"] {
        emit("alp", s);
    }
    for s in [&b"ABCD"[..], b"abcd", b"AB  ", b"A1", b"  "] {
        emit("upr", s);
    }
    for s in [&b"abcd"[..], b"ABCD", b"ab  ", b"a1", b"  "] {
        emit("lwr", s);
    }
}
