/* Oracle harness for intrinsic.c cob_intr_*. Links the built libcob, cob_init()s the runtime, then calls
 * each exported intrinsic on a fixed battery of input fields and prints `label <result-field hex bytes>`.
 * The Rust evaluator (cob_intr_rows) runs the same battery and prints byte-identical lines.
 *
 * Build: gcc -O2 -I$PREFIX/include intrinsic_harness.c -o intrinsic_harness -L$PREFIX/lib -lcob
 */
#include <stdio.h>
#include <string.h>
#include <libcob.h>

static void dump(const char *label, cob_field *r) {
	printf("%s ", label);
	if (r && r->data) {
		for (size_t i = 0; i < r->size; i++) printf("%02x", r->data[i]);
	}
	printf("\n");
}

static cob_field mkf(const char *data, int size, int type, int digits, int scale, int flags) {
	static cob_field_attr attrs[64];
	static int na = 0;
	cob_field_attr *a = &attrs[na++ % 64];
	a->type = type; a->digits = digits; a->scale = scale; a->flags = flags; a->pic = NULL;
	cob_field f;
	f.size = size; f.data = (unsigned char *)data; f.attr = a;
	return f;
}

#define ALNUM 0x21
#define DISP  0x10
#define HAVE_SIGN 0x01

int main(int argc, char **argv) {
	cob_init(argc, argv);

	/* The decimal intrinsics read COB_MODULE_PTR->decimal_point; generated code sets the current module
	 * in its prologue. Set a minimal one (matching the pinned LC_ALL=C / default config). */
	cob_global *g = cob_get_global_ptr();
	static cob_module mod;
	memset(&mod, 0, sizeof(mod));
	mod.decimal_point = '.';
	mod.currency_symbol = '$';
	mod.numeric_separator = ',';
	g->cob_current_module = &mod;

	{ cob_field f = mkf("A", 1, ALNUM, 0, 0, 0); dump("ord_A", cob_intr_ord(&f)); }
	{ cob_field f = mkf("066", 3, DISP, 3, 0, 0); dump("char_66", cob_intr_char(&f)); }
	{ cob_field f = mkf("HELLO", 5, ALNUM, 0, 0, 0); dump("blen_5", cob_intr_byte_length(&f)); }
	{ cob_field f = mkf("WORLD!", 6, ALNUM, 0, 0, 0); dump("len_6", cob_intr_length(&f)); }
	{ cob_field f = mkf("aB3xZ", 5, ALNUM, 0, 0, 0); dump("upper", cob_intr_upper_case(0, 0, &f)); }
	{ cob_field f = mkf("aB3xZ", 5, ALNUM, 0, 0, 0); dump("lower", cob_intr_lower_case(0, 0, &f)); }
	{ cob_field f = mkf("abcde", 5, ALNUM, 0, 0, 0); dump("rev", cob_intr_reverse(0, 0, &f)); }
	{ cob_field f = mkf("hello", 5, ALNUM, 0, 0, 0); dump("upper_rm", cob_intr_upper_case(2, 3, &f)); }
	/* signed S9(2)V99: -12.34 = "123t" (trailing negative overpunch '4'->'t'), +12.34 = "1234" */
	{ cob_field f = mkf("123t", 4, DISP, 4, 2, HAVE_SIGN); dump("sign_neg", cob_intr_sign(&f)); }
	{ cob_field f = mkf("1234", 4, DISP, 4, 2, HAVE_SIGN); dump("sign_pos", cob_intr_sign(&f)); }
	{ cob_field f = mkf("123t", 4, DISP, 4, 2, HAVE_SIGN); dump("abs_neg", cob_intr_abs(&f)); }
	{ cob_field f = mkf("123t", 4, DISP, 4, 2, HAVE_SIGN); dump("integer_neg", cob_intr_integer(&f)); }
	{ cob_field f = mkf("1234", 4, DISP, 4, 2, HAVE_SIGN); dump("integer_pos", cob_intr_integer(&f)); }
	{ cob_field f = mkf("123t", 4, DISP, 4, 2, HAVE_SIGN); dump("intpart_neg", cob_intr_integer_part(&f)); }
	{ cob_field f = mkf("1234", 4, DISP, 4, 2, HAVE_SIGN); dump("intpart_pos", cob_intr_integer_part(&f)); }
	{ cob_field f = mkf("20240229", 8, DISP, 8, 0, 0); dump("iod", cob_intr_integer_of_date(&f)); }
	{ cob_field f = mkf("00154794", 8, DISP, 8, 0, 0); dump("doi", cob_intr_date_of_integer(&f)); }
	{ cob_field f = mkf("2024060", 7, DISP, 7, 0, 0); dump("ioday", cob_intr_integer_of_day(&f)); }
	{ cob_field f = mkf("00154794", 8, DISP, 8, 0, 0); dump("doiy", cob_intr_day_of_integer(&f)); }
	{ cob_field f = mkf("-12.34  ", 8, ALNUM, 0, 0, 0); dump("numval", cob_intr_numval(&f)); }
	{ cob_field f = mkf("$1,234.56", 9, ALNUM, 0, 0, 0); dump("numvalc", cob_intr_numval_c(&f, NULL)); }
	/* MOD/REM: signed S9(3); 17="017", -17="01w" (neg overpunch '7'->'w'=0x77), 5="005" */
	{ cob_field a = mkf("017", 3, DISP, 3, 0, HAVE_SIGN), b = mkf("005", 3, DISP, 3, 0, HAVE_SIGN); dump("mod_p", cob_intr_mod(&a, &b)); }
	{ cob_field a = mkf("01w", 3, DISP, 3, 0, HAVE_SIGN), b = mkf("005", 3, DISP, 3, 0, HAVE_SIGN); dump("mod_n", cob_intr_mod(&a, &b)); }
	{ cob_field a = mkf("017", 3, DISP, 3, 0, HAVE_SIGN), b = mkf("005", 3, DISP, 3, 0, HAVE_SIGN); dump("rem_p", cob_intr_rem(&a, &b)); }
	{ cob_field a = mkf("01w", 3, DISP, 3, 0, HAVE_SIGN), b = mkf("005", 3, DISP, 3, 0, HAVE_SIGN); dump("rem_n", cob_intr_rem(&a, &b)); }
	{ cob_field a=mkf("AB",2,ALNUM,0,0,0),b=mkf("CD",2,ALNUM,0,0,0),c=mkf("EF",2,ALNUM,0,0,0); dump("concat", cob_intr_concatenate(0,0,3,&a,&b,&c)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("020",3,DISP,3,0,0),c=mkf("030",3,DISP,3,0,0); dump("sum", cob_intr_sum(3,&a,&b,&c)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("030",3,DISP,3,0,0),c=mkf("020",3,DISP,3,0,0); dump("max", cob_intr_max(3,&a,&b,&c)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("030",3,DISP,3,0,0),c=mkf("020",3,DISP,3,0,0); dump("min", cob_intr_min(3,&a,&b,&c)); }
	{ cob_field f=mkf("005",3,DISP,3,0,0); dump("fact5", cob_intr_factorial(&f)); }
	{ cob_field f=mkf("HI   ",5,ALNUM,0,0,0); dump("scl", cob_intr_stored_char_length(&f)); }
	dump("ndp", cob_intr_num_decimal_point());
	dump("nts", cob_intr_num_thousands_sep());
	dump("mdp", cob_intr_mon_decimal_point());
	dump("mts", cob_intr_mon_thousands_sep());
	dump("cur", cob_intr_currency_symbol());
	{ cob_field y=mkf("070",3,DISP,3,0,0),iv=mkf("020",3,DISP,3,0,0),cy=mkf("2024",4,DISP,4,0,0); dump("y2y", cob_intr_year_to_yyyy(3,&y,&iv,&cy)); }
	{ cob_field y=mkf("700101",6,DISP,6,0,0),iv=mkf("020",3,DISP,3,0,0),cy=mkf("2024",4,DISP,4,0,0); dump("d2y", cob_intr_date_to_yyyymmdd(3,&y,&iv,&cy)); }
	{ cob_field y=mkf("70001",5,DISP,5,0,0),iv=mkf("020",3,DISP,3,0,0),cy=mkf("2024",4,DISP,4,0,0); dump("dy2y", cob_intr_day_to_yyyyddd(3,&y,&iv,&cy)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("030",3,DISP,3,0,0),c=mkf("020",3,DISP,3,0,0); dump("ordmin", cob_intr_ord_min(3,&a,&b,&c)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("030",3,DISP,3,0,0),c=mkf("020",3,DISP,3,0,0); dump("ordmax", cob_intr_ord_max(3,&a,&b,&c)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("030",3,DISP,3,0,0),c=mkf("020",3,DISP,3,0,0); dump("range", cob_intr_range(3,&a,&b,&c)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("030",3,DISP,3,0,0),c=mkf("020",3,DISP,3,0,0); dump("midr", cob_intr_midrange(3,&a,&b,&c)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("030",3,DISP,3,0,0),c=mkf("020",3,DISP,3,0,0); dump("mean", cob_intr_mean(3,&a,&b,&c)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("030",3,DISP,3,0,0),c=mkf("020",3,DISP,3,0,0); dump("median", cob_intr_median(3,&a,&b,&c)); }
	{ cob_field a=mkf("010",3,DISP,3,0,0),b=mkf("020",3,DISP,3,0,0),c=mkf("030",3,DISP,3,0,0),d=mkf("040",3,DISP,3,0,0); dump("median4", cob_intr_median(4,&a,&b,&c,&d)); }
	{ cob_field f=mkf("\x00\x7f" "Gz",4,ALNUM,0,0,0); dump("hexof", cob_intr_hex_of(&f)); }
	{ cob_field f=mkf("007F417A",8,ALNUM,0,0,0); dump("hex2c", cob_intr_hex_to_char(&f)); }
	{ cob_field f=mkf("\x00\xa5",2,ALNUM,0,0,0); dump("bitof", cob_intr_bit_of(&f)); }
	{ cob_field f=mkf("0000000010100101",16,ALNUM,0,0,0); dump("bit2c", cob_intr_bit_to_char(&f)); }
	{ cob_field f=mkf("000000",6,DISP,6,2,HAVE_SIGN); dump("loalg_ds", cob_intr_lowest_algebraic(&f)); }
	{ cob_field f=mkf("000000",6,DISP,6,2,HAVE_SIGN); dump("hialg_ds", cob_intr_highest_algebraic(&f)); }
	{ cob_field f=mkf("000000",6,DISP,6,2,0); dump("loalg_du", cob_intr_lowest_algebraic(&f)); }
	{ cob_field f=mkf("ABCD",4,ALNUM,0,0,0); dump("hialg_a", cob_intr_highest_algebraic(&f)); }
	{ cob_field d=mkf("00154794",8,DISP,8,0,0),t=mkf("43200",5,DISP,5,0,0); dump("cdt", cob_intr_combined_datetime(&d,&t)); }
	{ cob_field f=mkf("1234",4,DISP,4,2,0); dump("frac", cob_intr_fraction_part(&f)); }
	{ cob_field f=mkf("0123",4,DISP,4,0,0); dump("frac0", cob_intr_fraction_part(&f)); }
	{ cob_field f=mkf("20240229",8,DISP,8,0,0); dump("tdate_ok", cob_intr_test_date_yyyymmdd(&f)); }
	{ cob_field f=mkf("20230229",8,DISP,8,0,0); dump("tdate_bad", cob_intr_test_date_yyyymmdd(&f)); }
	{ cob_field f=mkf("20241301",8,DISP,8,0,0); dump("tdate_mon", cob_intr_test_date_yyyymmdd(&f)); }
	{ cob_field f=mkf("2024060",7,DISP,7,0,0); dump("tday_ok", cob_intr_test_day_yyyyddd(&f)); }
	{ cob_field f=mkf("2023366",7,DISP,7,0,0); dump("tday_bad", cob_intr_test_day_yyyyddd(&f)); }
	{ cob_field f=mkf("  HELLO  ",9,ALNUM,0,0,0); dump("trim_b", cob_intr_trim(0,0,&f,0)); }
	{ cob_field f=mkf("  HELLO  ",9,ALNUM,0,0,0); dump("trim_l", cob_intr_trim(0,0,&f,1)); }
	{ cob_field f=mkf("  HELLO  ",9,ALNUM,0,0,0); dump("trim_t", cob_intr_trim(0,0,&f,2)); }
	{ cob_field o=mkf("MISSISSIPPI",11,ALNUM,0,0,0),m1=mkf("SS",2,ALNUM,0,0,0),r1=mkf("X",1,ALNUM,0,0,0),m2=mkf("PP",2,ALNUM,0,0,0),r2=mkf("Y",1,ALNUM,0,0,0); dump("subst", cob_intr_substitute(0,0,5,&o,&m1,&r1,&m2,&r2)); }
	{ cob_field o=mkf("Hello",5,ALNUM,0,0,0),m1=mkf("L",1,ALNUM,0,0,0),r1=mkf("_",1,ALNUM,0,0,0); dump("subst_c", cob_intr_substitute_case(0,0,3,&o,&m1,&r1)); }
	{ cob_field f=mkf("-12.34  ",8,ALNUM,0,0,0); dump("tnv_ok", cob_intr_test_numval(&f)); }
	{ cob_field f=mkf("12.3.4",6,ALNUM,0,0,0); dump("tnv_dd", cob_intr_test_numval(&f)); }
	{ cob_field f=mkf("12X4",4,ALNUM,0,0,0); dump("tnv_x", cob_intr_test_numval(&f)); }
	{ cob_field f=mkf("++5",3,ALNUM,0,0,0); dump("tnv_pp", cob_intr_test_numval(&f)); }
	{ cob_field f=mkf("12CR",4,ALNUM,0,0,0); dump("tnv_cr", cob_intr_test_numval(&f)); }
	{ cob_field f=mkf("12cr",4,ALNUM,0,0,0); dump("tnv_lc", cob_intr_test_numval(&f)); }
	{ cob_field f=mkf("    ",4,ALNUM,0,0,0); dump("tnv_sp", cob_intr_test_numval(&f)); }
	{ cob_field f=mkf("$1,234.56",9,ALNUM,0,0,0); dump("tnvc_ok", cob_intr_test_numval_c(&f,NULL)); }
	{ cob_field f=mkf("1,234",5,ALNUM,0,0,0); dump("tnvc_cma", cob_intr_test_numval_c(&f,NULL)); }
	{ cob_field f=mkf("1.2.3",5,ALNUM,0,0,0); dump("tnvc_dd", cob_intr_test_numval_c(&f,NULL)); }
	{ cob_field f=mkf("1.5E+10",7,ALNUM,0,0,0); dump("nvf_ok", cob_intr_test_numval_f(&f)); }
	{ cob_field f=mkf("1E5",3,ALNUM,0,0,0); dump("nvf_e5", cob_intr_test_numval_f(&f)); }
	{ cob_field f=mkf("-12.34",6,ALNUM,0,0,0); dump("nvf_neg", cob_intr_test_numval_f(&f)); }
	{ cob_field f=mkf("1.2.3",5,ALNUM,0,0,0); dump("nvf_dd", cob_intr_test_numval_f(&f)); }
	{ cob_field f=mkf("1E+",3,ALNUM,0,0,0); dump("nvf_ee", cob_intr_test_numval_f(&f)); }
	{ cob_field f=mkf("YYYYMMDD",8,ALNUM,0,0,0),d=mkf("20240229",8,ALNUM,0,0,0); dump("iofd_ymd", cob_intr_integer_of_formatted_date(&f,&d)); }
	{ cob_field f=mkf("YYYY-MM-DD",10,ALNUM,0,0,0),d=mkf("2024-02-29",10,ALNUM,0,0,0); dump("iofd_ymdh", cob_intr_integer_of_formatted_date(&f,&d)); }
	{ cob_field f=mkf("YYYYDDD",7,ALNUM,0,0,0),d=mkf("2024060",7,ALNUM,0,0,0); dump("iofd_ddd", cob_intr_integer_of_formatted_date(&f,&d)); }
	{ cob_field f=mkf("YYYYWwwD",8,ALNUM,0,0,0),d=mkf("2024W092",8,ALNUM,0,0,0); dump("iofd_www", cob_intr_integer_of_formatted_date(&f,&d)); }
	{ cob_field f=mkf("YYYY-Www-D",10,ALNUM,0,0,0),d=mkf("2024-W09-2",10,ALNUM,0,0,0); dump("iofd_wwwh", cob_intr_integer_of_formatted_date(&f,&d)); }
	{ cob_field f=mkf("YYYY-MM-DDThh:mm:ss",19,ALNUM,0,0,0),d=mkf("2024-02-29T12:00:00",19,ALNUM,0,0,0); dump("iofd_dt", cob_intr_integer_of_formatted_date(&f,&d)); }
	{ cob_field f=mkf("YYYYMMDD",8,ALNUM,0,0,0),d=mkf("20240230",8,ALNUM,0,0,0); dump("iofd_bad", cob_intr_integer_of_formatted_date(&f,&d)); }
	{ cob_field f=mkf("ZZZZ",4,ALNUM,0,0,0),d=mkf("x",1,ALNUM,0,0,0); dump("iofd_badf", cob_intr_integer_of_formatted_date(&f,&d)); }
	{ cob_field f=mkf("YYYYMMDD",8,ALNUM,0,0,0),d=mkf("16010101",8,ALNUM,0,0,0); dump("iofd_base", cob_intr_integer_of_formatted_date(&f,&d)); }
	{ cob_field f=mkf("YYYYMMDD",8,ALNUM,0,0,0),d=mkf("0000001",7,DISP,7,0,0); dump("fd_1ymd", cob_intr_formatted_date(0,0,&f,&d)); }
	{ cob_field f=mkf("YYYY-MM-DD",10,ALNUM,0,0,0),d=mkf("0000001",7,DISP,7,0,0); dump("fd_1ymdh", cob_intr_formatted_date(0,0,&f,&d)); }
	{ cob_field f=mkf("YYYYDDD",7,ALNUM,0,0,0),d=mkf("0000001",7,DISP,7,0,0); dump("fd_1ddd", cob_intr_formatted_date(0,0,&f,&d)); }
	{ cob_field f=mkf("YYYYWwwD",8,ALNUM,0,0,0),d=mkf("0000001",7,DISP,7,0,0); dump("fd_1www", cob_intr_formatted_date(0,0,&f,&d)); }
	{ cob_field f=mkf("YYYY-Www-D",10,ALNUM,0,0,0),d=mkf("0000001",7,DISP,7,0,0); dump("fd_1wwwh", cob_intr_formatted_date(0,0,&f,&d)); }
	{ cob_field f=mkf("YYYY-MM-DD",10,ALNUM,0,0,0),d=mkf("0154789",7,DISP,7,0,0); dump("fd_mod", cob_intr_formatted_date(0,0,&f,&d)); }
	{ cob_field f=mkf("YYYY-Www-D",10,ALNUM,0,0,0),d=mkf("0154789",7,DISP,7,0,0); dump("fd_modw", cob_intr_formatted_date(0,0,&f,&d)); }
	{ cob_field f=mkf("YYYYMMDD",8,ALNUM,0,0,0),d=mkf("0000000",7,DISP,7,0,0); dump("fd_inv", cob_intr_formatted_date(0,0,&f,&d)); }
	{ cob_field f=mkf("BAD",3,ALNUM,0,0,0),d=mkf("0000001",7,DISP,7,0,0); dump("fd_badf", cob_intr_formatted_date(0,0,&f,&d)); }
	return 0;
}
