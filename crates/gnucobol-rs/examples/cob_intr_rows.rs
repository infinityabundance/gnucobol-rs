//! Rust evaluator for the intrinsic.c differential. Runs the SAME fixed battery as `intrinsic_harness.c`
//! (the real exported `cob_intr_*` linked against libcob) through the Rust port, printing
//! `label <result-field hex bytes>` so the two streams diff byte-for-byte.

use gnucobol_rs::attr::{FieldAttr, COB_FLAG_HAVE_SIGN, COB_TYPE_ALPHANUMERIC, COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_DISPLAY};
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
    hexln("tnv_ok", &cob_intr_test_numval(b"-12.34  "));
    hexln("tnv_dd", &cob_intr_test_numval(b"12.3.4"));
    hexln("tnv_x", &cob_intr_test_numval(b"12X4"));
    hexln("tnv_pp", &cob_intr_test_numval(b"++5"));
    hexln("tnv_cr", &cob_intr_test_numval(b"12CR"));
    hexln("tnv_lc", &cob_intr_test_numval(b"12cr"));
    hexln("tnv_sp", &cob_intr_test_numval(b"    "));
    hexln("tnvc_ok", &cob_intr_test_numval_c(b"$1,234.56", None));
    hexln("tnvc_cma", &cob_intr_test_numval_c(b"1,234", None));
    hexln("tnvc_dd", &cob_intr_test_numval_c(b"1.2.3", None));
    hexln("nvf_ok", &cob_intr_test_numval_f(b"1.5E+10"));
    hexln("nvf_e5", &cob_intr_test_numval_f(b"1E5"));
    hexln("nvf_neg", &cob_intr_test_numval_f(b"-12.34"));
    hexln("nvf_dd", &cob_intr_test_numval_f(b"1.2.3"));
    hexln("nvf_ee", &cob_intr_test_numval_f(b"1E+"));
    hexln("iofd_ymd", &cob_intr_integer_of_formatted_date(b"YYYYMMDD", b"20240229"));
    hexln("iofd_ymdh", &cob_intr_integer_of_formatted_date(b"YYYY-MM-DD", b"2024-02-29"));
    hexln("iofd_ddd", &cob_intr_integer_of_formatted_date(b"YYYYDDD", b"2024060"));
    hexln("iofd_www", &cob_intr_integer_of_formatted_date(b"YYYYWwwD", b"2024W092"));
    hexln("iofd_wwwh", &cob_intr_integer_of_formatted_date(b"YYYY-Www-D", b"2024-W09-2"));
    hexln("iofd_dt", &cob_intr_integer_of_formatted_date(b"YYYY-MM-DDThh:mm:ss", b"2024-02-29T12:00:00"));
    hexln("iofd_bad", &cob_intr_integer_of_formatted_date(b"YYYYMMDD", b"20240230"));
    hexln("iofd_badf", &cob_intr_integer_of_formatted_date(b"ZZZZ", b"x"));
    hexln("iofd_base", &cob_intr_integer_of_formatted_date(b"YYYYMMDD", b"16010101"));
    let d7 = disp(7, 0, false);
    hexln("fd_1ymd", &cob_intr_formatted_date(0, 0, b"YYYYMMDD", b"0000001", &d7));
    hexln("fd_1ymdh", &cob_intr_formatted_date(0, 0, b"YYYY-MM-DD", b"0000001", &d7));
    hexln("fd_1ddd", &cob_intr_formatted_date(0, 0, b"YYYYDDD", b"0000001", &d7));
    hexln("fd_1www", &cob_intr_formatted_date(0, 0, b"YYYYWwwD", b"0000001", &d7));
    hexln("fd_1wwwh", &cob_intr_formatted_date(0, 0, b"YYYY-Www-D", b"0000001", &d7));
    hexln("fd_mod", &cob_intr_formatted_date(0, 0, b"YYYY-MM-DD", b"0154789", &d7));
    hexln("fd_modw", &cob_intr_formatted_date(0, 0, b"YYYY-Www-D", b"0154789", &d7));
    hexln("fd_inv", &cob_intr_formatted_date(0, 0, b"YYYYMMDD", b"0000000", &d7));
    hexln("fd_badf", &cob_intr_formatted_date(0, 0, b"BAD", b"0000001", &d7));
    hexln("tfdt_d", &cob_intr_test_formatted_datetime(b"YYYYMMDD", b"20240229"));
    hexln("tfdt_t", &cob_intr_test_formatted_datetime(b"hhmmss", b"120000"));
    hexln("tfdt_tdec", &cob_intr_test_formatted_datetime(b"hh:mm:ss.ss", b"12:00:00.50"));
    hexln("tfdt_dt", &cob_intr_test_formatted_datetime(b"YYYY-MM-DDThh:mm:ss", b"2024-02-29T12:00:00"));
    hexln("tfdt_bh", &cob_intr_test_formatted_datetime(b"hhmmss", b"250000"));
    hexln("tfdt_z", &cob_intr_test_formatted_datetime(b"hhmmssZ", b"120000Z"));
    hexln("tfdt_off", &cob_intr_test_formatted_datetime(b"hh:mm:ss+hh:mm", b"12:00:00+05:30"));
    hexln("tfdt_not", &cob_intr_test_formatted_datetime(b"YYYY-MM-DDThh:mm:ss", b"2024-02-29X12:00:00"));
    hexln("tfdt_bs", &cob_intr_test_formatted_datetime(b"hhmmss", b"120061"));
    hexln("tfdt_bad", &cob_intr_test_formatted_datetime(b"GARBAGE", b"x"));
    // Order mirrors the harness: each fractional case follows a zero-decimal one so libcob's shared
    // scratch decimal (cob_d1) is clean — see seconds_from_formatted_time's characterized-divergence note.
    hexln("sfft_noon", &cob_intr_seconds_from_formatted_time(b"hhmmss", b"120000"));
    hexln("sfft_dec", &cob_intr_seconds_from_formatted_time(b"hh:mm:ss.ss", b"12:00:00.50"));
    hexln("sfft_123", &cob_intr_seconds_from_formatted_time(b"hh:mm:ss", b"01:02:03"));
    hexln("sfft_eod", &cob_intr_seconds_from_formatted_time(b"hhmmss.sss", b"235959.125"));
    hexln("sfft_dt", &cob_intr_seconds_from_formatted_time(b"YYYY-MM-DDThh:mm:ss", b"2024-02-29T06:30:00"));
    hexln("sfft_bad", &cob_intr_seconds_from_formatted_time(b"hhmmss", b"250000"));
    // FORMATTED-TIME / FORMATTED-DATETIME: explicit-offset path (use_system_offset = false).
    let d7n = disp(7, 0, false);
    let o4 = disp(4, 0, false);
    let binneg = FieldAttr { field_type: COB_TYPE_NUMERIC_BINARY, digits: 9, scale: 0, flags: COB_FLAG_HAVE_SIGN };
    hexln("ft_plain", &cob_intr_formatted_time(0, 0, b"hhmmss", b"0043200", &d7n, None, false));
    hexln("ft_colon", &cob_intr_formatted_time(0, 0, b"hh:mm:ss", b"0043200", &d7n, None, false));
    hexln("ft_frac", &cob_intr_formatted_time(0, 0, b"hh:mm:ss.ss", b"4320050", &disp(7, 2, false), None, false));
    hexln("ft_z", &cob_intr_formatted_time(0, 0, b"hhmmssZ", b"0043200", &d7n, Some((b"0330", &o4)), false));
    hexln("ft_off", &cob_intr_formatted_time(0, 0, b"hh:mm:ss+hh:mm", b"0043200", &d7n, Some((b"0330", &o4)), false));
    hexln("ft_offneg", &cob_intr_formatted_time(0, 0, b"hh:mm:ss+hh:mm", b"0043200", &d7n, Some((b"\x88\xff\xff\xff", &binneg)), false));
    hexln("ft_inv", &cob_intr_formatted_time(0, 0, b"hhmmss", b"0090000", &d7n, None, false));
    hexln("fdt_plain", &cob_intr_formatted_datetime(0, 0, b"YYYY-MM-DDThh:mm:ss", b"0000001", &d7n, b"0043200", &d7n, None, false));
    hexln("fdt_z", &cob_intr_formatted_datetime(0, 0, b"YYYY-MM-DDThh:mm:ssZ", b"0000001", &d7n, b"0043200", &d7n, Some((b"0000", &o4)), false));
    hexln("fdt_ovf", &cob_intr_formatted_datetime(0, 0, b"YYYY-MM-DDThh:mm:ssZ", b"0000001", &d7n, b"0082800", &d7n, Some((b"\x88\xff\xff\xff", &binneg)), false));
    hexln("fdt_off", &cob_intr_formatted_datetime(0, 0, b"YYYY-MM-DDThh:mm:ss+hh:mm", b"0000001", &d7n, b"0043200", &d7n, Some((b"0330", &o4)), false));
    hexln("fdt_inv", &cob_intr_formatted_datetime(0, 0, b"BADFORMAT", b"0000001", &d7n, b"0043200", &d7n, None, false));
    hexln("nvf2_sci", &cob_intr_numval_f(b"1.5E+10"));
    hexln("nvf2_neg", &cob_intr_numval_f(b"-12.34"));
    hexln("nvf2_em3", &cob_intr_numval_f(b"1E-3"));
    hexln("nvf2_lead", &cob_intr_numval_f(b"000123.450"));
    hexln("nvf2_e2", &cob_intr_numval_f(b"-7.5e2"));
    hexln("nvf2_zero", &cob_intr_numval_f(b"0"));
    hexln("sqrt2", &cob_intr_sqrt(b"2", &disp(1, 0, false)));
    hexln("sqrt16", &cob_intr_sqrt(b"16", &disp(2, 0, false)));
    hexln("sqrt225", &cob_intr_sqrt(b"225", &disp(3, 2, false)));
    hexln("sqrt0", &cob_intr_sqrt(b"0", &disp(1, 0, false)));
    hexln("exp1", &cob_intr_exp(b"1", &disp(1, 0, false)));
    hexln("exp0", &cob_intr_exp(b"0", &disp(1, 0, false)));
    hexln("exp2", &cob_intr_exp(b"2", &disp(1, 0, false)));
    hexln("expn1", &cob_intr_exp(b"q", &disp(1, 0, true)));
    hexln("logv10", &cob_intr_log(b"10", &disp(2, 0, false)));
    hexln("log2", &cob_intr_log(b"2", &disp(1, 0, false)));
    hexln("log1", &cob_intr_log(b"1", &disp(1, 0, false)));
    hexln("l10_1k", &cob_intr_log10(b"1000", &disp(4, 0, false)));
    hexln("l10_100", &cob_intr_log10(b"100", &disp(3, 0, false)));
    hexln("l10_2", &cob_intr_log10(b"2", &disp(1, 0, false)));
    hexln("e10_2", &cob_intr_exp10(b"2", &disp(1, 0, false)));
    hexln("e10n1", &cob_intr_exp10(b"q", &disp(1, 0, true)));
    hexln("e10_3", &cob_intr_exp10(b"3", &disp(1, 0, false)));
    hexln("e10_h", &cob_intr_exp10(b"05", &disp(2, 1, false)));
    hexln("sin1", &cob_intr_sin(b"1", &disp(1, 0, false)));
    hexln("sin0", &cob_intr_sin(b"0", &disp(1, 0, false)));
    hexln("sin2", &cob_intr_sin(b"2", &disp(1, 0, false)));
    hexln("sin10", &cob_intr_sin(b"10", &disp(2, 0, false)));
    hexln("sinn1", &cob_intr_sin(b"q", &disp(1, 0, true)));
    hexln("cos1", &cob_intr_cos(b"1", &disp(1, 0, false)));
    hexln("cos0", &cob_intr_cos(b"0", &disp(1, 0, false)));
    hexln("cos2", &cob_intr_cos(b"2", &disp(1, 0, false)));
    hexln("tan1", &cob_intr_tan(b"1", &disp(1, 0, false)));
    hexln("tan0", &cob_intr_tan(b"0", &disp(1, 0, false)));
    hexln("pi", &cob_intr_pi());
    hexln("ee", &cob_intr_e());
    hexln("atan1", &cob_intr_atan(b"1", &disp(1, 0, false)));
    hexln("atan2", &cob_intr_atan(b"2", &disp(1, 0, false)));
    hexln("atan3", &cob_intr_atan(b"3", &disp(1, 0, false)));
    hexln("atan0", &cob_intr_atan(b"0", &disp(1, 0, false)));
    hexln("atann1", &cob_intr_atan(b"q", &disp(1, 0, true)));
    hexln("asin0", &cob_intr_asin(b"0", &disp(1, 0, false)));
    hexln("asin1", &cob_intr_asin(b"1", &disp(1, 0, false)));
    hexln("asinn1", &cob_intr_asin(b"q", &disp(1, 0, true)));
    hexln("asinh", &cob_intr_asin(b"5", &disp(1, 1, false)));
    hexln("asin_oor", &cob_intr_asin(b"2", &disp(1, 0, false)));
    hexln("acos0", &cob_intr_acos(b"0", &disp(1, 0, false)));
    hexln("acos1", &cob_intr_acos(b"1", &disp(1, 0, false)));
    hexln("acosn1", &cob_intr_acos(b"q", &disp(1, 0, true)));
    hexln("acosh", &cob_intr_acos(b"5", &disp(1, 1, false)));
    hexln("acos_oor", &cob_intr_acos(b"2", &disp(1, 0, false)));
    hexln("bo_add", &cob_intr_binop(b"010", &u3, b'+', b"020", &u3));
    hexln("bo_sub", &cob_intr_binop(b"030", &u3, b'-', b"012", &u3));
    hexln("bo_mul", &cob_intr_binop(b"006", &u3, b'*', b"007", &u3));
    hexln("bo_div", &cob_intr_binop(b"020", &u3, b'/', b"004", &u3));
    hexln("bo_pow", &cob_intr_binop(b"002", &u3, b'^', b"010", &u3));
    hexln("bo_powh", &cob_intr_binop(b"009", &u3, b'^', b"5", &disp(1, 1, false)));
    hexln("bo_and", &cob_intr_binop(b"012", &u3, b'a', b"010", &u3));
    hexln("bo_or", &cob_intr_binop(b"012", &u3, b'o', b"010", &u3));
    hexln("bo_xor", &cob_intr_binop(b"012", &u3, b'e', b"010", &u3));
    hexln("bo_shl", &cob_intr_binop(b"003", &u3, b'l', b"002", &u3));
    hexln("bo_not", &cob_intr_binop(b"005", &u3, b'n', b"005", &u3));
    hexln("annu", &cob_intr_annuity(b"005", &disp(3, 2, false), b"10", &disp(2, 0, false)));
    hexln("annu0", &cob_intr_annuity(b"000", &u3, b"05", &disp(2, 0, false)));
    hexln("pv", &cob_intr_present_value(b"010", &disp(3, 2, false), &[(b"100", &u3), (b"200", &u3)]));
    hexln("var", &cob_intr_variance(&[(b"002", &u3), (b"004", &u3), (b"006", &u3)]));
    hexln("sdev", &cob_intr_standard_deviation(&[(b"002", &u3), (b"004", &u3), (b"006", &u3)]));
    hexln("whenc", &cob_intr_when_compiled(0, 0, b"2024010112000000-0500", &an));
    hexln("whenc_rm", &cob_intr_when_compiled(3, 4, b"2024010112000000-0500", &an));
    hexln("cdate", &cob_intr_current_date(0, 0));
    hexln("fcd", &cob_intr_formatted_current_date(0, 0, b"YYYY-MM-DDThh:mm:ss"));
    hexln("fcd2", &cob_intr_formatted_current_date(0, 0, b"YYYYMMDDThhmmss"));
    hexln("mid", &cob_intr_module_id(b"MYMOD"));
    hexln("msrc", &cob_intr_module_source(b"mymod.cob"));
    hexln("mfd", &cob_intr_module_formatted_date(b"2024/02/29 12:34:56"));
    hexln("mdate", &cob_intr_module_date(20240229));
    hexln("mtime", &cob_intr_module_time(123456));
    hexln("mcaller", &cob_intr_module_caller_id(None));
    hexln("mpath", &cob_intr_module_path(None));
    hexln("exstat", &cob_intr_exception_status(None));
    hexln("exstmt", &cob_intr_exception_statement(None));
    hexln("exloc", &cob_intr_exception_location(None));
    hexln("exfile", &cob_intr_exception_file(None));
}
