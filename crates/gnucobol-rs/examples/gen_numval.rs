//! Generate FUNCTION NUMVAL cases (`GNURUST.INTRINSIC.NUMVAL.1`). Emits `label<TAB>input`. Test infra.
fn main() {
    let c: &[(&str, &str)] = &[
        ("plain","123.45"), ("spaces","  123  "), ("lead_neg","-123.45"), ("lead_pos","+123.45"),
        ("trail_neg","123.45-"), ("trail_pos","123.45+"), ("sp_neg","  -42 "), ("cr","123.45 CR"),
        ("db","123.45 DB"), ("lead_dot",".5"), ("leadzero","007"), ("zero","0"),
        ("big","12345678.9999"), ("frac3","1.234"),
    ];
    for (l, i) in c { println!("{l}\t{i}"); }
}
