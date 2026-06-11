//! Generate class-condition cases (`GNURUST.CLASS.1`): `label|pic|test|hex`. test ∈ num/snum/alp/upr/lwr.
//! `pic` is the real field PIC (the sweep declares it + a REDEFINES X(n) to inject the bytes).
fn main() {
    let mut id = 0u32;
    let mut emit = |pic: &str, test: &str, s: &[u8]| {
        let hex: String = s.iter().map(|b| format!("{b:02x}")).collect();
        println!("c{id}|{pic}|{test}|{hex}");
        id += 1;
    };
    for s in [&b"0012"[..], b" 12 ", b"12AB", b"0000", b"    ", b"999", b"00", b"7"] {
        emit(&format!("X({})", s.len()), "num", s);
    }
    // signed-numeric (trailing overpunch) on PIC S9(3).
    for s in [&b"012"[..], b"01r", b"01p", b"01y", b"01A", b"01z", b"01 ", b"999", b"r00", b"01/"] {
        emit("S9(3)", "snum", s);
    }
    for s in [&b"ABCD"[..], b"AB12", b"AB  ", b"abcd", b"    ", b"a1", b"Hello", b"MiXeD"] {
        emit(&format!("X({})", s.len()), "alp", s);
    }
    for s in [&b"ABCD"[..], b"abcd", b"AB  ", b"A1", b"  "] {
        emit(&format!("X({})", s.len()), "upr", s);
    }
    for s in [&b"abcd"[..], b"ABCD", b"ab  ", b"a1", b"  "] {
        emit(&format!("X({})", s.len()), "lwr", s);
    }
}
