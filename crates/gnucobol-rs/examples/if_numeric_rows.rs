//! Rust mirror for the numeric IF/EVALUATE slice sweep (`GNURUST.IF.NUMERIC.SLICE.1`).
use gnucobol_rs::if_eval::{Relop, SliceField};
use gnucobol_rs::if_numeric::*;
use gnucobol_rs::perform_slice::NumCond;
use std::io::BufRead;
fn fields() -> Vec<SliceField<'static>> {
    vec![SliceField { name: "N", offset: 0, size: 3 }, SliceField { name: "F", offset: 3, size: 2 }]
}
fn rec(n: i64) -> Vec<u8> { format!("{n:03}00").into_bytes() }
fn f_of(out: &[u8]) -> i64 { out[3..5].iter().fold(0i64, |a, &c| a * 10 + (c - b'0') as i64) }
fn iff(n: i64, op: Relop, v: i64, th: i64, el: i64) -> i64 {
    let fl = fields();
    f_of(&eval_if_numeric(&rec(n), &fl, &NumCond { field: "N", op, value: v }, &[MoveNum { value: th, target: "F" }], &[MoveNum { value: el, target: "F" }]))
}
fn ev(n: i64) -> i64 {
    let fl = fields();
    let whens: Vec<(i64, &[MoveNum])> = vec![(10, &[MoveNum { value: 1, target: "F" }]), (50, &[MoveNum { value: 5, target: "F" }])];
    f_of(&eval_evaluate_numeric(&rec(n), &fl, "N", &whens, &[MoveNum { value: 8, target: "F" }]))
}
fn compute(label: &str) -> i64 {
    match label {
        "gt100" => iff(50, Relop::Gt, 100, 1, 9),
        "lt100" => iff(50, Relop::Lt, 100, 1, 9),
        "eq50" => iff(50, Relop::Eq, 50, 7, 0),
        "ge50" => iff(50, Relop::Ge, 50, 1, 9),
        "ev50" => ev(50),
        "ev99" => ev(99),
        "num5gt9" => iff(5, Relop::Gt, 9, 1, 9),
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
