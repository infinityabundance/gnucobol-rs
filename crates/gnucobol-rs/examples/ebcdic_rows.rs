//! Print gnucobol-rs's embedded cp500 EBCDIC->ASCII table as 256 lower-hex bytes (one line), to be
//! compared byte-for-byte against the admitted oracle's `cob_load_collation` output. Test infra.
fn main() {
    let mut s = String::new();
    for b in 0u16..256 {
        let a = gnucobol_rs::translate_byte(gnucobol_rs::CodePage::Cp500, b as u8).unwrap();
        s.push_str(&format!("{a:02x}"));
    }
    println!("{s}");
}
