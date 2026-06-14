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
}
