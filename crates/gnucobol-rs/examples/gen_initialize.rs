//! Generate INITIALIZE byte-effect cases (`GNURUST.INITIALIZE.1`). Emits `label<TAB>reclen<TAB>lines` where
//! lines are the record's `05/10` declarations joined by `|`. Test infra.
fn main() {
    let cases: &[(&str, usize, &[&str])] = &[
        ("elem", 14, &["05 A PIC X(4).", "05 N PIC 9(3).", "05 SN PIC S9(3).", "05 P PIC S9(3) COMP-3.", "05 B PIC 9(4) COMP."]),
        ("filler", 6, &["05 X1 PIC X(2).", "05 FILLER PIC X(2).", "05 X2 PIC X(2)."]),
        ("redef", 6, &["05 BASE PIC X(4).", "05 RED REDEFINES BASE PIC 9(4).", "05 TAIL PIC X(2)."]),
        ("value", 5, &["05 A PIC X(3) VALUE \"ABC\".", "05 N PIC 9(2) VALUE 99."]),
        ("group", 8, &["05 G.", "10 GA PIC X(3).", "10 GN PIC 9(2).", "05 H PIC X(3)."]),
        ("packs", 5, &["05 P1 PIC S9(5) COMP-3.", "05 P2 PIC 9(3) COMP-3."]),
    ];
    for (label, reclen, lines) in cases {
        println!("{label}\t{reclen}\t{}", lines.join("|"));
    }
}
