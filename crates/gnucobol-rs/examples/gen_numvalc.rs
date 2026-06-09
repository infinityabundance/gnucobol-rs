//! Generate FUNCTION NUMVAL-C cases (`GNURUST.INTRINSIC.NUMVAL-C.1`). Emits `label<TAB>input`.
fn main() {
    let c: &[&str] = &["$1,234.56","1,234.56","1,234,567","-$1,234.56","$1,234.56-","$1,234.56CR","  $42.00  ","1234.56","0","$0.99"];
    for (id, inp) in c.iter().enumerate() { println!("v{id}\t{inp}"); }
}
