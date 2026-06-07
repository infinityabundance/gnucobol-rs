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

    // 16b financial decorations.
    for v in ["0", "5", "42", "907"] {
        emit("$ZZ9", v); // fixed currency
        emit("$$$9", v); // floating currency
        emit("***9", v); // star / check protection
        emit("9990", v); // literal '0' insertion (the slot-aware case)
    }
    for v in ["0.00", "5.50", "1234.56"] {
        emit("$ZZ,ZZ9.99", v);
        emit("$$$,$$9.99", v);
        emit("**,**9.99", v);
    }
    // CR / DB trailing sign (negative shows CR/DB, positive shows blanks).
    for v in ["0", "5", "-5", "-42"] {
        emit("ZZ9CR", v);
        emit("ZZ9DB", v);
    }
    for v in ["0.00", "12.34", "-12.34"] {
        emit("ZZ,ZZ9.99CR", v);
    }
    // B / slash insertions (date/blank-shaped edits).
    for v in ["0", "1234", "120534"] {
        emit(
            "99B99",
            &format!("{:04}", v.parse::<i64>().unwrap() % 10000),
        );
        emit(
            "99/99/99",
            &format!("{:06}", v.parse::<i64>().unwrap() % 1000000),
        );
    }
}
