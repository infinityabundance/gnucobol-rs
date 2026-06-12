/*
 * pow_harness.c — runtime-library oracle for gnucobol-rs / int_pow (GNURUST.INTPOW.1).
 *
 * Links the built libcob and calls the REAL cob_s32_pow / cob_s64_pow over a matrix of
 * (base, power), dumping the returned integer. The Rust port reproduces these values.
 * NOT shipped in any published crate; lab/ tooling only.
 *
 * Input  (stdin): label width base power     (width: 32 or 64)
 * Output (stdout): label result             (decimal; "SIGFPE" for the 0**negative crash case,
 *                  which the generator never emits so the process does not actually abort)
 */
#include <libcob.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    char line[256];
    cob_init(argc, argv);
    while (fgets(line, sizeof(line), stdin)) {
        char label[64];
        unsigned int width;
        long long base, power;
        if (line[0] == '#' || line[0] == '\n') continue;
        if (sscanf(line, "%63s %u %lld %lld", label, &width, &base, &power) != 4) continue;
        if (width == 32) {
            cob_s32_t r = cob_s32_pow((cob_s32_t)base, (cob_s32_t)power);
            printf("%s %d\n", label, r);
        } else {
            cob_s64_t r = cob_s64_pow((cob_s64_t)base, (cob_s64_t)power);
            printf("%s %lld\n", label, (long long)r);
        }
    }
    return 0;
}
