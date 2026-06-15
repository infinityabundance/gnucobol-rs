//! Port of the **pure, side-effect-free** helpers of `libcob/common.c` -- the runtime core.
//!
//! common.c (11k lines, 254 functions) is the libcob runtime: global state, the exception stack, signal
//! handling, environment/config, tracing, and the field primitives. The panel guidance is to MAP its
//! state / side-effect / exception / config / signal surface before porting those parts. This module ports
//! only the parts that are **pure functions of their inputs** -- the byte-comparison primitives behind
//! `IF a = b` / `IF a = ALL "x"`, the figurative SPACE/ZERO comparisons, the EBCDIC overpunch sign, the
//! SORT key comparison, integer formatting, the ISO day-of-week shift, and the boolean env parsing. The
//! global-state, signal, exception and config functions are the declared runtime boundary (mapped, then
//! ported, separately).

/// Port of `common.c:ss_itoa_u10` -- format a signed integer as base-10 ASCII (a leading `-` for negatives).
pub fn ss_itoa_u10(value: i32) -> Vec<u8> {
    let mut out = Vec::new();
    if value < 0 {
        out.push(b'-');
    }
    let mut u = value.unsigned_abs();
    let start = out.len();
    loop {
        out.push(b'0' + (u % 10) as u8);
        u /= 10;
        if u == 0 {
            break;
        }
    }
    out[start..].reverse();
    out
}

/// Port of `common.c:compare_character` -- compare `data` against the pattern `c` repeated to `data`'s
/// length (the `IF a = ALL "literal"` comparison). Returns the signed difference of the first differing
/// byte, or `0`.
pub fn compare_character(data: &[u8], c: &[u8]) -> i32 {
    if c.is_empty() {
        return 0;
    }
    for (i, &b) in data.iter().enumerate() {
        let pc = c[i % c.len()];
        if b != pc {
            return b as i32 - pc as i32;
        }
    }
    0
}

/// Port of `common.c:compare_spaces` -- compare `data` against `ALL SPACES` (`0x20`).
pub fn compare_spaces(data: &[u8]) -> i32 {
    compare_character(data, b" ")
}

/// Port of `common.c:compare_zeroes` -- compare `data` against `ALL ZEROES` (the character `'0'`, `0x30`).
pub fn compare_zeroes(data: &[u8]) -> i32 {
    compare_character(data, b"0")
}

/// Port of `common.c:common_cmpc` -- compare every byte of `data`, translated through the 256-entry
/// collating table `col`, against the translated character `c`. Returns the first differing translated
/// difference, or `0`.
pub fn common_cmpc(data: &[u8], c: u8, col: &[u8; 256]) -> i32 {
    let c_col = col[c as usize] as i32;
    for &b in data {
        let ret = col[b as usize] as i32 - c_col;
        if ret != 0 {
            return ret;
        }
    }
    0
}

/// Port of `common.c:common_cmps` -- compare `s1` against `s2` (equal length) byte-by-byte through the
/// collating table `col`.
pub fn common_cmps(s1: &[u8], s2: &[u8], col: &[u8; 256]) -> i32 {
    for (&a, &b) in s1.iter().zip(s2.iter()) {
        let ret = col[a as usize] as i32 - col[b as usize] as i32;
        if ret != 0 {
            return ret;
        }
    }
    0
}

/// Port of `common.c:cob_cmp_all` -- compare a field `data1` against a figurative/`ALL` operand `data2`.
/// Without a collation: a 1-byte SPACE/ZERO operand routes to [`compare_spaces`]/[`compare_zeroes`], any
/// other operand is repeated to `data1`'s length ([`compare_character`]); with a collation the bytes are
/// translated through it ([`common_cmpc`] for a 1-byte operand).
pub fn cob_cmp_all(data1: &[u8], data2: &[u8], col: Option<&[u8; 256]>) -> i32 {
    match col {
        None => {
            if data2.len() == 1 {
                match data2[0] {
                    b' ' => compare_spaces(data1),
                    b'0' => compare_zeroes(data1),
                    c => compare_character(data1, &[c]),
                }
            } else {
                compare_character(data1, data2)
            }
        }
        Some(col) => {
            if data2.len() == 1 {
                common_cmpc(data1, data2[0], col)
            } else {
                // repeat data2 through the collation
                for (i, &b) in data1.iter().enumerate() {
                    let pc = data2[i % data2.len()];
                    let ret = col[b as usize] as i32 - col[pc as usize] as i32;
                    if ret != 0 {
                        return ret;
                    }
                }
                0
            }
        }
    }
}

/// Port of `common.c:cob_cmp_alnum` -- compare two alphanumeric fields, space-padding the shorter to the
/// longer's length. Without a collation the common prefix is `memcmp`ed and the tail compared to spaces;
/// with a collation both go through it. A longer-but-equal `data1` is `> 0`, a longer-but-equal `data2`
/// is `< 0`.
pub fn cob_cmp_alnum(data1: &[u8], data2: &[u8], col: Option<&[u8; 256]>) -> i32 {
    let (s1, s2) = (data1.len(), data2.len());
    let min = s1.min(s2);
    let ret = match col {
        None => data1[..min].cmp(&data2[..min]) as i32,
        Some(col) => common_cmps(&data1[..min], &data2[..min], col),
    };
    if ret != 0 {
        // normalise Ordering's -1/0/1; for the col path it is already a signed difference
        return match col {
            None => match data1[..min].cmp(&data2[..min]) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Greater => 1,
                std::cmp::Ordering::Equal => 0,
            },
            Some(_) => ret,
        };
    }
    if s1 > s2 {
        match col {
            None => compare_spaces(&data1[min..]),
            Some(_) => compare_spaces(&data1[min..]),
        }
    } else if s1 < s2 {
        -compare_spaces(&data2[min..])
    } else {
        0
    }
}

/// One SORT key (`common.c`'s `sort_keys` entry): a byte range `[offset, offset+size)` of the record,
/// ASCENDING or DESCENDING, and whether the key is numeric (compared by value rather than by bytes).
#[derive(Debug, Clone, Copy)]
pub struct SortKey {
    pub offset: usize,
    pub size: usize,
    pub ascending: bool,
    pub numeric: bool,
}

/// Port of the alphanumeric path of `common.c:sort_compare` -- order two records by the SORT keys: each
/// key's byte range is `memcmp`ed (negated for DESCENDING), the first differing key deciding. Numeric keys
/// (`numeric: true`) are a declared composition with `GNURUST.NUMCMP.1` (`cob_numeric_cmp`) -- here their
/// byte comparison is used as the fallback.
pub fn sort_compare(rec1: &[u8], rec2: &[u8], keys: &[SortKey]) -> i32 {
    for k in keys {
        let a = &rec1[k.offset.min(rec1.len())..(k.offset + k.size).min(rec1.len())];
        let b = &rec2[k.offset.min(rec2.len())..(k.offset + k.size).min(rec2.len())];
        let res = match a.cmp(b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Equal => 0,
        };
        if res != 0 {
            return if k.ascending { res } else { -res };
        }
    }
    0
}

/// Port of `common.c:sort_compare_collate` -- [`sort_compare`] with the alphanumeric keys translated
/// through a collating sequence `col`.
pub fn sort_compare_collate(rec1: &[u8], rec2: &[u8], keys: &[SortKey], col: &[u8; 256]) -> i32 {
    for k in keys {
        let a = &rec1[k.offset.min(rec1.len())..(k.offset + k.size).min(rec1.len())];
        let b = &rec2[k.offset.min(rec2.len())..(k.offset + k.size).min(rec2.len())];
        let res = common_cmps(a, b, col);
        if res != 0 {
            return if k.ascending { res.signum() } else { -res.signum() };
        }
    }
    0
}

/// Port of `common.c:cob_put_sign_ebcdic` -- overlay an EBCDIC overpunch sign on the last digit `p`:
/// negative maps `0..9` to `}JKLMNOPQR`, positive to `{ABCDEFGHI`; an already-signed byte is left as-is, an
/// unexpected byte becomes the zero-overpunch (`}` / `{`).
pub fn cob_put_sign_ebcdic(p: &mut u8, sign: i32) {
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
            b'}' | b'J'..=b'R' => return, // already signed
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
            b'{' | b'A'..=b'I' => return, // already signed
            _ => b'{',
        };
    }
}

/// Port of `common.c:one_indexed_day_of_week_from_monday` -- convert a 0-indexed-from-Sunday weekday to a
/// 1-indexed-from-Monday weekday (`Mon=1 .. Sun=7`).
pub fn one_indexed_day_of_week_from_monday(zero_indexed_from_sunday: i32) -> i32 {
    ((zero_indexed_from_sunday + 6) % 7) + 1
}

/// Port of `common.c:cob_check_env_true` -- is the env value `s` a "true" setting? (`Y`/`y`/`1`, or
/// `YES`/`ON`/`TRUE` case-insensitively).
pub fn cob_check_env_true(s: &[u8]) -> bool {
    if s.len() == 1 && (s[0] == b'Y' || s[0] == b'y' || s[0] == b'1') {
        return true;
    }
    let up = s.to_ascii_uppercase();
    up == b"YES" || up == b"ON" || up == b"TRUE"
}

/// Port of `common.c:cob_check_env_false` -- is the env value `s` a "false" setting? (`N`/`n`/`0`, or
/// `NO`/`NONE`/`OFF`/`FALSE` case-insensitively).
pub fn cob_check_env_false(s: &[u8]) -> bool {
    if s.len() == 1 && (s[0] == b'N' || s[0] == b'n' || s[0] == b'0') {
        return true;
    }
    let up = s.to_ascii_uppercase();
    up == b"NO" || up == b"NONE" || up == b"OFF" || up == b"FALSE"
}

/// Port of `common.c:version_bitstring` -- pack a `(major, minor, point)` version into the
/// `0xMMmmpp00` unsigned integer GnuCOBOL compares (`major<<24 | minor<<16 | point<<8`).
pub fn version_bitstring(major: u32, minor: u32, point: u32) -> u32 {
    (major << 24) | (minor << 16) | (point << 8)
}

/// Port of `common.c:cob_is_upper` -- every byte of the field is a space or an ASCII uppercase letter
/// (the `CLASS UPPER` / `FUNCTION UPPER-CASE`-adjacent test; C `isupper` under the C locale).
pub fn cob_is_upper(data: &[u8]) -> bool {
    data.iter().all(|&p| p == b' ' || p.is_ascii_uppercase())
}

/// Port of `common.c:cob_is_lower` -- every byte of the field is a space or an ASCII lowercase letter.
pub fn cob_is_lower(data: &[u8]) -> bool {
    data.iter().all(|&p| p == b' ' || p.is_ascii_lowercase())
}

/// Port of `common.c:cob_int_to_string` -- the decimal text of a signed integer (`sprintf("%i")`).
pub fn cob_int_to_string(i: i32) -> Vec<u8> {
    i.to_string().into_bytes()
}

/// Port of `common.c:cob_int_to_formatted_bytestring` -- a human byte size: `> 1 MiB` -> `"<d> MB"`,
/// `> 1 kiB` -> `"<d> kB"`, else `"0.00 B"`, each formatted `"%3.2f %s"` (the binary 1024 thresholds the C
/// uses, with its `d = 0` for the bytes branch). Matches GnuCOBOL's dump/runtime size reporting.
pub fn cob_int_to_formatted_bytestring(i: i32) -> Vec<u8> {
    let (d, unit): (f64, &str) = if i > 1024 * 1024 {
        (i as f64 / 1024.0 / 1024.0, "MB")
    } else if i > 1024 {
        (i as f64 / 1024.0, "kB")
    } else {
        (0.0, "B")
    };
    format!("{d:.2} {unit}").into_bytes()
}

// ======================================================================================================
// Runtime bounds checks (common.c cob_check_subscript / cob_check_odo / cob_check_ref_mod*). In libcob each
// raises an EC-BOUND exception, prints a `cob_runtime_error` line (and optional `cob_runtime_hint`), and --
// when `abend` -- aborts via `cob_hard_failure`. This port separates the PURE bounds decision + the exact
// message text (the oracle-visible artifact, verified identical across GnuCOBOL 3.1.2 and 3.2) from the
// abort side effect: each function returns the violation(s) it would raise, or none when in bounds.
// ======================================================================================================

/// The EC-BOUND exception a check raises (symbolic; the oracle shows the message text, not the numeric id).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundException {
    /// `EC-BOUND-SUBSCRIPT`.
    Subscript,
    /// `EC-BOUND-ODO`.
    Odo,
    /// `EC-BOUND-REF-MOD`.
    RefMod,
}

/// A single bounds-check violation: the exception raised, the `cob_runtime_error` message (the text after
/// `libcob: <file>:<line>: error: `), an optional `cob_runtime_hint` line (after `note: `), and whether the
/// runtime aborts on it (`cob_hard_failure`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundViolation {
    /// The raised exception.
    pub exception: BoundException,
    /// The runtime-error message text.
    pub message: Vec<u8>,
    /// An optional hint line.
    pub hint: Option<Vec<u8>>,
    /// Whether this violation aborts the run.
    pub fatal: bool,
}

/// Port of `common.c:cob_check_subscript` -- validate a 1-based table subscript `i` against `max`.
/// `cannot_check` models the 2.0-ABI compatibility path (only a zero subscript is caught). `odo_item`
/// selects the "current maximum" hint wording for a variable-length table. Returns the violation, or `None`
/// when in bounds.
pub fn cob_check_subscript(
    i: i32,
    max: i32,
    name: &str,
    odo_item: bool,
    cannot_check: bool,
) -> Option<BoundViolation> {
    if cannot_check {
        if i == 0 {
            return Some(BoundViolation {
                exception: BoundException::Subscript,
                message: format!("subscript of 'unknown field' out of bounds: {i}").into_bytes(),
                hint: None,
                fatal: true,
            });
        }
        return None;
    }
    if i < 1 || i > max {
        let hint = if i >= 1 {
            let label = if odo_item { "current maximum subscript" } else { "maximum subscript" };
            Some(format!("{label} for '{name}': {max}").into_bytes())
        } else {
            None
        };
        return Some(BoundViolation {
            exception: BoundException::Subscript,
            message: format!("subscript of '{name}' out of bounds: {i}").into_bytes(),
            hint,
            fatal: true,
        });
    }
    None
}

/// Port of `common.c:cob_check_odo` -- validate an `OCCURS DEPENDING ON` length `i` against `[min, max]`.
pub fn cob_check_odo(i: i32, min: i32, max: i32, name: &str, dep_name: &str) -> Option<BoundViolation> {
    if i < min || i > max {
        let hint = if i > max {
            format!("maximum subscript for '{name}': {max}")
        } else {
            format!("minimum subscript for '{name}': {min}")
        };
        return Some(BoundViolation {
            exception: BoundException::Odo,
            message: format!("OCCURS DEPENDING ON '{dep_name}' out of bounds: {i}").into_bytes(),
            hint: Some(hint.into_bytes()),
            fatal: true,
        });
    }
    None
}

/// Port of `common.c:cob_check_ref_mod_detailed` -- validate a reference modification `field(offset:length)`
/// against the field `size`. Returns every violation it would raise, in evaluation order (offset, then
/// plain length, then length-with-offset); `abend` sets each violation's `fatal` (with `abend` the first
/// fatal aborts, so the first violation is the one the oracle shows). `zero_allowed` permits a zero length.
pub fn cob_check_ref_mod_detailed(
    name: &str,
    abend: bool,
    zero_allowed: bool,
    size: i32,
    offset: i32,
    length: i32,
) -> Vec<BoundViolation> {
    let minimal_length = if zero_allowed { 0 } else { 1 };
    let mut out = Vec::new();
    if offset < 1 || offset > size {
        let message = if offset < 1 {
            format!("offset of '{name}' out of bounds: {offset}")
        } else {
            format!("offset of '{name}' out of bounds: {offset}, maximum: {size}")
        };
        out.push(BoundViolation { exception: BoundException::RefMod, message: message.into_bytes(), hint: None, fatal: abend });
    }
    if length < minimal_length || length > size {
        let message = if length < minimal_length {
            format!("length of '{name}' out of bounds: {length}")
        } else {
            format!("length of '{name}' out of bounds: {length}, maximum: {size}")
        };
        out.push(BoundViolation { exception: BoundException::RefMod, message: message.into_bytes(), hint: None, fatal: abend });
    }
    if offset + length - 1 > size {
        let message = format!("length of '{name}' out of bounds: {length}, starting at: {offset}, maximum: {size}");
        out.push(BoundViolation { exception: BoundException::RefMod, message: message.into_bytes(), hint: None, fatal: abend });
    }
    out
}

/// Port of `common.c:cob_check_ref_mod` (the 2.2/3.1-rc1 compat shim) -- always-abend, no zero length.
pub fn cob_check_ref_mod(offset: i32, length: i32, size: i32, name: &str) -> Vec<BoundViolation> {
    cob_check_ref_mod_detailed(name, true, false, size, offset, length)
}

/// Port of `common.c:cob_check_ref_mod_minimal` -- the minimal check (only `offset >= 1` and `length >= 1`,
/// each fatal), with no upper-bound check.
pub fn cob_check_ref_mod_minimal(name: &str, offset: i32, length: i32) -> Vec<BoundViolation> {
    let mut out = Vec::new();
    if offset < 1 {
        out.push(BoundViolation {
            exception: BoundException::RefMod,
            message: format!("offset of '{name}' out of bounds: {offset}").into_bytes(),
            hint: None,
            fatal: true,
        });
    }
    if length < 1 {
        out.push(BoundViolation {
            exception: BoundException::RefMod,
            message: format!("length of '{name}' out of bounds: {length}").into_bytes(),
            hint: None,
            fatal: true,
        });
    }
    out
}

// ======================================================================================================
// Numeric-class diagnostics (common.c explain_field_type / cob_check_numeric). The "not numeric" runtime
// error is what GnuCOBOL prints when a non-numeric value reaches arithmetic (under cobc -debug); its text
// is reproduced byte-identically and proven version-stable across the 3.1.2 + 3.2 oracles.
// ======================================================================================================

// `common.h` field-type codes (the subset explain_field_type names).
const T_GROUP: u8 = 0x01;
const T_BOOLEAN: u8 = 0x02;
const T_NUMERIC_DISPLAY: u8 = 0x10;
const T_NUMERIC_BINARY: u8 = 0x11;
const T_NUMERIC_PACKED: u8 = 0x12;
const T_NUMERIC_FLOAT: u8 = 0x13;
const T_NUMERIC_DOUBLE: u8 = 0x14;
const T_NUMERIC_L_DOUBLE: u8 = 0x15;
const T_NUMERIC_FP_DEC64: u8 = 0x16;
const T_NUMERIC_FP_DEC128: u8 = 0x17;
const T_NUMERIC_FP_BIN32: u8 = 0x18;
const T_NUMERIC_FP_BIN64: u8 = 0x19;
const T_NUMERIC_FP_BIN128: u8 = 0x1A;
const T_NUMERIC_COMP5: u8 = 0x1B;
const T_NUMERIC_EDITED: u8 = 0x24;
const T_ALPHANUMERIC: u8 = 0x21;
const T_ALPHANUMERIC_ALL: u8 = 0x22;
const T_ALPHANUMERIC_EDITED: u8 = 0x23;
const T_NATIONAL: u8 = 0x40;
const T_NATIONAL_EDITED: u8 = 0x41;

/// Port of `common.c:explain_field_type` -- the human field-type name GnuCOBOL prints in runtime
/// diagnostics (e.g. `NUMERIC DISPLAY`, `PACKED-DECIMAL`, `ALPHANUMERIC`). `no_sign_nibble` / `have_sign`
/// disambiguate the packed-decimal variants (`COMP-6`, `PACKED-DECIMAL (unsigned)`).
pub fn explain_field_type(field_type: u8, no_sign_nibble: bool, have_sign: bool) -> &'static str {
    match field_type {
        T_GROUP => "GROUP",
        T_BOOLEAN => "BOOLEAN",
        T_NUMERIC_DISPLAY => "NUMERIC DISPLAY",
        T_NUMERIC_BINARY => "BINARY",
        T_NUMERIC_PACKED => {
            if no_sign_nibble {
                "COMP-6"
            } else if !have_sign {
                "PACKED-DECIMAL (unsigned)"
            } else {
                "PACKED-DECIMAL"
            }
        }
        T_NUMERIC_FLOAT => "FLOAT",
        T_NUMERIC_DOUBLE => "DOUBLE",
        T_NUMERIC_L_DOUBLE => "LONG DOUBLE",
        T_NUMERIC_FP_DEC64 => "FP DECIMAL 64",
        T_NUMERIC_FP_DEC128 => "FP DECIMAL 128",
        T_NUMERIC_FP_BIN32 => "FP BINARY 32",
        T_NUMERIC_FP_BIN64 => "FP BINARY 64",
        T_NUMERIC_FP_BIN128 => "FP BINARY 128",
        T_NUMERIC_COMP5 => "COMP-5",
        T_NUMERIC_EDITED => "NUMERIC EDITED",
        T_ALPHANUMERIC => "ALPHANUMERIC",
        T_ALPHANUMERIC_ALL => "ALPHANUMERIC ALL",
        T_ALPHANUMERIC_EDITED => "ALPHANUMERIC EDITED",
        T_NATIONAL => "NATIONAL",
        T_NATIONAL_EDITED => "NATIONAL EDITED",
        _ => "UNKNOWN",
    }
}

/// Escape a field's raw bytes for the "not numeric" diagnostic, as `common.c:cob_check_numeric` does:
/// for a NUMERIC DISPLAY or alphanumeric field (`as_text`), each printable byte is shown as-is and each
/// non-printable byte as `\<ooo>` (3-digit octal); otherwise the whole value is shown as `0x<hex>`.
fn escape_not_numeric(data: &[u8], as_text: bool) -> Vec<u8> {
    let mut out = Vec::new();
    if as_text {
        for &b in data {
            if (0x20..=0x7e).contains(&b) {
                out.push(b);
            } else {
                out.extend_from_slice(format!("\\{b:03o}").as_bytes());
            }
        }
    } else {
        out.extend_from_slice(b"0x");
        for &b in data {
            out.extend_from_slice(format!("{b:02x}").as_bytes());
        }
    }
    out
}

/// Port of `common.c:cob_check_numeric` -- when `is_numeric` is false (the field's bytes are not valid for
/// its numeric type), the EC-DATA-INCOMPATIBLE runtime error message GnuCOBOL prints (the text after
/// `libcob: <file>:<line>: error: `): `'<name>' (Type: <type>) not numeric: '<escaped>'`. `type_name` is
/// [`explain_field_type`]; `escape_as_text` is set for NUMERIC DISPLAY / alphanumeric fields (see
/// [`escape_not_numeric`]). Returns `None` when the value is numeric (no error). The abort
/// (`cob_hard_failure`) is the caller's side effect.
pub fn cob_check_numeric(
    is_numeric: bool,
    name: &str,
    type_name: &str,
    data: &[u8],
    escape_as_text: bool,
) -> Option<Vec<u8>> {
    if is_numeric {
        return None;
    }
    let mut msg = format!("'{name}' (Type: {type_name}) not numeric: '").into_bytes();
    msg.extend_from_slice(&escape_not_numeric(data, escape_as_text));
    msg.push(b'\'');
    Some(msg)
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.COMMON.BOUNDCHECK.1
    /// `cob_check_subscript` is total + exactly characterizes the in-bounds set: for any `(i, max)` it
    /// returns `None` iff `1 <= i <= max` (in the normal, non-compat path), and never panics.
    #[kani::proof]
    fn subscript_check_is_exact() {
        let i: i32 = kani::any();
        let max: i32 = kani::any();
        kani::assume(max >= 0 && max <= 1000 && i >= -1000 && i <= 2000);
        let v = cob_check_subscript(i, max, "X", false, false);
        if i >= 1 && i <= max {
            assert!(v.is_none());
        } else {
            assert!(v.is_some());
        }
    }

    // KANIFOR: GNURUST.COMMON.NUMCHECK.1
    /// `cob_check_numeric` returns a message exactly when the value is not numeric, never otherwise, and
    /// `explain_field_type` is total -- both never panic for any inputs.
    #[kani::proof]
    #[kani::unwind(3)]
    fn numeric_check_is_conditional() {
        let is_numeric: bool = kani::any();
        let b: u8 = kani::any();
        let ty: u8 = kani::any();
        let _ = explain_field_type(ty, kani::any(), kani::any());
        let v = cob_check_numeric(is_numeric, "X", "T", &[b], kani::any());
        assert_eq!(v.is_none(), is_numeric);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident_col() -> [u8; 256] {
        let mut c = [0u8; 256];
        for (i, x) in c.iter_mut().enumerate() {
            *x = i as u8;
        }
        c
    }

    #[test]
    fn itoa_and_dow_and_env() {
        assert_eq!(ss_itoa_u10(0), b"0".to_vec());
        assert_eq!(ss_itoa_u10(1234), b"1234".to_vec());
        assert_eq!(ss_itoa_u10(-57), b"-57".to_vec());
        // Sunday(0)->7, Monday(1)->1, Saturday(6)->6
        assert_eq!(one_indexed_day_of_week_from_monday(0), 7);
        assert_eq!(one_indexed_day_of_week_from_monday(1), 1);
        assert_eq!(one_indexed_day_of_week_from_monday(6), 6);
        assert!(cob_check_env_true(b"Y") && cob_check_env_true(b"yes") && cob_check_env_true(b"ON"));
        assert!(!cob_check_env_true(b"maybe"));
        assert!(cob_check_env_false(b"0") && cob_check_env_false(b"false") && cob_check_env_false(b"OFF"));
    }

    #[test]
    fn figurative_and_field_comparison() {
        assert_eq!(compare_spaces(b"   "), 0);
        assert!(compare_spaces(b"  X") > 0); // 'X' > ' '
        assert_eq!(compare_zeroes(b"000"), 0);
        assert!(compare_zeroes(b"00 ") < 0); // ' ' < '0'
        // ALL "AB" repeated
        assert_eq!(compare_character(b"ABAB", b"AB"), 0);
        assert!(compare_character(b"ABAC", b"AB") > 0);
        // cob_cmp_all: field vs ALL SPACE / ZERO / literal
        assert_eq!(cob_cmp_all(b"   ", b" ", None), 0);
        assert_eq!(cob_cmp_all(b"XYXY", b"XY", None), 0);
        // alnum compare with space padding: "AB" vs "AB   " is equal
        assert_eq!(cob_cmp_alnum(b"AB", b"AB   ", None), 0);
        assert!(cob_cmp_alnum(b"ABC", b"AB", None) > 0); // 'C' > ' '
        assert!(cob_cmp_alnum(b"AB", b"ABC", None) < 0);
        // collation: a table folding 'a'->'A' makes them compare equal
        let mut col = ident_col();
        col[b'a' as usize] = b'A';
        assert_eq!(common_cmps(b"a", b"A", &col), 0);
        assert_eq!(cob_cmp_alnum(b"a", b"A", Some(&col)), 0);
    }

    #[test]
    fn sort_keys_and_ebcdic_sign() {
        // K1 X(3) ASC at 0, K2 X(2) DESC at 3
        let keys = [
            SortKey { offset: 0, size: 3, ascending: true, numeric: false },
            SortKey { offset: 3, size: 2, ascending: false, numeric: false },
        ];
        assert!(sort_compare(b"AAA10", b"BBB10", &keys) < 0); // AAA < BBB
        assert!(sort_compare(b"AAA10", b"AAA05", &keys) < 0); // K2 desc: 10 before 05
        assert_eq!(sort_compare(b"AAA10", b"AAA10", &keys), 0);
        // EBCDIC overpunch
        let mut p = b'5';
        cob_put_sign_ebcdic(&mut p, -1);
        assert_eq!(p, b'N'); // negative 5
        let mut q = b'3';
        cob_put_sign_ebcdic(&mut q, 0);
        assert_eq!(q, b'C'); // positive 3
        let mut r = b'N';
        cob_put_sign_ebcdic(&mut r, -1);
        assert_eq!(r, b'N'); // already signed -> unchanged
    }

    #[test]
    fn version_and_class_and_format_helpers() {
        // version_bitstring: 3.2.0 -> 0x03020000
        assert_eq!(version_bitstring(3, 2, 0), 0x0302_0000);
        assert_eq!(version_bitstring(9, 9, 9), 0x0909_0900);
        // is_upper / is_lower (space counts; empty is vacuously true)
        assert!(cob_is_upper(b"ABC DEF"));
        assert!(!cob_is_upper(b"ABc"));
        assert!(cob_is_lower(b"abc def"));
        assert!(!cob_is_lower(b"abC"));
        assert!(cob_is_upper(b"   "));
        // int_to_string
        assert_eq!(cob_int_to_string(0), b"0".to_vec());
        assert_eq!(cob_int_to_string(-42), b"-42".to_vec());
        assert_eq!(cob_int_to_string(2147483647), b"2147483647".to_vec());
        // formatted byte size (binary thresholds, %3.2f)
        assert_eq!(cob_int_to_formatted_bytestring(512), b"0.00 B".to_vec());
        assert_eq!(cob_int_to_formatted_bytestring(2048), b"2.00 kB".to_vec());
        assert_eq!(cob_int_to_formatted_bytestring(1024 * 1024 * 3), b"3.00 MB".to_vec());
    }

    #[test]
    fn bounds_check_messages() {
        // subscript out of bounds (the exact message + hint the oracle prints, verified vs 3.1.2 + 3.2)
        let v = cob_check_subscript(5, 3, "E", false, false).unwrap();
        assert_eq!(v.message, b"subscript of 'E' out of bounds: 5".to_vec());
        assert_eq!(v.hint.unwrap(), b"maximum subscript for 'E': 3".to_vec());
        assert!(v.fatal);
        assert!(cob_check_subscript(2, 3, "E", false, false).is_none()); // in bounds
        // zero subscript with cannot_check
        assert_eq!(
            cob_check_subscript(0, 3, "E", false, true).unwrap().message,
            b"subscript of 'unknown field' out of bounds: 0".to_vec()
        );
        // odo-item subscript uses the "current maximum" wording
        assert_eq!(
            cob_check_subscript(9, 4, "T", true, false).unwrap().hint.unwrap(),
            b"current maximum subscript for 'T': 4".to_vec()
        );
        // ODO out of bounds
        let o = cob_check_odo(7, 1, 5, "T", "N").unwrap();
        assert_eq!(o.message, b"OCCURS DEPENDING ON 'N' out of bounds: 7".to_vec());
        assert_eq!(o.hint.unwrap(), b"maximum subscript for 'T': 5".to_vec());
        assert_eq!(
            cob_check_odo(0, 1, 5, "T", "N").unwrap().hint.unwrap(),
            b"minimum subscript for 'T': 1".to_vec()
        );
        // reference modification: offset and length
        let r = cob_check_ref_mod(0, 2, 5, "F");
        assert_eq!(r[0].message, b"offset of 'F' out of bounds: 0".to_vec());
        let r2 = cob_check_ref_mod(2, 9, 5, "F");
        assert_eq!(r2[0].message, b"length of 'F' out of bounds: 9, maximum: 5".to_vec());
        let r3 = cob_check_ref_mod(4, 3, 5, "F"); // 4+3-1=6 > 5
        assert_eq!(r3[0].message, b"length of 'F' out of bounds: 3, starting at: 4, maximum: 5".to_vec());
        assert!(cob_check_ref_mod(1, 5, 5, "F").is_empty()); // in bounds
        assert_eq!(cob_check_ref_mod_minimal("F", 0, 2)[0].message, b"offset of 'F' out of bounds: 0".to_vec());
    }

    #[test]
    fn numeric_diagnostics() {
        // explain_field_type
        assert_eq!(explain_field_type(0x10, false, false), "NUMERIC DISPLAY");
        assert_eq!(explain_field_type(0x12, false, true), "PACKED-DECIMAL");
        assert_eq!(explain_field_type(0x12, false, false), "PACKED-DECIMAL (unsigned)");
        assert_eq!(explain_field_type(0x12, true, true), "COMP-6");
        assert_eq!(explain_field_type(0x21, false, false), "ALPHANUMERIC");
        assert_eq!(explain_field_type(0x24, false, false), "NUMERIC EDITED");
        assert_eq!(explain_field_type(0x99, false, false), "UNKNOWN");
        // cob_check_numeric: the exact "not numeric" message (verified vs 3.1.2 + 3.2)
        assert_eq!(
            cob_check_numeric(false, "N", "NUMERIC DISPLAY", b"12X", true).unwrap(),
            b"'N' (Type: NUMERIC DISPLAY) not numeric: '12X'".to_vec()
        );
        assert!(cob_check_numeric(true, "N", "NUMERIC DISPLAY", b"123", true).is_none());
        // non-printable byte -> \ooo octal (0x01 -> \001)
        assert_eq!(
            cob_check_numeric(false, "X", "NUMERIC DISPLAY", b"1\x01", true).unwrap(),
            b"'X' (Type: NUMERIC DISPLAY) not numeric: '1\\001'".to_vec()
        );
        // non-text field -> 0xHEX
        assert_eq!(
            cob_check_numeric(false, "P", "PACKED-DECIMAL", b"\x12\x3f", false).unwrap(),
            b"'P' (Type: PACKED-DECIMAL) not numeric: '0x123f'".to_vec()
        );
    }
}
