//! Faithful 1:1 port of the **sign** + top-level **comparison** helpers of `libcob/common.c`.
//!
//! This module ports the COBOL DISPLAY-field sign get/set helpers and the top-level `cob_cmp`
//! dispatcher. The functions carry their **exact C names** (the port-index matches by C name). They
//! are pure functions of explicit inputs: a `cob_field` in libcob is `(attr->type, data, attr->flags)`,
//! so here a field is modelled as a byte slice plus a small [`SignFlags`] record (the relevant
//! `COB_FLAG_*` bits) and, where the dispatch needs it, a `COB_TYPE_*` code.
//!
//! Citations: `cob_get_sign_ascii` (`common.c:1450`), `cob_put_sign_ascii` (`common.c:1516`),
//! `cob_get_sign_ebcdic` (`common.c:1580`), `locate_sign` (`common.c:3693`),
//! `cob_real_get_sign` (`common.c:3712`), `cob_real_put_sign` (`common.c:3763`),
//! `cob_cmp` (`common.c:3834`). Evidence kind: **source_port**.
//!
//! ASCII host only (`COB_EBCDIC_MACHINE` off): the EBCDIC-host branches of `cob_get_sign_ascii` /
//! `cob_put_sign_ascii` are the classified non-claim; the non-EBCDIC branch is ported. The runtime
//! `ebcdic_sign` module flag (whether the *data* uses an EBCDIC overpunch on an ASCII host) is passed
//! explicitly into the dispatchers.

#![forbid(unsafe_code)]

/// The `COB_FLAG_*` sign attribute bits of a `cob_field`'s attr that the sign helpers read
/// (`common.h:704-706`): `HAVE_SIGN`, `SIGN_SEPARATE`, `SIGN_LEADING`. `ebcdic_sign` is the
/// per-module flag (`COB_MODULE_PTR->ebcdic_sign`) selecting the EBCDIC overpunch on an ASCII host.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SignFlags {
    /// `COB_FLAG_HAVE_SIGN`.
    pub have_sign: bool,
    /// `COB_FLAG_SIGN_SEPARATE`.
    pub sign_separate: bool,
    /// `COB_FLAG_SIGN_LEADING`.
    pub sign_leading: bool,
    /// `COB_MODULE_PTR->ebcdic_sign`.
    pub ebcdic_sign: bool,
}

// `common.h` field-type codes used by the sign dispatchers + cob_cmp.
const COB_TYPE_NUMERIC: u8 = 0x10; // the "is numeric" bit (COB_FIELD_TYPE & COB_TYPE_NUMERIC)
const COB_TYPE_NUMERIC_DISPLAY: u8 = 0x10;
const COB_TYPE_NUMERIC_PACKED: u8 = 0x12;
const COB_TYPE_ALPHANUMERIC_ALL: u8 = 0x22;

/// `IS_VALID_DIGIT_DATA(c) = (unsigned char)(c - '0') <= 9` (`common.c:428`).
#[inline]
const fn is_valid_digit_data(c: u8) -> bool {
    c.wrapping_sub(0x30) <= 9
}

/// `COB_D2I(x) = (x) & 0x0F` (`coblocal.h:243`).
#[inline]
const fn cob_d2i(x: u8) -> u8 {
    x & 0x0F
}

/// `COB_I2D(x) = '0' + x` (`coblocal.h:244`).
#[inline]
const fn cob_i2d(x: u8) -> u8 {
    0x30u8.wrapping_add(x)
}

/// Port of `common.c:cob_get_sign_ascii` (non-EBCDIC branch, `common.c:1499-1508`). Reads the ASCII
/// overpunch sign of a single DISPLAY sign byte `p` and **normalizes it in place**: a negative
/// overpunch `'p'..'y'` (0x70..0x79) is cleared with `&= ~0x40` (→ a plain digit) and returns `-1`; a
/// plain digit is left as-is and returns `+1`; anything else is forced to `'0'` and returns `+1`.
pub fn cob_get_sign_ascii(p: &mut u8) -> i32 {
    if *p >= 0x70 && *p <= 0x79 {
        // 'p'..'y'
        *p &= !0x40; // 0x71 'q' -> 0x31 '1'
        return -1;
    }
    if is_valid_digit_data(*p) {
        // already without sign
        return 1;
    }
    *p = 0x30; // '0'
    1
}

/// Port of `common.c:cob_put_sign_ascii` (non-EBCDIC branch, `common.c:1566`). Overpunches a negative
/// sign onto the DISPLAY sign byte `p` by setting bit `0x40` (`'0'`→`'p'`, …). The C is unconditional;
/// the caller only invokes it when the sign is negative.
pub fn cob_put_sign_ascii(p: &mut u8) {
    *p |= 0x40;
}

/// Port of `common.c:cob_get_sign_ebcdic` (non-EBCDIC machine branch, `common.c:1609-1681`). Reads the
/// EBCDIC overpunch sign of a single DISPLAY sign byte `p` and **normalizes it in place** to a plain
/// digit: positive `{ABCDEFGHI` → `0..9`, sign `+1`; negative `}JKLMNOPQR` → `0..9`, sign `-1`; a plain
/// digit `'0'..'9'` is left as-is, sign `0` (unsigned); any other byte is coerced (`0x__` low nibble
/// out of range → `'0'`, else its low-nibble digit) and returns `+1`.
pub fn cob_get_sign_ebcdic(p: &mut u8) -> i32 {
    match *p {
        b'{' => {
            *p = b'0';
            1
        }
        b'A'..=b'I' => {
            *p = b'1' + (*p - b'A');
            1
        }
        b'}' => {
            *p = b'0';
            -1
        }
        b'J'..=b'R' => {
            *p = b'1' + (*p - b'J');
            -1
        }
        _ => {
            if (b'0'..=b'9').contains(p) {
                return 0;
            }
            if (*p & 0x0F) > 9 {
                *p = b'0';
            } else {
                *p = cob_i2d(cob_d2i(*p));
            }
            1
        }
    }
}

/// Port of `common.c:locate_sign` (`common.c:3693`). The index of the sign byte within a DISPLAY
/// field's storage: byte `0` when `SIGN LEADING`, otherwise the last byte (`size - 1`). A pure index
/// computation (the C returns `f->data` or `f->data + f->size - 1`).
#[inline]
pub fn locate_sign(size: usize, sign_leading: bool) -> usize {
    if sign_leading {
        0
    } else {
        size.saturating_sub(1)
    }
}

/// Port of `common.c:cob_real_get_sign` (`common.c:3712`). Get the sign of a DISPLAY or PACKED field.
///
/// `field_type` is the `COB_TYPE_*` code; `data` the field bytes (mutated in place for the
/// DISPLAY/overpunch cases exactly as the C "unpunches" via [`cob_get_sign_ascii`] /
/// [`cob_get_sign_ebcdic`]); `flags` the sign attribute bits; `adjust_ebcdic` selects the
/// `COB_GET_SIGN_ADJUST` path.
///
/// Returns: `1` = positive/non-negative, `-1` = negative, `2`/`-2` = positive/negative **adjusted**
/// (the `adjust_ebcdic` overpunch variants), `0` = neither DISPLAY nor PACKED. On the non-EBCDIC host
/// the `adjust_ebcdic && !ebcdic_sign` path reads the sign from the high nibble (`(*p & 0xF0) == 0x70`)
/// **without** mutating the data.
pub fn cob_real_get_sign(
    field_type: u8,
    data: &mut [u8],
    flags: SignFlags,
    adjust_ebcdic: bool,
) -> i32 {
    match field_type {
        COB_TYPE_NUMERIC_DISPLAY => {
            let idx = locate_sign(data.len(), flags.sign_leading);
            // The C reads/writes through `p`; an empty field has no sign byte to read.
            let p = match data.get(idx) {
                Some(&b) => b,
                None => return 1,
            };
            if flags.sign_separate {
                return if p == b'-' { -1 } else { 1 };
            }
            if is_valid_digit_data(p) {
                return 1;
            }
            if p == b' ' {
                return 1;
            }
            if adjust_ebcdic {
                if flags.ebcdic_sign {
                    // ASCII machine, ebcdic_sign data: adjusted variant returns +/-2.
                    return if cob_get_sign_ebcdic(&mut data[idx]) < 0 { -2 } else { 2 };
                }
                // ASCII machine, ascii sign, adjust: read from high nibble, no mutation.
                if (p & 0xF0) == 0x70 {
                    -1
                } else {
                    1
                }
            } else if flags.ebcdic_sign {
                cob_get_sign_ebcdic(&mut data[idx])
            } else {
                cob_get_sign_ascii(&mut data[idx])
            }
        }
        COB_TYPE_NUMERIC_PACKED => {
            // The C: COB_FIELD_NO_SIGN_NIBBLE -> 1 (modelled by an unsigned packed field having no
            // sign nibble; here a signed packed field reads the last byte's low nibble: 0x0D = -1).
            if !flags.have_sign {
                return 1;
            }
            match data.last() {
                Some(&b) if (b & 0x0F) == 0x0D => -1,
                _ => 1,
            }
        }
        _ => 0,
    }
}

/// Port of `common.c:cob_real_put_sign` (`common.c:3763`). Store the sign into a DISPLAY or PACKED
/// field. `SIGN SEPARATE` writes `'+'`/`'-'`; an `ebcdic_sign` field uses the EBCDIC overpunch
/// ([`crate::common::cob_put_sign_ebcdic`] semantics, inlined here to avoid a cross-module dep);
/// otherwise a negative sign overpunches via [`cob_put_sign_ascii`] (a non-negative ASCII sign is a
/// no-op). For PACKED the last byte's low nibble becomes `0x0D` (negative) or `0x0C` (positive). A
/// non-DISPLAY/PACKED field is a no-op.
pub fn cob_real_put_sign(field_type: u8, data: &mut [u8], flags: SignFlags, sign: i32) {
    match field_type {
        COB_TYPE_NUMERIC_DISPLAY => {
            let idx = locate_sign(data.len(), flags.sign_leading);
            if flags.sign_separate {
                let c = if sign == -1 { b'-' } else { b'+' };
                if let Some(slot) = data.get_mut(idx) {
                    if *slot != c {
                        *slot = c;
                    }
                }
            } else if flags.ebcdic_sign {
                if let Some(slot) = data.get_mut(idx) {
                    put_sign_ebcdic_byte(slot, sign);
                }
            } else if sign == -1 {
                if let Some(slot) = data.get_mut(idx) {
                    cob_put_sign_ascii(slot);
                }
            }
        }
        COB_TYPE_NUMERIC_PACKED => {
            if !flags.have_sign {
                return;
            }
            if let Some(p) = data.last_mut() {
                if sign == -1 {
                    *p = (*p & 0xF0) | 0x0D;
                } else {
                    *p = (*p & 0xF0) | 0x0C;
                }
            }
        }
        _ => {}
    }
}

/// `common.c:cob_put_sign_ebcdic` (`common.c:1686`), inlined byte form: overlay an EBCDIC overpunch
/// sign on the digit `p` (negative `0..9`→`}JKLMNOPQR`, positive `0..9`→`{ABCDEFGHI`; an already-signed
/// byte is left as-is; an unexpected byte becomes the zero-overpunch `}`/`{`). Kept private to avoid a
/// dependency on the `common` module; identical to that module's `cob_put_sign_ebcdic`.
fn put_sign_ebcdic_byte(p: &mut u8, sign: i32) {
    if sign == -1 {
        *p = match *p {
            b'0' => b'}',
            b'1' => b'J',
            b'2' => b'K',
            b'3' => b'L',
            b'4' => b'M',
            b'5' => b'N',
            b'6' => b'O',
            b'7' => b'P',
            b'8' => b'Q',
            b'9' => b'R',
            b'}' | b'J'..=b'R' => return,
            _ => b'}',
        };
    } else {
        *p = match *p {
            b'0' => b'{',
            b'1' => b'A',
            b'2' => b'B',
            b'3' => b'C',
            b'4' => b'D',
            b'5' => b'E',
            b'6' => b'F',
            b'7' => b'G',
            b'8' => b'H',
            b'9' => b'I',
            b'{' | b'A'..=b'I' => return,
            _ => b'{',
        };
    }
}

/// A `cob_field` operand for [`cob_cmp`], modelled as `(type, data, sign-flags)` — the parts of a
/// libcob `cob_field`/`cob_field_attr` the comparison reads.
#[derive(Debug, Clone, Copy)]
pub struct CmpField<'a> {
    /// `COB_FIELD_TYPE` (the `COB_TYPE_*` code).
    pub field_type: u8,
    /// The field's bytes (`f->data` over `f->size`).
    pub data: &'a [u8],
    /// The sign attribute bits.
    pub flags: SignFlags,
}

/// The outcome of [`cob_cmp`] for the paths this port covers, vs. the **declared boundary** where the
/// C delegates to the numeric engine (`cob_numeric_cmp` / `cob_cmp_int`), which lives behind
/// `GNURUST.NUMCMP.1` and is not re-ported here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpResult {
    /// A resolved signed comparison value (negative / zero / positive ordering).
    Value(i32),
    /// Both operands numeric → `cob_numeric_cmp(f1, f2)` (numeric-engine boundary).
    NumericCmp,
    /// A numeric operand vs. the `ALL "0"` figurative → `±cob_cmp_int(f, 0)` (sign carries the
    /// negation the C applies when `f1` is the ALL field).
    CmpIntZero {
        /// `true` when the C negates the result (`f1` is the `ALL "0"` operand).
        negate: bool,
    },
}

/// Port of `common.c:cob_cmp` (`common.c:3834`) — the top-level COBOL comparison entry. Dispatches on
/// the two operands' types:
///
/// * both numeric → `cob_numeric_cmp` ([`CmpResult::NumericCmp`], the numeric-engine boundary);
/// * one operand is the internal `ALPHANUMERIC_ALL` figurative (`ZERO`/`LOW-VALUE`/…): a 1-byte `'0'`
///   ALL against a numeric operand routes to `±cob_cmp_int(f, 0)` ([`CmpResult::CmpIntZero`]),
///   otherwise to [`crate::common::cob_cmp_all`] (negated when `f1` is the ALL field);
/// * otherwise an alphanumeric comparison via [`crate::common::cob_cmp_alnum`]. When exactly one
///   operand is numeric the C performs an intermediate MOVE-to-DISPLAY (drop sign + implied decimal)
///   then `cob_cmp_alnum`; this port models the **already-DISPLAY** alphanumeric path directly with the
///   collation-free `cob_cmp_alnum` and, for a signed DISPLAY operand, strips its sign with
///   [`cob_real_get_sign`] before comparing (the MOVE-to-DISPLAY of a non-DISPLAY numeric operand is
///   the numeric-engine boundary and is not performed here).
///
/// `col` is the optional collating sequence threaded into the alphanumeric/`ALL` comparisons.
pub fn cob_cmp(f1: &CmpField, f2: &CmpField, col: Option<&[u8; 256]>) -> CmpResult {
    let f1_is_numeric = (f1.field_type & COB_TYPE_NUMERIC) != 0;
    let f2_is_numeric = (f2.field_type & COB_TYPE_NUMERIC) != 0;

    // both numeric -> direct numeric compare
    if f1_is_numeric && f2_is_numeric {
        return CmpResult::NumericCmp;
    }

    // one is an internal ALL field (ZERO, LOW-VALUE, ...)
    if f2.field_type == COB_TYPE_ALPHANUMERIC_ALL {
        if f2.data.len() == 1 && f2.data[0] == b'0' && f1_is_numeric {
            return CmpResult::CmpIntZero { negate: false };
        }
        return CmpResult::Value(crate::common::cob_cmp_all(f1.data, f2.data, col));
    }
    if f1.field_type == COB_TYPE_ALPHANUMERIC_ALL {
        if f1.data.len() == 1 && f1.data[0] == b'0' && f2_is_numeric {
            return CmpResult::CmpIntZero { negate: true };
        }
        return CmpResult::Value(-crate::common::cob_cmp_all(f2.data, f1.data, col));
    }

    // all else -> alphanumeric comparison. When exactly one operand is numeric it is (here) a DISPLAY
    // numeric; a signed DISPLAY operand has its sign stripped first (mirroring COB_GET_SIGN before
    // cob_cmp_alnum), the C then restoring it -- restoration is invisible to the pure result.
    if f1_is_numeric || f2_is_numeric {
        if f1_is_numeric && f1.flags.have_sign {
            let mut buf = f1.data.to_vec();
            let _ = cob_real_get_sign(COB_TYPE_NUMERIC_DISPLAY, &mut buf, f1.flags, false);
            return CmpResult::Value(crate::common::cob_cmp_alnum(&buf, f2.data, col));
        }
        if f2_is_numeric && f2.flags.have_sign {
            let mut buf = f2.data.to_vec();
            let _ = cob_real_get_sign(COB_TYPE_NUMERIC_DISPLAY, &mut buf, f2.flags, false);
            return CmpResult::Value(crate::common::cob_cmp_alnum(f1.data, &buf, col));
        }
        return CmpResult::Value(crate::common::cob_cmp_alnum(f1.data, f2.data, col));
    }

    // both data not numeric: compare as string
    CmpResult::Value(crate::common::cob_cmp_alnum(f1.data, f2.data, col))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(have_sign: bool, sep: bool, lead: bool, ebc: bool) -> SignFlags {
        SignFlags { have_sign, sign_separate: sep, sign_leading: lead, ebcdic_sign: ebc }
    }

    #[test]
    fn get_put_sign_ascii() {
        // 'r' = 0x72 = '2' with negative overpunch -> '2', sign -1
        let mut p = b'r';
        assert_eq!(cob_get_sign_ascii(&mut p), -1);
        assert_eq!(p, b'2');
        // plain digit unchanged, +1
        let mut d = b'5';
        assert_eq!(cob_get_sign_ascii(&mut d), 1);
        assert_eq!(d, b'5');
        // junk -> '0', +1
        let mut j = b'#';
        assert_eq!(cob_get_sign_ascii(&mut j), 1);
        assert_eq!(j, b'0');
        // put: '3' -> 's' (0x33|0x40 = 0x73)
        let mut q = b'3';
        cob_put_sign_ascii(&mut q);
        assert_eq!(q, b's');
    }

    #[test]
    fn get_sign_ebcdic_overpunch() {
        // positive {ABCDEFGHI
        let mut p = b'{';
        assert_eq!(cob_get_sign_ebcdic(&mut p), 1);
        assert_eq!(p, b'0');
        let mut a = b'C';
        assert_eq!(cob_get_sign_ebcdic(&mut a), 1);
        assert_eq!(a, b'3');
        // negative }JKLMNOPQR
        let mut n = b'}';
        assert_eq!(cob_get_sign_ebcdic(&mut n), -1);
        assert_eq!(n, b'0');
        let mut r = b'N'; // N is the 5th of JKLMN... -> '5'
        assert_eq!(cob_get_sign_ebcdic(&mut r), -1);
        assert_eq!(r, b'5');
        // plain digit -> unsigned 0
        let mut d = b'7';
        assert_eq!(cob_get_sign_ebcdic(&mut d), 0);
        assert_eq!(d, b'7');
    }

    #[test]
    fn put_sign_ebcdic_field_matches_fsign_oracle() {
        // `-fsign=EBCDIC` stores a signed S9(3) DISPLAY field with the EBCDIC overpunch on the trailing
        // digit. Oracle (built cobc, the field REDEFINED as X(3) to expose the raw bytes):
        //   -123 -> "12" + 'L' (0x4C);  +123 -> "12" + 'C' (0x43).  vs the default ASCII -123 -> "12s".
        let ebc = flags(true, false, false, true);
        let mut neg = *b"123";
        cob_real_put_sign(COB_TYPE_NUMERIC_DISPLAY, &mut neg, ebc, -1);
        assert_eq!(&neg, b"12L");
        let mut pos = *b"123";
        cob_real_put_sign(COB_TYPE_NUMERIC_DISPLAY, &mut pos, ebc, 1);
        assert_eq!(&pos, b"12C");
        // decode is the inverse (cob_get_sign_ebcdic, tested in get_sign_ebcdic_overpunch): 'L' -> -3.
        // the default ASCII convention stores -3 as the overpunch 's' (0x73) instead.
        let asc = flags(true, false, false, false);
        let mut an = *b"123";
        cob_real_put_sign(COB_TYPE_NUMERIC_DISPLAY, &mut an, asc, -1);
        assert_eq!(&an, b"12s");
    }

    #[test]
    fn locate_sign_index() {
        assert_eq!(locate_sign(4, true), 0); // leading
        assert_eq!(locate_sign(4, false), 3); // trailing
        assert_eq!(locate_sign(0, false), 0); // empty saturates
    }

    #[test]
    fn real_get_sign_display_ascii() {
        // trailing ascii overpunch: "34r" (r = neg 2) -> sign -1, byte normalized to '2'
        let mut d = b"34r".to_vec();
        assert_eq!(cob_real_get_sign(0x10, &mut d, flags(true, false, false, false), false), -1);
        assert_eq!(&d, b"342");
        // separate sign: leading '-' -> -1, no mutation of others
        let mut s = b"-12".to_vec();
        assert_eq!(cob_real_get_sign(0x10, &mut s, flags(true, true, true, false), false), -1);
        assert_eq!(&s, b"-12");
        // plain trailing digit -> +1
        let mut q = b"123".to_vec();
        assert_eq!(cob_real_get_sign(0x10, &mut q, flags(true, false, false, false), false), 1);
        // adjust path, ascii sign, high nibble 0x70 -> -1 without mutation
        let mut adj = vec![b'1', b'2', 0x72];
        assert_eq!(cob_real_get_sign(0x10, &mut adj, flags(true, false, false, false), true), -1);
        assert_eq!(adj[2], 0x72); // not mutated in the readonly adjust path
        // ebcdic adjust -> +/-2
        let mut e = b"12}".to_vec();
        assert_eq!(cob_real_get_sign(0x10, &mut e, flags(true, false, false, true), true), -2);
    }

    #[test]
    fn real_get_sign_packed_and_other() {
        // packed signed, last low nibble 0x0D -> -1
        let mut p = vec![0x12u8, 0x3D];
        assert_eq!(cob_real_get_sign(0x12, &mut p, flags(true, false, false, false), false), -1);
        // packed signed, 0x0C -> +1
        let mut q = vec![0x12u8, 0x3C];
        assert_eq!(cob_real_get_sign(0x12, &mut q, flags(true, false, false, false), false), 1);
        // non display/packed -> 0
        let mut a = b"ABC".to_vec();
        assert_eq!(cob_real_get_sign(0x21, &mut a, flags(false, false, false, false), false), 0);
    }

    #[test]
    fn real_put_sign() {
        // separate, negative -> '-' at leading
        let mut s = b"+12".to_vec();
        cob_real_put_sign(0x10, &mut s, flags(true, true, true, false), -1);
        assert_eq!(&s, b"-12");
        // ascii overpunch negative on trailing: '3' -> 's'
        let mut a = b"123".to_vec();
        cob_real_put_sign(0x10, &mut a, flags(true, false, false, false), -1);
        assert_eq!(&a, b"12s");
        // ascii positive -> no-op
        let mut b = b"123".to_vec();
        cob_real_put_sign(0x10, &mut b, flags(true, false, false, false), 1);
        assert_eq!(&b, b"123");
        // packed negative -> low nibble 0x0D
        let mut p = vec![0x12u8, 0x3F];
        cob_real_put_sign(0x12, &mut p, flags(true, false, false, false), -1);
        assert_eq!(p[1], 0x3D);
        // ebcdic overpunch negative: trailing '3' -> 'L'
        let mut e = b"123".to_vec();
        cob_real_put_sign(0x10, &mut e, flags(true, false, false, true), -1);
        assert_eq!(&e, b"12L");
    }

    #[test]
    fn cob_cmp_dispatch() {
        let num = |d: &'static [u8]| CmpField { field_type: 0x10, data: d, flags: SignFlags::default() };
        let alnum = |d: &'static [u8]| CmpField { field_type: 0x21, data: d, flags: SignFlags::default() };
        let all = |d: &'static [u8]| CmpField { field_type: 0x22, data: d, flags: SignFlags::default() };

        // both numeric -> numeric boundary
        assert_eq!(cob_cmp(&num(b"12"), &num(b"34"), None), CmpResult::NumericCmp);
        // numeric vs ALL "0" -> cmp_int zero (no negate when f2 is the ALL field)
        assert_eq!(cob_cmp(&num(b"12"), &all(b"0"), None), CmpResult::CmpIntZero { negate: false });
        // ALL "0" (f1) vs numeric -> negate
        assert_eq!(cob_cmp(&all(b"0"), &num(b"12"), None), CmpResult::CmpIntZero { negate: true });
        // alnum vs ALL "AB" -> cob_cmp_all value (repeat compare equal)
        assert_eq!(cob_cmp(&alnum(b"ABAB"), &all(b"AB"), None), CmpResult::Value(0));
        // two alnum -> string compare, space padded
        assert_eq!(cob_cmp(&alnum(b"AB"), &alnum(b"AB  "), None), CmpResult::Value(0));
        // alnum vs unsigned DISPLAY numeric -> alnum compare
        match cob_cmp(&alnum(b"123"), &num(b"123"), None) {
            CmpResult::Value(0) => {}
            other => panic!("expected Value(0), got {other:?}"),
        }
    }
}
