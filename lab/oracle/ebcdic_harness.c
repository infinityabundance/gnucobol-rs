/* ebcdic_harness.c — dump the admitted oracle's EBCDIC->ASCII table via libcob's cob_load_collation
 * (the function cobc itself uses to embed tables). Prints 256 lower-hex bytes, one line. Verifies
 * that gnucobol-rs's embedded cp500 table is exactly the one the admitted oracle produces.
 * Input (argv[1]): collation name (e.g. "ebcdic500_ascii8bit"). NOT shipped in any crate.
 */
#include <libcob.h>
#include <stdio.h>
int main(int argc, char **argv) {
    cob_u8_t ebc2asc[256];
    const char *name = argc > 1 ? argv[1] : "ebcdic500_ascii8bit";
    cob_init(argc, argv);
    if (cob_load_collation(name, ebc2asc, NULL) != 0) { fprintf(stderr, "load failed: %s\n", name); return 2; }
    for (int i = 0; i < 256; i++) printf("%02x", ebc2asc[i]);
    printf("\n");
    return 0;
}
