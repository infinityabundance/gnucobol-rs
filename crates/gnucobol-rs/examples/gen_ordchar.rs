//! Generate FUNCTION ORD/CHAR cases (`GNURUST.INTRINSIC.ORD-CHAR.1`). `label<TAB>op(ORD|CHAR)<TAB>arg`
//! (arg = a single char for ORD, a 1..=256 number for CHAR).
fn main() {
    let ord: &[&str] = &["A", "0", "9", "z", "M", " ", "~"];
    let chr: &[&str] = &["66", "49", "65", "97", "1", "256", "128", "33"];
    let mut id = 0;
    for a in ord { println!("o{id}\tORD\t{a}"); id += 1; }
    for n in chr { println!("o{id}\tCHAR\t{n}"); id += 1; }
}
