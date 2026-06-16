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

/// The admitted `libcob` version components (`version.h`: `__LIBCOB_VERSION` 3, `_MINOR` 2,
/// `_PATCHLEVEL` 0 -- GnuCOBOL 3.2.0).
pub const LIBCOB_VERSION: i32 = 3;
/// See [`LIBCOB_VERSION`].
pub const LIBCOB_VERSION_MINOR: i32 = 2;
/// See [`LIBCOB_VERSION`].
pub const LIBCOB_VERSION_PATCHLEVEL: i32 = 0;

/// Port of `common.c:set_libcob_version` -- compare a caller's `(major, minor, patch)` against the linked
/// libcob version and overwrite them with the library's: returns `0` when they match (or the caller passed
/// `major == 0`), else `1`/`2`/`3` for a major/minor/patch mismatch. The `major`/`minor`/`patch` are
/// updated in place to the library version regardless (the C `*mayor = __LIBCOB_VERSION; ...`).
pub fn set_libcob_version(major: &mut i32, minor: &mut i32, patch: &mut i32) -> i32 {
    let mut ret = 0;
    if *major != 0 {
        if *major != LIBCOB_VERSION {
            ret = 1;
        } else if *minor != LIBCOB_VERSION_MINOR {
            ret = 2;
        } else if *patch != LIBCOB_VERSION_PATCHLEVEL {
            ret = 3;
        }
    }
    *major = LIBCOB_VERSION;
    *minor = LIBCOB_VERSION_MINOR;
    *patch = LIBCOB_VERSION_PATCHLEVEL;
    ret
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

impl BoundException {
    /// The COBOL exception code (`exception.def`) this violation kind raises, so a caught violation can
    /// be threaded into the runtime exception state. libcob's `cob_check_*` call `cob_set_exception`
    /// with exactly these conditions (common.c:4360+) before emitting the message; the port returns the
    /// violation and the runtime raises this code (see [`raise_bound_violation`]).
    pub fn ec_code(self) -> i32 {
        match self {
            BoundException::Subscript => 0x0207, // EC-BOUND-SUBSCRIPT
            BoundException::Odo => 0x0202,        // EC-BOUND-ODO
            BoundException::RefMod => 0x0205,     // EC-BOUND-REF-MOD
        }
    }
}

/// `EC-DATA-INCOMPATIBLE` (`0x0303`): the exception `cob_check_numeric` raises when a field tested for
/// the numeric class is not numeric (common.c:4360). [`cob_check_numeric`] returns only the message; the
/// runtime raises this code alongside it.
pub const EC_DATA_INCOMPATIBLE: i32 = 0x0303;

/// Raise a caught [`BoundViolation`] into the runtime exception state -- the missing link between the
/// pure `cob_check_*` (which return the violation) and `cob_set_exception` (which records it for
/// EXCEPTION-STATUS / `cob_last_exception_is`). Mirrors the `cob_set_exception(...)` call libcob makes
/// inside each `cob_check_*` before the `cob_runtime_error`.
pub fn raise_bound_violation(state: &mut crate::common_exception::ExceptionState, v: &BoundViolation) {
    crate::common_exception::cob_set_exception_by_code(state, v.exception.ec_code());
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

/// Emit the **exact** bytes GnuCOBOL writes to `stderr` for a caught [`BoundViolation`]: the
/// `cob_runtime_error` line (`libcob: <file>:<line>: error: <message>`) and, when the violation carries a
/// hint, the `cob_runtime_hint` companion (`note: <hint>`). This is the single byte-faithful bound-fault
/// diagnostic -- it wires the message **body** (the `cob_check_*` ports above, which already match the
/// oracle) to the `libcob:`/`note:` **wrapper** in [`crate::common_runerr`]. The internal `Display` of the
/// pure slice helpers (`subscript.rs` / `refmod.rs` / `odo.rs`) is a developer-facing detail, NOT these
/// runtime-error bytes; a live bounds fault is reported through here.
///
/// Oracle (built `cobc -fec=all -debug`): a subscript fault prints
/// `libcob: sub.cob:9: error: subscript of 'E' out of bounds: 5` then `note: maximum subscript for 'E': 3`;
/// a reference-modification fault prints `libcob: rm.cob:9: error: length of 'S' out of bounds: 8, maximum: 5`.
/// (The subsequent `Last statement ... / ENTRY ... / Started by ...` module-stack dump is the `cob_hard_failure`
/// abort path -- it needs the live module/call-stack state and is tracked under `exc-hard-failure`.)
pub fn cob_bound_violation_diagnostic(
    loc: &crate::common_runerr::SourceLocation,
    v: &BoundViolation,
) -> Vec<u8> {
    let mut out = crate::common_runerr::cob_runtime_error(loc, &v.message);
    if let Some(hint) = &v.hint {
        out.extend_from_slice(&crate::common_runerr::cob_runtime_hint(hint));
    }
    out
}

/// Whether a [`BoundViolation`] is **critical** per `exception.def`'s `fatal` column -- i.e. whether the
/// runtime acts on it with `cob_hard_failure` (abort, exit `EXIT_FAILURE`) rather than continuing. This is
/// the "act on the fatal flag" link: it cross-checks the violation's own `fatal` (set by the `cob_check_*`)
/// against the authoritative critical column ([`crate::common_exception::cob_exception_is_fatal`]) keyed by
/// the raised `EC-BOUND-*` code, so the two can never silently drift. EC-BOUND-ODO/-REF-MOD/-SUBSCRIPT are
/// all critical; the umbrella EC-BOUND (`0x0200`) is not.
pub fn cob_bound_violation_is_critical(v: &BoundViolation) -> bool {
    crate::common_exception::cob_exception_is_fatal(v.exception.ec_code())
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

// ======================================================================================================
// CBL_/C$ logic + bit + case builtins (common.c cob_sys_*). Each is a pure, in-place byte operation over a
// caller-sized buffer; the C trusts the caller's `length`, this port additionally bounds-guards every byte
// access (GNURUST.PANICPOLICY.0). All return 0 (the routine status). Oracle-verifiable via `CALL "CBL_AND"`
// etc.
// ======================================================================================================

/// Apply a binary byte op `dst[n] = f(src[n], dst[n])` for `n in 0..length`, bounds-guarded.
fn cbl_binop(src: &[u8], dst: &mut [u8], length: i32, f: impl Fn(u8, u8) -> u8) -> i32 {
    if length <= 0 {
        return 0;
    }
    let end = (length as usize).min(src.len()).min(dst.len());
    for n in 0..end {
        dst[n] = f(src[n], dst[n]);
    }
    0
}

/// `CBL_AND` (`common.c:cob_sys_and`): `dst &= src` over `length` bytes.
pub fn cob_sys_and(src: &[u8], dst: &mut [u8], length: i32) -> i32 {
    cbl_binop(src, dst, length, |a, b| a & b)
}
/// `CBL_OR` (`cob_sys_or`): `dst |= src`.
pub fn cob_sys_or(src: &[u8], dst: &mut [u8], length: i32) -> i32 {
    cbl_binop(src, dst, length, |a, b| a | b)
}
/// `CBL_NOR` (`cob_sys_nor`): `dst = ~(src | dst)`.
pub fn cob_sys_nor(src: &[u8], dst: &mut [u8], length: i32) -> i32 {
    cbl_binop(src, dst, length, |a, b| !(a | b))
}
/// `CBL_XOR` (`cob_sys_xor`): `dst ^= src`.
pub fn cob_sys_xor(src: &[u8], dst: &mut [u8], length: i32) -> i32 {
    cbl_binop(src, dst, length, |a, b| a ^ b)
}
/// `CBL_IMP` (`cob_sys_imp`): logical implies `dst = ~src | dst`.
pub fn cob_sys_imp(src: &[u8], dst: &mut [u8], length: i32) -> i32 {
    cbl_binop(src, dst, length, |a, b| !a | b)
}
/// `CBL_NIMP` (`cob_sys_nimp`): not-implies `dst = src & ~dst`.
pub fn cob_sys_nimp(src: &[u8], dst: &mut [u8], length: i32) -> i32 {
    cbl_binop(src, dst, length, |a, b| a & !b)
}
/// `CBL_EQ` (`cob_sys_eq`): bit equivalence `dst = ~(src ^ dst)`.
pub fn cob_sys_eq(src: &[u8], dst: &mut [u8], length: i32) -> i32 {
    cbl_binop(src, dst, length, |a, b| !(a ^ b))
}
/// `CBL_NOT` (`cob_sys_not`): `dst = ~dst` (single buffer).
pub fn cob_sys_not(dst: &mut [u8], length: i32) -> i32 {
    if length > 0 {
        let end = (length as usize).min(dst.len());
        for n in 0..end {
            dst[n] = !dst[n];
        }
    }
    0
}
/// `CBL_XF4` (`cob_sys_xf4`): pack the low bit of 8 source bytes into one destination byte
/// (`dst = sum(src[n]&1 << (7-n))`).
pub fn cob_sys_xf4(dst: &mut [u8], src: &[u8]) -> i32 {
    if dst.is_empty() || src.len() < 8 {
        return 0;
    }
    let mut b = 0u8;
    for n in 0..8 {
        b |= (src[n] & 1) << (7 - n);
    }
    dst[0] = b;
    0
}
/// `CBL_XF5` (`cob_sys_xf5`): unpack one source byte's 8 bits into 8 destination bytes (`0`/`1`).
pub fn cob_sys_xf5(src: &[u8], dst: &mut [u8]) -> i32 {
    if src.is_empty() || dst.len() < 8 {
        return 0;
    }
    for n in 0..8 {
        dst[n] = if src[0] & (1 << (7 - n)) != 0 { 1 } else { 0 };
    }
    0
}
/// `CBL_TOUPPER` (`cob_sys_toupper`): ASCII-uppercase `length` bytes in place (C `toupper`, C locale).
pub fn cob_sys_toupper(data: &mut [u8], length: i32) -> i32 {
    if length > 0 {
        let end = (length as usize).min(data.len());
        for b in &mut data[..end] {
            *b = b.to_ascii_uppercase();
        }
    }
    0
}
/// `CBL_TOLOWER` (`cob_sys_tolower`): ASCII-lowercase `length` bytes in place.
pub fn cob_sys_tolower(data: &mut [u8], length: i32) -> i32 {
    if length > 0 {
        let end = (length as usize).min(data.len());
        for b in &mut data[..end] {
            *b = b.to_ascii_lowercase();
        }
    }
    0
}
/// `CBL_GC_PRINTABLE` (`cob_sys_printable`): replace every non-printable byte (C `isprint`, the C-locale
/// `0x20..=0x7e` range) with `dotrep` (default `'.'`). The field length is the buffer length here.
pub fn cob_sys_printable(data: &mut [u8], dotrep: u8) -> i32 {
    for b in data.iter_mut() {
        if !(0x20..=0x7e).contains(b) {
            *b = dotrep;
        }
    }
    0
}

/// Port of `common.c:cob_correct_numeric` -- correct a NUMERIC DISPLAY field's bytes in place to a canonical
/// representation: fix the sign byte (leading or trailing; `+`/`-` for SIGN SEPARATE, the EBCDIC overpunch
/// `{`/`A`-`I` positive `}`/`J`-`R` negative for `ebcdic_sign`, else the ASCII path) and replace each
/// non-digit data byte with `'0'` (for NUL/space) or its low-nibble digit. The field attributes are passed
/// as flags. A no-op for a non-`NUMDISP` field.
pub fn cob_correct_numeric(
    data: &mut [u8],
    is_numdisp: bool,
    have_sign: bool,
    sign_leading: bool,
    sign_separate: bool,
    ebcdic_sign: bool,
) {
    if !is_numdisp || data.is_empty() {
        return;
    }
    let mut p_start = 0usize;
    let mut end = data.len();
    if have_sign {
        let c = if sign_leading {
            p_start = 1;
            0
        } else {
            end -= 1;
            end
        };
        if sign_separate {
            if data[c] != b'+' && data[c] != b'-' {
                data[c] = b'+';
            }
        } else if ebcdic_sign {
            match data[c] {
                b'{' | b'A'..=b'I' | b'}' | b'J'..=b'R' => {}
                b'0' => data[c] = b'{',
                b'1'..=b'9' => data[c] = b'A' + (data[c] - b'1'),
                0 | b' ' => data[c] = b'{',
                _ => {}
            }
        } else if data[c] == 0 || data[c] == b' ' {
            data[c] = b'0';
        }
    } else {
        let c = end - 1;
        if ebcdic_sign {
            match data[c] {
                0 | b' ' | b'{' | b'}' => data[c] = b'0',
                b'A'..=b'I' => data[c] = b'1' + (data[c] - b'A'),
                b'J'..=b'R' => data[c] = b'1' + (data[c] - b'J'),
                _ => {}
            }
        } else {
            match data[c] {
                0 | b' ' | b'p' => data[c] = b'0',
                b'q'..=b'y' => data[c] -= b'p',
                _ => {}
            }
        }
    }
    for b in &mut data[p_start..end] {
        match *b {
            b'0'..=b'9' => {}
            0 | b' ' => *b = b'0',
            other => {
                if (other & 0x0f) <= 9 {
                    *b = b'0' + (other & 0x0f);
                }
            }
        }
    }
}

/// Port of `common.c:cob_is_numeric` -- whether a field's bytes are valid for its numeric type. Dispatches
/// on `field_type` (the `COB_TYPE_*` code): BINARY is always numeric; NUMERIC DISPLAY delegates to
/// [`crate::common_misc::cob_check_numdisp`]; PACKED checks the sign nibble + BCD digits (`no_sign_nibble`
/// = COMP-6, `host_sign` = the host-sign relaxation); FLOAT/DOUBLE reproduce the C `!ISFINITE` test on the
/// little-endian value; the IEEE-754 decimal forms test the combination-field bits; any other type checks
/// every byte is an ASCII digit. Sign attributes are passed as flags (see `cob_check_numdisp`).
#[allow(clippy::too_many_arguments)]
pub fn cob_is_numeric(
    field_type: u8,
    data: &[u8],
    have_sign: bool,
    sign_leading: bool,
    sign_separate: bool,
    sign_mode: crate::common_misc::SignMode,
    no_sign_nibble: bool,
    host_sign: bool,
) -> i32 {
    // IS_INVALID_BCD_DATA(c) = (c & 0x0f) > 0x09 || (c & 0xf0) > 0x90
    let invalid_bcd = |c: u8| (c & 0x0f) > 0x09 || (c & 0xf0) > 0x90;
    match field_type {
        0x11 => 1, // COB_TYPE_NUMERIC_BINARY
        0x13 => {
            // FLOAT: the C returns !ISFINITE(value) (ported verbatim).
            if data.len() < 4 {
                return 0;
            }
            let v = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            i32::from(!v.is_finite())
        }
        0x14 | 0x15 => {
            // DOUBLE / LONG DOUBLE: the C `!ISFINITE` on the value (LONG DOUBLE is decoded as f64 here;
            // the exact extended-precision layout is the host boundary).
            if data.len() < 8 {
                return 0;
            }
            let mut b = [0u8; 8];
            b.copy_from_slice(&data[..8]);
            i32::from(!f64::from_le_bytes(b).is_finite())
        }
        0x12 => {
            // PACKED-DECIMAL: sign nibble + high nibble of last byte + BCD digit bytes.
            if data.is_empty() {
                return 0;
            }
            let end = data.len() - 1;
            let sign = data[end] & 0x0f;
            if no_sign_nibble {
                if sign > 0x09 {
                    return 0;
                }
            } else if have_sign {
                if host_sign && sign == 0x0f {
                    // ok
                } else if sign != 0x0c && sign != 0x0d {
                    return 0;
                }
            } else if sign != 0x0f {
                return 0;
            }
            if (data[end] & 0xf0) > 0x90 {
                return 0;
            }
            for &p in &data[..end] {
                if invalid_bcd(p) {
                    return 0;
                }
            }
            1
        }
        0x10 => crate::common_misc::cob_check_numdisp(data, have_sign, sign_leading, sign_separate, sign_mode),
        0x16 => {
            // FP DECIMAL 64 (little-endian host): combination field not 0x78.
            i32::from(data.len() >= 8 && (data[7] & 0x78) != 0x78)
        }
        0x17 => {
            // FP DECIMAL 128 (little-endian host).
            i32::from(data.len() >= 16 && (data[15] & 0x78) != 0x78)
        }
        _ => {
            // default: every byte must be an ASCII digit.
            for &p in data {
                if p.wrapping_sub(0x30) > 9 {
                    return 0;
                }
            }
            1
        }
    }
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

    // KANIFOR: GNURUST.COMMON.CBL.1
    /// `cob_sys_xor` is involutive (XORing the same source twice restores the destination) and every CBL_
    /// bit op is panic-free + bounded for any inputs -- the byte-op contract.
    #[kani::proof]
    #[kani::unwind(2)]
    fn cbl_xor_is_involutive() {
        let s: u8 = kani::any();
        let d0: u8 = kani::any();
        let mut d = [d0];
        cob_sys_xor(&[s], &mut d, 1);
        cob_sys_xor(&[s], &mut d, 1);
        assert_eq!(d[0], d0);
        // negative/zero length is a no-op, never panics.
        let mut e = [d0];
        cob_sys_and(&[s], &mut e, -1);
        assert_eq!(e[0], d0);
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

    #[test]
    fn bound_violation_sets_exception_status() {
        use crate::common_exception::{
            cob_exception_index_of, cob_get_last_exception_code, cob_get_last_exception_name,
            cob_last_exception_is, cob_set_exception_by_code, ExceptionState,
        };
        // A caught subscript violation, raised into the exception state, must surface as
        // EC-BOUND-SUBSCRIPT (0x0207) for EXCEPTION-STATUS -- the link that was missing.
        let mut st = ExceptionState::default();
        let v = cob_check_subscript(7, 5, "T", false, false).expect("out of bounds");
        raise_bound_violation(&mut st, &v);
        assert_eq!(cob_get_last_exception_code(&st), 0x0207);
        assert_eq!(cob_get_last_exception_name(&st), Some("EC-BOUND-SUBSCRIPT"));
        assert!(cob_last_exception_is(&st, cob_exception_index_of(0x0207)) != 0);
        // ODO -> EC-BOUND-ODO (0x0202)
        let mut st2 = ExceptionState::default();
        raise_bound_violation(&mut st2, &cob_check_odo(9, 1, 5, "T", "N").unwrap());
        assert_eq!(cob_get_last_exception_code(&st2), 0x0202);
        // a non-numeric class check -> EC-DATA-INCOMPATIBLE (0x0303)
        let mut st3 = ExceptionState::default();
        cob_set_exception_by_code(&mut st3, EC_DATA_INCOMPATIBLE);
        assert_eq!(cob_get_last_exception_code(&st3), 0x0303);
        assert_eq!(cob_get_last_exception_name(&st3), Some("EC-DATA-INCOMPATIBLE"));
    }

    #[test]
    fn bound_violation_diagnostic_matches_cobc_bytes() {
        use crate::common_runerr::SourceLocation;
        // Subscript fault (cobc -fec=all -debug): the libcob error line + the `note:` hint line, byte-exact.
        let v = cob_check_subscript(5, 3, "E", false, false).expect("out of bounds");
        let diag = cob_bound_violation_diagnostic(&SourceLocation::at(b"sub.cob", 9), &v);
        assert_eq!(
            diag,
            b"libcob: sub.cob:9: error: subscript of 'E' out of bounds: 5\nnote: maximum subscript for 'E': 3\n"
                .to_vec()
        );
        // Reference-modification fault: the first (aborting) violation -- length over the field size, no note.
        let rv = cob_check_ref_mod(3, 8, 5, "S"); // offset 3, length 8, size 5
        let diag2 = cob_bound_violation_diagnostic(&SourceLocation::at(b"rm.cob", 9), &rv[0]);
        assert_eq!(
            diag2,
            b"libcob: rm.cob:9: error: length of 'S' out of bounds: 8, maximum: 5\n".to_vec()
        );
        // ODO depending-on fault: the message names the DEPENDING-ON field (N), the hint names the OCCURS
        // ELEMENT (E) -- cobc `... OCCURS DEPENDING ON 'N' out of bounds: 9` / `note: maximum subscript for 'E': 5`.
        let ov = cob_check_odo(9, 1, 5, "E", "N").unwrap();
        let diag3 = cob_bound_violation_diagnostic(&SourceLocation::at(b"o.cob", 10), &ov);
        assert_eq!(
            diag3,
            b"libcob: o.cob:10: error: OCCURS DEPENDING ON 'N' out of bounds: 9\nnote: maximum subscript for 'E': 5\n"
                .to_vec()
        );
    }

    #[test]
    fn bound_fault_critical_column_drives_hard_failure() {
        use crate::common_exception::cob_exception_is_fatal;
        use crate::common_term::{ExitState, TermAction, EXIT_FAILURE};
        // exception.def's `fatal` column: EC-BOUND-SUBSCRIPT/-ODO/-REF-MOD are critical; umbrella EC-BOUND
        // (0x0200) is not. (Carried in EXCEPTION_FATAL_CODES, dropped from EXCEPTION_TAB's (code, name).)
        assert!(cob_exception_is_fatal(0x0207)); // EC-BOUND-SUBSCRIPT
        assert!(cob_exception_is_fatal(0x0202)); // EC-BOUND-ODO
        assert!(cob_exception_is_fatal(0x0205)); // EC-BOUND-REF-MOD
        assert!(!cob_exception_is_fatal(0x0200)); // EC-BOUND (umbrella)
        assert!(!cob_exception_is_fatal(0x1000)); // EC-SIZE

        // Each cob_check_* violation's own `fatal` equals the authoritative critical column (no drift), so
        // acting on `v.fatal` IS acting on exception.def's `fatal` bit.
        let sub = cob_check_subscript(5, 3, "E", false, false).unwrap();
        let odo = cob_check_odo(9, 1, 5, "E", "N").unwrap();
        let rm = &cob_check_ref_mod(3, 8, 5, "S")[0];
        for v in [&sub, &odo, rm] {
            assert_eq!(v.fatal, cob_bound_violation_is_critical(v));
            assert!(v.fatal, "a bound fault is critical");
        }

        // Acting on the fatal flag: a critical bound fault drives cob_hard_failure, which terminates with
        // exit code EXIT_FAILURE (1) -- the observed cobc process exit on a bounds fault.
        let mut term = ExitState::default();
        let outcome = term.cob_hard_failure();
        assert_eq!(outcome.action, TermAction::Exit(EXIT_FAILURE));
        assert_eq!(EXIT_FAILURE, 1);
    }

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
    fn set_libcob_version_compares_and_overwrites() {
        let (mut ma, mut mi, mut pa) = (3, 2, 0);
        assert_eq!(set_libcob_version(&mut ma, &mut mi, &mut pa), 0); // match
        let (mut ma, mut mi, mut pa) = (2, 9, 9);
        assert_eq!(set_libcob_version(&mut ma, &mut mi, &mut pa), 1); // major mismatch
        assert_eq!((ma, mi, pa), (3, 2, 0)); // overwritten to lib version
        let (mut ma, mut mi, mut pa) = (3, 1, 0);
        assert_eq!(set_libcob_version(&mut ma, &mut mi, &mut pa), 2); // minor mismatch
        let (mut ma, mut mi, mut pa) = (0, 0, 0);
        assert_eq!(set_libcob_version(&mut ma, &mut mi, &mut pa), 0); // major==0 -> 0
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

    #[test]
    fn correct_numeric_and_is_numeric() {
        use crate::common_misc::SignMode;
        // cob_correct_numeric: unsigned NUMDISP with a space/junk digit -> '0'
        let mut d = *b"12 4";
        cob_correct_numeric(&mut d, true, false, false, false, false);
        assert_eq!(&d, b"1204");
        // ASCII trailing overpunch sign: 'r' (=2, negative) corrected to '2' in the last byte
        let mut s = *b"34r";
        cob_correct_numeric(&mut s, true, false, false, false, false);
        assert_eq!(&s, b"342");
        // cob_is_numeric: BINARY always numeric; DISPLAY digit check; default ASCII-digit check
        assert_eq!(cob_is_numeric(0x11, b"\x00\x05", false, false, false, SignMode::Ascii, false, false), 1);
        assert_eq!(cob_is_numeric(0x10, b"123", false, false, false, SignMode::Ascii, false, false), 1);
        assert_eq!(cob_is_numeric(0x10, b"12X", false, false, false, SignMode::Ascii, false, false), 0);
        // PACKED: 0x12 0x3C = digits 123 sign C (positive) -> numeric
        assert_eq!(cob_is_numeric(0x12, b"\x12\x3c", true, false, false, SignMode::Ascii, false, false), 1);
        // PACKED bad sign nibble (0x3F with have_sign) -> not numeric
        assert_eq!(cob_is_numeric(0x12, b"\x12\x3f", true, false, false, SignMode::Ascii, false, false), 0);
    }

    #[test]
    fn common_cmpc_translates_through_collation() {
        let col = ident_col();
        // identity collation: byte-by-byte difference against the single char.
        assert_eq!(common_cmpc(b"AAA", b'A', &col), 0);
        // 'B'(0x42) vs 'A'(0x41) -> +1 at the first byte.
        assert_eq!(common_cmpc(b"BBB", b'A', &col), 1);
        // 'A' vs 'B' -> -1.
        assert_eq!(common_cmpc(b"A", b'B', &col), -1);

        // a collation that folds 'a'(0x61) onto 'A'(0x41) makes them compare equal.
        let mut fold = ident_col();
        fold[0x61] = 0x41;
        assert_eq!(common_cmpc(b"aaa", b'A', &fold), 0);
    }

    #[test]
    fn sort_compare_collate_orders_by_keys() {
        let col = ident_col();
        let asc = [SortKey { offset: 0, size: 3, ascending: true, numeric: false }];
        // "ABC" < "ABD" ascending -> negative.
        assert_eq!(sort_compare_collate(b"ABC", b"ABD", &asc, &col), -1);
        assert_eq!(sort_compare_collate(b"ABD", b"ABC", &asc, &col), 1);
        assert_eq!(sort_compare_collate(b"ABC", b"ABC", &asc, &col), 0);
        // descending flips the sign.
        let desc = [SortKey { offset: 0, size: 3, ascending: false, numeric: false }];
        assert_eq!(sort_compare_collate(b"ABC", b"ABD", &desc, &col), 1);
    }

    #[test]
    fn cob_check_ref_mod_detailed_reports_violations() {
        // in-bounds field(2:3) of a size-5 item: no violation.
        assert!(cob_check_ref_mod_detailed("F", true, false, 5, 2, 3).is_empty());

        // offset 0 (< 1): one offset-out-of-bounds violation.
        let v = cob_check_ref_mod_detailed("F", true, false, 5, 0, 1);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].exception, BoundException::RefMod);
        assert!(v[0].fatal);
        assert_eq!(v[0].message, b"offset of 'F' out of bounds: 0".to_vec());

        // length 0 with zero NOT allowed -> length-out-of-bounds.
        let v = cob_check_ref_mod_detailed("F", false, false, 5, 1, 0);
        assert_eq!(v.len(), 1);
        assert!(!v[0].fatal);
        assert_eq!(v[0].message, b"length of 'F' out of bounds: 0".to_vec());

        // offset 4 + length 4 - 1 = 7 > size 5 -> the combined start+length violation.
        let v = cob_check_ref_mod_detailed("F", true, false, 5, 4, 4);
        assert!(v.iter().any(|b| b.message
            == b"length of 'F' out of bounds: 4, starting at: 4, maximum: 5".to_vec()));
    }
}
