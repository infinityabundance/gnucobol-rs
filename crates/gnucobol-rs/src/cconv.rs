//! 1:1 port of cconv.c — GnuCOBOL's character-conversion helpers: hex-digit/byte parsing, the 7-bit
//! ASCII upper/lower case-fold tables (`cob_toupper`/`cob_tolower`), `cob_field_to_string` (rtrim a
//! field's content into a buffer with optional case folding), and `cob_load_collation` (load a `.ttbl`
//! translation table). The case tables are the GnuCOBOL "C-locale 7-bit" fold — only `A-Z`/`a-z` change,
//! everything else (including bytes ≥ 128) is left untouched; this is NOT locale-aware folding.
//!
//! All 9 cconv.c functions have named counterparts. `cob_load_collation` is the one function that does
//! filesystem + environment I/O (faithful to the C); the other eight are pure.
#![forbid(unsafe_code)]

/// `enum cob_case_modifier` (coblocal.h): how `cob_field_to_string` folds case while copying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CobCase {
    /// `CCM_NONE` — copy verbatim.
    None,
    /// `CCM_LOWER` — `cob_tolower` (7-bit ASCII).
    Lower,
    /// `CCM_UPPER` — `cob_toupper` (7-bit ASCII).
    Upper,
    /// `CCM_LOWER_LOCALE` — C-library `tolower` (locale). Under the pinned `LC_ALL=C.UTF-8` this equals
    /// `CCM_LOWER` for ASCII and leaves bytes ≥ 128 unchanged.
    LowerLocale,
    /// `CCM_UPPER_LOCALE` — C-library `toupper` (locale); see [`CobCase::LowerLocale`].
    UpperLocale,
}

/// `lower_tab` (cconv.c:34): `a-z → A-Z`, every other byte `0`. Used by [`cob_toupper`].
const fn build_lower_tab() -> [u8; 256] {
    let mut t = [0u8; 256];
    let mut c = b'a';
    while c <= b'z' {
        t[c as usize] = c - 32;
        c += 1;
    }
    t
}
/// `upper_tab` (cconv.c:62): `A-Z → a-z`, every other byte `0`. Used by [`cob_tolower`].
const fn build_upper_tab() -> [u8; 256] {
    let mut t = [0u8; 256];
    let mut c = b'A';
    while c <= b'Z' {
        t[c as usize] = c + 32;
        c += 1;
    }
    t
}
const LOWER_TAB: [u8; 256] = build_lower_tab();
const UPPER_TAB: [u8; 256] = build_upper_tab();

/// The C `isspace` (C locale): space, `\t`, `\n`, `\v`, `\f`, `\r`.
fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// `cob_toupper (c)` (cconv.c:239): 7-bit ASCII upper-casing via `lower_tab` (non-letters unchanged).
pub fn cob_toupper(c: u8) -> u8 {
    let e = LOWER_TAB[c as usize];
    if e != 0 {
        e
    } else {
        c
    }
}

/// `cob_tolower (c)` (cconv.c:250): 7-bit ASCII lower-casing via `upper_tab` (non-letters unchanged).
pub fn cob_tolower(c: u8) -> u8 {
    let e = UPPER_TAB[c as usize];
    if e != 0 {
        e
    } else {
        c
    }
}

/// `cob_convert_hex_digit (h)` (cconv.c:97): a hex digit char → `0..15`, or `-1` if not a hex digit.
pub fn cob_convert_hex_digit(h: u8) -> i32 {
    if h.is_ascii_digit() {
        return (h - b'0') as i32;
    }
    let h = cob_toupper(h);
    if (b'A'..=b'F').contains(&h) {
        10 + (h - b'A') as i32
    } else {
        -1
    }
}

/// `cob_convert_hex_byte (h)` (cconv.c:106): two hex chars `h[0]h[1]` → a byte `0..255`, or `-1`.
pub fn cob_convert_hex_byte(h: &[u8]) -> i32 {
    let d1 = cob_convert_hex_digit(h.first().copied().unwrap_or(0));
    let d2 = cob_convert_hex_digit(h.get(1).copied().unwrap_or(0));
    if d1 < 0 || d2 < 0 {
        -1
    } else {
        d1 * 16 + d2
    }
}

/// `cob_skip_blanks (s)` (cconv.c:118): the suffix of `s` after any leading whitespace.
pub fn cob_skip_blanks(s: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < s.len() && is_space(s[i]) {
        i += 1;
    }
    &s[i..]
}

/// A `cob_field` reference for [`cob_field_to_string`]: the declared `size` and the (possibly absent,
/// for a BASED/LINKAGE item) `data`.
pub struct FieldRef<'a> {
    pub size: usize,
    pub data: Option<&'a [u8]>,
}

/// `cob_field_to_string (f, str, maxsize, target_case)` (cconv.c:264): copy `f`'s right-trimmed content
/// (trailing spaces and NULs removed) into `out` (acting as the `maxsize` buffer), optionally case-folding,
/// and NUL-terminate. Returns the number of bytes copied, or a negative error code:
/// `-1` NULL field, `-2` zero-size field, `-3` NULL data address, `-4` content longer than `maxsize`.
pub fn cob_field_to_string(f: Option<&FieldRef>, out: &mut [u8], target_case: CobCase) -> i32 {
    let maxsize = out.len();
    let write_msg = |out: &mut [u8], msg: &[u8]| {
        // snprintf(str, maxsize, "%s", msg) + str[maxsize-1] = 0
        if maxsize == 0 {
            return;
        }
        let n = msg.len().min(maxsize.saturating_sub(1));
        out[..n].copy_from_slice(&msg[..n]);
        out[n] = 0;
        out[maxsize - 1] = 0;
    };
    let f = match f {
        None => {
            write_msg(out, b"NULL field");
            return -1;
        }
        Some(f) => f,
    };
    if f.size == 0 {
        if maxsize > 0 {
            out[0] = 0;
        }
        return -2;
    }
    let data = match f.data {
        None => {
            write_msg(out, b"field with NULL address");
            return -3;
        }
        Some(d) => d,
    };
    // rtrim: end = last index in [0, size) whose byte is not ' ' and not 0; the C walks back from size-1.
    let size = f.size.min(data.len());
    if size == 0 {
        if maxsize > 0 {
            out[0] = 0;
        }
        return 0;
    }
    let mut end = size - 1;
    while end > 0 {
        if data[end] != b' ' && data[end] != 0 {
            break;
        }
        end -= 1;
    }
    if data[end] == b' ' || data[end] == 0 {
        if maxsize > 0 {
            out[0] = 0;
        }
        return 0;
    }
    // note: the specified max does not contain the low-value
    if end > maxsize {
        if maxsize > 0 {
            out[0] = 0;
        }
        return -4;
    }
    let mut s = 0usize;
    for &b in &data[..=end] {
        out[s] = match target_case {
            CobCase::None => b,
            CobCase::Lower | CobCase::LowerLocale => cob_tolower(b),
            CobCase::Upper | CobCase::UpperLocale => cob_toupper(b),
        };
        s += 1;
    }
    out[s] = 0;
    (end + 1) as i32
}

/// `init_upper_lower ()` (cconv.c:343): the non-`HAVE_DESIGNATED_INITS` path that fills the case tables
/// from `plower_tab`/`plower_val`. Returns `(lower_tab, upper_tab)`, identical to the compile-time
/// [`LOWER_TAB`]/[`UPPER_TAB`] constants the admitted (designated-init) build uses.
pub fn init_upper_lower() -> ([u8; 256], [u8; 256]) {
    let plower_tab = b"abcdefghijklmnopqrstuvwxyz";
    let plower_val = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut lower = [0u8; 256];
    for (p, v) in plower_tab.iter().zip(plower_val.iter()) {
        lower[*p as usize] = *v;
    }
    let mut upper = [0u8; 256];
    for (p, v) in plower_val.iter().zip(plower_tab.iter()) {
        upper[*p as usize] = *v;
    }
    (lower, upper)
}

/// `cob_init_cconv (lptr)` (cconv.c:364): module init. The case tables are compile-time constants in this
/// port (the designated-init path), so this is a no-op.
pub fn cob_init_cconv() {}

/// `cob_load_collation (col_name, ebcdic_to_ascii, ascii_to_ebcdic)` (cconv.c:130): load a translation
/// table. `col_name` is used as a path when it begins with `.`/`/`, else `$COB_CONFIG_DIR/<col_name>.ttbl`.
/// The file is hex bytes (whitespace-separated, `#` comments); 256 bytes fills `ebcdic_to_ascii` and the
/// inverse fills `ascii_to_ebcdic`, 512 bytes fills both halves directly. Returns `0` on success, `-1` on
/// any error. This is the one cconv function that does filesystem + environment I/O (faithful to libcob).
pub fn cob_load_collation(
    col_name: &str,
    ebcdic_to_ascii: Option<&mut [u8]>,
    ascii_to_ebcdic: Option<&mut [u8]>,
) -> i32 {
    let cn = col_name.as_bytes();
    let is_path = cn.first() == Some(&b'.') || cn.first() == Some(&b'/');
    let filename: std::path::PathBuf = if is_path {
        std::path::PathBuf::from(col_name)
    } else {
        let config_dir = std::env::var("COB_CONFIG_DIR").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(format!("{config_dir}/{col_name}.ttbl"))
    };

    let contents = match std::fs::read(&filename) {
        Ok(c) => c,
        Err(_) => return -1,
    };

    let mut table = [0u8; 512];
    let mut i = 0usize;
    let mut had_error = false;
    // Iterate "lines" (the C uses fgets); split on '\n'.
    for line in contents.split(|&b| b == b'\n') {
        let mut hexptr = cob_skip_blanks(line);
        while !hexptr.is_empty() && hexptr[0] != b'#' {
            let c = cob_convert_hex_byte(hexptr);
            if c < 0 {
                had_error = true;
            }
            if i < 512 {
                table[i] = c as u8; // (unsigned char)(-1) == 255, exactly as the C stores it
                i += 1;
            } else {
                return -1; // too much data
            }
            // advance past the 2 hex chars + following blanks
            hexptr = if hexptr.len() >= 2 {
                cob_skip_blanks(&hexptr[2..])
            } else {
                &[]
            };
        }
    }

    if i != 256 && i != 512 {
        return -1; // not enough / too much data
    }

    if let Some(e2a) = ebcdic_to_ascii {
        let n = e2a.len().min(256);
        e2a[..n].copy_from_slice(&table[..n]);
    }
    if let Some(a2e) = ascii_to_ebcdic {
        if i == 512 {
            let n = a2e.len().min(256);
            a2e[..n].copy_from_slice(&table[256..256 + n]);
        } else {
            for k in 0..256 {
                let idx = table[k] as usize;
                if idx < a2e.len() {
                    a2e[idx] = k as u8;
                }
            }
        }
    }

    if had_error {
        -1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_fold_is_7bit_ascii_only() {
        assert_eq!(cob_toupper(b'a'), b'A');
        assert_eq!(cob_toupper(b'z'), b'Z');
        assert_eq!(cob_toupper(b'A'), b'A'); // already upper, lower_tab['A']==0
        assert_eq!(cob_toupper(b'5'), b'5');
        assert_eq!(cob_toupper(0xE9), 0xE9); // byte >= 128 unchanged (not locale)
        assert_eq!(cob_tolower(b'A'), b'a');
        assert_eq!(cob_tolower(b'Z'), b'z');
        assert_eq!(cob_tolower(b'a'), b'a');
        assert_eq!(cob_tolower(b'!'), b'!');
        // init_upper_lower must reproduce the const tables exactly.
        let (lo, up) = init_upper_lower();
        for c in 0u8..=255 {
            let e_up = if lo[c as usize] != 0 { lo[c as usize] } else { c };
            let e_lo = if up[c as usize] != 0 { up[c as usize] } else { c };
            assert_eq!(cob_toupper(c), e_up);
            assert_eq!(cob_tolower(c), e_lo);
        }
    }

    #[test]
    fn hex_parsing() {
        assert_eq!(cob_convert_hex_digit(b'0'), 0);
        assert_eq!(cob_convert_hex_digit(b'9'), 9);
        assert_eq!(cob_convert_hex_digit(b'a'), 10);
        assert_eq!(cob_convert_hex_digit(b'F'), 15);
        assert_eq!(cob_convert_hex_digit(b'g'), -1);
        assert_eq!(cob_convert_hex_byte(b"ff"), 255);
        assert_eq!(cob_convert_hex_byte(b"00"), 0);
        assert_eq!(cob_convert_hex_byte(b"4A"), 0x4A);
        assert_eq!(cob_convert_hex_byte(b"zz"), -1);
    }

    #[test]
    fn skip_blanks() {
        assert_eq!(cob_skip_blanks(b"   abc"), b"abc");
        assert_eq!(cob_skip_blanks(b"\t\n x"), b"x");
        assert_eq!(cob_skip_blanks(b"abc"), b"abc");
    }

    #[test]
    fn field_to_string_rtrim_and_fold() {
        let mut out = [0u8; 32];
        let f = FieldRef { size: 8, data: Some(b"HeLLo   ") };
        let n = cob_field_to_string(Some(&f), &mut out, CobCase::Lower);
        assert_eq!(n, 5);
        assert_eq!(&out[..6], b"hello\0");
        let n = cob_field_to_string(Some(&f), &mut out, CobCase::Upper);
        assert_eq!(n, 5);
        assert_eq!(&out[..6], b"HELLO\0");
        // all spaces -> 0
        let blank = FieldRef { size: 4, data: Some(b"    ") };
        assert_eq!(cob_field_to_string(Some(&blank), &mut out, CobCase::None), 0);
        // NULL field / zero size / NULL data
        assert_eq!(cob_field_to_string(None, &mut out, CobCase::None), -1);
        assert_eq!(cob_field_to_string(Some(&FieldRef { size: 0, data: Some(b"") }), &mut out, CobCase::None), -2);
        assert_eq!(cob_field_to_string(Some(&FieldRef { size: 4, data: None }), &mut out, CobCase::None), -3);
    }
}
