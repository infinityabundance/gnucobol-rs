//! Emit numeric->edited ENCODE cases (`GNURUST.16c`) as `label<TAB>pic<TAB>value` TSV. The sweep MOVEs
//! each value into the edited field and DISPLAYs its bytes; the Rust `edited_encode_rows` mirror calls
//! `encode_edited` and checks it reproduces those oracle bytes EXACTLY. Test infra. Includes floating +/- sign strings (sign-aware glyph).
fn main() {
    let mut id = 0u32;
    let mut emit = |pic: &str, value: &str| {
        println!("E{id}\t{pic}\t{value}");
        id += 1;
    };
    // unsigned integer edits.
    for v in ["0", "5", "42", "907"] {
        emit("ZZ9", v);
        emit("ZZZ9", v);
        emit("9999", v);
    }
    for v in ["0", "12", "1234", "99999"] {
        emit("ZZ,ZZ9", v);
        emit("Z,ZZ9", if v == "99999" { "9999" } else { v });
    }
    // unsigned decimal edits.
    for v in ["0.00", "0.07", "5.50", "1234.56", "99999.99"] {
        emit("ZZ,ZZ9.99", v);
    }
    for v in ["0.0", "3.5", "42.9"] {
        emit("ZZ9.9", v);
    }
    // fixed signs — leading & trailing, + and -.
    for v in ["0", "5", "-5", "42", "-907"] {
        emit("-ZZ9", v);
        emit("ZZ9-", v);
        emit("+ZZ9", v);
        emit("ZZ9+", v);
        emit("+9999", v);
    }
    for v in ["0.00", "12.34", "-12.34", "-1234.56"] {
        emit("-ZZ,ZZ9.99", v);
        emit("ZZ,ZZ9.99-", v);
        emit("+9999.99", v);
    }
    // floating currency / fixed currency / star (check protection).
    for v in ["0", "5", "42", "907"] {
        emit("$ZZ9", v);
        emit("$$$9", v);
        emit("***9", v);
        emit("9990", v);
    }
    for v in ["0.00", "5.50", "1234.56"] {
        emit("$ZZ,ZZ9.99", v);
        emit("$$$,$$9.99", v);
        emit("**,**9.99", v);
        emit("****.99", if v == "1234.56" { "42.00" } else { v });
    }
    // CR / DB trailing sign (+ a $-float + CR combo, and a B/DB combo).
    for v in ["0", "5", "-5", "-42"] {
        emit("ZZ9CR", v);
        emit("ZZ9DB", v);
    }
    for v in ["0.00", "12.34", "-12.34"] {
        emit("ZZ,ZZ9.99CR", v);
        emit("$$,$$9.99CR", v);
    }
    emit("9(4).99BBDB", "7");
    emit("9(4).99BBDB", "-7");
    emit("ZZZ9.99-", "-12.5");
    // B / slash insertions.
    for v in ["0", "1234", "120534"] {
        emit("99B99", &format!("{:04}", v.parse::<i64>().unwrap() % 10000));
        emit("99/99/99", &format!("{:06}", v.parse::<i64>().unwrap() % 1000000));
    }
    // floating +/- sign strings (sign-aware: + shows +/-, - shows space/-).
    for v in ["0", "5", "-5", "12.5", "-12.5"] {
        let val = if v.contains('.') { v.to_string() } else { format!("{v}.00") };
        emit("++++9.99", &val);
        emit("----9.99", &val);
    }
    for v in ["0", "7", "-7", "42", "-42"] {
        emit("++++9", v);
        emit("----9", v);
        emit("$$$$9", v);
    }
}
