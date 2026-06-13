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
    hexln("concat", &cob_intr_concatenate(0, 0, &[b"AB", b"CD", b"EF"]));
    let u3 = disp(3, 0, false);
    hexln("sum", &cob_intr_sum(&[(b"010", &u3), (b"020", &u3), (b"030", &u3)]));
    hexln("max", &cob_intr_max(&[(b"010", &u3), (b"030", &u3), (b"020", &u3)]));
    hexln("min", &cob_intr_min(&[(b"010", &u3), (b"030", &u3), (b"020", &u3)]));
    hexln("fact5", &cob_intr_factorial(b"005", &u3));
    hexln("scl", &cob_intr_stored_char_length(b"HI   "));
    hexln("ndp", &cob_intr_num_decimal_point());
    hexln("nts", &cob_intr_num_thousands_sep());
    hexln("mdp", &cob_intr_mon_decimal_point());
    hexln("mts", &cob_intr_mon_thousands_sep());
    hexln("cur", &cob_intr_currency_symbol());
    hexln("y2y", &cob_intr_year_to_yyyy(70, 20, 2024));
    hexln("d2y", &cob_intr_date_to_yyyymmdd(700101, 20, 2024));
    hexln("dy2y", &cob_intr_day_to_yyyyddd(70001, 20, 2024));
    let abc: &[(&[u8], &FieldAttr)] = &[(b"010", &u3), (b"030", &u3), (b"020", &u3)];
    hexln("ordmin", &cob_intr_ord_min(abc));
    hexln("ordmax", &cob_intr_ord_max(abc));
    hexln("range", &cob_intr_range(abc));
    hexln("midr", &cob_intr_midrange(abc));
    hexln("mean", &cob_intr_mean(abc));
    hexln("median", &cob_intr_median(abc));
    hexln("median4", &cob_intr_median(&[(b"010", &u3), (b"020", &u3), (b"030", &u3), (b"040", &u3)]));
    hexln("hexof", &cob_intr_hex_of(b"\x00\x7fGz"));
    hexln("hex2c", &cob_intr_hex_to_char(b"007F417A"));
    hexln("bitof", &cob_intr_bit_of(b"\x00\xa5"));
    hexln("bit2c", &cob_intr_bit_to_char(b"0000000010100101"));
    let ds = disp(6, 2, true);
    let du = disp(6, 2, false);
    hexln("loalg_ds", &cob_intr_lowest_algebraic(6, &ds));
    hexln("hialg_ds", &cob_intr_highest_algebraic(6, &ds));
    hexln("loalg_du", &cob_intr_lowest_algebraic(6, &du));
    hexln("hialg_a", &cob_intr_highest_algebraic(4, &an));
    hexln("cdt", &cob_intr_combined_datetime(b"00154794", &disp(8, 0, false), b"43200", &disp(5, 0, false)));
    hexln("frac", &cob_intr_fraction_part(b"1234", &disp(4, 2, false)));
    hexln("frac0", &cob_intr_fraction_part(b"0123", &disp(4, 0, false)));
    hexln("tdate_ok", &cob_intr_test_date_yyyymmdd(b"20240229", &disp(8, 0, false)));
    hexln("tdate_bad", &cob_intr_test_date_yyyymmdd(b"20230229", &disp(8, 0, false)));
    hexln("tdate_mon", &cob_intr_test_date_yyyymmdd(b"20241301", &disp(8, 0, false)));
    hexln("tday_ok", &cob_intr_test_day_yyyyddd(b"2024060", &disp(7, 0, false)));
    hexln("tday_bad", &cob_intr_test_day_yyyyddd(b"2023366", &disp(7, 0, false)));
    hexln("trim_b", &cob_intr_trim(0, 0, b"  HELLO  ", &an, 0));
    hexln("trim_l", &cob_intr_trim(0, 0, b"  HELLO  ", &an, 1));
    hexln("trim_t", &cob_intr_trim(0, 0, b"  HELLO  ", &an, 2));
    let subst_pairs: &[(&[u8], &[u8])] = &[(b"SS", b"X"), (b"PP", b"Y")];
    hexln("subst", &cob_intr_substitute(0, 0, b"MISSISSIPPI", subst_pairs));
    let subst_c_pairs: &[(&[u8], &[u8])] = &[(b"L", b"_")];
    hexln("subst_c", &cob_intr_substitute_case(0, 0, b"Hello", subst_c_pairs));
}
