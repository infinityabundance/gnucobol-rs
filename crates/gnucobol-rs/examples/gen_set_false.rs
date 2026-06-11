//! Deterministic generator of `SET condition-name TO FALSE` cases (`GNURUST.12B`). Each line:
//! `label|pic-decl|size|88def|falselit`. The oracle declares `88 C VALUE<def> WHEN SET TO FALSE IS
//! <falselit>`, runs `SET C TO FALSE`, and dumps the parent bytes; the Rust mirror runs `set_88_false`.
use gnucobol_rs::{build_field, Usage};

fn size_of(pic: &str, comp3: bool) -> Option<usize> {
    let u = if comp3 { Usage::Comp3 } else { Usage::Display };
    build_field(pic, u, false, false).ok().map(|p| p.size)
}

fn main() {
    let mut id = 0u64;
    let mut emit = |pic: &str, comp3: bool, def: &str, fls: &str| {
        if let Some(size) = size_of(pic, comp3) {
            let decl = if comp3 { format!("{pic} USAGE COMP-3") } else { pic.to_string() };
            println!("f{id}|{decl}|{size}|{def}|{fls}");
            id += 1;
        }
    };
    // Alphanumeric: false literal differs from the true value(s).
    emit("X(1)", false, "la:A", "fa:Z");
    emit("X(3)", false, "la:AB", "fa:ZZ");
    emit("X(3)", false, "la:A;la:B;la:C", "fa:Q");
    emit("X(3)", false, "ra:AB:AM", "fa:ZZ");
    emit("X(5)", false, "la:HELLO", "fa:BYE");
    // Numeric DISPLAY: false literal outside the values.
    for pic in ["9", "9(2)", "9(3)", "S9", "S9(3)"] {
        emit(pic, false, "ln:1", "fn:0");
        emit(pic, false, "ln:5;ln:7;ln:9", "fn:0");
        emit(pic, false, "rn:1:3", "fn:0");
    }
    emit("S9", false, "ln:-3", "fn:1");
    emit("S9(3)", false, "rn:-5:-1", "fn:0");
    emit("S9V9", false, "ln:-1.5", "fn:0.5");
    emit("9(3)V99", false, "rn:050.00:099.99", "fn:0");
    // COMP-3: false literal differs.
    for pic in ["S9(3)", "S9(5)", "S9(3)V99", "9(5)", "S9(7)V99", "9(3)"] {
        emit(pic, true, "ln:1", "fn:0");
        emit(pic, true, "rn:1:5", "fn:0");
    }
    emit("S9(3)", true, "ln:-9", "fn:0");
}
