/* gnucobol-rs-ffi.h -- C-ABI for the native-Rust gnucobol-rs libcob algorithms.
 *
 * Link libgnucobol_rs_ffi (cdylib/staticlib) where you would link libcob. The cob_field / cob_field_attr
 * layouts match GnuCOBOL 3.2's libcob common.h, so the same field buffers work unchanged.
 *
 * License: LGPL-3.0-or-later. This is gnucobol-rs (a faithful, line-cited native-Rust port of libcob,
 * NOT clean-room), NOT GnuCOBOL and not affiliated with the GNU project.
 */
#ifndef GNUCOBOL_RS_FFI_H
#define GNUCOBOL_RS_FFI_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* cob_field_attr (libcob common.h:1097) */
typedef struct cobrs_field_attr {
	unsigned short type;    /* COB_TYPE_* */
	unsigned short digits;  /* digit count */
	short          scale;   /* field scale */
	unsigned short flags;   /* COB_FLAG_* */
	const void    *pic;     /* picture string (opaque) */
} cob_field_attr;

/* cob_field (libcob common.h:1107) */
typedef struct cobrs_field {
	size_t                 size;  /* byte length */
	unsigned char         *data;  /* data bytes */
	const cob_field_attr  *attr;  /* attributes */
} cob_field;

/* MOVE src -> dst, converting category/usage (libcob cob_move). */
void cob_move(const cob_field *src, cob_field *dst);

/* The field's value as int / long long (libcob cob_get_int / cob_get_llint). */
int       cob_get_int(const cob_field *f);
long long cob_get_llint(const cob_field *f);

/* Store an int into a field (libcob cob_set_int). */
void cob_set_int(cob_field *f, int n);

/* The reproduced GnuCOBOL version, e.g. "3.2.0" (gnucobol-rs convenience, not a libcob symbol). */
const char *cobrs_version(void);

#ifdef __cplusplus
}
#endif

#endif /* GNUCOBOL_RS_FFI_H */
