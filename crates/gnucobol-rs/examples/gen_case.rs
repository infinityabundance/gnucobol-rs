//! Generate FUNCTION UPPER-CASE/LOWER-CASE/REVERSE cases (`GNURUST.INTRINSIC.CASE.1`). `label<TAB>op<TAB>input`.
fn main() {
    let c: &[(&str, &str)] = &[
        ("UPPER-CASE","aB3 z!"), ("UPPER-CASE","hello"), ("UPPER-CASE","ABC"),
        ("LOWER-CASE","Ab3 Z!"), ("LOWER-CASE","WORLD"), ("LOWER-CASE","abc"),
        ("REVERSE","ab c"), ("REVERSE","12345"), ("REVERSE","x"), ("REVERSE","abcdefgh"),
    ];
    for (id, (op, inp)) in c.iter().enumerate() { println!("c{id}\t{op}\t{inp}"); }
}
