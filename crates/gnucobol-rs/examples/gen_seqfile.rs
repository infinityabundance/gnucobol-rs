//! Generate sequential-file READ cases (`GNURUST.FILE.SEQUENTIAL.1`). Emits
//! `label<TAB>org(RECORD|LINE)<TAB>record_len<TAB>file_hex`. Test infra.
fn hx(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
fn main() {
    let cases: &[(&str, &str, &[u8])] = &[
        ("r_exact", "RECORD", b"01234567ABCDEFGH"),
        ("r_partial", "RECORD", b"01234567ABCDEFGHWXYZ"),
        ("r_partial_first", "RECORD", b"WXYZ"),
        ("r_single", "RECORD", b"01234567"),
        ("l_short", "LINE", b"AB\nCD\nEF\n"),
        ("l_long", "LINE", b"AB\nCDEFGHIJKL\nXY\n"),
        ("l_notrail", "LINE", b"AB\nXY"),
        ("l_midempty", "LINE", b"AB\n\nEF\n"),
        ("l_exact8", "LINE", b"CDEFGHIJ\n"),
        ("l_long_eof", "LINE", b"CDEFGHIJKL"),
    ];
    for (label, org, bytes) in cases {
        println!("{label}\t{org}\t8\t{}", hx(bytes));
    }
}
