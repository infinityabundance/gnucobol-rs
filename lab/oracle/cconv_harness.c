/* Oracle harness for cconv.c. Exercises the EXPORTED cconv functions of the built libcob (the static
 * helpers cob_convert_hex_digit/byte/skip_blanks/init_upper_lower are covered transitively):
 *   - cob_toupper / cob_tolower over all 256 bytes
 *   - cob_field_to_string over a fixed grid of (data, size, case)
 *   - cob_load_collation over the .ttbl files passed as argv (absolute paths)
 * Prints one line per result; the Rust evaluator (cconv_rows) prints byte-identical lines.
 *
 * Build: gcc -O2 -I$PREFIX/include cconv_harness.c -o cconv_harness -L$PREFIX/lib -lcob
 * Run:   cconv_harness <ttbl-path> [<ttbl-path> ...]
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <libcob.h>

/* coblocal.h is an internal (uninstalled) header; forward-declare the exported cconv ABI we exercise. */
enum cob_case_modifier { CCM_NONE, CCM_LOWER, CCM_UPPER, CCM_LOWER_LOCALE, CCM_UPPER_LOCALE };
extern int cob_field_to_string(const cob_field *, void *, const size_t, const enum cob_case_modifier);
extern int cob_load_collation(const char *, cob_u8_t *, cob_u8_t *);
extern unsigned char cob_toupper(const unsigned char);
extern unsigned char cob_tolower(const unsigned char);
extern void cob_init_cconv(cob_global *);

/* cconv.o's only non-libc dependency is cob_runtime_error (called on parse errors); stub it so we can
 * link the EXACT oracle object (extracted from libcob.a) without the whole runtime + its db/ncurses deps. */
void cob_runtime_error(const char *fmt, ...) { (void)fmt; }

static void put_hex(const char *tag, const unsigned char *b, int n) {
	printf("%s", tag);
	for (int i = 0; i < n; i++) printf("%02x", b[i]);
	printf("\n");
}

static void f2s_case(const char *label, const char *data, int size, enum cob_case_modifier cm) {
	cob_field_attr a = { COB_TYPE_ALPHANUMERIC, 0, 0, 0, NULL };
	cob_field f;
	f.size = size;
	f.data = (unsigned char *)data;
	f.attr = &a;
	char out[64];
	memset(out, 0, sizeof out);
	int r = cob_field_to_string(&f, out, sizeof(out), cm);
	/* print return + the bytes up to the first NUL (the produced string) */
	printf("F2S %s %d ", label, r);
	for (size_t i = 0; i < sizeof(out) && out[i]; i++) printf("%02x", (unsigned char)out[i]);
	printf("\n");
}

int main(int argc, char **argv) {
	cob_init_cconv(NULL);

	unsigned char up[256], lo[256];
	for (int c = 0; c < 256; c++) {
		up[c] = cob_toupper((unsigned char)c);
		lo[c] = cob_tolower((unsigned char)c);
	}
	put_hex("TOUPPER ", up, 256);
	put_hex("TOLOWER ", lo, 256);

	f2s_case("none8",  "HeLLo   ", 8, CCM_NONE);
	f2s_case("low8",   "HeLLo   ", 8, CCM_LOWER);
	f2s_case("up8",    "HeLLo   ", 8, CCM_UPPER);
	f2s_case("lowloc", "HeLLo   ", 8, CCM_LOWER_LOCALE);
	f2s_case("uploc",  "HeLLo   ", 8, CCM_UPPER_LOCALE);
	f2s_case("blank",  "    ",     4, CCM_NONE);
	f2s_case("trail0", "AB\0\0",   4, CCM_NONE);
	f2s_case("full",   "ABCDE",    5, CCM_UPPER);
	f2s_case("mixed",  "Ab9$X",    5, CCM_LOWER);
	f2s_case("one",    "Q",        1, CCM_LOWER);

	for (int k = 1; k < argc; k++) {
		unsigned char e2a[256], a2e[256];
		memset(e2a, 0, 256);
		memset(a2e, 0, 256);
		int r = cob_load_collation(argv[k], e2a, a2e);
		/* tag by basename */
		const char *base = strrchr(argv[k], '/');
		base = base ? base + 1 : argv[k];
		printf("COLL %s %d\n", base, r);
		char tag[300];
		snprintf(tag, sizeof tag, "E2A %s ", base);
		put_hex(tag, e2a, 256);
		snprintf(tag, sizeof tag, "A2E %s ", base);
		put_hex(tag, a2e, 256);
	}
	return 0;
}
