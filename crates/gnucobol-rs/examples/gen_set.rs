//! Deterministic generator of `SET condition-name TO TRUE` cases (`GNURUST.12`). Each line:
//! `label|pic|size|88def` (size from the sealed build_field, used by the oracle's REDEFINES). The
//! oracle `SET`s the condition TRUE and dumps the parent bytes; the Rust mirror runs `set_88_true`.

use gnucobol_rs::{build_field, Usage};

fn size_of(pic: &str) -> Option<usize> {
    build_field(pic, Usage::Display, false, false)
        .ok()
        .map(|p| p.size)
}
fn size_comp3(pic: &str) -> Option<usize> {
    build_field(pic, Usage::Comp3, false, false)
        .ok()
        .map(|p| p.size)
}

fn main() {
    let mut id = 0u64;
    let mut emit = |pic: &str, comp3: bool, def: &str| {
        let sz = if comp3 { size_comp3(pic) } else { size_of(pic) };
        if let Some(size) = sz {
            // the oracle needs the real declared PIC incl. usage; carry a usage tag in the pic field.
            let decl = if comp3 {
                format!("{pic} USAGE COMP-3")
            } else {
                pic.to_string()
            };
            println!("s{id}|{decl}|{size}|{def}");
            id += 1;
        }
    };

    // Alphanumeric: single, multiple (first), range (lower bound).
    emit("X(1)", false, "la:A");
    emit("X(3)", false, "la:A");
    emit("X(3)", false, "la:AB");
    emit("X(3)", false, "la:A;la:B;la:C");
    emit("X(3)", false, "la:M;la:A;la:Z"); // first = M
    emit("X(3)", false, "ra:AB:AM");
    emit("X(1)", false, "ra:A:Z");

    // Numeric DISPLAY: single, multiple (first), range (lower).
    for pic in ["9", "9(2)", "9(3)", "S9", "S9(3)"] {
        emit(pic, false, "ln:1");
        emit(pic, false, "ln:5;ln:7;ln:9");
        emit(pic, false, "rn:1:3");
    }
    emit("S9", false, "ln:-3");
    emit("S9(3)", false, "rn:-5:-1"); // lower bound -5
    emit("S9V9", false, "rn:1.5:2.5");
    emit("S9V9", false, "ln:-1.5");
    emit("9(3)V99", false, "rn:050.00:099.99");

    // COMP-3: single, range lower.
    for pic in ["S9(3)", "S9(5)", "S9(3)V99", "9(5)", "S9(7)V99", "9(3)"] {
        emit(pic, true, "ln:1");
        emit(pic, true, "rn:1:5");
        emit(pic, true, "ln:0;ln:2;ln:4"); // first = 0
    }
    // More alphanumeric multi/range coverage.
    for d in ["la:AA;la:ZZ", "ra:AA:MM", "la:HELLO", "ra:000:999"] {
        emit("X(5)", false, d);
    }
    // Numeric multi-value where first is mid/high.
    emit("9(2)", false, "ln:42;ln:7;ln:1");
    emit("S9(3)", false, "ln:-99;ln:1");
    emit("9(4)V99", false, "rn:0012.34:9999.99"); // lower bound 0012.34
}
