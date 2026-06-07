//! Deterministic generator of LEVEL-88 sweep cases (`GNURUST.11`). Each line:
//! `label|pic|mvkind:mv|88def`  where mvkind ∈ {A,N}; 88def is `;`-separated entries each
//! `la:s` | `ln:n` | `ra:a:b` | `rn:a:b`. The oracle MOVEs `mv` into the parent and evaluates the
//! 88; the Rust mirror encodes the same value (via the sealed value_image) and runs eval_88.

fn main() {
    let mut id = 0u64;
    let mut emit = |pic: &str, mvkind: char, mv: &str, def: &str| {
        println!("c{id}|{pic}|{mvkind}:{mv}|{def}");
        id += 1;
    };

    // --- Alphanumeric parents ---
    for mv in ["A", "B", "C", "D"] {
        emit("X(1)", 'A', mv, "la:A");
        emit("X(1)", 'A', mv, "la:A;la:B;la:C");
        emit("X(1)", 'A', mv, "ra:A:C");
    }
    for mv in ["X", "Y", "Z", "W"] {
        emit("X(1)", 'A', mv, "ra:X:Z");
    }
    for mv in ["AB", "AC", "ZZ"] {
        emit("X(3)", 'A', mv, "la:AB");
        emit("X(3)", 'A', mv, "ra:AA:AM");
    }

    // --- Numeric DISPLAY parents ---
    for mv in ["0", "5", "6", "9"] {
        emit("9", 'N', mv, "ln:5");
        emit("9", 'N', mv, "ln:1;ln:2;ln:5");
        emit("9", 'N', mv, "rn:1:7");
    }
    for mv in ["05", "15", "20", "25", "99"] {
        emit("9(2)", 'N', mv, "rn:10:20");
        emit("9(2)", 'N', mv, "ln:15");
    }
    // signed
    for mv in ["3", "-3", "0", "-5"] {
        emit("S9", 'N', mv, "ln:-3");
        emit("S9", 'N', mv, "rn:-5:-1");
    }
    // scaled (V) signed
    for mv in ["1.5", "2.0", "2.5", "3.0", "-1.5"] {
        emit("S9V9", 'N', mv, "rn:1.5:2.5");
        emit("S9V9", 'N', mv, "ln:-1.5");
    }
    // wider scaled
    for mv in ["012.34", "099.99", "100.00", "050.00", "099.98"] {
        emit("9(3)V99", 'N', mv, "rn:000.00:099.99");
        emit("9(3)V99", 'N', mv, "ln:050.00;ln:099.99");
    }

    // More alphanumeric coverage: multi-char literals, padded comparison, multi-value + range mix.
    for mv in ["AA", "AM", "AZ", "BA"] {
        emit("X(2)", 'A', mv, "la:AA;la:AM;la:AZ");
        emit("X(2)", 'A', mv, "ra:AA:AZ");
    }
    for mv in ["A", "M", "N", "Z", "0", "9"] {
        emit("X(1)", 'A', mv, "ra:A:M;ra:N:Z"); // two ranges
    }
    // Numeric: multi single values, larger ranges, boundary values (THRU is inclusive).
    for mv in ["00", "01", "07", "08", "50", "99"] {
        emit("9(2)", 'N', mv, "ln:1;ln:7;ln:50");
        emit("9(2)", 'N', mv, "rn:1:7;rn:50:99"); // two ranges, boundaries
    }
    // Signed range crossing zero.
    for mv in ["-9", "-1", "0", "1", "9"] {
        emit("S9", 'N', mv, "rn:-2:2");
    }
}
