#![forbid(unsafe_code)]
//! Port of a cluster of **pure, side-effect-free** helpers from `libcob/common.c` -- the programmable
//! COBOL switch array, the OMITTED / no-op markers, the alpha class check, the boolean-string parser,
//! pointer read/write through a field's bytes, and the NUMERIC DISPLAY sign/digit validator.
//!
//! Each function below is a faithful 1:1 port of the named `common.c` routine. The only routine that
//! carries mutable runtime state in C is the switch array (`static int cob_switch[COB_SWITCH_MAX + 1]`,
//! mutated by `cob_get_switch` / `cob_set_switch`); per the port conventions that global static is
//! modelled here as an explicit [`Switches`] struct rather than a hidden global.
//!
//! `cob_is_upper` / `cob_is_lower` already live in `common.rs` and are intentionally NOT re-ported here.

/// `common.c:399` -- `#define COB_SWITCH_MAX 36` (maximum switch index; the backing array is
/// `cob_switch[COB_SWITCH_MAX + 1]`, i.e. indices `0..=36`).
pub const COB_SWITCH_MAX: i32 = 36;

/// Number of slots in the switch array: `COB_SWITCH_MAX + 1` == 37.
pub const COB_SWITCH_COUNT: usize = (COB_SWITCH_MAX as usize) + 1;

/// Port of the `static int cob_switch[COB_SWITCH_MAX + 1]` global state from `common.c`, modelled
/// explicitly (per the port conventions, NOT a global static). Holds the programmable COBOL
/// SWITCH-0..n values accessed by [`Switches::get`] (`common.c:cob_get_switch`) and
/// [`Switches::set`] (`common.c:cob_set_switch`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Switches([i32; COB_SWITCH_COUNT]);

impl Default for Switches {
    fn default() -> Self {
        Switches([0; COB_SWITCH_COUNT])
    }
}

impl Switches {
    /// A fresh switch array, all switches `0` (the C `static` zero-initialised array).
    pub fn new() -> Self {
        Switches::default()
    }

    /// Port of `common.c:cob_get_switch` -- return switch `n`'s value, or `0` for an out-of-range
    /// index. The C guard is `if (n < 0 || n > COB_SWITCH_MAX) return 0;`, so the valid range is
    /// `0..=COB_SWITCH_MAX` inclusive.
    pub fn get(&self, n: i32) -> i32 {
        if n < 0 || n > COB_SWITCH_MAX {
            return 0;
        }
        self.0[n as usize]
    }

    /// Port of `common.c:cob_set_switch` -- set switch `n` to `0` when `flag == 0`, to `1` when
    /// `flag == 1`, and leave it unchanged for any other `flag` value. Out-of-range `n` is a no-op.
    /// Mirrors the C exactly: only the literal values `0` and `1` mutate the slot.
    pub fn set(&mut self, n: i32, flag: i32) {
        if n < 0 || n > COB_SWITCH_MAX {
            return;
        }
        if flag == 0 {
            self.0[n as usize] = 0;
        } else if flag == 1 {
            self.0[n as usize] = 1;
        }
    }
}

/// Port of `common.c:cob_get_switch` -- convenience free-function form reading from an explicit
/// [`Switches`] state (the C signature is `int cob_get_switch(const int n)` over the global array).
pub fn cob_get_switch(switches: &Switches, n: i32) -> i32 {
    switches.get(n)
}

/// Port of `common.c:cob_set_switch` -- convenience free-function form mutating an explicit
/// [`Switches`] state (the C signature is `void cob_set_switch(const int n, const int flag)`).
pub fn cob_set_switch(switches: &mut Switches, n: i32, flag: i32) {
    switches.set(n, flag)
}

/// Port of `common.c:cob_nop` -- an empty no-op routine. In C it exists only to block certain
/// portable compiler optimisations; it has no observable effect.
pub fn cob_nop() {}

/// Port of `common.c:cob_is_omitted` -- a field/param is OMITTED iff its data pointer is NULL
/// (`return f->data == NULL;`). The C "is the pointer NULL" test is modelled here as a boolean:
/// `data_is_null == true` corresponds to a NULL `f->data`, i.e. an omitted parameter.
/// Returns `1` (omitted) / `0` (present), faithful to the C `int` return.
pub fn cob_is_omitted(data_is_null: bool) -> i32 {
    if data_is_null {
        1
    } else {
        0
    }
}

/// Port of `common.c:cob_is_alpha` -- the COBOL `IS ALPHABETIC` class check: every byte of the
/// field must be a space (`0x20`) or an ASCII alphabetic character (`isalpha`). Returns `1` when
/// the whole field is alphabetic (or empty), `0` otherwise.
///
/// Note: C `isalpha` is locale-dependent; in the default "C" locale (the GnuCOBOL runtime default)
/// it is exactly `A-Z` / `a-z`, which is what is ported here.
pub fn cob_is_alpha(data: &[u8]) -> i32 {
    for &p in data {
        // 0x20 == space ; ASCII alpha A-Z (0x41..=0x5A) or a-z (0x61..=0x7A)
        let is_space = p == 0x20;
        let is_upper = (0x41..=0x5A).contains(&p);
        let is_lower = (0x61..=0x7A).contains(&p);
        if !(is_space || is_upper || is_lower) {
            return 0;
        }
    }
    1
}

/// Port of `common.c:translate_boolean_to_int` -- parse a boolean-ish C string into an int.
///
/// Faithful return contract from the C source:
/// * NULL or empty string -> `2`
/// * a single-character `"0"` / `"1"` -> `0` / `1` (via `COB_D2I`, i.e. low nibble)
/// * `"!"` (the pre-translated "never / not set" marker) -> `-1`
/// * case-insensitive `true` / `t` / `on` / `yes` / `y` -> `1`
/// * case-insensitive `false` / `f` / `off` / `no` / `n` -> `0`
/// * anything else -> `2`
///
/// The C input is a `const char*`; a NULL pointer is modelled here as `None`.
pub fn translate_boolean_to_int(ptr: Option<&[u8]>) -> i32 {
    // NULL ptr or empty string ( *ptr == 0 ) -> 2
    let bytes = match ptr {
        None => return 2,
        Some(b) if b.is_empty() => return 2,
        Some(b) => b,
    };
    // In C the string is NUL-terminated; treat a leading NUL byte as the empty string.
    if bytes[0] == 0 {
        return 2;
    }

    // single character "0" or "1": *(ptr+1) == 0 && (*ptr == '0' || *ptr == '1')
    // "single char" here means the second byte terminates the string (NUL) or the slice ends.
    let second_is_terminator = bytes.len() < 2 || bytes[1] == 0;
    if second_is_terminator && (bytes[0] == 0x30 || bytes[0] == 0x31) {
        // COB_D2I(x) == (x) & 0x0F : 0x30 -> 0, 0x31 -> 1
        return (bytes[0] & 0x0F) as i32;
    }

    // Compare against the recognised words, honouring the C NUL terminator (strcmp / strcasecmp
    // stop at the first NUL), so trim the slice at its first NUL byte first.
    let s: &[u8] = match bytes.iter().position(|&b| b == 0) {
        Some(i) => &bytes[..i],
        None => bytes,
    };

    // strcmp(ptr, "!") == 0  -> -1  (exact, case-sensitive)
    if s == b"!" {
        return -1;
    }

    if eq_ignore_ascii_case(s, b"true")
        || eq_ignore_ascii_case(s, b"t")
        || eq_ignore_ascii_case(s, b"on")
        || eq_ignore_ascii_case(s, b"yes")
        || eq_ignore_ascii_case(s, b"y")
    {
        return 1;
    }

    if eq_ignore_ascii_case(s, b"false")
        || eq_ignore_ascii_case(s, b"f")
        || eq_ignore_ascii_case(s, b"off")
        || eq_ignore_ascii_case(s, b"no")
        || eq_ignore_ascii_case(s, b"n")
    {
        return 0;
    }

    2
}

/// Helper modelling C `strcasecmp(a, b) == 0` for ASCII (the only alphabet in the recognised words).
fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (&x, &y) in a.iter().zip(b.iter()) {
        if x.to_ascii_lowercase() != y.to_ascii_lowercase() {
            return false;
        }
    }
    true
}

/// Port of `common.c:cob_get_pointer` -- read a host pointer-sized value out of a field's bytes.
///
/// The C body is `memcpy(&tmptr, srcptr, sizeof(void *))` returning the bytes reinterpreted as a
/// pointer. Reinterpreting raw bytes as a live pointer is the host-boundary part that cannot be a
/// pure Rust function; what is pure and faithful is the *byte transfer*: read `sizeof(void *)` bytes
/// in host (native) byte order into an integer of that width. This port returns that integer.
///
/// The boundary is explicit: the result depends on the **host pointer size** (`POINTER_SIZE`,
/// here `usize`/`u64`) and **host endianness** (native), exactly as the C `memcpy` does. Returns
/// `None` if `srcptr` has fewer than `POINTER_SIZE` bytes (the C reads unconditionally; this port
/// bounds-guards instead of reading out of range).
pub fn cob_get_pointer(srcptr: &[u8]) -> Option<u64> {
    let n = POINTER_SIZE;
    if srcptr.len() < n {
        return None;
    }
    let mut buf = [0u8; 8];
    // Native byte order: copy the first `n` bytes into the low `n` bytes for LE hosts, but to be
    // faithful to memcpy on the host we reconstruct using from_ne_bytes over an 8-byte window.
    // For pointer sizes < 8 we build an exact-width native-endian integer then widen.
    match n {
        8 => {
            buf.copy_from_slice(&srcptr[..8]);
            Some(u64::from_ne_bytes(buf))
        }
        4 => {
            let mut b4 = [0u8; 4];
            b4.copy_from_slice(&srcptr[..4]);
            Some(u32::from_ne_bytes(b4) as u64)
        }
        2 => {
            let mut b2 = [0u8; 2];
            b2.copy_from_slice(&srcptr[..2]);
            Some(u16::from_ne_bytes(b2) as u64)
        }
        _ => {
            // Fallback for any other pointer width: assemble native-endian.
            let mut v: u64 = 0;
            if cfg!(target_endian = "little") {
                for i in (0..n).rev() {
                    v = (v << 8) | srcptr[i] as u64;
                }
            } else {
                for i in 0..n {
                    v = (v << 8) | srcptr[i] as u64;
                }
            }
            Some(v)
        }
    }
}

/// Port of `move.c:cob_put_pointer` -- write a host pointer-sized value into a field's bytes.
///
/// The C body is `memcpy(mem, &val, sizeof(void *))`. As with [`cob_get_pointer`] the live-pointer
/// reinterpretation is the host boundary; the pure, faithful part is the byte transfer: write the
/// low `POINTER_SIZE` bytes of `val` into `mem` in host (native) byte order. Returns `false`
/// (writing nothing) if `mem` has fewer than `POINTER_SIZE` bytes (bounds guard for the C memcpy).
pub fn cob_put_pointer(val: u64, mem: &mut [u8]) -> bool {
    let n = POINTER_SIZE;
    if mem.len() < n {
        return false;
    }
    let ne = val.to_ne_bytes(); // 8 bytes, native order
    if cfg!(target_endian = "little") {
        // low `n` bytes are the first `n` of the native (little-endian) representation
        mem[..n].copy_from_slice(&ne[..n]);
    } else {
        // big-endian host: low `n` bytes are the last `n` of the 8-byte native representation
        mem[..n].copy_from_slice(&ne[8 - n..]);
    }
    true
}

/// Host pointer width in bytes (`sizeof(void *)`), the boundary that [`cob_get_pointer`] /
/// [`cob_put_pointer`] depend on. Equals `usize`'s byte width on the build target.
pub const POINTER_SIZE: usize = core::mem::size_of::<usize>();

/// EBCDIC sign mode for [`cob_check_numdisp`], standing in for `COB_MODULE_PTR->ebcdic_sign`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignMode {
    /// Non-EBCDIC (ASCII) overpunch sign handling.
    Ascii,
    /// EBCDIC overpunch sign handling.
    Ebcdic,
}

/// `common.c:427` -- `#define IS_INVALID_DIGIT_DATA(c) ((unsigned char)(c - '0') > 9)`: true when
/// `c` is not one of the ASCII digits `0x30..=0x39`.
fn is_invalid_digit_data(c: u8) -> bool {
    // (c - '0') as unsigned char > 9 ; '0' == 0x30
    c.wrapping_sub(0x30) > 9
}

/// Port of `common.c:cob_check_numdisp` -- validate the bytes of a NUMERIC DISPLAY field: every
/// digit position must be an ASCII digit, and (for a signed field) the sign position must be a
/// valid sign or overpunch character. Returns `1` when valid, `0` otherwise.
///
/// The field's sign attributes are taken as explicit parameters in place of the C `COB_FIELD_*`
/// flag macros, and the EBCDIC sign mode in place of `COB_MODULE_PTR->ebcdic_sign`:
/// * `have_sign`     == `COB_FIELD_HAVE_SIGN(f)`
/// * `sign_leading`  == `COB_FIELD_SIGN_LEADING(f)`
/// * `sign_separate` == `COB_FIELD_SIGN_SEPARATE(f)`
/// * `sign_mode`     == EBCDIC vs ASCII overpunch handling
///
/// EBCDIC overpunch characters (per the C `switch`): positive `{` then `ABCDEFGHI` (= 0..9),
/// negative `}` then `JKLMNOPQR` (= 0..9). The non-EBCDIC (ASCII) embedded-sign overpunch set is
/// `p q r s t u v w x y`.
pub fn cob_check_numdisp(
    data: &[u8],
    have_sign: bool,
    sign_leading: bool,
    sign_separate: bool,
    sign_mode: SignMode,
) -> i32 {
    // p = f->data ; end = p + f->size. Modelled as a [start, end) index window into `data`.
    let mut start: usize = 0;
    let mut end: usize = data.len();

    if have_sign {
        // Pull off the sign byte: leading -> *p++ ; trailing -> *(--end).
        // Bounds guard: an empty signed field cannot carry a sign byte -> invalid.
        if end == 0 {
            return 0;
        }
        let c: u8 = if sign_leading {
            let v = data[start];
            start += 1;
            v
        } else {
            end -= 1;
            data[end]
        };

        if sign_separate {
            // c must be '+' (0x2B) or '-' (0x2D)
            if c != 0x2B && c != 0x2D {
                return 0;
            }
        } else if is_invalid_digit_data(c) {
            // embedded sign / overpunch: validate against the mode's character set
            match sign_mode {
                SignMode::Ebcdic => {
                    // { A B C D E F G H I  (positive 0-9)  } J K L M N O P Q R (negative 0-9)
                    let ok = matches!(
                        c,
                        0x7B // '{'
                            | 0x41..=0x49 // 'A'..='I'
                            | 0x7D // '}'
                            | 0x4A..=0x52 // 'J'..='R'
                    );
                    if !ok {
                        return 0;
                    }
                }
                SignMode::Ascii => {
                    // p q r s t u v w x y  == 0x70..=0x79
                    let ok = (0x70..=0x79).contains(&c);
                    if !ok {
                        return 0;
                    }
                }
            }
        }
    }

    // remaining bytes [start, end) must all be valid ASCII digits
    let mut p = start;
    while p < end {
        if is_invalid_digit_data(data[p]) {
            return 0;
        }
        p += 1;
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switches_default_all_zero() {
        let s = Switches::new();
        for n in 0..=COB_SWITCH_MAX {
            assert_eq!(s.get(n), 0);
        }
    }

    #[test]
    fn switches_set_and_get() {
        let mut s = Switches::new();
        s.set(0, 1);
        assert_eq!(s.get(0), 1);
        s.set(0, 0);
        assert_eq!(s.get(0), 0);
        s.set(COB_SWITCH_MAX, 1);
        assert_eq!(s.get(COB_SWITCH_MAX), 1);
    }

    #[test]
    fn switches_set_other_flag_is_noop() {
        let mut s = Switches::new();
        s.set(3, 1);
        s.set(3, 7); // only 0 and 1 mutate
        assert_eq!(s.get(3), 1);
        s.set(3, -5);
        assert_eq!(s.get(3), 1);
    }

    #[test]
    fn switches_out_of_range() {
        let mut s = Switches::new();
        assert_eq!(s.get(-1), 0);
        assert_eq!(s.get(COB_SWITCH_MAX + 1), 0);
        s.set(-1, 1); // no panic, no effect
        s.set(COB_SWITCH_MAX + 1, 1);
        assert_eq!(s.get(0), 0);
    }

    #[test]
    fn switches_free_function_forms() {
        let mut s = Switches::new();
        cob_set_switch(&mut s, 5, 1);
        assert_eq!(cob_get_switch(&s, 5), 1);
    }

    #[test]
    fn nop_does_nothing() {
        cob_nop();
    }

    #[test]
    fn is_omitted_maps_null() {
        assert_eq!(cob_is_omitted(true), 1);
        assert_eq!(cob_is_omitted(false), 0);
    }

    #[test]
    fn is_alpha_basic() {
        assert_eq!(cob_is_alpha(b"ABCdef"), 1);
        assert_eq!(cob_is_alpha(b"AB cd"), 1); // spaces allowed
        assert_eq!(cob_is_alpha(b"   "), 1);
        assert_eq!(cob_is_alpha(b""), 1); // empty -> all (vacuously) alpha
        assert_eq!(cob_is_alpha(b"AB1"), 0); // digit
        assert_eq!(cob_is_alpha(b"AB-"), 0); // punctuation
    }

    #[test]
    fn translate_boolean_null_and_empty() {
        assert_eq!(translate_boolean_to_int(None), 2);
        assert_eq!(translate_boolean_to_int(Some(b"")), 2);
        assert_eq!(translate_boolean_to_int(Some(&[0u8])), 2); // leading NUL
    }

    #[test]
    fn translate_boolean_single_digit() {
        assert_eq!(translate_boolean_to_int(Some(b"0")), 0);
        assert_eq!(translate_boolean_to_int(Some(b"1")), 1);
        // two-char "01" is NOT the single-digit case
        assert_eq!(translate_boolean_to_int(Some(b"01")), 2);
    }

    #[test]
    fn translate_boolean_never_marker() {
        assert_eq!(translate_boolean_to_int(Some(b"!")), -1);
        // case-sensitive: "!" only, nothing else maps to -1
        assert_eq!(translate_boolean_to_int(Some(b"!!")), 2);
    }

    #[test]
    fn translate_boolean_true_words() {
        for w in [&b"true"[..], b"True", b"TRUE", b"t", b"T", b"on", b"ON", b"yes", b"YeS", b"y", b"Y"] {
            assert_eq!(translate_boolean_to_int(Some(w)), 1, "word {:?}", w);
        }
    }

    #[test]
    fn translate_boolean_false_words() {
        for w in [&b"false"[..], b"FALSE", b"f", b"F", b"off", b"OFF", b"no", b"NO", b"n", b"N"] {
            assert_eq!(translate_boolean_to_int(Some(w)), 0, "word {:?}", w);
        }
    }

    #[test]
    fn translate_boolean_unknown() {
        assert_eq!(translate_boolean_to_int(Some(b"maybe")), 2);
        assert_eq!(translate_boolean_to_int(Some(b"2")), 2);
    }

    #[test]
    fn translate_boolean_stops_at_nul() {
        // "true\0xx" should match "true" (strcasecmp stops at NUL)
        assert_eq!(translate_boolean_to_int(Some(b"true\0xx")), 1);
    }

    #[test]
    fn pointer_roundtrip() {
        let mut buf = vec![0u8; POINTER_SIZE];
        let v: u64 = if POINTER_SIZE >= 8 {
            0x0123_4567_89AB_CDEF
        } else {
            0x89AB_CDEF
        };
        assert!(cob_put_pointer(v, &mut buf));
        assert_eq!(cob_get_pointer(&buf), Some(v));
    }

    #[test]
    fn pointer_get_bounds_guard() {
        let short = vec![0u8; POINTER_SIZE - 1];
        assert_eq!(cob_get_pointer(&short), None);
    }

    #[test]
    fn pointer_put_bounds_guard() {
        let mut short = vec![0u8; POINTER_SIZE - 1];
        assert!(!cob_put_pointer(1, &mut short));
    }

    #[test]
    fn pointer_native_byte_order() {
        // A known value should land in memory in native byte order; reading it back must agree.
        let mut buf = vec![0u8; POINTER_SIZE];
        cob_put_pointer(0xFF, &mut buf);
        if cfg!(target_endian = "little") {
            assert_eq!(buf[0], 0xFF);
        } else {
            assert_eq!(buf[POINTER_SIZE - 1], 0xFF);
        }
    }

    #[test]
    fn numdisp_unsigned_valid() {
        // unsigned: all digits
        assert_eq!(
            cob_check_numdisp(b"12345", false, false, false, SignMode::Ascii),
            1
        );
        assert_eq!(
            cob_check_numdisp(b"12X45", false, false, false, SignMode::Ascii),
            0
        );
        assert_eq!(
            cob_check_numdisp(b"", false, false, false, SignMode::Ascii),
            1
        );
    }

    #[test]
    fn numdisp_sign_separate() {
        // trailing separate sign '+' / '-'
        assert_eq!(
            cob_check_numdisp(b"123+", true, false, true, SignMode::Ascii),
            1
        );
        assert_eq!(
            cob_check_numdisp(b"123-", true, false, true, SignMode::Ascii),
            1
        );
        assert_eq!(
            cob_check_numdisp(b"123X", true, false, true, SignMode::Ascii),
            0
        );
        // leading separate
        assert_eq!(
            cob_check_numdisp(b"-123", true, true, true, SignMode::Ascii),
            1
        );
    }

    #[test]
    fn numdisp_sign_separate_empty_invalid() {
        assert_eq!(
            cob_check_numdisp(b"", true, false, true, SignMode::Ascii),
            0
        );
    }

    #[test]
    fn numdisp_ascii_overpunch() {
        // trailing embedded sign, ASCII set p..y on last byte; rest digits.
        // 'p' == 0x70 valid overpunch
        assert_eq!(
            cob_check_numdisp(b"12p", true, false, false, SignMode::Ascii),
            1
        );
        // a plain digit in the sign position is fine too (not IS_INVALID_DIGIT_DATA)
        assert_eq!(
            cob_check_numdisp(b"125", true, false, false, SignMode::Ascii),
            1
        );
        // 'z' (0x7A) is NOT in p..y -> invalid
        assert_eq!(
            cob_check_numdisp(b"12z", true, false, false, SignMode::Ascii),
            0
        );
    }

    #[test]
    fn numdisp_ebcdic_overpunch() {
        // '{' positive zero, 'A'..'I' positive 1-9, '}' negative zero, 'J'..'R' negative 1-9
        assert_eq!(
            cob_check_numdisp(b"12{", true, false, false, SignMode::Ebcdic),
            1
        );
        assert_eq!(
            cob_check_numdisp(b"12I", true, false, false, SignMode::Ebcdic),
            1
        );
        assert_eq!(
            cob_check_numdisp(b"12}", true, false, false, SignMode::Ebcdic),
            1
        );
        assert_eq!(
            cob_check_numdisp(b"12R", true, false, false, SignMode::Ebcdic),
            1
        );
        // 'S' (0x53) is just past 'R' -> not a valid EBCDIC overpunch
        assert_eq!(
            cob_check_numdisp(b"12S", true, false, false, SignMode::Ebcdic),
            0
        );
        // leading EBCDIC sign byte
        assert_eq!(
            cob_check_numdisp(b"{12", true, true, false, SignMode::Ebcdic),
            1
        );
    }

    #[test]
    fn numdisp_bad_digit_in_body() {
        // valid trailing sign but a non-digit in the body -> invalid
        assert_eq!(
            cob_check_numdisp(b"1X3+", true, false, true, SignMode::Ascii),
            0
        );
    }

    #[test]
    fn is_invalid_digit_data_table() {
        assert!(!is_invalid_digit_data(0x30)); // '0'
        assert!(!is_invalid_digit_data(0x39)); // '9'
        assert!(is_invalid_digit_data(0x2F)); // '/'
        assert!(is_invalid_digit_data(0x3A)); // ':'
        assert!(is_invalid_digit_data(0x41)); // 'A'
    }
}
