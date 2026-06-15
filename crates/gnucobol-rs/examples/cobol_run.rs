//! END-TO-END EXECUTION of a real COBOL program on the gnucobol-rs RUNTIME -- no `cobc`, no `libcob`
//! linked. This hand-wires (one statement at a time) the program below into the ported runtime
//! primitives and writes the program's exact stdout bytes. The companion sweep
//! (`lab/oracle/cobol_run_sweep.sh`) compiles+runs the SAME source with `cobc` and diffs the output:
//! a byte-identical result is direct evidence that the 13/13-file libcob runtime port is
//! execution-complete for this program. (The piece still missing for a turn-key "compiler" is the
//! lexer/parser/codegen FRONT-END that would do this wiring automatically; the runtime substrate it
//! would target is what this proves.)
//!
//! The program executed:
//!
//! ```cobol
//!        IDENTIFICATION DIVISION.
//!        PROGRAM-ID. DEMO.
//!        DATA DIVISION.
//!        WORKING-STORAGE SECTION.
//!        01 WS-A   PIC 9(5) VALUE 100.
//!        01 WS-B   PIC 9(5) VALUE 250.
//!        01 WS-RES PIC ZZ,ZZ9.
//!        PROCEDURE DIVISION.
//!            ADD WS-A TO WS-B.
//!            MOVE WS-B TO WS-RES.
//!            DISPLAY "TOTAL=" WS-RES.
//!            STOP RUN.
//! ```

use gnucobol_rs::arith::{cob_arith, cob_divide, Op, Round};
use gnucobol_rs::attr::{FieldAttr, COB_TYPE_NUMERIC_DISPLAY};
use gnucobol_rs::edited::encode_edited;
use gnucobol_rs::termio::{cob_display, DisplaySettings};
use gnucobol_rs::value::Decimal;
use std::io::Write;

/// A `PIC 9(digits)` unsigned `USAGE DISPLAY` field attribute (zoned decimal, integer).
fn pic9(digits: u16) -> FieldAttr {
    FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits, scale: 0, flags: 0 }
}

/// DISPLAY of an alphanumeric literal followed by an edited field, then newline -- the bytes a
/// `DISPLAY "lit" FIELD.` statement writes.
fn display_line(lit: &[u8], field: &[u8]) -> Vec<u8> {
    let lit_attr = FieldAttr { field_type: 0x21 /* alphanumeric */, digits: 0, scale: 0, flags: 0 };
    let mut out = Vec::new();
    cob_display(true, &[(lit, &lit_attr), (field, &lit_attr)], &DisplaySettings::default(), &mut out);
    out
}

/// `prog_add`:  ADD WS-A TO WS-B / MOVE WS-B TO WS-RES (PIC ZZ,ZZ9) / DISPLAY "TOTAL=" WS-RES.
/// WS-A=100, WS-B=250 -> 350 -> "   350".
fn prog_add() -> Vec<u8> {
    let attr5 = pic9(5);
    let b = cob_arith(Op::Add, b"00250", &attr5, b"00100", &attr5, Round::Truncate).expect("ADD");
    let dec = Decimal { negative: false, digits: b.iter().map(|c| c - b'0').collect(), scale: 0 };
    let res = encode_edited("ZZ,ZZ9", &dec).expect("MOVE");
    display_line(b"TOTAL=", &res)
}

/// `prog_mul`:  MULTIPLY WS-P BY WS-Q GIVING WS-R / MOVE WS-R TO WS-RE (PIC ZZ,ZZ9) / DISPLAY.
/// WS-P=12, WS-Q=4 -> 48 -> "    48".
fn prog_mul() -> Vec<u8> {
    let attr5 = pic9(5);
    let attr3 = pic9(3);
    let r = cob_arith(Op::Multiply, b"00012", &attr5, b"004", &attr3, Round::Truncate).expect("MUL");
    let dec = Decimal { negative: false, digits: r.iter().map(|c| c - b'0').collect(), scale: 0 };
    let re = encode_edited("ZZ,ZZ9", &dec).expect("MOVE");
    display_line(b"PRODUCT=", &re)
}

/// `prog_div`:  DIVIDE WS-N BY WS-D GIVING WS-Q / MOVE WS-Q TO WS-QE (PIC Z,ZZ9.99) / DISPLAY.
/// WS-N=1000.00, WS-D=8 -> 125.00 -> "  125.00".
fn prog_div() -> Vec<u8> {
    // WS-N PIC 9(6)V99 = "00100000" (1000.00); WS-D PIC 9 = "8"; GIVING WS-Q PIC 9(4)V99 (scale 2).
    let n_attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 8, scale: 2, flags: 0 };
    let q_attr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: 6, scale: 2, flags: 0 };
    let q = cob_divide(b"00100000", &n_attr, b"8", &pic9(1), &q_attr, Round::Truncate).expect("DIV");
    let dec = Decimal { negative: false, digits: q.iter().map(|c| c - b'0').collect(), scale: 2 };
    let qe = encode_edited("Z,ZZ9.99", &dec).expect("MOVE");
    display_line(b"QUOTIENT=", &qe)
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_else(|| "add".into());
    let out = match which.as_str() {
        "add" => prog_add(),
        "mul" => prog_mul(),
        "div" => prog_div(),
        other => {
            eprintln!("unknown program '{other}' (add|mul|div)");
            std::process::exit(2);
        }
    };
    std::io::stdout().write_all(&out).unwrap();
}
