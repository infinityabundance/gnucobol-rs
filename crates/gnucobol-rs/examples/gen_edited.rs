//! Emit edited-picture decode cases (`GNURUST.16a`) as `label<TAB>pic<TAB>value` TSV. The sweep
//! builds one COBOL program that MOVEs each value into the edited field and DISPLAYs its bytes; the
//! Rust `edited_rows` mirror decodes those bytes back and checks the recovered value. Test infra.

fn main() {
    let mut id = 0u32;
    let mut emit = |pic: &str, value: &str| {
        println!("L{id}\t{pic}\t{value}");
        id += 1;
    };
    // Unsigned integer edits (non-negative values; an unsigned PIC drops the sign).
    for v in ["0", "5", "42", "907"] {
        emit("ZZ9", v);
        emit("ZZZ9", v);
    }
    for v in ["0", "12", "1234", "99999"] {
        emit("ZZ,ZZ9", v);
        emit("Z,ZZ9", if v == "99999" { "9999" } else { v });
    }
    // Unsigned decimal edits.
    for v in ["0.00", "0.07", "5.50", "1234.56", "99999.99"] {
        emit("ZZ,ZZ9.99", v);
    }
    for v in ["0.0", "3.5", "42.9"] {
        emit("ZZ9.9", v);
    }
    // Signed edits — leading and trailing, both `+` and `-`.
    for v in ["0", "5", "-5", "42", "-907"] {
        emit("-ZZ9", v);
        emit("ZZ9-", v);
    }
    for v in ["0", "7", "-7", "123"] {
        emit("+ZZ9", v);
        emit("ZZ9+", v);
    }
    for v in ["0.00", "12.34", "-12.34", "-1234.56"] {
        emit("-ZZ,ZZ9.99", v);
        emit("ZZ,ZZ9.99-", v);
    }
}
