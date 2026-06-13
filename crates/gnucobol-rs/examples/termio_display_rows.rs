//! Differential evaluator for termio.c `cob_display_common` (the `DISPLAY`-bytes core). One source of
//! truth: with `--cob` it emits the cobc oracle program (typed fields with VALUEs, each DISPLAYed with a
//! label); with no arg it builds the SAME field storage in Rust (via the sealed `cob_move` / float
//! encoders) and prints `label=<cob_display_common bytes>`. The sweep diffs the two streams.

use gnucobol_rs::attr::{
    FieldAttr, COB_FLAG_BINARY_SWAP, COB_FLAG_HAVE_SIGN, COB_TYPE_ALPHANUMERIC,
    COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_DOUBLE,
    COB_TYPE_NUMERIC_FLOAT, COB_TYPE_NUMERIC_PACKED,
};
use gnucobol_rs::termio::{cob_display_common, DisplaySettings};

/// A test field: label, COBOL PIC + USAGE, decimal value string, and the Rust field type.
struct Case {
    label: &'static str,
    pic: &'static str,
    usage: &'static str,
    value: &'static str,
    ftype: u16,
    digits: u16,
    scale: i16,
    signed: bool,
}

const CASES: &[Case] = &[
    // pretty numeric DISPLAY (default pretty-display: yes)
    Case { label: "d_u5", pic: "9(5)", usage: "DISPLAY", value: "42", ftype: COB_TYPE_NUMERIC_DISPLAY, digits: 5, scale: 0, signed: false },
    Case { label: "d_s4p", pic: "S9(4)", usage: "DISPLAY", value: "1234", ftype: COB_TYPE_NUMERIC_DISPLAY, digits: 4, scale: 0, signed: true },
    Case { label: "d_s4n", pic: "S9(4)", usage: "DISPLAY", value: "-1234", ftype: COB_TYPE_NUMERIC_DISPLAY, digits: 4, scale: 0, signed: true },
    Case { label: "d_sv2", pic: "S9(3)V99", usage: "DISPLAY", value: "-12.34", ftype: COB_TYPE_NUMERIC_DISPLAY, digits: 5, scale: 2, signed: true },
    Case { label: "d_uv2", pic: "9(3)V99", usage: "DISPLAY", value: "12.34", ftype: COB_TYPE_NUMERIC_DISPLAY, digits: 5, scale: 2, signed: false },
    Case { label: "d_v3", pic: "S9(2)V999", usage: "DISPLAY", value: "-1.5", ftype: COB_TYPE_NUMERIC_DISPLAY, digits: 5, scale: 3, signed: true },
    // packed COMP-3
    Case { label: "p_sv2n", pic: "S9(3)V99", usage: "COMP-3", value: "-12.34", ftype: COB_TYPE_NUMERIC_PACKED, digits: 5, scale: 2, signed: true },
    Case { label: "p_u4", pic: "9(4)", usage: "COMP-3", value: "789", ftype: COB_TYPE_NUMERIC_PACKED, digits: 4, scale: 0, signed: false },
    // binary COMP (pretty path)
    Case { label: "b_s4n", pic: "S9(4)", usage: "COMP", value: "-1234", ftype: COB_TYPE_NUMERIC_BINARY, digits: 4, scale: 0, signed: true },
    Case { label: "b_uv2", pic: "9(3)V99", usage: "COMP", value: "5.67", ftype: COB_TYPE_NUMERIC_BINARY, digits: 5, scale: 2, signed: false },
    // float COMP-2 / COMP-1 — exactly-representable values (so cobc's literal->double == Rust's parse;
    // the %G form selection / clean_double is still fully exercised: f-form, e-form, signs, decimals).
    Case { label: "f2", pic: "", usage: "COMP-2", value: "12.5", ftype: COB_TYPE_NUMERIC_DOUBLE, digits: 0, scale: 0, signed: true },
    Case { label: "f2b", pic: "", usage: "COMP-2", value: "-0.0625", ftype: COB_TYPE_NUMERIC_DOUBLE, digits: 0, scale: 0, signed: true },
    Case { label: "f2c", pic: "", usage: "COMP-2", value: "0.0009765625", ftype: COB_TYPE_NUMERIC_DOUBLE, digits: 0, scale: 0, signed: true },
    Case { label: "f2e", pic: "", usage: "COMP-2", value: "1.0E16", ftype: COB_TYPE_NUMERIC_DOUBLE, digits: 0, scale: 0, signed: true },
    Case { label: "f1", pic: "", usage: "COMP-1", value: "1.5", ftype: COB_TYPE_NUMERIC_FLOAT, digits: 0, scale: 0, signed: true },
    // alphanumeric
    Case { label: "an", pic: "X(6)", usage: "DISPLAY", value: "", ftype: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, signed: false },
];

/// Zoned DISPLAY image of a signed integer magnitude at `digits` (trailing ASCII overpunch sign).
fn disp(digits: usize, mag: u64, neg: bool) -> Vec<u8> {
    let mut s: Vec<u8> = format!("{mag:0digits$}").into_bytes();
    if neg {
        let l = s.len() - 1;
        s[l] = (s[l] - b'0') + 0x70; // ASCII trailing overpunch for negative (p..y)
    }
    s
}

/// (magnitude as integer at the field scale, negative?) from a decimal string.
fn mag_scale(value: &str, scale: i16) -> (u64, bool) {
    let neg = value.starts_with('-');
    let v = value.trim_start_matches(['-', '+']);
    let (int_part, frac_part) = match v.split_once('.') {
        Some((a, b)) => (a.to_string(), b.to_string()),
        None => (v.to_string(), String::new()),
    };
    let mut frac = frac_part;
    while (frac.len() as i16) < scale {
        frac.push('0');
    }
    frac.truncate(scale.max(0) as usize);
    let combined = format!("{int_part}{frac}");
    (combined.parse::<u64>().unwrap_or(0), neg)
}

fn build_storage(c: &Case) -> (Vec<u8>, FieldAttr) {
    let flags = if c.signed { COB_FLAG_HAVE_SIGN } else { 0 };
    match c.ftype {
        COB_TYPE_NUMERIC_DISPLAY => {
            let (mag, neg) = mag_scale(c.value, c.scale);
            let bytes = disp(c.digits as usize, mag, neg && c.signed);
            (bytes, FieldAttr { field_type: c.ftype, digits: c.digits, scale: c.scale, flags })
        }
        COB_TYPE_NUMERIC_PACKED | COB_TYPE_NUMERIC_BINARY => {
            // build a zoned source, cob_move into the target storage (sealed encoders).
            let (mag, neg) = mag_scale(c.value, c.scale);
            let src = disp(c.digits as usize, mag, neg && c.signed);
            let src_attr =
                FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: c.digits, scale: c.scale, flags };
            let dsize = if c.ftype == COB_TYPE_NUMERIC_PACKED {
                (c.digits as usize) / 2 + 1
            } else {
                bin_size(c.digits)
            };
            let dflags = if c.ftype == COB_TYPE_NUMERIC_BINARY { flags | COB_FLAG_BINARY_SWAP } else { flags };
            let dattr = FieldAttr { field_type: c.ftype, digits: c.digits, scale: c.scale, flags: dflags };
            let mut buf = vec![0u8; dsize];
            let _ = gnucobol_rs::cob_move(&src, &src_attr, &mut buf, &dattr);
            (buf, dattr)
        }
        COB_TYPE_NUMERIC_DOUBLE => {
            let v: f64 = c.value.parse().unwrap();
            (v.to_le_bytes().to_vec(), FieldAttr { field_type: c.ftype, digits: 0, scale: 0, flags: 0 })
        }
        COB_TYPE_NUMERIC_FLOAT => {
            let v: f32 = c.value.parse().unwrap();
            (v.to_le_bytes().to_vec(), FieldAttr { field_type: c.ftype, digits: 0, scale: 0, flags: 0 })
        }
        _ => {
            // alphanumeric: "HELLO " padded to 6
            let mut b = b"HELLO ".to_vec();
            b.truncate(6);
            (b, FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 })
        }
    }
}

/// COMP byte size for a digit count (GnuCOBOL default binary-size table).
fn bin_size(digits: u16) -> usize {
    match digits {
        1..=2 => 1,
        3..=4 => 2,
        5..=6 => 3,
        7..=9 => 4,
        10..=11 => 5,
        12..=14 => 6,
        15..=16 => 7,
        17..=18 => 8,
        _ => 8,
    }
}

fn emit_cob() {
    println!(">>SOURCE FORMAT FREE");
    println!("IDENTIFICATION DIVISION.");
    println!("PROGRAM-ID. TD.");
    println!("DATA DIVISION.");
    println!("WORKING-STORAGE SECTION.");
    for (i, c) in CASES.iter().enumerate() {
        if c.ftype == COB_TYPE_ALPHANUMERIC {
            println!("01 V{i} PIC X(6) VALUE \"HELLO\".");
        } else if c.usage == "COMP-1" || c.usage == "COMP-2" {
            println!("01 V{i} USAGE {} VALUE {}.", c.usage, c.value);
        } else {
            println!("01 V{i} PIC {} USAGE {} VALUE {}.", c.pic, c.usage, c.value);
        }
    }
    println!("PROCEDURE DIVISION.");
    for (i, c) in CASES.iter().enumerate() {
        println!("    DISPLAY \"{}=\" V{i}.", c.label);
    }
    println!("    STOP RUN.");
}

fn main() {
    if std::env::args().any(|a| a == "--cob") {
        emit_cob();
        return;
    }
    let settings = DisplaySettings::default();
    for c in CASES {
        let (bytes, attr) = build_storage(c);
        let mut out = Vec::new();
        cob_display_common(&bytes, &attr, &settings, &mut out);
        println!("{}={}", c.label, String::from_utf8_lossy(&out));
    }
}
