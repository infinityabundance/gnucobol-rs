//! Rust evaluator for the intrinsic.c differential. Runs the SAME fixed battery as `intrinsic_harness.c`
//! (the real exported `cob_intr_*` linked against libcob) through the Rust port, printing
//! `label <result-field hex bytes>` so the two streams diff byte-for-byte.

use gnucobol_rs::attr::{FieldAttr, COB_FLAG_HAVE_SIGN, COB_TYPE_ALPHANUMERIC, COB_TYPE_NUMERIC_DISPLAY};
use gnucobol_rs::intrinsic::*;

fn hexln(label: &str, r: &(Vec<u8>, FieldAttr)) {
    let mut s = String::from(label);
    s.push(' ');
    for b in &r.0 {
        s.push_str(&format!("{b:02x}"));
    }
    println!("{s}");
}

fn disp(digits: u16, scale: i16, signed: bool) -> FieldAttr {
    FieldAttr {
        field_type: COB_TYPE_NUMERIC_DISPLAY,
        digits,
        scale,
        flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
    }
}

fn main() {
    let an = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: 0, scale: 0, flags: 0 };
    let _ = an;

    hexln("ord_A", &cob_intr_ord(b"A"));
    hexln("char_66", &cob_intr_char(b"066", &disp(3, 0, false)));
    hexln("blen_5", &cob_intr_byte_length(5));
    hexln("len_6", &cob_intr_length(6));
    hexln("upper", &cob_intr_upper_case(0, 0, b"aB3xZ"));
    hexln("lower", &cob_intr_lower_case(0, 0, b"aB3xZ"));
    hexln("rev", &cob_intr_reverse(0, 0, b"abcde"));
    hexln("upper_rm", &cob_intr_upper_case(2, 3, b"hello"));
    let s = disp(4, 2, true);
    hexln("sign_neg", &cob_intr_sign(b"123t", &s));
    hexln("sign_pos", &cob_intr_sign(b"1234", &s));
    hexln("abs_neg", &cob_intr_abs(b"123t", &s));
    hexln("integer_neg", &cob_intr_integer(b"123t", &s));
    hexln("integer_pos", &cob_intr_integer(b"1234", &s));
    hexln("intpart_neg", &cob_intr_integer_part(b"123t", &s));
    hexln("intpart_pos", &cob_intr_integer_part(b"1234", &s));
    hexln("iod", &cob_intr_integer_of_date(b"20240229", &disp(8, 0, false)));
    hexln("doi", &cob_intr_date_of_integer(b"00154794", &disp(8, 0, false)));
    hexln("ioday", &cob_intr_integer_of_day(b"2024060", &disp(7, 0, false)));
    hexln("doiy", &cob_intr_day_of_integer(b"00154794", &disp(8, 0, false)));
    hexln("numval", &cob_intr_numval(b"-12.34  "));
    hexln("numvalc", &cob_intr_numval_c(b"$1,234.56"));
    let s3 = disp(3, 0, true);
    hexln("mod_p", &cob_intr_mod(b"017", &s3, b"005", &s3));
    hexln("mod_n", &cob_intr_mod(b"01w", &s3, b"005", &s3));
    hexln("rem_p", &cob_intr_rem(b"017", &s3, b"005", &s3));
    hexln("rem_n", &cob_intr_rem(b"01w", &s3, b"005", &s3));
}
