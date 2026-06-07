/*
 * arith_harness.c — runtime-library oracle for gnucobol-rs / arith (GNURUST.7).
 *
 * Links the built libcob and exercises the REAL cob_add / cob_sub / cob_mul on cob_fields,
 * dumping the destination (f1) bytes after the operation. The Rust port reproduces these bytes.
 * NOT shipped in any published crate; lab/ tooling only.
 *
 * Input  (stdin): label op a_type a_dig a_scale a_flags a_size a_hex
 *                          b_type b_dig b_scale b_flags b_size b_hex opt
 *   op : 1=ADD 2=SUB 3=MUL ;  opt: 0=truncate, 1=COB_STORE_ROUND (nearest-away default mode)
 * Output (stdout): label  f1_hex   (bytes of f1 after f1 := f1 op f2)
 */
#include <libcob.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static cob_module  module_storage;
static cob_global *cobglob;

static int hexval(int c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}
static int parse_hex(const char *s, unsigned char *buf, size_t size) {
    for (size_t i = 0; i < size; i++) {
        int hi = hexval((unsigned char)s[2*i]), lo = hexval((unsigned char)s[2*i+1]);
        if (hi < 0 || lo < 0) return -1;
        buf[i] = (unsigned char)((hi << 4) | lo);
    }
    return 0;
}

int main(int argc, char **argv) {
    char line[4096];
    cob_init(argc, argv);
    memset(&module_storage, 0, sizeof(module_storage));
    module_storage.decimal_point = '.';
    module_storage.currency_symbol = '$';
    module_storage.numeric_separator = ',';
    module_storage.ebcdic_sign = 0;
    module_storage.flag_binary_truncate = 1;
    { cob_module *mp = &module_storage; cob_module_global_enter(&mp, &cobglob, 0, 0, 0); }

    while (fgets(line, sizeof(line), stdin)) {
        char label[256], a_hex[2050], b_hex[2050];
        unsigned int op, a_type, a_dig, a_flags, a_size, b_type, b_dig, b_flags, b_size, opt;
        int a_scale, b_scale;
        unsigned char a_data[1024], b_data[1024];
        cob_field_attr a_attr, b_attr;
        cob_field a_field, b_field;

        if (line[0] == '#' || line[0] == '\n') continue;
        if (sscanf(line, "%255s %u %u %u %d %u %u %2049s %u %u %d %u %u %2049s %u",
                   label, &op, &a_type, &a_dig, &a_scale, &a_flags, &a_size, a_hex,
                   &b_type, &b_dig, &b_scale, &b_flags, &b_size, b_hex, &opt) != 15) {
            fprintf(stderr, "bad line: %s", line); continue;
        }
        if (a_size > sizeof(a_data) || b_size > sizeof(b_data)) continue;
        if (parse_hex(a_hex, a_data, a_size) || parse_hex(b_hex, b_data, b_size)) continue;

        a_attr.type = a_type; a_attr.digits = a_dig; a_attr.scale = a_scale; a_attr.flags = a_flags; a_attr.pic = NULL;
        b_attr.type = b_type; b_attr.digits = b_dig; b_attr.scale = b_scale; b_attr.flags = b_flags; b_attr.pic = NULL;
        a_field.size = a_size; a_field.data = a_data; a_field.attr = &a_attr;
        b_field.size = b_size; b_field.data = b_data; b_field.attr = &b_attr;

        switch (op) {
            case 1: cob_add(&a_field, &b_field, (int)opt); break;
            case 2: cob_sub(&a_field, &b_field, (int)opt); break;
            case 3: cob_mul(&a_field, &b_field, (int)opt); break;
            default: break;
        }

        printf("%s ", label);
        for (size_t i = 0; i < a_size; i++) printf("%02x", a_data[i]);
        printf("\n");
    }
    return 0;
}
