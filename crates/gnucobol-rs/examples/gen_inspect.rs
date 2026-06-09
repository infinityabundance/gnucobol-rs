//! Generate INSPECT cases (`GNURUST.INSPECT.1`). Emits
//! `label<TAB>op(TALLY|REPL|CONV)<TAB>target<TAB>mode(all|leading|first|chars)<TAB>a1<TAB>a2<TAB>region`.
//! region is ""|"before:X"|"after:X". Test infra.
fn main() {
    let c: &[(&str, &str, &str, &str, &str, &str, &str)] = &[
        ("t_all", "TALLY", "ABABAA", "all", "A", "", ""),
        ("t_leading", "TALLY", "AABBAA", "leading", "A", "", ""),
        ("t_chars", "TALLY", "ABCDEF", "chars", "", "", ""),
        ("t_overlap", "TALLY", "AAAAAA", "all", "AA", "", ""),
        ("t_before", "TALLY", "ABXABY", "all", "A", "", "before:X"),
        ("t_after", "TALLY", "ABXABY", "all", "A", "", "after:X"),
        ("r_all", "REPL", "ABABAA", "all", "A", "B", ""),
        ("r_leading", "REPL", "AABBAA", "leading", "A", "X", ""),
        ("r_first", "REPL", "ABABAA", "first", "A", "Z", ""),
        ("r_after", "REPL", "ABXABY", "all", "A", "Z", "after:X"),
        ("c_basic", "CONV", "CABFED", "", "ABCDEF", "UVWXYZ", ""),
        ("c_before", "CONV", "ABXABY", "", "AB", "XY", "before:X"),
    ];
    for (l, op, t, m, a1, a2, r) in c {
        println!("{l}\t{op}\t{t}\t{m}\t{a1}\t{a2}\t{r}");
    }
}
