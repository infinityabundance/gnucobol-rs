//! Generate DIVIDE...REMAINDER receiving-field byte cases (`GNURUST.REMAINDER.1`). Emits
//! `label<TAB>form<TAB>a<TAB>a_pic<TAB>a_use<TAB>b<TAB>b_pic<TAB>b_use<TAB>c_pic<TAB>c_use<TAB>c_sz<TAB>d_pic<TAB>d_use<TAB>d_sz`.
//! form = BY (q := a/b) or INTO (q := b/a). c = quotient receiver, d = remainder receiver. The REMAINDER
//! forms use the un-rounded quotient, so no ROUNDED column. Non-zero divisors only (n/0 is fail-closed).
fn usage_size(pic_digits: usize, usage: &str) -> usize {
    if usage == "COMP-3" { pic_digits / 2 + 1 } else { pic_digits }
}
fn main() {
    let (a_pic, b_pic) = ("S9(5)V99", "S9(5)V99");
    let quot_kinds = [("S9(5)", 5usize), ("S9(5)V99", 7usize)]; // integer + scaled quotient
    let rem_kinds = [("S9(5)V99", 7usize)];
    let usages = ["DISPLAY", "COMP-3"];
    // sign coverage + exact (zero remainder) + non-exact + scaled
    let pairs = [
        ("10.00", "3.00"), ("10.00", "-3.00"), ("-10.00", "3.00"), ("-10.00", "-3.00"),
        ("10.00", "5.00"), ("1.00", "2.00"), ("1.00", "3.00"), ("100.00", "8.00"),
        ("7.00", "4.00"), ("99.99", "7.00"), ("20.00", "6.00"), ("17.00", "5.00"),
    ];
    let mut id = 0u32;
    for a_use in usages {
        for b_use in usages {
            for (c_pic, c_dig) in quot_kinds {
                for c_use in usages {
                    for (d_pic, d_dig) in rem_kinds {
                        for d_use in usages {
                            for form in ["BY", "INTO"] {
                                for (a, b) in pairs {
                                    let divisor = if form == "BY" { b } else { a };
                                    if divisor.trim_start_matches('-').parse::<f64>().unwrap_or(0.0) == 0.0 {
                                        continue;
                                    }
                                    let csz = usage_size(c_dig, c_use);
                                    let dsz = usage_size(d_dig, d_use);
                                    println!("m{id}\t{form}\t{a}\t{a_pic}\t{a_use}\t{b}\t{b_pic}\t{b_use}\t{c_pic}\t{c_use}\t{csz}\t{d_pic}\t{d_use}\t{dsz}");
                                    id += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
