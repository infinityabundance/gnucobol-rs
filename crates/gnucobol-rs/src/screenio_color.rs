//! Port of the color / attribute *computation* helpers of GnuCOBOL 3.2 `libcob/screenio.c`
//! (`cob_get_color_pair`, `cob_activate_color_pair`, `cob_to_curses_color`,
//! `adjust_attr_from_extended_color`, `adjust_attr_from_color_field`,
//! `get_cob_color_from_color_value`, `handle_control_field_color`, `screen_attr_cmp`,
//! `adjust_attr_from_control_field`, `cob_screen_attr`).
//!
//! These functions translate a COBOL `SCREEN SECTION` item's attributes -- `FOREGROUND-COLOR` /
//! `BACKGROUND-COLOR`, `HIGHLIGHT` / `LOWLIGHT` / `REVERSE-VIDEO` / `BLINK` / `UNDERLINE`, the ACU/RM
//! numeric `COLOR` field, and the named `CONTROL` field -- into curses **attribute flag words**
//! (`chtype` style bits) and **color-pair numbers**. The arithmetic and the bit logic are entirely
//! deterministic and are reproduced here byte-for-byte from the C.
//!
//! ## Claim boundary (read this)
//!
//! The functions ported here are the *pure computation*: given the COBOL attribute/color inputs they
//! return the exact curses `A_*` style word, the exact `COLOR_*` numbers, and the exact color-pair
//! registry decisions GnuCOBOL computes. The **ncurses leaf calls** are the documented boundary and are
//! **not** performed here:
//!   * `init_pair` / `pair_content` (the live curses color-pair table) -- modeled as an explicit
//!     [`ColorPairRegistry`] struct so the *selection arithmetic* of `cob_get_color_pair` is exercised
//!     without touching a terminal.
//!   * `color_set` / `attrset` / `bkgdset` (applying the pair) -- [`cob_activate_color_pair`] returns the
//!     `COLOR_PAIR(n)` attr value it would hand to ncurses; the side-effecting call is the boundary.
//!   * `attrset` / `attron` and the `clear`/`clrtoeol`/`clrtobot`/`cob_beep` screen side-effects in
//!     `cob_screen_attr` -- [`cob_screen_attr`] returns the assembled style word + the resolved color
//!     pair + the (possibly-adjusted) `attr` flags; emitting them to the terminal is the boundary.
//!
//! The `A_OVERLINE` / `A_LEFTLINE` / `A_RIGHTLINE` style bits are guarded by `#if defined (...)` in the
//! C and are **not defined** on the admitted host's ncurses 6.6 build (`libncursesw.so.6`); the port
//! reproduces the admitted host -- those `OVERLINE`/`LEFTLINE`/`RIGHTLINE` COBOL attributes therefore
//! contribute no style bit, exactly as on the oracle. See [`A_OVERLINE`] etc.
//!
//! `cob_field` operands (`cob_get_int`, the `CONTROL` text) are passed here as already-parsed integers
//! / byte slices: the field-decode layer (`numeric.c` / `move.c`) is sealed in its own courts and is
//! intentionally not re-entered here.
#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// curses COLOR_* constants (ncurses 6.6, the admitted host).
// `cob_to_curses_color` maps a COBOL color number onto exactly these.
// ---------------------------------------------------------------------------

/// curses `COLOR_BLACK`.
pub const COLOR_BLACK: i16 = 0;
/// curses `COLOR_RED`.
pub const COLOR_RED: i16 = 1;
/// curses `COLOR_GREEN`.
pub const COLOR_GREEN: i16 = 2;
/// curses `COLOR_YELLOW`.
pub const COLOR_YELLOW: i16 = 3;
/// curses `COLOR_BLUE`.
pub const COLOR_BLUE: i16 = 4;
/// curses `COLOR_MAGENTA`.
pub const COLOR_MAGENTA: i16 = 5;
/// curses `COLOR_CYAN`.
pub const COLOR_CYAN: i16 = 6;
/// curses `COLOR_WHITE`.
pub const COLOR_WHITE: i16 = 7;

// ---------------------------------------------------------------------------
// COBOL color numbers (`enum cob_colors`, common.h). These are the *COBOL* color
// indices 0..7, NOT the curses COLOR_* values; `cob_to_curses_color` translates.
// ---------------------------------------------------------------------------

/// COBOL color `BLACK` (`enum cob_colors`).
pub const COB_SCREEN_BLACK: i32 = 0;
/// COBOL color `BLUE`.
pub const COB_SCREEN_BLUE: i32 = 1;
/// COBOL color `GREEN`.
pub const COB_SCREEN_GREEN: i32 = 2;
/// COBOL color `CYAN`.
pub const COB_SCREEN_CYAN: i32 = 3;
/// COBOL color `RED`.
pub const COB_SCREEN_RED: i32 = 4;
/// COBOL color `MAGENTA`.
pub const COB_SCREEN_MAGENTA: i32 = 5;
/// COBOL color `YELLOW`.
pub const COB_SCREEN_YELLOW: i32 = 6;
/// COBOL color `WHITE`.
pub const COB_SCREEN_WHITE: i32 = 7;

// ---------------------------------------------------------------------------
// COB_SCREEN_* attribute flags (`cob_flags_t`, common.h) -- 1 << n, cob_flags_t is s64.
// ---------------------------------------------------------------------------

/// `AUTO` / `AUTO-SKIP` (`1 << 4`).
pub const COB_SCREEN_AUTO: i64 = 1 << 4;
/// `BEEP` / `BELL` (`1 << 5`).
pub const COB_SCREEN_BELL: i64 = 1 << 5;
/// `BLANK LINE` (`1 << 6`).
pub const COB_SCREEN_BLANK_LINE: i64 = 1 << 6;
/// `BLANK SCREEN` (`1 << 7`).
pub const COB_SCREEN_BLANK_SCREEN: i64 = 1 << 7;
/// `BLINK` (`1 << 8`).
pub const COB_SCREEN_BLINK: i64 = 1 << 8;
/// `ERASE EOL` (`1 << 9`).
pub const COB_SCREEN_ERASE_EOL: i64 = 1 << 9;
/// `ERASE EOS` (`1 << 10`).
pub const COB_SCREEN_ERASE_EOS: i64 = 1 << 10;
/// `FULL` / `LENGTH-CHECK` (`1 << 11`).
pub const COB_SCREEN_FULL: i64 = 1 << 11;
/// `HIGH` / `HIGHLIGHT` (`1 << 12`).
pub const COB_SCREEN_HIGHLIGHT: i64 = 1 << 12;
/// `LOW` / `LOWLIGHT` (`1 << 13`).
pub const COB_SCREEN_LOWLIGHT: i64 = 1 << 13;
/// `REQUIRED` (`1 << 14`).
pub const COB_SCREEN_REQUIRED: i64 = 1 << 14;
/// `REVERSE` / `REVERSE-VIDEO` (`1 << 15`).
pub const COB_SCREEN_REVERSE: i64 = 1 << 15;
/// `NO-ECHO` / `OFF` (`1 << 16`).
pub const COB_SCREEN_SECURE: i64 = 1 << 16;
/// `UNDERLINE` (`1 << 17`).
pub const COB_SCREEN_UNDERLINE: i64 = 1 << 17;
/// `OVERLINE` (`1 << 18`).
pub const COB_SCREEN_OVERLINE: i64 = 1 << 18;
/// `PROMPT` (`1 << 19`).
pub const COB_SCREEN_PROMPT: i64 = 1 << 19;
/// `UPDATE` (`1 << 20`).
pub const COB_SCREEN_UPDATE: i64 = 1 << 20;
/// `ECHO` (mapped to `NO_ECHO` flag, `1 << 24`).
pub const COB_SCREEN_NO_ECHO: i64 = 1 << 24;
/// `LEFTLINE` (`1 << 25`).
pub const COB_SCREEN_LEFTLINE: i64 = 1 << 25;
/// `UPPER` (`1 << 28`).
pub const COB_SCREEN_UPPER: i64 = 1 << 28;
/// `LOWER` (`1 << 29`).
pub const COB_SCREEN_LOWER: i64 = 1 << 29;
/// `CONVERT` (`1 << 30`).
pub const COB_SCREEN_CONV: i64 = 1 << 30;
/// `GRAPHICS` (`1 << 31`).
pub const COB_SCREEN_GRAPHICS: i64 = 1 << 31;
/// `RIGHTLINE` (`1 << 32`).
pub const COB_SCREEN_RIGHTLINE: i64 = 1 << 32;
/// `TAB` (`1 << 33`).
pub const COB_SCREEN_TAB: i64 = 1 << 33;
/// `PROTECT` (`NO_UPDATE`, `1 << 34`).
pub const COB_SCREEN_NO_UPDATE: i64 = 1 << 34;
/// `GRID` (`1 << 36`).
pub const COB_SCREEN_GRID: i64 = 1 << 36;

// ---------------------------------------------------------------------------
// curses chtype attribute (style) bits -- NCURSES_BITS(1, shift) == 1 << (shift + 8),
// NCURSES_ATTR_SHIFT == 8 (ncurses 6.6, admitted host).
// ---------------------------------------------------------------------------

/// curses `A_NORMAL` -- no style bits.
pub const A_NORMAL: u64 = 0;
/// curses `A_UNDERLINE` (`NCURSES_BITS(1,9)` == `1 << 17`).
pub const A_UNDERLINE: u64 = 1 << 17;
/// curses `A_REVERSE` (`NCURSES_BITS(1,10)` == `1 << 18`).
pub const A_REVERSE: u64 = 1 << 18;
/// curses `A_BLINK` (`NCURSES_BITS(1,11)` == `1 << 19`).
pub const A_BLINK: u64 = 1 << 19;
/// curses `A_DIM` (`NCURSES_BITS(1,12)` == `1 << 20`).
pub const A_DIM: u64 = 1 << 20;
/// curses `A_BOLD` (`NCURSES_BITS(1,13)` == `1 << 21`).
pub const A_BOLD: u64 = 1 << 21;

/// `A_OVERLINE` is **not defined** on the admitted host's ncurses build; the `#if defined (A_OVERLINE)`
/// guard in `cob_screen_attr` is false, so `COB_SCREEN_OVERLINE` contributes no style bit. Recorded as
/// `None` to make that boundary explicit.
pub const A_OVERLINE: Option<u64> = None;
/// `A_LEFTLINE` -- not defined on the admitted host (see [`A_OVERLINE`]).
pub const A_LEFTLINE: Option<u64> = None;
/// `A_RIGHTLINE` -- not defined on the admitted host (see [`A_OVERLINE`]).
pub const A_RIGHTLINE: Option<u64> = None;

/// `COLOR_PAIR(n)` -- the curses macro that packs a color-pair number into a `chtype` attribute:
/// `NCURSES_BITS(n,0) & A_COLOR` == `(n << 8) & ((256-1) << 8)`. This is the value `attrset`/`bkgdset`
/// receive; computing it is in scope, issuing the call is the boundary.
pub fn cob_color_pair(n: i16) -> u64 {
    // A_COLOR == NCURSES_BITS(((1U) << 8) - 1U, 0) == 255 << 8.
    const A_COLOR: u64 = 255 << 8;
    (((n as u64) & 0xFFFF_FFFF) << 8) & A_COLOR
}

// ===========================================================================
// cob_to_curses_color
// ===========================================================================

/// Port of `cob_to_curses_color` (`screenio.c`). Maps a COBOL color number onto a curses `COLOR_*`
/// constant. Only the low 3 bits select the color (bit 3 / value 8 is the "extended attribute" flag,
/// handled by [`adjust_attr_from_extended_color`]); a color `< 0` or `> 15` is invalid.
///
/// Returns `Ok(curses_color)` on success (mirrors C returning `0` and writing `*curses_color`), or
/// `Err(())` for an invalid color (mirrors C returning `-1` and leaving `*curses_color` untouched).
pub fn cob_to_curses_color(cob_color: i32) -> Result<i16, ()> {
    if cob_color < 0 || cob_color > 15 {
        // "invalid" color - nothing to do
        return Err(());
    }
    // compat for MF/ACU/... only use first 3 bits -> 0-7,
    // bit 4 is "included highlight/blink attribute"
    match cob_color & 7 {
        COB_SCREEN_BLACK => Ok(COLOR_BLACK),
        COB_SCREEN_BLUE => Ok(COLOR_BLUE),
        COB_SCREEN_GREEN => Ok(COLOR_GREEN),
        COB_SCREEN_CYAN => Ok(COLOR_CYAN),
        COB_SCREEN_RED => Ok(COLOR_RED),
        COB_SCREEN_MAGENTA => Ok(COLOR_MAGENTA),
        COB_SCREEN_YELLOW => Ok(COLOR_YELLOW),
        COB_SCREEN_WHITE => Ok(COLOR_WHITE),
        _ => Err(()),
    }
}

// ===========================================================================
// adjust_attr_from_extended_color
// ===========================================================================

/// Port of `adjust_attr_from_extended_color` (`screenio.c`). For a "valid" color (`>= 0`) whose bit 3
/// (value `8`) is set -- the MF/ACU "extended" color carrying a highlight/blink -- OR the `BLINK` flag
/// (background side) / the `HIGHLIGHT` flag (foreground side) into `*attr`. Mutates `attr` in place,
/// matching the C `cob_flags_t *attr`.
pub fn adjust_attr_from_extended_color(attr: &mut i64, color: i32, handle_background: i32) {
    // check for "valid" color and "extended attribute" set
    if color >= 0 && (color & 8) != 0 {
        if handle_background != 0 {
            *attr |= COB_SCREEN_BLINK;
        } else {
            *attr |= COB_SCREEN_HIGHLIGHT;
        }
    }
}

// ===========================================================================
// adjust_attr_from_color_field
// ===========================================================================

/// Port of `adjust_attr_from_color_field` (`screenio.c`). Decomposes the ACU/RM numeric `COLOR` field
/// (a sum of attribute weights) into attribute flags plus foreground/background curses colors,
/// subtracting each recognized weight in descending order.
///
/// NOTE (faithful bug reproduction): the C assigns `COB_SCREEN_CYAN` (== `3`) -- *not* `COLOR_CYAN`
/// (== `6`) -- for both the background-cyan and foreground-cyan branches. That off-by mapping is
/// reproduced verbatim. `col_attr` is the already-decoded integer value of the field (`cob_get_int`).
/// Mutates `attr`, `fg_color`, `bg_color` in place.
pub fn adjust_attr_from_color_field(
    attr: &mut i64,
    col_attr_in: i32,
    fg_color: &mut i16,
    bg_color: &mut i16,
) {
    let mut col_attr = col_attr_in; // added numeric value

    if col_attr >= 131072 {
        col_attr -= 131072;
        // BACKGROUND-HIGH pending
    }
    if col_attr >= 65536 {
        col_attr -= 65536;
        // BACKGROUND-LOW pending
    }
    if col_attr >= 32768 {
        col_attr -= 32768;
        // Protected
    }
    if col_attr >= 16384 {
        col_attr -= 16384;
        *attr |= COB_SCREEN_BLINK;
    }
    if col_attr >= 8192 {
        col_attr -= 8192;
        *attr |= COB_SCREEN_UNDERLINE;
    }
    if col_attr >= 4096 {
        col_attr -= 4096;
        *attr |= COB_SCREEN_HIGHLIGHT;
    }
    if col_attr >= 2048 {
        col_attr -= 2048;
        *attr |= COB_SCREEN_LOWLIGHT;
    }
    if col_attr >= 1024 {
        col_attr -= 1024;
        *attr |= COB_SCREEN_REVERSE;
    }

    if col_attr >= 256 {
        col_attr -= 256;
        *bg_color = COLOR_WHITE;
    } else if col_attr >= 224 {
        col_attr -= 224;
        *bg_color = COLOR_YELLOW;
    } else if col_attr >= 192 {
        col_attr -= 192;
        *bg_color = COLOR_MAGENTA;
    } else if col_attr >= 160 {
        col_attr -= 160;
        *bg_color = COLOR_RED;
    } else if col_attr >= 128 {
        col_attr -= 128;
        *bg_color = COB_SCREEN_CYAN as i16; // C: COB_SCREEN_CYAN (== 3), not COLOR_CYAN
    } else if col_attr >= 96 {
        col_attr -= 96;
        *bg_color = COLOR_GREEN;
    } else if col_attr >= 64 {
        col_attr -= 64;
        *bg_color = COLOR_BLUE;
    } else if col_attr >= 32 {
        col_attr -= 32;
        *bg_color = COLOR_BLACK;
    }

    if col_attr >= 8 {
        col_attr -= 8;
        *fg_color = COLOR_WHITE;
    } else if col_attr >= 7 {
        col_attr -= 7;
        *fg_color = COLOR_YELLOW;
    } else if col_attr >= 6 {
        col_attr -= 6;
        *fg_color = COLOR_MAGENTA;
    } else if col_attr >= 5 {
        col_attr -= 5;
        *fg_color = COLOR_RED;
    } else if col_attr >= 4 {
        col_attr -= 4;
        *fg_color = COB_SCREEN_CYAN as i16; // C: COB_SCREEN_CYAN (== 3), not COLOR_CYAN
    } else if col_attr >= 3 {
        col_attr -= 3;
        *fg_color = COLOR_GREEN;
    } else if col_attr >= 2 {
        col_attr -= 2;
        *fg_color = COLOR_BLUE;
    } else if col_attr >= 1 {
        col_attr -= 1;
        *fg_color = COLOR_BLACK;
    }
    // (col_attr is fully consumed; the final value is discarded, as in C)
    let _ = col_attr;
}

// ===========================================================================
// get_cob_color_from_color_value
// ===========================================================================

/// Port of `get_cob_color_from_color_value` (`screenio.c`). Translates a `CONTROL`-field color token
/// (`p`, length `len`) into a COBOL color number: first try `strtol`-style base-10 parsing of a leading
/// numeric prefix; if no digits parsed, fall back to the exact length-keyed color name table
/// (`"BLACK"`, `"WHITE"`, `"GREEN"`, `"BLUE"`, `"CYAN"`, `"RED"`, `"MAGENTA"`, `"YELLOW"`). Returns
/// `-1` for an unrecognized token (mirrors the C `-1`).
pub fn get_cob_color_from_color_value(p: &[u8], len: usize) -> i32 {
    // translate number: strtol base 10 -- parse a leading optional sign + digits; if no digit
    // consumed (endptr == p) fall through to the text table.
    {
        if let Some(n) = strtol_base10_prefix(p) {
            return n;
        }
    }

    // text translation -- exact, length-keyed (memcmp of the literal C lengths).
    if len == 5 {
        if mem_eq(p, b"BLACK") {
            return COB_SCREEN_BLACK;
        }
        if mem_eq(p, b"WHITE") {
            return COB_SCREEN_WHITE;
        }
        if mem_eq(p, b"GREEN") {
            return COB_SCREEN_GREEN;
        }
        return -1;
    }
    if len == 4 {
        if mem_eq(p, b"BLUE") {
            return COB_SCREEN_BLUE;
        }
        if mem_eq(p, b"CYAN") {
            return COB_SCREEN_CYAN;
        }
        return -1;
    }
    if len == 3 {
        if mem_eq(p, b"RED") {
            return COB_SCREEN_RED;
        }
        return -1;
    }
    if len == 7 {
        if mem_eq(p, b"MAGENTA") {
            return COB_SCREEN_MAGENTA;
        }
        return -1;
    }
    if len == 6 && mem_eq(p, b"YELLOW") {
        return COB_SCREEN_YELLOW;
    }

    -1
}

/// `memcmp(p, lit, lit.len()) == 0` over the leading `lit.len()` bytes (the C `memcmp` guarded by the
/// matching `len ==` check). Returns false if `p` is shorter than `lit`.
fn mem_eq(p: &[u8], lit: &[u8]) -> bool {
    p.len() >= lit.len() && &p[..lit.len()] == lit
}

/// Reproduce the relevant behaviour of `strtol(p, &endptr, 10)` for the color path: parse an optional
/// leading `+`/`-` sign followed by one or more ASCII digits. Returns `Some(value)` when at least one
/// digit is consumed (C: `endptr != p`), else `None` (C falls through to the text table). Leading
/// whitespace is also skipped, matching `strtol`. Saturates rather than overflowing.
fn strtol_base10_prefix(p: &[u8]) -> Option<i32> {
    let mut i = 0usize;
    // strtol skips leading isspace()
    while i < p.len() && matches!(p[i], b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c) {
        i += 1;
    }
    let mut neg = false;
    if i < p.len() && (p[i] == 0x2b || p[i] == 0x2d) {
        // '+' / '-'
        neg = p[i] == 0x2d;
        i += 1;
    }
    let digit_start = i;
    let mut acc: i64 = 0;
    while i < p.len() && p[i].is_ascii_digit() {
        acc = acc.saturating_mul(10).saturating_add((p[i] - b'0') as i64);
        i += 1;
    }
    if i == digit_start {
        // no digits consumed -> endptr == p (no number)
        return None;
    }
    if neg {
        acc = -acc;
    }
    // clamp to int range as a 32-bit C int would hold
    if acc > i32::MAX as i64 {
        acc = i32::MAX as i64;
    } else if acc < i32::MIN as i64 {
        acc = i32::MIN as i64;
    }
    Some(acc as i32)
}

// ===========================================================================
// handle_control_field_color
// ===========================================================================

/// Port of `handle_control_field_color` (`screenio.c`). Parse a `CONTROL`-field color token, translate
/// to a curses color, store it into `*curses_color`, and fold any "extended" highlight/blink attribute
/// into `*attr`. Returns `0` on success and `-1` if the token is not a valid color (mirrors the C int
/// return, on which `adjust_attr_from_control_field` decides whether to push the token back).
pub fn handle_control_field_color(
    attr: &mut i64,
    p: &[u8],
    len: usize,
    curses_color: &mut i16,
    handle_background: i32,
) -> i32 {
    let cob_color = get_cob_color_from_color_value(p, len);

    // translate to curses, early error return if not possible
    let curses_color_val = match cob_to_curses_color(cob_color) {
        Ok(v) => v,
        Err(()) => return -1,
    };

    // take over color
    *curses_color = curses_color_val;

    // attribute via renamed colors
    adjust_attr_from_extended_color(attr, cob_color, handle_background);

    0
}

// ===========================================================================
// control_attrs table + screen_attr_cmp
// ===========================================================================

/// A `CONTROL`-field keyword and the `cob_flags_t` it sets (`struct parse_control`). A `cobflag` of
/// `0` marks a *color* keyword (FOREGROUND/BACKGROUND/FCOLOR/BCOLOR); `-1` marks a known-but-not-applied
/// keyword (skipped). Mirrors `control_attrs[]` -- the entries are in the exact C order, which is
/// `strcmp`-sorted so `screen_attr_cmp` can `bsearch` it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseControl {
    /// The (upper-case) keyword token.
    pub keyword: &'static [u8],
    /// The `cob_flags_t` value (`0` = color keyword, `-1` = ignored).
    pub cobflag: i64,
}

/// Port of `control_attrs[]` (`screenio.c`) -- the binary-sorted known-attribute table. Order is the
/// verbatim C order (sorted by `strcmp` on the keyword) so [`screen_attr_cmp`]-based binary search
/// resolves the same entry the oracle does.
pub const CONTROL_ATTRS: &[ParseControl] = &[
    ParseControl { keyword: b"AUTO", cobflag: COB_SCREEN_AUTO },
    ParseControl { keyword: b"AUTO-SKIP", cobflag: COB_SCREEN_AUTO },
    ParseControl { keyword: b"BACKGROUND-COLOR", cobflag: 0 },
    ParseControl { keyword: b"BACKGROUND-COLOUR", cobflag: 0 },
    ParseControl { keyword: b"BCOLOR", cobflag: 0 },
    ParseControl { keyword: b"BEEP", cobflag: COB_SCREEN_BELL },
    ParseControl { keyword: b"BELL", cobflag: COB_SCREEN_BELL },
    ParseControl { keyword: b"BLANK LINE", cobflag: COB_SCREEN_BLANK_LINE },
    ParseControl { keyword: b"BLANK SCREEN", cobflag: COB_SCREEN_BLANK_SCREEN },
    ParseControl { keyword: b"BLINK", cobflag: COB_SCREEN_BLINK },
    ParseControl { keyword: b"CONVERT", cobflag: COB_SCREEN_CONV },
    ParseControl { keyword: b"ECHO", cobflag: COB_SCREEN_NO_ECHO },
    ParseControl { keyword: b"EMPTY-CHECK", cobflag: -1 },
    ParseControl { keyword: b"ERASE EOL", cobflag: COB_SCREEN_ERASE_EOL },
    ParseControl { keyword: b"ERASE EOS", cobflag: COB_SCREEN_ERASE_EOS },
    ParseControl { keyword: b"FCOLOR", cobflag: 0 },
    ParseControl { keyword: b"FOREGROUND-COLOR", cobflag: 0 },
    ParseControl { keyword: b"FOREGROUND-COLOUR", cobflag: 0 },
    ParseControl { keyword: b"FULL", cobflag: COB_SCREEN_FULL },
    ParseControl { keyword: b"GRAPHICS", cobflag: COB_SCREEN_GRAPHICS },
    ParseControl { keyword: b"GRID", cobflag: COB_SCREEN_GRID },
    ParseControl { keyword: b"HIGH", cobflag: COB_SCREEN_HIGHLIGHT },
    ParseControl { keyword: b"HIGHLIGHT", cobflag: COB_SCREEN_HIGHLIGHT },
    ParseControl { keyword: b"JUST", cobflag: -1 },
    ParseControl { keyword: b"JUSTIFY", cobflag: -1 },
    ParseControl { keyword: b"LEFTLINE", cobflag: COB_SCREEN_LEFTLINE },
    ParseControl { keyword: b"LENGTH-CHECK", cobflag: COB_SCREEN_FULL },
    ParseControl { keyword: b"LOW", cobflag: COB_SCREEN_LOWLIGHT },
    ParseControl { keyword: b"LOWER", cobflag: COB_SCREEN_LOWER },
    ParseControl { keyword: b"LOWLIGHT", cobflag: COB_SCREEN_LOWLIGHT },
    ParseControl { keyword: b"NO-ECHO", cobflag: COB_SCREEN_SECURE },
    ParseControl { keyword: b"OFF", cobflag: COB_SCREEN_SECURE },
    ParseControl { keyword: b"OVERLINE", cobflag: COB_SCREEN_OVERLINE },
    ParseControl { keyword: b"PROMPT", cobflag: COB_SCREEN_PROMPT },
    ParseControl { keyword: b"PROTECT", cobflag: COB_SCREEN_NO_UPDATE },
    ParseControl { keyword: b"REQUIRED", cobflag: COB_SCREEN_REQUIRED },
    ParseControl { keyword: b"REVERSE", cobflag: COB_SCREEN_REVERSE },
    ParseControl { keyword: b"REVERSE-VIDEO", cobflag: COB_SCREEN_REVERSE },
    ParseControl { keyword: b"RIGHT-JUSTIFY", cobflag: -1 },
    ParseControl { keyword: b"RIGHTLINE", cobflag: COB_SCREEN_RIGHTLINE },
    ParseControl { keyword: b"TAB", cobflag: COB_SCREEN_TAB },
    ParseControl { keyword: b"TRAILING", cobflag: -1 },
    ParseControl { keyword: b"TRAILING-SIGN", cobflag: -1 },
    ParseControl { keyword: b"UNDERLINE", cobflag: COB_SCREEN_UNDERLINE },
    ParseControl { keyword: b"UPDATE", cobflag: COB_SCREEN_UPDATE },
    ParseControl { keyword: b"UPPER", cobflag: COB_SCREEN_UPPER },
    ParseControl { keyword: b"ZERO-FILL", cobflag: -1 },
];

/// Port of `screen_attr_cmp` (`screenio.c`) -- the `bsearch`/`qsort` comparator: `strcmp(key,
/// entry->keyword)`. The C key is a NUL-terminated string; here it is a byte slice (the token). Returns
/// the sign of the `strcmp`-style comparison as `-1` / `0` / `1`.
pub fn screen_attr_cmp(key: &[u8], entry: &ParseControl) -> i32 {
    c_strcmp(key, entry.keyword)
}

/// `strcmp`-style lexicographic byte comparison (`unsigned char` semantics, NUL-terminator implied by
/// slice end). Returns `-1`, `0`, or `1`.
fn c_strcmp(a: &[u8], b: &[u8]) -> i32 {
    let n = a.len().min(b.len());
    let mut i = 0;
    while i < n {
        if a[i] != b[i] {
            return if a[i] < b[i] { -1 } else { 1 };
        }
        i += 1;
    }
    // shorter (its NUL terminator '\0' == 0) sorts before the longer.
    match a.len().cmp(&b.len()) {
        core::cmp::Ordering::Less => -1,
        core::cmp::Ordering::Equal => 0,
        core::cmp::Ordering::Greater => 1,
    }
}

/// Binary search of [`CONTROL_ATTRS`] for `key` using [`screen_attr_cmp`], reproducing the C `bsearch`.
/// Returns the matched entry, or `None` (the C `NULL`).
pub fn bsearch_control_attr(key: &[u8]) -> Option<&'static ParseControl> {
    let mut lo = 0isize;
    let mut hi = CONTROL_ATTRS.len() as isize - 1;
    while lo <= hi {
        let mid = (lo + hi) / 2;
        let entry = &CONTROL_ATTRS[mid as usize];
        match screen_attr_cmp(key, entry) {
            0 => return Some(entry),
            c if c < 0 => hi = mid - 1,
            _ => lo = mid + 1,
        }
    }
    None
}

// ===========================================================================
// adjust_attr_from_control_field
// ===========================================================================

/// Tokenize a `CONTROL` field's text the way `adjust_attr_from_control_field` does after
/// `cob_field_to_string(..., CCM_UPPER)`: split on `,; \n\r\t`, and when a token contains `=` push the
/// pre-`=` part, then a standalone `"="`, then continue from after the `=`. The text is taken as
/// already upper-cased (the case-fold is the sealed `cob_field_to_string` layer). Returns the token
/// list (capped at `COB_MINI_MAX` like the C, using the same cap value).
fn tokenize_control(buffer: &[u8]) -> Vec<Vec<u8>> {
    /// `COB_MINI_MAX` -- the C token cap (`token[COB_MINI_BUFF]`, break at `COB_MINI_MAX`).
    const COB_MINI_MAX: usize = 255;
    let delim = |c: u8| matches!(c, b',' | b';' | b' ' | b'\n' | b'\r' | b'\t');

    // First pass: strtok-style split into raw tokens.
    let mut raw: Vec<Vec<u8>> = Vec::new();
    let mut i = 0usize;
    while i < buffer.len() {
        while i < buffer.len() && delim(buffer[i]) {
            i += 1;
        }
        if i >= buffer.len() {
            break;
        }
        let start = i;
        while i < buffer.len() && !delim(buffer[i]) {
            i += 1;
        }
        raw.push(buffer[start..i].to_vec());
    }

    // Second pass: the C `=` splitting on each strtok token.
    let mut tokens: Vec<Vec<u8>> = Vec::new();
    for tok in raw {
        if tokens.len() == COB_MINI_MAX {
            break;
        }
        // C: only splits when p[1] != 0 (token length >= 2) and an '=' is present.
        if tok.len() >= 2 {
            if let Some(eq) = tok.iter().position(|&c| c == b'=') {
                // not first char -> push the first part
                if eq != 0 {
                    tokens.push(tok[..eq].to_vec());
                    if tokens.len() == COB_MINI_MAX {
                        break;
                    }
                }
                // push the equal sign
                tokens.push(b"=".to_vec());
                if tokens.len() == COB_MINI_MAX {
                    break;
                }
                // remainder after '=' (C advances p = q+1; if *p == 0 it skips to next strtok token,
                // which for our pre-split tokens means simply nothing more to push here).
                let rest = &tok[eq + 1..];
                if !rest.is_empty() {
                    tokens.push(rest.to_vec());
                    if tokens.len() == COB_MINI_MAX {
                        break;
                    }
                }
                continue;
            }
        }
        tokens.push(tok);
    }
    tokens
}

/// Port of `adjust_attr_from_control_field` (`screenio.c`). Tokenize the (upper-cased) `CONTROL` text,
/// then walk the tokens applying / clearing attribute flags, handling the `NO` negation prefix, the
/// two-token `ERASE EOL/EOS` and `BLANK LINE/SCREEN` keywords, and color keywords (with their optional
/// `IS` / `=` separator). Mutates `attr`, `fg_color`, `bg_color` in place. `control_text` is the
/// already-upper-cased field text (the `cob_field_to_string` decode is the sealed boundary).
pub fn adjust_attr_from_control_field(
    attr: &mut i64,
    control_text: &[u8],
    fg_color: &mut i16,
    bg_color: &mut i16,
) {
    let tokens = tokenize_control(control_text);
    let max_tokens = tokens.len();
    if max_tokens == 0 {
        // no token - early exit
        return;
    }

    let mut i = 0usize;
    while i < max_tokens {
        let mut keyword: &[u8] = &tokens[i];
        let mut len = keyword.len();

        let mut no_indicator = 0;
        let bg_color_token;

        // negation
        if len == 2 && &keyword[..2] == b"NO" {
            i += 1;
            if i == max_tokens {
                break;
            }
            keyword = &tokens[i];
            len = keyword.len();
            no_indicator = 1;
        }

        // two-token keywords -- ERASE EOL / ERASE EOS
        let mut combined: Vec<u8> = Vec::new();
        if len == 5 && mem_eq(keyword, b"ERASE") {
            i += 1;
            if i == max_tokens {
                break;
            }
            keyword = &tokens[i];
            len = keyword.len();
            if len == 3 && mem_eq(keyword, b"EOL") {
                combined = b"ERASE EOL".to_vec();
            } else if len == 3 && mem_eq(keyword, b"EOS") {
                combined = b"ERASE EOS".to_vec();
            } else {
                i += 1;
                continue;
            }
        } else if len == 5 && mem_eq(keyword, b"BLANK") {
            i += 1;
            if i == max_tokens {
                break;
            }
            keyword = &tokens[i];
            len = keyword.len();
            if len == 4 && mem_eq(keyword, b"LINE") {
                combined = b"BLANK LINE".to_vec();
            } else if len == 6 && mem_eq(keyword, b"SCREEN") {
                combined = b"BLANK SCREEN".to_vec();
            } else {
                i += 1;
                continue;
            }
        }
        // After a successful two-token merge, `keyword` becomes the combined literal.
        let lookup_key: Vec<u8> = if combined.is_empty() {
            keyword.to_vec()
        } else {
            combined.clone()
        };

        // find token and get its attribute
        {
            match bsearch_control_attr(&lookup_key) {
                None => {
                    // skip unknown
                    i += 1;
                    continue;
                }
                Some(ca) if ca.cobflag == -1 => {
                    // not implemented
                    i += 1;
                    continue;
                }
                Some(ca) if ca.cobflag != 0 => {
                    // normal attribute - apply and go on
                    if no_indicator == 0 {
                        *attr |= ca.cobflag;
                    } else {
                        *attr &= !(ca.cobflag);
                    }
                    i += 1;
                    continue;
                }
                Some(_) => {
                    // cobflag == 0 -> color attribute, fall through below
                }
            }
        }

        // color attribute - check next token. C: bg_color_token = *keyword == 'B'.
        bg_color_token = lookup_key.first() == Some(&b'B');
        i += 1;
        if i == max_tokens {
            break;
        }
        let mut ckey: &[u8] = &tokens[i];
        let mut clen = ckey.len();

        // skip optional IS / = token
        if (clen == 2 && mem_eq(ckey, b"IS")) || (clen == 1 && ckey[0] == b'=') {
            i += 1;
            if i == max_tokens {
                break;
            }
            ckey = &tokens[i];
            clen = ckey.len();
        }

        // parse and handle color keyword
        {
            let ret = if bg_color_token {
                handle_control_field_color(attr, ckey, clen, bg_color, 1)
            } else {
                handle_control_field_color(attr, ckey, clen, fg_color, 0)
            };
            // if we could not parse the keyword as color value, push the non-color keyword back.
            if ret != 0 {
                i -= 1;
            }
        }
        let _ = no_indicator;
        i += 1;
    }
}

// ===========================================================================
// cob_get_color_pair / cob_activate_color_pair  (+ explicit registry)
// ===========================================================================

/// An explicit model of the curses color-pair table that `cob_get_color_pair` consults via
/// `pair_content` / `init_pair`. The live ncurses table is the boundary; this struct lets the
/// *selection arithmetic* run deterministically off-terminal.
///
/// `fore_color` / `back_color` are the terminal's default colors read during `cob_screen_init`
/// (pair 0). `pairs[n] = Some((fg, bg))` is a defined pair; `None` is a free slot. `max_color_pairs`
/// models `COLOR_PAIRS` (clamped to `SHRT_MAX - 1`).
#[derive(Debug, Clone)]
pub struct ColorPairRegistry {
    /// Terminal default foreground (curses color), pair 0's fg.
    pub fore_color: i16,
    /// Terminal default background (curses color), pair 0's bg.
    pub back_color: i16,
    /// `pairs[n]` = the content of curses color-pair `n` (`None` = undefined / free).
    pub pairs: Vec<Option<(i16, i16)>>,
    /// `COLOR_PAIRS` (clamped to `SHRT_MAX - 1`).
    pub max_color_pairs: i16,
}

impl ColorPairRegistry {
    /// A fresh registry with the given terminal defaults and `max_color_pairs` total pairs, pair 1
    /// reserved as "all black" (`fg=0, bg=0`) exactly as `cob_screen_init` sets it. `pairs[n]` for
    /// `n >= 2` start undefined (`pair_content` would report `(0, 0)` for an unused pair on ncurses,
    /// which `cob_get_color_pair` treats as a free slot -- modeled here as `None`).
    pub fn new(fore_color: i16, back_color: i16, max_color_pairs: i16) -> Self {
        let cap = if max_color_pairs < 0 { 0 } else { max_color_pairs as usize };
        let mut pairs = vec![None; cap];
        if cap > 0 {
            pairs[0] = Some((fore_color, back_color)); // pair 0 = terminal default
        }
        if cap > 1 {
            pairs[1] = Some((0, 0)); // reserved "all black"
        }
        ColorPairRegistry { fore_color, back_color, pairs, max_color_pairs }
    }

    /// `pair_content(n, &fg, &bg)`: an undefined pair reports `(0, 0)` (the ncurses default), which the
    /// caller treats as "free".
    fn pair_content(&self, n: i16) -> (i16, i16) {
        match self.pairs.get(n as usize).and_then(|p| *p) {
            Some((fg, bg)) => (fg, bg),
            None => (0, 0),
        }
    }

    /// `init_pair(n, fg, bg)`: define a previously-free pair. The live curses call is the boundary;
    /// here it records the definition so subsequent `cob_get_color_pair` calls see it.
    fn init_pair(&mut self, n: i16, fg: i16, bg: i16) {
        if let Some(slot) = self.pairs.get_mut(n as usize) {
            *slot = Some((fg, bg));
        }
    }
}

/// `SHRT_MAX` for the `COLOR_PAIRS` clamp in `cob_get_color_pair`.
pub const SHRT_MAX: i16 = 32767;

/// Port of `cob_get_color_pair` (`screenio.c`). Resolve the curses color-pair number for a
/// `(fg_color, bg_color)` request against the registry: pair `0` is the terminal default, pair `1` is
/// the reserved all-black; otherwise scan pairs `2..max` for an already-defined match or a free slot
/// (which it then `init_pair`s). Returns `0` (default) when none is free.
///
/// The `init_pair` write lands in the explicit [`ColorPairRegistry`] (the boundary), so the *selection*
/// is fully reproduced; `&mut registry` stands in for the curses global table.
pub fn cob_get_color_pair(registry: &mut ColorPairRegistry, fg_color: i16, bg_color: i16) -> i16 {
    // default color (defined from terminal, read during init)
    if fg_color == registry.fore_color && bg_color == registry.back_color {
        return 0;
    }
    // reserved color "all black", defined during init
    if fg_color == 0 && bg_color == 0 {
        return 1;
    }

    // limit to COLOR_PAIRS / SHRT_MAX - 1
    let max_clr_pairs = if registry.max_color_pairs < SHRT_MAX {
        registry.max_color_pairs
    } else {
        SHRT_MAX - 1
    };

    let mut color_pair_number: i16 = 2;
    while color_pair_number < max_clr_pairs {
        let (fg_defined, bg_defined) = registry.pair_content(color_pair_number);

        // already defined this color pair?
        if fg_defined == fg_color && bg_defined == bg_color {
            return color_pair_number;
        }
        // spare pair, defined as requested
        if fg_defined == 0 && bg_defined == 0 {
            registry.init_pair(color_pair_number, fg_color, bg_color);
            return color_pair_number;
        }
        color_pair_number += 1;
    }

    // none left - return default
    0
}

/// Port of `cob_activate_color_pair` (`screenio.c`). The C calls `color_set`/`attrset` + `bkgdset` with
/// `COLOR_PAIR(n)` and returns the curses status `ret`. The terminal side-effect is the boundary; the
/// in-scope computation is the `COLOR_PAIR(n)` attribute value it applies, returned here so a caller can
/// see exactly which pair attribute would be set. (C returns the curses `OK`/`ERR` status, which has no
/// pure analogue -- the meaningful pure output is the packed pair attr.)
pub fn cob_activate_color_pair(color_pair_number: i16) -> u64 {
    cob_color_pair(color_pair_number)
}

// ===========================================================================
// cob_screen_attr
// ===========================================================================

/// The non-boundary result of [`cob_screen_attr`]: the assembled curses style word, the resolved
/// color-pair number (when the terminal has color), and the (possibly attribute-field-adjusted) COBOL
/// `attr` flags that the C returns. The actual `attrset`/`attron`/`color_set` and the
/// `clear`/`clrtoeol`/`clrtobot`/`cob_beep` screen side-effects are the boundary and are *not* performed
/// here; their triggering flags remain visible in `attr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenAttrResult {
    /// The curses `chtype` style word (`A_BOLD | A_REVERSE | ...`) the C would `attron`.
    pub styles: u64,
    /// The resolved curses foreground color (`-1` if unset and color is off).
    pub fg_color: i16,
    /// The resolved curses background color (`-1` if unset and color is off).
    pub bg_color: i16,
    /// The color-pair number selected (`Some` only when `cob_has_color`), else `None`.
    pub color_pair: Option<i16>,
    /// The COBOL `attr` flags as returned by `cob_screen_attr` (after extended-color / color-field /
    /// control-field adjustment) -- this is the C function's actual return value.
    pub attr: i64,
}

/// Whether the `cob_screen_attr` call is for a `DISPLAY` (controls the BLANK LINE / ERASE EOL/EOS
/// branch) or an `ACCEPT`. Mirrors the relevant `enum screen_statement` discrimination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenStatement {
    /// `DISPLAY_STATEMENT`.
    Display,
    /// `ACCEPT_STATEMENT` (and any non-DISPLAY statement).
    Accept,
}

/// Port of `cob_screen_attr` (`screenio.c`) -- the main attribute assembler. Given the optional
/// `FOREGROUND-COLOR` / `BACKGROUND-COLOR` integer values (`cob_get_int(fgc/bgc)`, `None` => the C
/// `-1`), the incoming COBOL `attr` flag word, an optional numeric `COLOR` field value, and an optional
/// (upper-cased) `CONTROL` field text, it:
///   1. pre-sets curses fg/bg via [`cob_to_curses_color`];
///   2. folds extended-color highlight/blink into `attr`;
///   3. applies the numeric `COLOR` field ([`adjust_attr_from_color_field`]) and the `CONTROL` field
///      ([`adjust_attr_from_control_field`]) overrides;
///   4. builds the curses style word from the (adjusted) `attr` flags;
///   5. when `has_color`, resolves the color pair via [`cob_get_color_pair`] against `registry`.
///
/// Returns a [`ScreenAttrResult`]; the `attr` field is the C function's return value. The
/// `attrset`/`attron`, `cob_get_color_pair`+`cob_activate_color_pair` *application*, and the
/// BLANK/ERASE/BELL screen side-effects are the documented boundary (their flags stay visible in the
/// returned `attr`). The `A_OVERLINE`/`A_LEFTLINE`/`A_RIGHTLINE` bits are omitted, matching the admitted
/// host ncurses build where those macros are undefined.
#[allow(clippy::too_many_arguments)]
pub fn cob_screen_attr(
    cob_fg_color: Option<i32>,
    cob_bg_color: Option<i32>,
    mut attr: i64,
    control_text: Option<&[u8]>,
    color_field_value: Option<i32>,
    _stmt: ScreenStatement,
    has_color: bool,
    registry: &mut ColorPairRegistry,
) -> ScreenAttrResult {
    let cob_fg_color = cob_fg_color.unwrap_or(-1);
    let cob_bg_color = cob_bg_color.unwrap_or(-1);
    let mut fg_color: i16 = -1;
    let mut bg_color: i16 = -1;
    let mut styles: u64 = A_NORMAL;

    // pre-set color value (cob_to_curses_color leaves fg/bg untouched on invalid -> stays -1)
    if let Ok(v) = cob_to_curses_color(cob_fg_color) {
        fg_color = v;
    }
    if let Ok(v) = cob_to_curses_color(cob_bg_color) {
        bg_color = v;
    }

    // attribute via renamed colors
    adjust_attr_from_extended_color(&mut attr, cob_fg_color, 0);
    adjust_attr_from_extended_color(&mut attr, cob_bg_color, 1);

    // ACU / RM? extension that may override colors + some attributes
    if let Some(col) = color_field_value {
        adjust_attr_from_color_field(&mut attr, col, &mut fg_color, &mut bg_color);
    }

    // CONTROL - extension to override attributes and colors
    if let Some(ctrl) = control_text {
        adjust_attr_from_control_field(&mut attr, ctrl, &mut fg_color, &mut bg_color);
    }

    // curses attributes from (possibly adjusted) COBOL attributes
    if attr & COB_SCREEN_REVERSE != 0 {
        styles |= A_REVERSE;
    }
    if attr & COB_SCREEN_HIGHLIGHT != 0 {
        styles |= A_BOLD;
    }
    if attr & COB_SCREEN_LOWLIGHT != 0 {
        styles |= A_DIM;
    }
    if attr & COB_SCREEN_BLINK != 0 {
        styles |= A_BLINK;
    }
    if attr & COB_SCREEN_UNDERLINE != 0 {
        styles |= A_UNDERLINE;
    }
    // A_OVERLINE / A_LEFTLINE / A_RIGHTLINE are #if defined-guarded and undefined on the admitted
    // host's ncurses build -- so COB_SCREEN_OVERLINE/LEFTLINE/RIGHTLINE add no style bit (boundary).
    if let Some(bit) = A_OVERLINE {
        if attr & COB_SCREEN_OVERLINE != 0 {
            styles |= bit;
        }
    }
    if let Some(bit) = A_LEFTLINE {
        if attr & COB_SCREEN_LEFTLINE != 0 {
            styles |= bit;
        }
    }
    if let Some(bit) = A_RIGHTLINE {
        if attr & COB_SCREEN_RIGHTLINE != 0 {
            styles |= bit;
        }
    }

    // apply attributes -- attrset(A_NORMAL) + attron(styles) is the boundary.

    // apply colors
    let color_pair = if has_color {
        if fg_color == -1 {
            fg_color = registry.fore_color;
        }
        if bg_color == -1 {
            bg_color = registry.back_color;
        }
        let n = cob_get_color_pair(registry, fg_color, bg_color);
        // cob_activate_color_pair(n) -- the attrset/bkgdset is the boundary.
        Some(n)
    } else {
        None
    };

    // BLANK SCREEN / BLANK LINE / ERASE EOL/EOS / BELL screen side-effects are the boundary;
    // their triggering flags stay visible in `attr`.

    ScreenAttrResult { styles, fg_color, bg_color, color_pair, attr }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cob_to_curses_color_maps_each() {
        assert_eq!(cob_to_curses_color(0), Ok(COLOR_BLACK));
        assert_eq!(cob_to_curses_color(1), Ok(COLOR_BLUE));
        assert_eq!(cob_to_curses_color(2), Ok(COLOR_GREEN));
        assert_eq!(cob_to_curses_color(3), Ok(COLOR_CYAN));
        assert_eq!(cob_to_curses_color(4), Ok(COLOR_RED));
        assert_eq!(cob_to_curses_color(5), Ok(COLOR_MAGENTA));
        assert_eq!(cob_to_curses_color(6), Ok(COLOR_YELLOW));
        assert_eq!(cob_to_curses_color(7), Ok(COLOR_WHITE));
        // bit 3 (value 8) is masked off -> 8 maps like 0 (black)
        assert_eq!(cob_to_curses_color(8), Ok(COLOR_BLACK));
        assert_eq!(cob_to_curses_color(15), Ok(COLOR_WHITE));
        // out of range
        assert_eq!(cob_to_curses_color(-1), Err(()));
        assert_eq!(cob_to_curses_color(16), Err(()));
    }

    #[test]
    fn extended_color_sets_highlight_or_blink() {
        // foreground extended color -> HIGHLIGHT
        let mut a = 0i64;
        adjust_attr_from_extended_color(&mut a, 8 | COB_SCREEN_GREEN, 0);
        assert_eq!(a, COB_SCREEN_HIGHLIGHT);
        // background extended color -> BLINK
        let mut b = 0i64;
        adjust_attr_from_extended_color(&mut b, 8 | COB_SCREEN_RED, 1);
        assert_eq!(b, COB_SCREEN_BLINK);
        // no bit 3 -> nothing
        let mut c = 0i64;
        adjust_attr_from_extended_color(&mut c, COB_SCREEN_RED, 0);
        assert_eq!(c, 0);
        // invalid (< 0) -> nothing even if bit 3 conceptually set
        let mut d = 0i64;
        adjust_attr_from_extended_color(&mut d, -8, 0);
        assert_eq!(d, 0);
    }

    #[test]
    fn color_field_decomposes_weights() {
        // 16384 (BLINK) + 8192 (UNDERLINE) + 256 (bg WHITE) + 8 (fg WHITE)
        let mut attr = 0i64;
        let (mut fg, mut bg) = (-1i16, -1i16);
        adjust_attr_from_color_field(&mut attr, 16384 + 8192 + 256 + 8, &mut fg, &mut bg);
        assert_eq!(attr, COB_SCREEN_BLINK | COB_SCREEN_UNDERLINE);
        assert_eq!(bg, COLOR_WHITE);
        assert_eq!(fg, COLOR_WHITE);

        // the faithful CYAN bug: bg 128 -> COB_SCREEN_CYAN (3), not COLOR_CYAN (6)
        let mut attr2 = 0i64;
        let (mut fg2, mut bg2) = (-1i16, -1i16);
        adjust_attr_from_color_field(&mut attr2, 128 + 4, &mut fg2, &mut bg2);
        assert_eq!(bg2, COB_SCREEN_CYAN as i16); // == 3
        assert_eq!(fg2, COB_SCREEN_CYAN as i16); // fg 4 -> also 3
        assert_ne!(bg2, COLOR_CYAN); // distinct from real cyan (6)

        // REVERSE + LOWLIGHT + HIGHLIGHT bits
        let mut attr3 = 0i64;
        let (mut fg3, mut bg3) = (-1i16, -1i16);
        adjust_attr_from_color_field(&mut attr3, 1024 + 2048 + 4096, &mut fg3, &mut bg3);
        assert_eq!(attr3, COB_SCREEN_REVERSE | COB_SCREEN_LOWLIGHT | COB_SCREEN_HIGHLIGHT);
    }

    #[test]
    fn color_value_from_text_and_number() {
        assert_eq!(get_cob_color_from_color_value(b"BLACK", 5), COB_SCREEN_BLACK);
        assert_eq!(get_cob_color_from_color_value(b"WHITE", 5), COB_SCREEN_WHITE);
        assert_eq!(get_cob_color_from_color_value(b"GREEN", 5), COB_SCREEN_GREEN);
        assert_eq!(get_cob_color_from_color_value(b"BLUE", 4), COB_SCREEN_BLUE);
        assert_eq!(get_cob_color_from_color_value(b"CYAN", 4), COB_SCREEN_CYAN);
        assert_eq!(get_cob_color_from_color_value(b"RED", 3), COB_SCREEN_RED);
        assert_eq!(get_cob_color_from_color_value(b"MAGENTA", 7), COB_SCREEN_MAGENTA);
        assert_eq!(get_cob_color_from_color_value(b"YELLOW", 6), COB_SCREEN_YELLOW);
        // numeric prefix wins
        assert_eq!(get_cob_color_from_color_value(b"5", 1), 5);
        assert_eq!(get_cob_color_from_color_value(b"12", 2), 12);
        // unknown
        assert_eq!(get_cob_color_from_color_value(b"PURPLE", 6), -1);
        assert_eq!(get_cob_color_from_color_value(b"XX", 2), -1);
    }

    #[test]
    fn handle_control_field_color_paths() {
        // valid fg color name -> sets fg, returns 0
        let mut attr = 0i64;
        let mut fg = -1i16;
        assert_eq!(handle_control_field_color(&mut attr, b"RED", 3, &mut fg, 0), 0);
        assert_eq!(fg, COLOR_RED);
        assert_eq!(attr, 0);
        // extended color (8 + 4 = 12 = bright red) on bg -> BLINK
        let mut attr2 = 0i64;
        let mut bg = -1i16;
        assert_eq!(handle_control_field_color(&mut attr2, b"12", 2, &mut bg, 1), 0);
        assert_eq!(bg, COLOR_RED); // 12 & 7 == 4 == red
        assert_eq!(attr2, COB_SCREEN_BLINK);
        // invalid color -> -1, no change
        let mut attr3 = 0i64;
        let mut fg3 = -1i16;
        assert_eq!(handle_control_field_color(&mut attr3, b"NOPE", 4, &mut fg3, 0), -1);
        assert_eq!(fg3, -1);
    }

    #[test]
    fn screen_attr_cmp_and_bsearch() {
        // comparator sign
        assert_eq!(screen_attr_cmp(b"AUTO", &CONTROL_ATTRS[0]), 0);
        assert!(screen_attr_cmp(b"AAA", &CONTROL_ATTRS[0]) < 0);
        assert!(screen_attr_cmp(b"ZZZ", &CONTROL_ATTRS[0]) > 0);
        // bsearch finds known entries
        assert_eq!(bsearch_control_attr(b"REVERSE").unwrap().cobflag, COB_SCREEN_REVERSE);
        assert_eq!(bsearch_control_attr(b"HIGHLIGHT").unwrap().cobflag, COB_SCREEN_HIGHLIGHT);
        assert_eq!(bsearch_control_attr(b"FOREGROUND-COLOR").unwrap().cobflag, 0);
        assert_eq!(bsearch_control_attr(b"JUST").unwrap().cobflag, -1);
        assert!(bsearch_control_attr(b"NOSUCHTHING").is_none());
    }

    #[test]
    fn control_attrs_is_strcmp_sorted() {
        // the bsearch correctness invariant: the table must be strcmp-ascending.
        for w in CONTROL_ATTRS.windows(2) {
            assert!(c_strcmp(w[0].keyword, w[1].keyword) < 0,
                "out of order: {:?} !< {:?}",
                core::str::from_utf8(w[0].keyword), core::str::from_utf8(w[1].keyword));
        }
    }

    #[test]
    fn control_field_applies_named_attrs() {
        // "HIGHLIGHT REVERSE" -> both flags
        let mut attr = 0i64;
        let (mut fg, mut bg) = (-1i16, -1i16);
        adjust_attr_from_control_field(&mut attr, b"HIGHLIGHT REVERSE", &mut fg, &mut bg);
        assert_eq!(attr, COB_SCREEN_HIGHLIGHT | COB_SCREEN_REVERSE);

        // "NO REVERSE" clears it
        let mut attr2 = COB_SCREEN_REVERSE | COB_SCREEN_BLINK;
        let (mut fg2, mut bg2) = (-1i16, -1i16);
        adjust_attr_from_control_field(&mut attr2, b"NO REVERSE", &mut fg2, &mut bg2);
        assert_eq!(attr2, COB_SCREEN_BLINK);

        // two-token "ERASE EOL" / "BLANK SCREEN"
        let mut attr3 = 0i64;
        let (mut fg3, mut bg3) = (-1i16, -1i16);
        adjust_attr_from_control_field(&mut attr3, b"ERASE EOL, BLANK SCREEN", &mut fg3, &mut bg3);
        assert_eq!(attr3, COB_SCREEN_ERASE_EOL | COB_SCREEN_BLANK_SCREEN);
    }

    #[test]
    fn control_field_color_keywords() {
        // "FOREGROUND-COLOR IS RED" -> fg red
        let mut attr = 0i64;
        let (mut fg, mut bg) = (-1i16, -1i16);
        adjust_attr_from_control_field(&mut attr, b"FOREGROUND-COLOR IS RED", &mut fg, &mut bg);
        assert_eq!(fg, COLOR_RED);

        // "BCOLOR = BLUE" -> bg blue (the '=' becomes its own token, then skipped)
        let mut attr2 = 0i64;
        let (mut fg2, mut bg2) = (-1i16, -1i16);
        adjust_attr_from_control_field(&mut attr2, b"BCOLOR = BLUE", &mut fg2, &mut bg2);
        assert_eq!(bg2, COLOR_BLUE);

        // "FCOLOR 6" -> fg yellow (cob color 6)
        let mut attr3 = 0i64;
        let (mut fg3, mut bg3) = (-1i16, -1i16);
        adjust_attr_from_control_field(&mut attr3, b"FCOLOR 6", &mut fg3, &mut bg3);
        assert_eq!(fg3, COLOR_YELLOW);
    }

    #[test]
    fn color_pair_registry_selection() {
        // defaults fg=7 bg=0, 16 pairs
        let mut reg = ColorPairRegistry::new(7, 0, 16);
        // request the terminal default -> pair 0
        assert_eq!(cob_get_color_pair(&mut reg, 7, 0), 0);
        // all-black -> reserved pair 1
        assert_eq!(cob_get_color_pair(&mut reg, 0, 0), 1);
        // a fresh pair -> allocated at slot 2
        assert_eq!(cob_get_color_pair(&mut reg, COLOR_RED, COLOR_BLUE), 2);
        assert_eq!(reg.pairs[2], Some((COLOR_RED, COLOR_BLUE)));
        // requesting the same again finds the existing pair 2
        assert_eq!(cob_get_color_pair(&mut reg, COLOR_RED, COLOR_BLUE), 2);
        // a different one -> slot 3
        assert_eq!(cob_get_color_pair(&mut reg, COLOR_GREEN, COLOR_MAGENTA), 3);
    }

    #[test]
    fn color_pair_exhaustion_returns_default() {
        // only pairs 0,1 plus slots 2..3 (max_clr_pairs loop runs 2..< max).
        // With max 3, the loop only visits pair 2; fill it then request another -> 0.
        let mut reg = ColorPairRegistry::new(7, 0, 4);
        assert_eq!(cob_get_color_pair(&mut reg, COLOR_RED, COLOR_BLUE), 2); // slot 2
        assert_eq!(cob_get_color_pair(&mut reg, COLOR_GREEN, COLOR_RED), 3); // slot 3
        // pairs 2,3 used; loop bound is < 4, so only 2,3 visited; next request finds none free.
        assert_eq!(cob_get_color_pair(&mut reg, COLOR_WHITE, COLOR_RED), 0);
    }

    #[test]
    fn activate_color_pair_packs_attr() {
        // COLOR_PAIR(n) == (n << 8) & (255 << 8)
        assert_eq!(cob_activate_color_pair(0), 0);
        assert_eq!(cob_activate_color_pair(1), 1 << 8);
        assert_eq!(cob_activate_color_pair(5), 5 << 8);
        // pair numbers above 255 wrap within the 8-bit color field
        assert_eq!(cob_activate_color_pair(256), 0);
        assert_eq!(cob_activate_color_pair(257), 1 << 8);
    }

    #[test]
    fn screen_attr_assembles_styles_and_pair() {
        let mut reg = ColorPairRegistry::new(7, 0, 16);
        // fg=red(4), bg=blue(1) cobol colors; attr HIGHLIGHT|REVERSE
        let r = cob_screen_attr(
            Some(COB_SCREEN_RED),
            Some(COB_SCREEN_BLUE),
            COB_SCREEN_HIGHLIGHT | COB_SCREEN_REVERSE,
            None,
            None,
            ScreenStatement::Display,
            true,
            &mut reg,
        );
        assert_eq!(r.styles, A_BOLD | A_REVERSE);
        assert_eq!(r.fg_color, COLOR_RED);
        assert_eq!(r.bg_color, COLOR_BLUE);
        // a fresh (red, blue) pair allocated at slot 2
        assert_eq!(r.color_pair, Some(2));
        // attr returned unchanged (no extended colors / fields)
        assert_eq!(r.attr, COB_SCREEN_HIGHLIGHT | COB_SCREEN_REVERSE);
    }

    #[test]
    fn screen_attr_extended_color_folds_in() {
        let mut reg = ColorPairRegistry::new(7, 0, 16);
        // fg = 8 | green = bright green -> folds HIGHLIGHT into attr
        let r = cob_screen_attr(
            Some(8 | COB_SCREEN_GREEN),
            None,
            0,
            None,
            None,
            ScreenStatement::Accept,
            false,
            &mut reg,
        );
        assert!(r.attr & COB_SCREEN_HIGHLIGHT != 0);
        assert_eq!(r.styles, A_BOLD);
        assert_eq!(r.fg_color, COLOR_GREEN);
        // no color (has_color = false) -> no pair
        assert_eq!(r.color_pair, None);
    }

    #[test]
    fn screen_attr_control_overrides() {
        let mut reg = ColorPairRegistry::new(7, 0, 16);
        // no fg/bg numbers, CONTROL sets UNDERLINE + fg yellow
        let r = cob_screen_attr(
            None,
            None,
            0,
            Some(b"UNDERLINE FCOLOR YELLOW"),
            None,
            ScreenStatement::Display,
            true,
            &mut reg,
        );
        assert!(r.attr & COB_SCREEN_UNDERLINE != 0);
        assert_eq!(r.styles, A_UNDERLINE);
        assert_eq!(r.fg_color, COLOR_YELLOW);
        // bg unset -> falls back to terminal default back_color (0)
        assert_eq!(r.bg_color, 0);
    }

    #[test]
    fn screen_attr_no_color_unset_stays_negative() {
        let mut reg = ColorPairRegistry::new(7, 0, 16);
        let r = cob_screen_attr(
            None, None, 0, None, None, ScreenStatement::Display, false, &mut reg,
        );
        // has_color false -> fg/bg left at -1, no pair
        assert_eq!(r.fg_color, -1);
        assert_eq!(r.bg_color, -1);
        assert_eq!(r.color_pair, None);
        assert_eq!(r.styles, A_NORMAL);
    }
}
