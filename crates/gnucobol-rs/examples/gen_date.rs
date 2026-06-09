//! Generate date-intrinsic cases (`GNURUST.INTRINSIC.DATE.1`). `label<TAB>op<TAB>arg`. op = IOD|DOI|IODY|DYOI.
fn main() {
    let iod = [16010101u32,16010102,18000101,19000228,19000301,20000101,20000229,20240229,20231231,19991231,99991231];
    let doi = [1i64,2,145731,145732,154557,100000,2000000,3000000];
    let iody = [1601001u32,2000001,2024060,2024366,2023365,9999365];
    let dyoi = [1i64,145732,154557,100000,2000000];
    let mut id = 0;
    for v in iod { println!("d{id}\tIOD\t{v}"); id += 1; }
    for v in doi { println!("d{id}\tDOI\t{v}"); id += 1; }
    for v in iody { println!("d{id}\tIODY\t{v}"); id += 1; }
    for v in dyoi { println!("d{id}\tDYOI\t{v}"); id += 1; }
}
