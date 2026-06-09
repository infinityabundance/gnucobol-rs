//! Generate FUNCTION LENGTH cases (`GNURUST.INTRINSIC.LENGTH.1`). Emits `label<TAB>pic<TAB>usage`. Test infra.
fn main() {
    let c: &[(&str, &str, &str)] = &[
        ("x5","X(5)","DISPLAY"), ("n5","9(5)","DISPLAY"), ("s8v2","S9(8)V99","DISPLAY"),
        ("s3c3","S9(3)","COMP-3"), ("n4c","9(4)","COMP"), ("n7c3","9(7)","COMP-3"),
        ("s9c","S9(9)","COMP"), ("x1","X(1)","DISPLAY"), ("n3","9(3)","DISPLAY"),
        ("s5v2c3","S9(5)V99","COMP-3"), ("n9c5","9(9)","COMP-5"), ("x10","X(10)","DISPLAY"),
    ];
    for (l, p, u) in c { println!("{l}\t{p}\t{u}"); }
}
