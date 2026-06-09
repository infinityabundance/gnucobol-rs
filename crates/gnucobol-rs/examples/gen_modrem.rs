//! Generate FUNCTION MOD/REM cases (`GNURUST.INTRINSIC.MOD-REM.1`). Emits `label<TAB>op(MOD|REM)<TAB>a<TAB>b`.
fn main() {
    let pairs: &[(i64, i64)] = &[(17,5),(-17,5),(17,-5),(-17,-5),(15,5),(0,5),(100,7),(-100,7),(23,-4),(-23,-4)];
    let mut id = 0;
    for op in ["MOD","REM"] {
        for (a,b) in pairs { println!("m{id}\t{op}\t{a}\t{b}"); id += 1; }
    }
}
