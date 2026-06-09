//! Generate FUNCTION INTEGER / INTEGER-PART cases (`GNURUST.INTRINSIC.INTEGER.1`). Emits `label<TAB>op<TAB>x`.
fn main() {
    let vals = ["3.7","-3.7","2.5","-2.5","3.0","-3.0","-0.1","0.9","100.99","-100.99"];
    let mut id = 0;
    for op in ["INTEGER","INTEGER-PART"] {
        for v in vals { println!("g{id}\t{op}\t{v}"); id += 1; }
    }
}
