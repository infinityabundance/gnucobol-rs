//! Generate DIVIDE receiving-field byte cases (`GNURUST.19`). Emits
//! `label<TAB>form<TAB>rounded<TAB>a_val<TAB>a_pic<TAB>a_usage<TAB>b_val<TAB>b_pic<TAB>b_usage<TAB>c_pic<TAB>c_usage<TAB>c_size`.
//! form = BY (C := A/B) or INTO (C := B/A). Non-zero divisors only (n/0 is a fail-closed non-claim). Test infra.

fn usage_size(pic_digits: usize, usage: &str) -> usize {
    if usage == "COMP-3" {
        pic_digits / 2 + 1
    } else {
        pic_digits
    }
}

fn main() {
    // (pic, digits) for operands and receivers
    let op_kinds = [("S9(5)V99", 7usize)];
    let recv_kinds = [("S9(5)V99", 7usize), ("S9(3)V9", 4usize)];
    let usages = ["DISPLAY", "COMP-3"];
    // (a, b) operand pairs covering the dangerous traps
    let pairs = [
        ("10.00", "3.00"),
        ("20.00", "3.00"),
        ("1.00", "2.00"),
        ("1.00", "3.00"),
        ("-10.00", "3.00"),
        ("10.00", "-3.00"),
        ("-1.00", "-2.00"),
        ("0.00", "7.00"),
        ("100.00", "8.00"),
        ("7.00", "4.00"),
        ("99.99", "7.00"),
        ("5.00", "1.00"),
    ];
    let mut id = 0u32;
    for (a_pic, a_dig) in op_kinds {
        for a_use in usages {
            for (b_pic, b_dig) in op_kinds {
                for b_use in usages {
                    for (c_pic, c_dig) in recv_kinds {
                        for c_use in usages {
                            for form in ["BY", "INTO"] {
                                for rounded in [0, 1] {
                                    for (a, b) in pairs {
                                        // Divisor is `b` for `A BY B`, `a` for `A INTO B`. Divide-by-zero
                                        // is a fail-closed NON-CLAIM (no SIZE ERROR court) -> exclude.
                                        let divisor = if form == "BY" { b } else { a };
                                        if divisor
                                            .trim_start_matches('-')
                                            .parse::<f64>()
                                            .unwrap_or(0.0)
                                            == 0.0
                                        {
                                            continue;
                                        }
                                        let csz = usage_size(c_dig, c_use);
                                        println!("d{id}\t{form}\t{rounded}\t{a}\t{a_pic}\t{a_use}\t{b}\t{b_pic}\t{b_use}\t{c_pic}\t{c_use}\t{csz}",
                                                 a_pic = a_pic, a_use = a_use, b_pic = b_pic, b_use = b_use);
                                        let _ = (a_dig, b_dig);
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
}
