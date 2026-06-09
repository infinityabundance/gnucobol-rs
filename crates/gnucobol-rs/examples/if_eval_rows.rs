//! Rust mirror for the IF/EVALUATE slice sweep (`GNURUST.IF.EVALUATE.SLICE.1`). Reads `label=[T]` and
//! recomputes the resulting T via eval_if/eval_evaluate. PASS=n FAIL=n.
use gnucobol_rs::if_eval::Operand::*;
use gnucobol_rs::if_eval::*;
use std::io::BufRead;
fn fields() -> Vec<SliceField<'static>> {
    vec![SliceField { name: "A", offset: 0, size: 3 }, SliceField { name: "T", offset: 3, size: 4 }]
}
fn rec(a: &str) -> Vec<u8> {
    let mut av = a.as_bytes().to_vec(); av.resize(3, b' ');
    av.extend_from_slice(b"----"); av
}
fn t(out: &[u8]) -> String { String::from_utf8_lossy(&out[3..7]).into_owned() }
fn iff(a: &str, op: Relop, rhs: &[u8], th: &[u8], el: Operand) -> String {
    let f = fields();
    t(&eval_if(&rec(a), &f, &Condition { left: Field("A"), op, right: Literal(rhs) },
        &[MoveStmt { source: Literal(th), target: "T" }], &[MoveStmt { source: el, target: "T" }]))
}
fn ev(a: &str) -> String {
    let f = fields();
    let whens: Vec<(&[u8], &[MoveStmt])> = vec![
        (b"A", &[MoveStmt { source: Literal(b"AAA"), target: "T" }]),
        (b"B", &[MoveStmt { source: Literal(b"BEE"), target: "T" }]),
    ];
    let other = [MoveStmt { source: Literal(b"OTH"), target: "T" }];
    t(&eval_evaluate(&rec(a), &f, "A", &whens, &other))
}
fn compute(label: &str) -> String {
    match label {
        "if_eq"      => iff("BBB", Relop::Eq, b"BBB", b"YES", Literal(b"NO")),
        "if_gt"      => iff("BBB", Relop::Gt, b"AAA", b"GT", Literal(b"LE")),
        "if_lt_else" => iff("BBB", Relop::Lt, b"AAA", b"Y", Field("A")),
        "if_ne"      => iff("BBB", Relop::Ne, b"BBB", b"NE", Literal(b"EQ")),
        "if_ge"      => iff("BBB", Relop::Ge, b"BBB", b"GE", Literal(b"X")),
        "if_le"      => iff("BBB", Relop::Le, b"AAA", b"LE", Literal(b"GT")),
        "eval_B"     => ev("B"),
        "eval_Z"     => ev("Z"),
        "eval_A"     => ev("A"),
        _ => String::new(),
    }
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((label, content)) = line.split_once('=') else { continue };
        let oracle = content.trim_start_matches('[').trim_end_matches(']');
        if compute(label) == oracle { pass += 1; } else { println!("{label} FAIL mine=[{}] oracle=[{oracle}]", compute(label)); fail += 1; }
    }
    println!("PASS={pass} FAIL={fail}");
}
