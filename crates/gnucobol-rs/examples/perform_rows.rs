//! Rust mirror for the PERFORM slice sweep (`GNURUST.PERFORM.SLICE.1`). Reads `label=<n>` and recomputes the
//! resulting C (or I) via eval_perform. PASS=n FAIL=n.
use gnucobol_rs::if_eval::{Relop, SliceField};
use gnucobol_rs::perform_slice::*;
use std::io::BufRead;
fn fields() -> Vec<SliceField<'static>> {
    vec![SliceField { name: "C", offset: 0, size: 3 }, SliceField { name: "I", offset: 3, size: 3 }]
}
fn parse(b: &[u8]) -> i64 { b.iter().fold(0i64, |a, &c| if c.is_ascii_digit() { a * 10 + (c - b'0') as i64 } else { a }) }
fn run(c0: i64, form: &PerformForm) -> Vec<u8> {
    let f = fields();
    let body = [AddOp { target: "C", amount: 1 }];
    eval_perform(format!("{c0:03}000").as_bytes(), &f, form, &body)
}
fn compute(label: &str) -> i64 {
    let cval = |out: &[u8]| parse(&out[0..3]);
    let ival = |out: &[u8]| parse(&out[3..6]);
    let ge5 = || NumCond { field: "C", op: Relop::Ge, value: 5 };
    match label {
        "times3" => cval(&run(0, &PerformForm::Times(3))),
        "times0" => cval(&run(0, &PerformForm::Times(0))),
        "until5" => cval(&run(0, &PerformForm::Until(ge5()))),
        "until_already" => cval(&run(7, &PerformForm::Until(ge5()))),
        "vary_body_c" => cval(&run(0, &PerformForm::Varying { var: "I", from: 1, by: 1, until: NumCond { field: "I", op: Relop::Gt, value: 4 } })),
        "vary_body_i" => ival(&run(0, &PerformForm::Varying { var: "I", from: 1, by: 1, until: NumCond { field: "I", op: Relop::Gt, value: 4 } })),
        "vary_by3_c" => cval(&run(0, &PerformForm::Varying { var: "I", from: 2, by: 3, until: NumCond { field: "I", op: Relop::Gt, value: 10 } })),
        "vary_by3_i" => ival(&run(0, &PerformForm::Varying { var: "I", from: 2, by: 3, until: NumCond { field: "I", op: Relop::Gt, value: 10 } })),
        "vary_none_c" => cval(&run(0, &PerformForm::Varying { var: "I", from: 5, by: 1, until: NumCond { field: "I", op: Relop::Gt, value: 2 } })),
        "vary_none_i" => ival(&run(0, &PerformForm::Varying { var: "I", from: 5, by: 1, until: NumCond { field: "I", op: Relop::Gt, value: 2 } })),
        _ => -1,
    }
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((label, oracle)) = line.split_once('=') else { continue };
        let o: i64 = oracle.trim().trim_start_matches('0').parse().unwrap_or(if oracle.trim().chars().all(|c| c == '0') { 0 } else { -1 });
        if compute(label) == o { pass += 1; } else { println!("{label} FAIL mine={} oracle={o}", compute(label)); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
