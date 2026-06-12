/*
 * logical_harness.c — runtime oracle for gnucobol-rs / logical (GNURUST.LOGICAL.1).
 *
 * Links the built libcob and calls the REAL cob_logical_and/or/xor/not/left/right over cob_decimals
 * built from int64 operands, then reads the result back as a 20-digit unsigned DISPLAY value. The
 * public libcob.h does not expose the internal cob_decimal type, so we declare it here (it is
 * { mpz_t value; int scale; }) and the needed externs. NOT shipped in any published crate.
 *
 * Input  (stdin): label op v0 v1     (op: and|or|xor|not|shl|shr ; v* are signed decimal int64)
 * Output (stdout): label result      (the u64 result as a decimal string)
 */
#include <libcob.h>
#include <gmp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct __cob_decimal { mpz_t value; int scale; } cob_decimal;

extern void cob_decimal_init (cob_decimal *);
extern void cob_decimal_set_llint (cob_decimal *, long long);
extern int  cob_decimal_get_field (cob_decimal *, cob_field *, const int);
extern void cob_logical_and (cob_decimal *, cob_decimal *);
extern void cob_logical_or (cob_decimal *, cob_decimal *);
extern void cob_logical_xor (cob_decimal *, cob_decimal *);
extern void cob_logical_not (cob_decimal *, cob_decimal *);
extern void cob_logical_left (cob_decimal *, cob_decimal *);
extern void cob_logical_right (cob_decimal *, cob_decimal *);

int main(int argc, char **argv) {
    char line[256];
    cob_init(argc, argv);
    cob_decimal d0, d1;
    cob_decimal_init(&d0);
    cob_decimal_init(&d1);

    while (fgets(line, sizeof(line), stdin)) {
        char label[64], op[8];
        long long v0, v1;
        if (line[0] == '#' || line[0] == '\n') continue;
        if (sscanf(line, "%63s %7s %lld %lld", label, op, &v0, &v1) != 4) continue;

        cob_decimal_set_llint(&d0, v0);
        cob_decimal_set_llint(&d1, v1);

        if (!strcmp(op, "and")) cob_logical_and(&d0, &d1);
        else if (!strcmp(op, "or")) cob_logical_or(&d0, &d1);
        else if (!strcmp(op, "xor")) cob_logical_xor(&d0, &d1);
        else if (!strcmp(op, "not")) cob_logical_not(&d0, &d1);
        else if (!strcmp(op, "shl")) cob_logical_left(&d0, &d1);
        else if (!strcmp(op, "shr")) cob_logical_right(&d0, &d1);
        else continue;

        unsigned char buf[20];
        cob_field_attr a;
        a.type = COB_TYPE_NUMERIC_DISPLAY; a.digits = 20; a.scale = 0; a.flags = 0; a.pic = NULL;
        cob_field f; f.size = 20; f.data = buf; f.attr = &a;
        memset(buf, '0', sizeof(buf));
        cob_decimal_get_field(&d0, &f, 0);

        int i = 0;
        while (i < 19 && buf[i] == '0') i++;
        printf("%s ", label);
        fwrite(buf + i, 1, 20 - i, stdout);
        printf("\n");
    }
    return 0;
}
