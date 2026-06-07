/*
 * decimal_harness.c — runtime-library oracle for gnucobol-rs.
 *
 * Links the *built* upstream libcob and exercises the REAL cob_move() and the
 * packed/zoned/display field helpers, dumping raw destination bytes. This is the
 * decoupled "runtime library" oracle shape: it bypasses the cobc front-end so the
 * MOVE byte-transform is tested directly. The Rust port (gnucobol-rs) consumes
 * the identical input rows via examples/rows and must produce byte-identical output.
 *
 * NOT shipped in any published crate; lab/ tooling only.
 *
 * Input  (stdin, one case per line, space-separated):
 *   label s_type s_digits s_scale s_flags s_size s_hex  d_type d_digits d_scale d_flags d_size
 * Output (stdout, one line per case):
 *   label d_hex            (destination field bytes after cob_move, lower-hex)
 *
 * types : 16=DISPLAY(0x10) 18=PACKED(0x12)   flags: 1 HAVE_SIGN | 2 SIGN_SEPARATE
 *         | 4 SIGN_LEADING | 256 NO_SIGN_NIBBLE   (decimal, matching common.h)
 */
#include <libcob.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <limits.h>
#include <locale.h>
#include <stddef.h>

static cob_module  module_storage;
static cob_global *cobglob;

static int hexval(int c) {
	if (c >= '0' && c <= '9') return c - '0';
	if (c >= 'a' && c <= 'f') return c - 'a' + 10;
	if (c >= 'A' && c <= 'F') return c - 'A' + 10;
	return -1;
}

/* parse 2*size hex chars into buf; returns 0 on success */
static int parse_hex(const char *s, unsigned char *buf, size_t size) {
	size_t i;
	for (i = 0; i < size; i++) {
		int hi = hexval((unsigned char)s[2 * i]);
		int lo = hexval((unsigned char)s[2 * i + 1]);
		if (hi < 0 || lo < 0) return -1;
		buf[i] = (unsigned char)((hi << 4) | lo);
	}
	return 0;
}

/* GNURUST.CABI-FIELD.0 / NUMCONST.0 / CMODEL.0 / CLOCALE.0:
 * emit the *built oracle's* real ABI, numeric constants, C integer model, and C locale so the
 * Rust port's constants are validated against the actual library, not a header guess. */
static void selfcheck(void) {
	printf("# cob_field/cob_field_attr C ABI (GNURUST.CABI-FIELD.0)\n");
	printf("sizeof_cob_field=%zu alignof_cob_field=%zu\n",
	       sizeof(cob_field), _Alignof(cob_field));
	printf("offset_size=%zu offset_data=%zu offset_attr=%zu\n",
	       offsetof(cob_field, size), offsetof(cob_field, data), offsetof(cob_field, attr));
	printf("sizeof_cob_field_attr=%zu alignof_cob_field_attr=%zu\n",
	       sizeof(cob_field_attr), _Alignof(cob_field_attr));
	printf("offset_type=%zu offset_digits=%zu offset_scale=%zu offset_flags=%zu offset_pic=%zu\n",
	       offsetof(cob_field_attr, type), offsetof(cob_field_attr, digits),
	       offsetof(cob_field_attr, scale), offsetof(cob_field_attr, flags),
	       offsetof(cob_field_attr, pic));
	printf("# numeric/type/flag constants (GNURUST.NUMCONST.0)\n");
	printf("COB_MAX_DIGITS=%d\n", COB_MAX_DIGITS);
	printf("COB_TYPE_NUMERIC_DISPLAY=0x%02x COB_TYPE_NUMERIC_PACKED=0x%02x\n",
	       COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED);
	printf("COB_FLAG_HAVE_SIGN=0x%04x COB_FLAG_SIGN_SEPARATE=0x%04x "
	       "COB_FLAG_SIGN_LEADING=0x%04x COB_FLAG_NO_SIGN_NIBBLE=0x%04x\n",
	       COB_FLAG_HAVE_SIGN, COB_FLAG_SIGN_SEPARATE,
	       COB_FLAG_SIGN_LEADING, COB_FLAG_NO_SIGN_NIBBLE);
	printf("# C integer model (GNURUST.CMODEL.0)\n");
	printf("sizeof_char=%zu sizeof_short=%zu sizeof_int=%zu sizeof_long=%zu "
	       "sizeof_longlong=%zu sizeof_size_t=%zu char_signed=%d\n",
	       sizeof(char), sizeof(short), sizeof(int), sizeof(long),
	       sizeof(long long), sizeof(size_t), (CHAR_MIN < 0) ? 1 : 0);
	{
		const unsigned int one = 1u;
		printf("endian=%s\n", (*(const unsigned char *)&one) ? "little" : "big");
	}
	printf("# C locale at process start (GNURUST.CLOCALE.0)\n");
	{
		const char *loc = setlocale(LC_ALL, NULL);
		const struct lconv *lc = localeconv();
		printf("setlocale_LC_ALL=%s decimal_point=%s thousands_sep=%s\n",
		       loc ? loc : "(null)",
		       (lc && lc->decimal_point && *lc->decimal_point) ? lc->decimal_point : "(empty)",
		       (lc && lc->thousands_sep && *lc->thousands_sep) ? lc->thousands_sep : "(empty)");
	}
}

int main(int argc, char **argv) {
	char line[4096];

	cob_init(argc, argv);

	if (argc > 1 && strcmp(argv[1], "--selfcheck") == 0) {
		selfcheck();
		return 0;
	}

	/* Provide a valid current module so the DISPLAY sign path
	   (cob_real_get_sign -> COB_MODULE_PTR->ebcdic_sign) is safe.
	   Zeroed => ebcdic_sign = 0 => the ASCII overpunch path. */
	memset(&module_storage, 0, sizeof(module_storage));
	module_storage.decimal_point     = '.';
	module_storage.currency_symbol   = '$';
	module_storage.numeric_separator  = ',';
	module_storage.ebcdic_sign        = 0;
	module_storage.flag_binary_truncate = 1;
	{
		cob_module *mp = &module_storage;
		cob_module_global_enter(&mp, &cobglob, 0, 0, 0);
	}

	while (fgets(line, sizeof(line), stdin)) {
		char label[256], s_hex[2050];
		unsigned int s_type, s_digits, s_flags, s_size;
		int          s_scale;
		unsigned int d_type, d_digits, d_flags, d_size;
		int          d_scale;
		unsigned char s_data[1024], d_data[1024];
		cob_field_attr s_attr, d_attr;
		cob_field s_field, d_field;
		size_t i;

		if (line[0] == '#' || line[0] == '\n') continue;
		if (sscanf(line, "%255s %u %u %d %u %u %2049s %u %u %d %u %u",
			   label, &s_type, &s_digits, &s_scale, &s_flags, &s_size, s_hex,
			   &d_type, &d_digits, &d_scale, &d_flags, &d_size) != 12) {
			fprintf(stderr, "bad line: %s", line);
			continue;
		}
		if (s_size > sizeof(s_data) || d_size > sizeof(d_data)) {
			fprintf(stderr, "size too big: %s", line);
			continue;
		}
		if (parse_hex(s_hex, s_data, s_size) != 0) {
			fprintf(stderr, "bad hex: %s", line);
			continue;
		}

		s_attr.type = (unsigned short)s_type;
		s_attr.digits = (unsigned short)s_digits;
		s_attr.scale = (short)s_scale;
		s_attr.flags = (unsigned short)s_flags;
		s_attr.pic = NULL;
		s_field.size = s_size;
		s_field.data = s_data;
		s_field.attr = &s_attr;

		d_attr.type = (unsigned short)d_type;
		d_attr.digits = (unsigned short)d_digits;
		d_attr.scale = (short)d_scale;
		d_attr.flags = (unsigned short)d_flags;
		d_attr.pic = NULL;
		memset(d_data, 0, d_size);
		d_field.size = d_size;
		d_field.data = d_data;
		d_field.attr = &d_attr;

		cob_move(&s_field, &d_field);

		printf("%s ", label);
		for (i = 0; i < d_size; i++) printf("%02x", d_data[i]);
		printf("\n");
	}
	return 0;
}
