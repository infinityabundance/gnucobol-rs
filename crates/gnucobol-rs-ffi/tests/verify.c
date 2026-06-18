/* Links against EITHER libgnucobol_rs_ffi OR the real libcob -- the cob_field ABI is identical, so the
 * same source proves the shim is a drop-in. Exercises cob_move (display->COMP-3), cob_get_int, cob_set_int. */
#include <stdio.h>
#include <stddef.h>

typedef struct { unsigned short type; unsigned short digits; short scale; unsigned short flags; const void *pic; } cob_field_attr;
typedef struct { size_t size; unsigned char *data; const cob_field_attr *attr; } cob_field;

extern void cob_move(const cob_field*, cob_field*);
extern int  cob_get_int(const cob_field*);
extern void cob_set_int(cob_field*, int);

#define DISPLAY 0x10
#define PACKED  0x12

int main(void) {
    unsigned char sd[4] = {'1','2','3','4'};
    cob_field_attr sa = { DISPLAY, 4, 0, 0, 0 };
    cob_field src = { 4, sd, &sa };
    unsigned char dd[3] = {0,0,0};
    cob_field_attr da = { PACKED, 4, 0, 0, 0 };
    cob_field dst = { 3, dd, &da };
    cob_move(&src, &dst);
    printf("move display->comp3: %02x %02x %02x\n", dd[0], dd[1], dd[2]);
    printf("get_int(comp3): %d\n", cob_get_int(&dst));
    unsigned char xd[4] = {0,0,0,0};
    cob_field_attr xa = { DISPLAY, 4, 0, 0, 0 };
    cob_field x = { 4, xd, &xa };
    cob_set_int(&x, 5678);
    printf("set_int display: %c%c%c%c\n", xd[0], xd[1], xd[2], xd[3]);
    return 0;
}
