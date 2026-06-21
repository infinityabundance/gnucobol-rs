//! Port of `screenio.c` -- the `SCREEN SECTION` `DISPLAY`/`ACCEPT` engine. GnuCOBOL drives **ncurses**
//! (`initscr`/`move`/`addstr`/`attrset`/`refresh`/`endwin`); the literal bytes written to the terminal are
//! ncurses's terminfo-optimized output, not anything `screenio.c` spells out itself. A *literal-byte*
//! faithful port therefore reproduces, in pure Rust, the exact escape-sequence stream ncurses emits for the
//! admitted terminal -- captured deterministically from the oracle under a pty (`screenio_sweep`).
//!
//! ## Claim boundary (read this)
//!
//! This module reproduces the byte stream for **`TERM=xterm` under the admitted host's ncurses 6.6**
//! (`libncursesw.so.6`, `terminfo` xterm). The init/teardown framing (`smcup` … `rmcup`) and the
//! cursor-movement optimization are ncurses/terminfo artifacts; a *different* terminal or ncurses build
//! emits *different* bytes. That terminal-dependence is the explicit non-claim. What is proven is that, for
//! the admitted terminal, the native emitter is **byte-identical** to GnuCOBOL's screen output -- so a
//! migration that records/replays terminal traces sees the same bytes.
//!
//! This is the foundation court (`GNURUST.SCREENIO.INIT.1`): the screen init/teardown framing plus a
//! positioned literal/`FROM` `DISPLAY`. Attributes, multi-field layout, numeric editing, and `ACCEPT`
//! input are sealed in follow-on courts.
#![forbid(unsafe_code)]

/// The terminal the reproduced bytes are admitted against (the non-claim is "anything else").
pub const ADMITTED_TERM: &str = "xterm";
/// The ncurses build whose terminfo-optimized output is reproduced.
pub const ADMITTED_NCURSES: &str = "ncurses 6.6 (libncursesw.so.6)";

/// ncurses/xterm screen **init prologue** -- the exact bytes GnuCOBOL's `cob_screen_init` produces via
/// ncurses `initscr` + `start_color` and the libcob default color setup, on first screen I/O. Decomposed:
/// `\e[?1049h` smcup (alternate screen) · `\e[22;0;0t` push window title · `\e[1;24r` DECSTBM scroll
/// region · `\e(B` G0=ASCII · `\e[m` SGR reset · `\e[4l` insert-mode off · `\e[?7h` autowrap on ·
/// `\e[?1h\e=` application cursor-keys + keypad · `\e[39;49m` default fg/bg · `\e[?12;25h` cursor blink +
/// visible · `\e[?1006;1000h` SGR-pixel + normal mouse tracking · `\e[39;49m` default fg/bg · `\e[37m`
/// fg white · `\e[40m` bg black (the libcob default `COLOR_PAIR`) · `\e[H` home · `\e[2J` clear screen.
pub const INIT_PROLOGUE: &[u8] = b"\x1b[?1049h\x1b[22;0;0t\x1b[1;24r\x1b(B\x1b[m\x1b[4l\x1b[?7h\x1b[?1h\x1b=\x1b[39;49m\x1b[?12;25h\x1b[?1006;1000h\x1b[39;49m\x1b[37m\x1b[40m\x1b[H\x1b[2J";

/// The libcob "press a key to exit" pause text (emitted by the implicit screen pause at `STOP RUN` while a
/// screen is active), exactly as the runtime spells it (note the trailing space).
pub const PAUSE_PROMPT: &[u8] = b"end of program, please press a key to exit ";

/// ncurses/xterm screen **teardown epilogue** -- the exact bytes `endwin` produces. Decomposed:
/// `\e[?1006;1000l` mouse off · `\e[39;49m\r` default fg/bg + CR · `\e[24d` VPA last line · `\e[K` clear to
/// EOL · `\e[24;1H` CUP 24,1 · `\e[?12l` cursor blink off · `\e[?25h` cursor visible · `\e[?1049l` rmcup
/// (leave alternate screen) · `\e[23;0;0t\r` pop window title + CR · `\e[?1l` normal cursor-keys · `\e>`
/// keypad numeric mode.
pub const TEARDOWN_EPILOGUE: &[u8] = b"\x1b[?1006;1000l\x1b[39;49m\r\x1b[24d\x1b[K\x1b[24;1H\x1b[?12l\x1b[?25h\x1b[?1049l\x1b[23;0;0t\r\x1b[?1l\x1b>";

/// The default terminal height the scroll region implies (`\e[1;24r`): rows 1..=24. Used to place the
/// pause prompt and the teardown's last-line clear.
pub const SCREEN_ROWS: i32 = 24;

/// A monochrome SCREEN SECTION display attribute (the ones whose terminal bytes do not trigger a
/// whole-screen color repaint). Each maps to the SGR parameter ncurses emits via `set_attributes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenAttr {
    /// `HIGHLIGHT` (bold) -- SGR `1`.
    Highlight,
    /// `LOWLIGHT` (dim) -- SGR `2`.
    Lowlight,
    /// `UNDERLINE` -- SGR `4`.
    Underline,
    /// `BLINK` -- SGR `5`.
    Blink,
    /// `REVERSE-VIDEO` -- SGR `7`.
    Reverse,
}

impl ScreenAttr {
    /// The SGR numeric parameter for this attribute (`\e[0;<n>m`).
    fn sgr_code(self) -> u8 {
        match self {
            ScreenAttr::Highlight => 1,
            ScreenAttr::Lowlight => 2,
            ScreenAttr::Underline => 4,
            ScreenAttr::Blink => 5,
            ScreenAttr::Reverse => 7,
        }
    }
}

/// A single positioned screen item of a `DISPLAY`: 1-based COBOL `LINE`/`COLUMN`, the literal/`FROM`
/// bytes to write (already space-padded to the item's `PIC` size, as the compiler presents it), and an
/// optional monochrome display attribute wrapping the field.
#[derive(Debug, Clone)]
pub struct ScreenItem {
    /// COBOL `LINE` (1-based).
    pub line: i32,
    /// COBOL `COLUMN` (1-based).
    pub column: i32,
    /// The item's display bytes.
    pub data: Vec<u8>,
    /// An optional monochrome display attribute (`HIGHLIGHT`/`REVERSE-VIDEO`/...).
    pub attr: Option<ScreenAttr>,
}

impl ScreenItem {
    /// A plain (no-attribute) positioned field.
    pub fn plain(line: i32, column: i32, data: Vec<u8>) -> Self {
        ScreenItem { line, column, data, attr: None }
    }
    /// A positioned field carrying a monochrome display attribute.
    pub fn with_attr(line: i32, column: i32, data: Vec<u8>, attr: ScreenAttr) -> Self {
        ScreenItem { line, column, data, attr: Some(attr) }
    }
}

/// The SGR "attribute on" sequence ncurses emits before an attributed field: the ASCII-charset
/// designation, the `set_attributes` SGR `\e[0;<n>m`, then the default-color restore. Observed byte-exact
/// against the oracle (`screenio_attr_sweep`).
fn sgr_on(attr: ScreenAttr) -> Vec<u8> {
    let mut v = b"\x1b(B\x1b[0;".to_vec();
    v.extend_from_slice(attr.sgr_code().to_string().as_bytes());
    v.extend_from_slice(b"m\x1b[39;49m\x1b[37m\x1b[40m");
    v
}

/// The SGR "attribute off" sequence ncurses emits after an attributed field: charset designation, `sgr0`
/// reset `\e[m`, then the default-color restore. Constant for every monochrome attribute.
fn sgr_off() -> Vec<u8> {
    b"\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m".to_vec()
}

/// Append `n` spaces (the cheapest right-move ncurses chooses for a short column advance: each overwrites a
/// known blank).
fn spaces(out: &mut Vec<u8>, n: i32) {
    for _ in 0..n {
        out.push(b' ');
    }
}
/// Append a CSI numeric command `\e[<n><final>` (VPA `d`, CHA/HPA `G`).
fn csi1(out: &mut Vec<u8>, n: i32, fin: u8) {
    out.extend_from_slice(b"\x1b[");
    out.extend_from_slice(n.to_string().as_bytes());
    out.push(fin);
}
/// Append a CUP `\e[<row>;<col>H` (direct cursor address, 1-based -- COBOL `LINE`/`COLUMN` map straight in).
fn cup(out: &mut Vec<u8>, line: i32, col: i32) {
    out.extend_from_slice(b"\x1b[");
    out.extend_from_slice(line.to_string().as_bytes());
    out.push(b';');
    out.extend_from_slice(col.to_string().as_bytes());
    out.push(b'H');
}

/// Build the candidate byte sequences for a horizontal move within row `ty`, from column `sc` to `tx`,
/// matching ncurses `relative_move`'s column handling. Forward: space-fill always; column-address HPA
/// `\e[<tx>G` only when it is the cheaper choice (advance of 5..=7 columns -- ncurses does not reach for HPA
/// on longer same-row runs, preferring direct cursor-address there). Backward: backspaces. (HPA candidates
/// are listed first so they win a byte-count tie, as the oracle does.)
fn horiz_candidates(sc: i32, tx: i32) -> Vec<Vec<u8>> {
    if tx == sc {
        return vec![Vec::new()];
    }
    let mut out = Vec::new();
    if tx > sc {
        let delta = tx - sc;
        if (5..=7).contains(&delta) {
            let mut h = Vec::new();
            csi1(&mut h, tx, b'G');
            out.push(h);
        }
        let mut s = Vec::new();
        spaces(&mut s, delta);
        out.push(s);
    } else {
        out.push(vec![0x08; (sc - tx) as usize]); // backspaces
    }
    out
}

/// Reproduce ncurses's `mvcur` choice for a move from `(fy,fx)` to `(ty,tx)` on the admitted xterm/ncurses
/// 6.6 terminal: enumerate the move strategies ncurses considers -- (1) keep the column and move
/// vertically (VPA `\e[<r>d`, or `cuu1 \e[A` for up-one) then move horizontally from `fx`; (2) carriage
/// return to column 1, move vertically, then horizontally from 1; (3) `home \e[H` for the exact `(1,1)`
/// target; (4) direct cursor-address CUP `\e[<r>;<c>H` -- and emit the shortest, with local/CR strategies
/// winning a byte-count tie over CUP (the empirically pinned tie-break, `screenio_grid_sweep` /
/// `screenio_multi_sweep`). This single function reproduces the from-home first-field move, every
/// inter-field move of a multi-field `DISPLAY`, and the post-field move to the pause prompt.
fn mvcur(fy: i32, fx: i32, ty: i32, tx: i32) -> Vec<u8> {
    let mut cands: Vec<Vec<u8>> = Vec::new();

    // Strategy 1: vertical (keeping the column) + horizontal from fx.
    let mut verticals: Vec<Vec<u8>> = Vec::new();
    if ty == fy {
        verticals.push(Vec::new());
    } else {
        if ty == fy - 1 {
            verticals.push(b"\x1b[A".to_vec()); // cuu1 (up one)
        }
        let mut v = Vec::new();
        csi1(&mut v, ty, b'd'); // VPA
        verticals.push(v);
    }
    for v in &verticals {
        for h in horiz_candidates(fx, tx) {
            let mut c = v.clone();
            c.extend_from_slice(&h);
            cands.push(c);
        }
    }

    // Strategy 2: CR to column 1, then vertical, then horizontal from column 1.
    {
        let mut c = vec![b'\r'];
        if ty != fy {
            csi1(&mut c, ty, b'd');
        }
        // After CR the column is 1; only a forward (or empty) horizontal applies.
        c.extend_from_slice(&horiz_candidates(1, tx)[0]);
        cands.push(c);
    }

    // Strategy 3: home for the exact (1,1) target.
    if ty == 1 && tx == 1 {
        cands.push(b"\x1b[H".to_vec());
    }

    // Strategy 4: direct cursor-address (CUP) -- listed last so it loses ties to the local strategies.
    {
        let mut c = Vec::new();
        cup(&mut c, ty, tx);
        cands.push(c);
    }

    // Shortest wins; the first-generated wins a tie (local/CR/home before CUP).
    cands.into_iter().min_by_key(|c| c.len()).unwrap_or_default()
}

/// Reproduce the full terminal byte stream of a `SCREEN SECTION` `DISPLAY` of `items` followed by `STOP
/// RUN` (which, with a screen active, prints the pause prompt and tears the screen down): the init
/// prologue, each item positioned + written, the pause prompt on the line below the lowest item, then the
/// teardown epilogue -- byte-identical to GnuCOBOL on the admitted terminal.
pub fn display_and_stop(items: &[ScreenItem]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(INIT_PROLOGUE);
    // The cursor starts at home `(1,1)` after the clear; each field is reached by the general `mvcur`, then
    // written (advancing the cursor by its length), in `DISPLAY` order.
    let (mut cy, mut cx) = (1, 1);
    for it in items {
        out.extend_from_slice(&mvcur(cy, cx, it.line, it.column));
        // A monochrome attribute wraps the field in an SGR on/off pair; the cursor still advances only by
        // the data length (the SGR sequences move nothing).
        if let Some(a) = it.attr {
            out.extend_from_slice(&sgr_on(a));
            out.extend_from_slice(&it.data);
            out.extend_from_slice(&sgr_off());
        } else {
            out.extend_from_slice(&it.data);
        }
        cy = it.line;
        cx = it.column + it.data.len() as i32;
    }
    // The pause prompt sits one row below where the cursor ended (the last field's row), at column 1.
    let prompt_line = cy + 1;
    out.extend_from_slice(&mvcur(cy, cx, prompt_line, 1));
    out.extend_from_slice(PAUSE_PROMPT);
    out.extend_from_slice(TEARDOWN_EPILOGUE);
    out
}

// ===========================================================================================
// GNURUST.SCREENIO.COLOR.1 -- the COLORED SCREEN SECTION field (full ncurses repaint).
// ===========================================================================================
//
// A monochrome attribute (`HIGHLIGHT` etc.) only wraps its field in an SGR on/off pair -- the
// rest of the screen is untouched, so `display_and_stop` handles it inline. A *color* clause
// (`FOREGROUND-COLOR`/`BACKGROUND-COLOR`) is different: the first use of a non-default color pair
// makes ncurses repaint the whole touched region. The observable byte stream is therefore not a
// simple positioned write but ncurses's `wclear` + top-down `TransformLine` sequence. This court
// reproduces that stream byte-for-byte against the admitted xterm/ncurses 6.6 oracle.
//
// ## The two genuinely new facts this court pins
//
// 1. **The COBOL->curses color permutation.** COBOL/IBM color numbers order the 3 color bits as
//    (bit0=blue, bit1=green, bit2=red); curses/ANSI order them (bit0=red, bit1=green, bit2=blue).
//    So a COBOL color maps to its curses color by *reversing the low three bits*
//    (`curses_color`): COBOL 1 (blue) -> 4, COBOL 4 (red) -> 1, COBOL 6 (brown) -> 3, etc. The
//    foreground SGR is then `30 + curses_color`, the background `40 + curses_color`. Verified
//    byte-exact for every one of the 8x8 color combinations.
//
// 2. **The default color pair is free.** COBOL `FOREGROUND-COLOR 7 BACKGROUND-COLOR 0`
//    (white-on-black) is ncurses color pair 0, which is always allocated and needs no SGR -- so a
//    field in the default colors triggers *no* repaint and falls back to the plain
//    `display_and_stop` byte stream. Only a non-default pair forces the full-screen repaint.
//
// ## The repaint shape (R >= 2, the sealed envelope)
//
// For a single colored field at COBOL `LINE R`, `COLUMN C` the body between the screen-clear and
// the pause prompt is, exactly:
//
// * `\e[<R+1>d` -- VPA parking the cursor on the row below the field (where the prompt will land),
// * the default-restore SGR `\e(B\e[m\e[39;49m\e[37m\e[40m` ([`RESET_DEFAULTS`]),
// * `\e[J` -- erase from there to the bottom of the screen (clears every row at/below the field),
// * the **top-down repaint of the rows above the field**: `\e[H\e[K` clears row 1, then each row
//   `2..=R-1` is `\e[<r>d\e[K`,
// * the **field-row positioning** (the leading-blank handling ncurses's `TransformLine` chooses):
//   when the field starts within 5 columns (`C-1 <= 4`) it space-fills from column 1
//   (`\e[<R>d` then `C-1` spaces); otherwise it cursor-addresses just before the field and clears
//   the leading blanks in one shot (`\e[<R>;<C-1>H\e[1K` then a single space onto column `C`),
// * the field itself: the two color SGRs, the data bytes, then `RESET_DEFAULTS` and `\e[K` to
//   clear the rest of the field row,
// * the prompt move -- the *same* [`mvcur`] cost model used everywhere else, from the cursor's
//   end position `(R, C+len)` to `(R+1, 1)`.
//
// The `R == 1` single-row-screen case uses a different `\e[A`-based positioning and is an explicit
// **non-claim** of this court (a colored field on the very first line is a rare edge; it is left to
// a follow-on). The court's sealed envelope is `R >= 2`.

/// The default ncurses foreground color (white) -- COBOL `FOREGROUND-COLOR 7`. Together with
/// [`DEFAULT_BG`] this is color pair 0, which needs no SGR and triggers no repaint.
pub const DEFAULT_FG: u8 = 7;
/// The default ncurses background color (black) -- COBOL `BACKGROUND-COLOR 0`.
pub const DEFAULT_BG: u8 = 0;

/// The SGR sequence that restores ncurses's default attributes + colors (charset designation,
/// `sgr0`, default fg/bg, then the libcob default white-on-black pair). Emitted by every attribute
/// or color reset; identical to the bytes [`sgr_off`] produces.
pub const RESET_DEFAULTS: &[u8] = b"\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m";

/// Map a COBOL color number (0..=7) to its curses/ANSI color number by reversing the low three
/// bits. COBOL orders the color bits (blue, green, red) low-to-high; curses orders them
/// (red, green, blue). So COBOL `1` (blue) -> `4`, `2` (green) -> `2`, `4` (red) -> `1`,
/// `6` (brown/yellow) -> `3`, and the symmetric values (`0`, `7`, plus `2`/`5`) are fixed points.
/// Inputs above 7 are masked to three bits (defensive; the compiler only emits 0..=7).
pub fn curses_color(cobol_color: u8) -> u8 {
    let n = cobol_color & 0b111;
    ((n & 0b001) << 2) | (n & 0b010) | ((n & 0b100) >> 2)
}

/// Append the foreground/background color SGRs for a COBOL `(fg, bg)` pair: `\e[<30+fg'>m` then
/// `\e[<40+bg'>m`, where `fg'`/`bg'` are the [`curses_color`]-mapped numbers.
fn color_sgr(out: &mut Vec<u8>, fg: u8, bg: u8) {
    csi1_str(out, 30 + curses_color(fg) as i32, b'm');
    csi1_str(out, 40 + curses_color(bg) as i32, b'm');
}

/// Append a CSI sequence `\e[<n><final>` (same shape as [`csi1`]; named to read clearly at the
/// SGR call sites where `n` is a color code rather than a row/column).
fn csi1_str(out: &mut Vec<u8>, n: i32, fin: u8) {
    csi1(out, n, fin);
}

/// Reproduce the full terminal byte stream of a `SCREEN SECTION` `DISPLAY` of a single field
/// carrying explicit `FOREGROUND-COLOR fg` / `BACKGROUND-COLOR bg` (COBOL color numbers 0..=7),
/// followed by `STOP RUN` -- byte-identical to GnuCOBOL on the admitted xterm/ncurses 6.6 terminal
/// (`GNURUST.SCREENIO.COLOR.1`).
///
/// When `(fg, bg)` is the default pair ([`DEFAULT_FG`], [`DEFAULT_BG`]) ncurses needs no color and
/// no repaint, so this delegates to the plain `display_and_stop` stream. Otherwise it emits the
/// full `wclear` repaint described in the module section above. The sealed envelope is `line >= 2`;
/// `line == 1` (a colored field on the very first row) is the documented non-claim.
pub fn color_display_and_stop(line: i32, column: i32, data: &[u8], fg: u8, bg: u8) -> Vec<u8> {
    // Default color pair -> no repaint; this is observably the plain positioned-write stream.
    if fg == DEFAULT_FG && bg == DEFAULT_BG {
        return display_and_stop(&[ScreenItem::plain(line, column, data.to_vec())]);
    }

    let mut out = Vec::new();
    // The colour prologue is the standard init prologue with the field's colour-pair SGR injected
    // just before the home+clear `\e[H\e[2J` -- ncurses's `start_color`/`init_pair` pre-selects the
    // pair that the first painted field will use, so the SGR appears already at screen setup. Split
    // the constant before its trailing 7-byte `\e[H\e[2J` and slot the colour SGR in.
    let home_clear_len = b"\x1b[H\x1b[2J".len();
    let split = INIT_PROLOGUE.len() - home_clear_len;
    out.extend_from_slice(&INIT_PROLOGUE[..split]);
    color_sgr(&mut out, fg, bg);
    out.extend_from_slice(&INIT_PROLOGUE[split..]);

    // (1) Park on the row below the field, restore defaults, erase from there to the bottom.
    csi1(&mut out, line + 1, b'd'); // VPA R+1
    out.extend_from_slice(RESET_DEFAULTS);
    out.extend_from_slice(b"\x1b[J"); // ED: erase to end of screen

    // (2) Repaint the rows above the field top-down: row 1 via home+clear, rows 2..=R-1 via
    //     VPA+clear. (Each row is blank, so a clear-to-EOL from column 1 blanks the whole line.)
    out.extend_from_slice(b"\x1b[H\x1b[K");
    for r in 2..line {
        csi1(&mut out, r, b'd');
        out.extend_from_slice(b"\x1b[K");
    }

    // (3) Position onto the field row. Few leading blanks (<=4) -> space-fill from column 1;
    //     more -> cursor-address just before the field and clear the leading blanks with \e[1K.
    if column - 1 <= 4 {
        csi1(&mut out, line, b'd'); // VPA R (column stays 1)
        spaces(&mut out, column - 1);
    } else {
        cup(&mut out, line, column - 1);
        out.extend_from_slice(b"\x1b[1K");
        out.push(b' ');
    }

    // (4) The colored field: fg/bg SGRs, the data, then restore defaults + clear the rest of row.
    color_sgr(&mut out, fg, bg);
    out.extend_from_slice(data);
    out.extend_from_slice(RESET_DEFAULTS);
    out.extend_from_slice(b"\x1b[K");

    // (5) Prompt move -- the shared mvcur cost model, from end-of-field to (R+1, 1).
    let end_col = column + data.len() as i32;
    out.extend_from_slice(&mvcur(line, end_col, line + 1, 1));
    out.extend_from_slice(PAUSE_PROMPT);
    out.extend_from_slice(TEARDOWN_EPILOGUE);
    out
}

// ===========================================================================================
// GNURUST.SCREENIO.NUMEDIT.1 -- a NUMERIC-EDITED field DISPLAY (zero-suppression / sign / CR-DB).
// ===========================================================================================
//
// A `SCREEN SECTION` field with an *edited* PIC (`ZZ,ZZ9.99`, `$$,$$9.99`, `-9(5).99`, `9(4).99CR`,
// `ZZ,ZZ9.99-` ...) `FROM` a numeric source displays the **edited representation** of the value --
// the bytes the move/edit engine produces (`edited::encode_edited`, sealed separately). That edited
// string is right-aligned in the field, so it characteristically carries **leading blanks** (from
// zero-suppression or a non-printing sign) and sometimes a short run of **trailing blanks** (a `CR`/
// `DB` on a positive value shows as two spaces; a trailing fixed sign on a positive value as one).
//
// This court reproduces, byte-for-byte, how ncurses paints that edited string on the cleared screen.
// The numeric editing itself is NOT re-proved here (it is the move.c court); this court proves the
// **screen positioning of an edited field**, which is the genuinely new screenio behaviour:
//
// * **Leading blanks are skipped.** The cleared screen is already all-blank, so ncurses does not
//   write the leading spaces -- it moves the cursor straight to the first non-blank column
//   (`col + first_nonblank`) via the shared [`mvcur`] cost model and writes from there.
// * **The written run goes to the end of the field** (`edited[first_nonblank..]`), which includes any
//   short trailing-blank run -- ncurses space-fills those (cheaper than a cursor move for the 1-2
//   trailing blanks a `CR`/`DB`/sign produces). The cursor therefore ends at the field's logical end
//   `col + width`, from which the pause-prompt move is taken.
// * **An all-blank field** (e.g. `ZZZZ.ZZ` of zero) writes *nothing*: ncurses simply positions the
//   cursor at the field end `col + width` and moves on.
//
// Sealed envelope: a single edited field on `LINE >= 1`, whose trailing-blank run is short enough that
// ncurses space-fills rather than cursor-skips it (true for all standard numeric editing -- `CR`/`DB`
// give 2, a trailing sign 1). A pathological PIC with a long interior/trailing blank run (5+), or
// multiple edited fields, is the declared non-claim.

/// Reproduce the full terminal byte stream of a `SCREEN SECTION` `DISPLAY` of a single NUMERIC-EDITED
/// field whose already-edited bytes are `edited` (produced by [`crate::edited::encode_edited`] for the
/// field's PIC + value), followed by `STOP RUN` -- byte-identical to GnuCOBOL on the admitted
/// xterm/ncurses 6.6 terminal (`GNURUST.SCREENIO.NUMEDIT.1`).
///
/// `edited` is the FULL field image including its leading (and any trailing) blanks; this function
/// supplies only the screen positioning: skip the leading blanks, write the first-non-blank-to-end
/// run, and leave the cursor at the field end (`column + edited.len()`). An all-blank `edited` writes
/// nothing and parks the cursor at the field end.
pub fn display_edited_and_stop(line: i32, column: i32, edited: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(INIT_PROLOGUE);

    let width = edited.len() as i32;
    let first_nonblank = edited.iter().position(|&b| b != b' ');
    let (mut cy, mut cx) = (1, 1);
    match first_nonblank {
        None => {
            // All-blank field: nothing to paint; the cursor parks at the field's logical end.
            let end = column + width;
            out.extend_from_slice(&mvcur(cy, cx, line, end));
            cy = line;
            cx = end;
        }
        Some(s) => {
            let s = s as i32;
            // Position at the first non-blank column and write the remainder (trailing blanks, if
            // any, are written as spaces -- ncurses space-fills the short tail).
            out.extend_from_slice(&mvcur(cy, cx, line, column + s));
            out.extend_from_slice(&edited[s as usize..]);
            cy = line;
            cx = column + width; // the cursor ends at the field's logical end.
        }
    }

    let prompt_line = line + 1;
    out.extend_from_slice(&mvcur(cy, cx, prompt_line, 1));
    out.extend_from_slice(PAUSE_PROMPT);
    out.extend_from_slice(TEARDOWN_EPILOGUE);
    out
}

// ===========================================================================================
// GNURUST.SCREENIO.ACCEPT.1 -- a SCREEN SECTION ACCEPT of an alphanumeric input field.
// ===========================================================================================
//
// `ACCEPT` of a `SCREEN SECTION` field reads keystrokes into the field. Its observable terminal
// byte stream (captured under a pty, input fed then EOF) is, for a single alphanumeric `USING`/`TO`
// field of width W at `LINE`/`COLUMN`:
//
// 1. **Position** at the field start via the shared [`mvcur`] cost model (from home).
// 2. **The input prompt:** the field is shown as W underscores `_` (ncurses paints the field's
//    initial blank-but-promptable content; the default prompt char is `_`).
// 3. **Reposition to the field start** so typing overwrites the prompt: a same-row *backward* move
//    whose encoding is the cheapest of column-address HPA `\e[<col>G`, backspaces, or
//    carriage-return-plus-spaces -- with HPA winning a byte-count tie over backspaces, and
//    backspaces over CR ([`accept_reposition`]).
// 4. **Echo** of the typed characters (up to W), verbatim.
// 5. **The field-full backspace:** if the typed text fills the whole field, after echoing the last
//    character the cursor sits one past the field, so ncurses emits a single backspace to keep the
//    cursor on the last field cell.
// 6. **Teardown** -- on EOF the implicit `STOP RUN` pause is skipped (no "press a key" prompt), so
//    the stream goes straight to the standard [`TEARDOWN_EPILOGUE`].
//
// ## Sealed envelope (verified byte-identical against BOTH GnuCOBOL 3.2 and 3.1.2)
//
// A single alphanumeric field of **width 1..=6**, plain printable input terminated by Enter then EOF
// -- INCLUDING **overflow** input that exceeds the field width (`GNURUST.SCREENIO.ACCEPT.2`, the
// BEL/overwrite tail in step (6) below). Width >= 7 is the declared non-claim: ncurses then paints
// the prompt with the `rep` capability (`_\e[<W-1>b`) and the post-`rep` reposition becomes
// terminfo-internal and position-dependent. Field editing keys (arrows, backspace-during-input,
// function keys), numeric/`USING` validation, and any terminal but the admitted `TERM=xterm` /
// ncurses 6.6 are likewise out of scope.

/// The default ncurses prompt character an `ACCEPT` field shows for each empty position.
pub const ACCEPT_PROMPT_CHAR: u8 = b'_';

/// Reproduce ncurses's same-row **backward** cursor move from `from_col` to `to_col` (the
/// reposition from the field end back to the field start after painting the prompt). Candidates: the
/// column-address HPA `\e[<to_col>G`, a run of backspaces, or carriage-return + `to_col-1` spaces;
/// the shortest wins, and on a byte-count tie HPA beats backspaces beats CR (the empirically pinned
/// order, `screenio_accept_sweep`). Distinct from the general [`mvcur`], whose backward path only
/// considers backspaces -- the ACCEPT reposition additionally reaches for HPA.
fn accept_reposition(from_col: i32, to_col: i32) -> Vec<u8> {
    // (priority, bytes) -- lower priority wins a length tie.
    let mut cands: Vec<(u8, Vec<u8>)> = Vec::new();
    // HPA `\e[<to_col>G`.
    let mut hpa = b"\x1b[".to_vec();
    hpa.extend_from_slice(to_col.to_string().as_bytes());
    hpa.push(b'G');
    cands.push((0, hpa));
    // Backspaces.
    cands.push((1, vec![0x08; (from_col - to_col).max(0) as usize]));
    // CR + (to_col-1) spaces.
    let mut cr = vec![b'\r'];
    cr.extend(std::iter::repeat(b' ').take((to_col - 1).max(0) as usize));
    cands.push((2, cr));
    cands.into_iter().min_by_key(|(prio, b)| (b.len(), *prio)).map(|(_, b)| b).unwrap_or_default()
}

/// Reproduce the full terminal byte stream of a `SCREEN SECTION` `ACCEPT` of a single alphanumeric
/// field of width `width` at `LINE line` / `COLUMN col`, given the printable `typed` input (the
/// characters entered before Enter), followed by `STOP RUN` at EOF -- byte-identical to GnuCOBOL on
/// the admitted xterm/ncurses 6.6 terminal (`GNURUST.SCREENIO.ACCEPT.1`).
///
/// Sealed envelope: `1 <= width <= 6` and `typed.len() <= width` (see the module section above);
/// width >= 7 (the `rep`-painted prompt) and overflow input are documented non-claims.
pub fn accept_field_and_stop(line: i32, col: i32, width: i32, typed: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(INIT_PROLOGUE);

    // (1) Position at the field start, then (2) paint the W-underscore input prompt.
    out.extend_from_slice(&mvcur(1, 1, line, col));
    for _ in 0..width {
        out.push(ACCEPT_PROMPT_CHAR);
    }
    // (3) Reposition from the field end (col + width) back to the field start.
    out.extend_from_slice(&accept_reposition(col + width, col));
    // (4) Echo the typed characters, capped at the field width.
    let echoed = typed.len().min(width as usize);
    out.extend_from_slice(&typed[..echoed]);
    // (5) Field-full backspace: a full field leaves the cursor one past the field; ncurses backs up.
    if typed.len() as i32 >= width {
        out.push(0x08);
    }
    // (6) OVERFLOW (GNURUST.SCREENIO.ACCEPT.2): typing past the field width. With the cursor parked on
    // the last cell, each excess key rings the bell `\a`; if the key differs from the character
    // currently shown in that cell it overwrites it (write the char, then a backspace to stay on the
    // cell), otherwise the bell is all that is emitted (overwriting a cell with its own value is a
    // no-op). The "currently shown" character starts as the last filled cell and updates on each
    // overwrite.
    if typed.len() as i32 > width {
        let mut last_shown = typed[width as usize - 1];
        for &ch in &typed[width as usize..] {
            out.push(0x07); // BEL
            if ch != last_shown {
                out.push(ch);
                out.push(0x08); // backspace back onto the last cell
                last_shown = ch;
            }
        }
    }
    // (7) EOF -> no pause prompt; straight to teardown.
    out.extend_from_slice(TEARDOWN_EPILOGUE);
    out
}

// ===========================================================================================
// GNURUST.SCREENIO.LINEDIFF.1 -- the multi-DISPLAY same-row refresh line-diff (ncurses doupdate /
// TransformLine), SEALED for the two-DISPLAY overwrite envelope.
// ===========================================================================================
//
// When two `DISPLAY` statements write the same screen row, the second is NOT a fresh positioned
// write: ncurses keeps a virtual screen, and on refresh `TransformLine` diffs the new row against
// the old and emits the minimal update. A `DISPLAY` of a literal at `(row, col)` writes its text at
// `[col, col+len)` and blanks `[col+len, EOL)` in the virtual row; the refresh then repositions to
// the first changed column, writes the changed run, and erases the now-blank tail. This was the
// long-documented hard case ("the overlapping same-row clr_eol line-diff"); it is reproduced here
// byte-for-byte against BOTH GnuCOBOL 3.2 and 3.1.2 (`screenio_linediff_sweep`, 297-case grid).
//
// ## The model (every rule oracle-pinned)
//
// For the second DISPLAY, with the cursor at `end1` (end of the first write), the new virtual line
// is diffed against the old; `first`/`last` are the first/last changed columns. The update is the
// CHEAPEST of these candidates (cost = move bytes + written bytes + tail), ties broken by the
// listed priority -- exactly ncurses's `EmitRange` + `onscreen_mvcur`:
//
// * **CR (priority 1):** carriage-return to column 1, then write from column 1 -- literal spaces for
//   the leading blanks (cursor advance, never `rep`-compressed) then the content. This is the path
//   that "rewrites leading unchanged cells" when reaching `first` precisely would cost more.
// * **The relative move to `first` (priority 0), restricted by distance:** a backward move uses
//   repeated backspaces when the distance is **<= 4**, else column-address `HPA \e[<c>G` (priority 2);
//   a forward gap uses spaces when **<= 4**, else `HPA`; a zero move writes in place. ncurses switches
//   from repeated `cub1`/`cuf1` to `column_address` exactly at distance 5.
// * **The cursor-uncertainty CUP (priority 3):** when the backward distance is **>= 8 AND the target
//   column is >= 9**, ncurses treats the prior column as unknown and HPA is SUPPRESSED -- the move is
//   forced to a direct `CUP \e[<row>;<col>H` even though HPA would be shorter. (Below that boundary
//   HPA wins on cost; this is the one rule that is not a pure cost decision.)
//
// The written run runs to its last non-blank cell; the trailing run of cells that changed to blank
// is erased with `clr_eol \e[K` when **>= 2 cells**, a single space when exactly **1**, nothing when
// **0**. Identical runs of **>= 7** characters use the `rep` capability `\e[<n>b`.
//
// ## Sealed envelope + non-claims
//
// TWO `DISPLAY`s to the same `row` (the second overwriting/extending/overlapping the first). The
// declared follow-ons (each a separate doupdate sub-case): **three or more** same-row DISPLAYs (the
// clear-to-EOL batches differently across 3+ refreshes -- a DISPLAY's trailing erase is deferred when
// another write to the row follows); **distant isolated** survivors needing a jump+single-space; the
// **multi-ROW** diff (vertical `mvcur` between changed rows); attributes/colour on the diffed text;
// any terminal but the admitted `TERM=xterm` / ncurses 6.6.

/// The 1-based virtual-screen width the line-diff models (the admitted 80-column terminal).
const LINEDIFF_WIDTH: usize = 80;

/// Append `\e[<n><final>` (VPA `d`, HPA/CHA `G`). Local mirror for the line-diff emitter.
fn ld_csi1(out: &mut Vec<u8>, n: i32, fin: u8) {
    out.extend_from_slice(b"\x1b[");
    out.extend_from_slice(n.to_string().as_bytes());
    out.push(fin);
}

/// Write `bytes` with ncurses's `rep` run-length encoding: a run of `>= 7` identical bytes becomes
/// the byte followed by `\e[<run-1>b`. Shorter runs are literal.
fn ld_rep_write(bytes: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < bytes.len() {
        let mut j = i;
        while j < bytes.len() && bytes[j] == bytes[i] {
            j += 1;
        }
        let run = j - i;
        if run >= 7 {
            out.push(bytes[i]);
            ld_csi1(out, (run - 1) as i32, b'b');
        } else {
            out.extend_from_slice(&bytes[i..j]);
        }
        i = j;
    }
}

/// Emit the write of the changed run starting at 0-based column `start` through `last` (inclusive),
/// over the new virtual line `new`: literal leading spaces (cursor advance) for the run's leading
/// blanks, then `rep`-encoded content, then the trailing erase (`clr_eol` for >= 2 blanked cells, a
/// space for 1, nothing for 0). Returns `(bytes, end_cursor_1based)`.
fn ld_emit_from(new: &[u8], start: usize, last: usize) -> (Vec<u8>, i32) {
    let mut out = Vec::new();
    let seg_first_nb = (start..=last).find(|&k| new[k] != b' ');
    let seg_last_nb = (start..=last).rev().find(|&k| new[k] != b' ');
    match (seg_first_nb, seg_last_nb) {
        (Some(fnb), Some(lnb)) => {
            for _ in start..fnb {
                out.push(b' '); // literal leading spaces
            }
            ld_rep_write(&new[fnb..=lnb], &mut out);
            let trailing = last - lnb;
            if trailing >= 2 {
                out.extend_from_slice(b"\x1b[K");
            } else if trailing == 1 {
                out.push(b' ');
            }
            let end = (lnb + 1) + if trailing == 1 { 1 } else { 0 } + 1;
            (out, end as i32)
        }
        _ => {
            // the whole changed region is blank
            let n = last - start + 1;
            if n >= 2 {
                out.extend_from_slice(b"\x1b[K");
                (out, (start + 1) as i32)
            } else {
                out.push(b' ');
                (out, (start + 2) as i32)
            }
        }
    }
}

/// The `TransformLine` update of the second DISPLAY: diff `old` vs `new`, choose the cheapest
/// reposition+write per the model above, return `(bytes, end_cursor_1based)`. `end_col` is the
/// 1-based cursor column after the previous write; `row` is the 1-based screen row.
fn ld_transform(old: &[u8], new: &[u8], end_col: i32, row: i32) -> (Vec<u8>, i32) {
    let diffs: Vec<usize> = (0..LINEDIFF_WIDTH).filter(|&k| old[k] != new[k]).collect();
    if diffs.is_empty() {
        return (Vec::new(), end_col);
    }
    let first = diffs[0];
    let last = *diffs.last().unwrap();
    let fc = (first + 1) as i32; // 1-based first changed column
    let back = end_col - fc;
    // candidates: (cost, priority, bytes, end_cursor)
    let mut cands: Vec<(usize, u8, Vec<u8>, i32)> = Vec::new();
    let add = |prio: u8, mv: &[u8], start: usize, cands: &mut Vec<(usize, u8, Vec<u8>, i32)>| {
        let (body, end) = ld_emit_from(new, start, last);
        let mut bytes = mv.to_vec();
        bytes.extend_from_slice(&body);
        cands.push((mv.len() + body.len(), prio, bytes, end));
    };
    // CR: write from column 1.
    add(1, b"\r", 0, &mut cands);
    let suppressed = back >= 8 && fc >= 9;
    if !suppressed {
        if fc < end_col {
            // backward: backspaces up to 4, else HPA.
            if back <= 4 {
                add(0, &vec![0x08u8; back as usize], first, &mut cands);
            } else {
                let mut h = Vec::new();
                ld_csi1(&mut h, fc, b'G');
                add(2, &h, first, &mut cands);
            }
        } else if fc > end_col {
            // forward gap: spaces up to 4, else HPA.
            let fwd = fc - end_col;
            if fwd <= 4 {
                add(0, &vec![b' '; fwd as usize], first, &mut cands);
            } else {
                let mut h = Vec::new();
                ld_csi1(&mut h, fc, b'G');
                add(2, &h, first, &mut cands);
            }
        } else {
            add(0, b"", first, &mut cands); // already at first
        }
    }
    // CUP fallback (and the only horizontal in the cursor-uncertainty region).
    let mut c = Vec::new();
    c.extend_from_slice(b"\x1b[");
    c.extend_from_slice(row.to_string().as_bytes());
    c.push(b';');
    c.extend_from_slice(fc.to_string().as_bytes());
    c.push(b'H');
    add(3, &c, first, &mut cands);

    cands.sort_by_key(|(cost, prio, _, _)| (*cost, *prio));
    let best = cands.into_iter().next().unwrap();
    (best.2, best.3)
}

/// The home -> first-field move for the first DISPLAY (VPA + spaces, or `HPA` for a 5..=7 advance,
/// or `CUP`; VPA+spaces preferred on a tie). 1-based.
fn ld_move_first(col: i32, row: i32) -> Vec<u8> {
    let mut cands: Vec<(usize, u8, Vec<u8>)> = Vec::new();
    let mut vpa_sp = Vec::new();
    ld_csi1(&mut vpa_sp, row, b'd');
    for _ in 0..(col - 1) {
        vpa_sp.push(b' ');
    }
    cands.push((vpa_sp.len(), 0, vpa_sp));
    if (5..=7).contains(&(col - 1)) {
        let mut v = Vec::new();
        ld_csi1(&mut v, row, b'd');
        ld_csi1(&mut v, col, b'G');
        cands.push((v.len(), 1, v));
    }
    let mut cup = Vec::new();
    cup.extend_from_slice(b"\x1b[");
    cup.extend_from_slice(row.to_string().as_bytes());
    cup.push(b';');
    cup.extend_from_slice(col.to_string().as_bytes());
    cup.push(b'H');
    cands.push((cup.len(), 2, cup));
    cands.sort_by_key(|(l, p, _)| (*l, *p));
    cands.into_iter().next().unwrap().2
}

/// The post-DISPLAY move to the pause-prompt row at column 1 (VPA + backspaces/spaces, CR+VPA, or
/// CUP). 1-based, from cursor column `fx` on the field row to `(ty, tx)`.
fn ld_move_final(fx: i32, ty: i32, tx: i32) -> Vec<u8> {
    let mut cands: Vec<(usize, u8, Vec<u8>)> = Vec::new();
    let mut a = Vec::new();
    ld_csi1(&mut a, ty, b'd');
    if fx >= tx {
        for _ in 0..(fx - tx) {
            a.push(0x08);
        }
    } else {
        for _ in 0..(tx - fx) {
            a.push(b' ');
        }
    }
    cands.push((a.len(), 0, a));
    let mut b = vec![b'\r'];
    ld_csi1(&mut b, ty, b'd');
    for _ in 0..(tx - 1) {
        b.push(b' ');
    }
    cands.push((b.len(), 1, b));
    let mut cup = Vec::new();
    cup.extend_from_slice(b"\x1b[");
    cup.extend_from_slice(ty.to_string().as_bytes());
    cup.push(b';');
    cup.extend_from_slice(tx.to_string().as_bytes());
    cup.push(b'H');
    cands.push((cup.len(), 2, cup));
    cands.sort_by_key(|(l, p, _)| (*l, *p));
    cands.into_iter().next().unwrap().2
}

/// Reproduce the full terminal byte stream of TWO `DISPLAY` statements to the same screen `row` --
/// the first at `(row, c1)` writing `d1`, the second at `(row, c2)` writing `d2` (overwriting /
/// extending / overlapping the first) -- followed by `STOP RUN`, byte-identical to GnuCOBOL on the
/// admitted xterm/ncurses 6.6 terminal (`GNURUST.SCREENIO.LINEDIFF.1`). This is the ncurses
/// `doupdate` / `TransformLine` line-diff (`clr_eol` trailing-erase, the leading-cell rewrite, the
/// cursor-uncertainty CUP, the backward/forward HPA distance thresholds). Sealed envelope: exactly
/// two same-row DISPLAYs (see the module section above for the declared follow-ons).
pub fn two_display_line_and_stop(row: i32, c1: i32, d1: &[u8], c2: i32, d2: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(INIT_PROLOGUE);

    // The virtual line: blanks, then each DISPLAY writes [col, col+len) and blanks [col+len, EOL).
    let mut line = vec![b' '; LINEDIFF_WIDTH];
    let apply = |line: &mut Vec<u8>, col: i32, data: &[u8]| {
        let c0 = (col - 1) as usize;
        for (i, &b) in data.iter().enumerate() {
            if c0 + i < LINEDIFF_WIDTH {
                line[c0 + i] = b;
            }
        }
        for k in (c0 + data.len())..LINEDIFF_WIDTH {
            line[k] = b' ';
        }
    };

    // First DISPLAY: position from home, write (rep-encoded).
    out.extend_from_slice(&ld_move_first(c1, row));
    ld_rep_write(d1, &mut out);
    apply(&mut line, c1, d1);
    let mut cx = c1 + d1.len() as i32;

    // Second DISPLAY: the TransformLine diff.
    let old = line.clone();
    apply(&mut line, c2, d2);
    let (body, end) = ld_transform(&old, &line, cx, row);
    out.extend_from_slice(&body);
    cx = end;

    // Pause prompt on the row below, then teardown.
    out.extend_from_slice(&ld_move_final(cx, row + 1, 1));
    out.extend_from_slice(PAUSE_PROMPT);
    out.extend_from_slice(TEARDOWN_EPILOGUE);
    out
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    // KANIFOR: GNURUST.SCREENIO.INIT.1
    /// A positioned single-field DISPLAY always brackets the exact ncurses init prologue and teardown
    /// epilogue and never panics, for any in-screen line/column and any byte payload. This is the framing
    /// invariant: the emitted stream is always a well-formed `PROLOGUE .. EPILOGUE` envelope.
    #[kani::proof]
    #[kani::unwind(4)]
    fn display_framing_envelope() {
        let line: i32 = kani::any();
        let column: i32 = kani::any();
        kani::assume(line >= 1 && line <= SCREEN_ROWS - 1);
        kani::assume(column >= 1 && column <= 80);
        let b: u8 = kani::any();
        let items = vec![ScreenItem { line, column, data: vec![b], attr: None }];
        let out = display_and_stop(&items);
        assert!(out.len() >= INIT_PROLOGUE.len() + TEARDOWN_EPILOGUE.len());
        assert_eq!(&out[..INIT_PROLOGUE.len()], INIT_PROLOGUE);
        assert_eq!(&out[out.len() - TEARDOWN_EPILOGUE.len()..], TEARDOWN_EPILOGUE);
    }

    // KANIFOR: GNURUST.SCREENIO.DISPLAY.2
    /// The mvcur cost model emits a bounded, well-formed move for any in-screen target: the move bytes are
    /// always non-empty-or-empty (col 1 row 1 is a no-op) and never exceed a small constant, and every
    /// branch (spaces / HPA / VPA / CUP) terminates. Proves the cursor reproduction is total + bounded.
    #[kani::proof]
    #[kani::unwind(6)]
    fn mvcur_is_bounded() {
        let fy: i32 = kani::any();
        let fx: i32 = kani::any();
        let ty: i32 = kani::any();
        let tx: i32 = kani::any();
        kani::assume(fy >= 1 && fy <= SCREEN_ROWS && fx >= 1 && fx <= 80);
        kani::assume(ty >= 1 && ty <= SCREEN_ROWS && tx >= 1 && tx <= 80);
        let out = mvcur(fy, fx, ty, tx);
        // Every move ncurses picks here is short: a CUP `\e[<=2;<=2H` is 8 bytes, the longest local path
        // (VPA + <=4 space-fill) is 9; never unbounded.
        assert!(out.len() <= 9);
    }

    // KANIFOR: GNURUST.SCREENIO.DISPLAY.3
    /// A two-field DISPLAY is always a well-formed prologue..epilogue envelope, for any two in-screen
    /// positions and any single-byte payloads -- the multi-field path never panics and always brackets the
    /// init/teardown framing.
    #[kani::proof]
    #[kani::unwind(6)]
    fn multi_field_envelope() {
        let l1: i32 = kani::any();
        let c1: i32 = kani::any();
        let l2: i32 = kani::any();
        let c2: i32 = kani::any();
        kani::assume(l1 >= 1 && l1 <= SCREEN_ROWS - 1 && c1 >= 1 && c1 <= 80);
        kani::assume(l2 >= 1 && l2 <= SCREEN_ROWS - 1 && c2 >= 1 && c2 <= 80);
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let out = display_and_stop(&[
            ScreenItem { line: l1, column: c1, data: vec![a], attr: None },
            ScreenItem { line: l2, column: c2, data: vec![b], attr: None },
        ]);
        assert_eq!(&out[..INIT_PROLOGUE.len()], INIT_PROLOGUE);
        assert_eq!(&out[out.len() - TEARDOWN_EPILOGUE.len()..], TEARDOWN_EPILOGUE);
    }

    // KANIFOR: GNURUST.SCREENIO.ATTR.1
    /// An attributed field always emits a well-formed envelope and contains both the SGR-on opener
    /// (`\e(B\e[0;`) and the field byte, for any attribute and position -- the attribute path never panics.
    #[kani::proof]
    #[kani::unwind(4)]
    fn attribute_envelope() {
        let line: i32 = kani::any();
        let column: i32 = kani::any();
        kani::assume(line >= 1 && line <= SCREEN_ROWS - 1 && column >= 1 && column <= 80);
        let sel: u8 = kani::any();
        let attr = match sel % 5 {
            0 => ScreenAttr::Highlight,
            1 => ScreenAttr::Lowlight,
            2 => ScreenAttr::Underline,
            3 => ScreenAttr::Blink,
            _ => ScreenAttr::Reverse,
        };
        let out = display_and_stop(&[ScreenItem::with_attr(line, column, vec![b'X'], attr)]);
        assert_eq!(&out[..INIT_PROLOGUE.len()], INIT_PROLOGUE);
        assert_eq!(&out[out.len() - TEARDOWN_EPILOGUE.len()..], TEARDOWN_EPILOGUE);
    }

    // KANIFOR: GNURUST.SCREENIO.COLOR.1
    /// A colored field DISPLAY always emits a well-formed prologue..epilogue envelope and never
    /// panics, for any in-screen position (R >= 2), any data byte, and any COBOL color pair. The
    /// color-mapping arithmetic stays in range (`curses_color` masks to three bits, so the SGR
    /// codes are always `30..=37` / `40..=47`).
    #[kani::proof]
    #[kani::unwind(6)]
    fn color_envelope_and_sgr_bounds() {
        let line: i32 = kani::any();
        let column: i32 = kani::any();
        kani::assume(line >= 2 && line <= SCREEN_ROWS - 1);
        kani::assume(column >= 1 && column <= 80);
        let fg: u8 = kani::any();
        let bg: u8 = kani::any();
        let b: u8 = kani::any();
        // curses_color is a total 3-bit permutation: always 0..=7.
        assert!(curses_color(fg) <= 7 && curses_color(bg) <= 7);
        let out = color_display_and_stop(line, column, &[b], fg, bg);
        assert_eq!(&out[..INIT_PROLOGUE.len()], INIT_PROLOGUE);
        assert_eq!(&out[out.len() - TEARDOWN_EPILOGUE.len()..], TEARDOWN_EPILOGUE);
    }

    // KANIFOR: GNURUST.SCREENIO.NUMEDIT.1
    /// A numeric-edited field DISPLAY always emits a well-formed prologue..epilogue envelope and never
    /// panics, for any in-screen position and any field image (including all-blank). The leading-blank
    /// scan + positioning is total.
    #[kani::proof]
    #[kani::unwind(5)]
    fn numedit_envelope() {
        let line: i32 = kani::any();
        let column: i32 = kani::any();
        kani::assume(line >= 1 && line <= SCREEN_ROWS - 1);
        kani::assume(column >= 1 && column <= 70);
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let out = display_edited_and_stop(line, column, &[a, b]);
        assert_eq!(&out[..INIT_PROLOGUE.len()], INIT_PROLOGUE);
        assert_eq!(&out[out.len() - TEARDOWN_EPILOGUE.len()..], TEARDOWN_EPILOGUE);
    }

    // KANIFOR: GNURUST.SCREENIO.ACCEPT.1
    // KANIFOR: GNURUST.SCREENIO.ACCEPT.2
    /// An ACCEPT field always emits a well-formed prologue..epilogue envelope and never panics, for
    /// any in-screen position, any width 1..=6, and any input -- including OVERFLOW (the input here is
    /// up to 3 bytes, which over-fills a width-1 or width-2 field, exercising the ACCEPT.2 BEL/
    /// overwrite tail). The reposition + echo + field-full + overflow logic is total.
    #[kani::proof]
    #[kani::unwind(8)]
    fn accept_envelope() {
        let line: i32 = kani::any();
        let col: i32 = kani::any();
        let width: i32 = kani::any();
        kani::assume(line >= 1 && line <= SCREEN_ROWS - 1);
        kani::assume(col >= 1 && col <= 70);
        kani::assume(width >= 1 && width <= 6);
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let c: u8 = kani::any();
        let out = accept_field_and_stop(line, col, width, &[a, b, c]);
        assert_eq!(&out[..INIT_PROLOGUE.len()], INIT_PROLOGUE);
        assert_eq!(&out[out.len() - TEARDOWN_EPILOGUE.len()..], TEARDOWN_EPILOGUE);
    }

    // KANIFOR: GNURUST.SCREENIO.LINEDIFF.1
    /// A two-DISPLAY same-row line-diff always emits a well-formed prologue..epilogue envelope and
    /// never panics, for any two in-screen positions and any single-byte payloads. The virtual-line
    /// diff + cost-search reposition is total.
    #[kani::proof]
    #[kani::unwind(4)]
    fn linediff_envelope() {
        let c1: i32 = kani::any();
        let c2: i32 = kani::any();
        kani::assume(c1 >= 1 && c1 <= 70);
        kani::assume(c2 >= 1 && c2 <= 70);
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let out = two_display_line_and_stop(2, c1, &[a], c2, &[b]);
        assert_eq!(&out[..INIT_PROLOGUE.len()], INIT_PROLOGUE);
        assert_eq!(&out[out.len() - TEARDOWN_EPILOGUE.len()..], TEARDOWN_EPILOGUE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_single_field_display_matches_oracle() {
        // `DISPLAY "X" LINE 2 COLUMN 3.` then `STOP RUN.` -- the exact 230-byte oracle capture
        // (TERM=xterm, ncurses 6.6). Proves the init framing + positioned write + pause + teardown.
        let items = vec![ScreenItem { line: 2, column: 3, data: b"X".to_vec(), attr: None }];
        let got = display_and_stop(&items);
        let want: &[u8] = b"\x1b[?1049h\x1b[22;0;0t\x1b[1;24r\x1b(B\x1b[m\x1b[4l\x1b[?7h\x1b[?1h\x1b=\x1b[39;49m\x1b[?12;25h\x1b[?1006;1000h\x1b[39;49m\x1b[37m\x1b[40m\x1b[H\x1b[2J\x1b[2d  X\r\x1b[3dend of program, please press a key to exit \x1b[?1006;1000l\x1b[39;49m\r\x1b[24d\x1b[K\x1b[24;1H\x1b[?12l\x1b[?25h\x1b[?1049l\x1b[23;0;0t\r\x1b[?1l\x1b>";
        assert_eq!(got, want);
    }

    /// The cursor-movement bytes between the screen clear and the pause prompt, for one positioned field
    /// -- exercising each branch of the mvcur cost model against the oracle-pinned encodings.
    fn move_and_field(line: i32, column: i32) -> Vec<u8> {
        let out = display_and_stop(&[ScreenItem { line, column, data: b"Z".to_vec(), attr: None }]);
        let start = out.windows(4).position(|w| w == b"\x1b[2J").unwrap() + 4;
        let end = out.windows(PAUSE_PROMPT.len()).position(|w| w == PAUSE_PROMPT).unwrap();
        out[start..end].to_vec()
    }

    #[test]
    fn mvcur_cost_model_branches() {
        // same row: <=4 cols -> spaces; 6..=8 -> HPA; >=9 -> CUP.
        assert_eq!(move_and_field(1, 1), b"Z\x1b[2d\x08"); // col-1: VPA + backspace to prompt
        assert_eq!(move_and_field(1, 5), b"    Z\r\x1b[2d"); // 4 spaces
        assert_eq!(move_and_field(1, 7), b"\x1b[7GZ\r\x1b[2d"); // HPA
        assert_eq!(move_and_field(1, 10), b"\x1b[1;10HZ\r\x1b[2d"); // CUP
        // row change: col<=3 -> VPA + spaces; else CUP.
        assert_eq!(move_and_field(2, 3), b"\x1b[2d  Z\r\x1b[3d");
        assert_eq!(move_and_field(3, 5), b"\x1b[3;5HZ\r\x1b[4d"); // CUP
        assert_eq!(move_and_field(10, 40), b"\x1b[10;40HZ\r\x1b[11d");
    }

    #[test]
    fn multi_field_inter_field_mvcur() {
        // Two fields, with the inter-field move whose origin is NOT home, then the prompt move from the
        // last field's row -- each oracle-pinned (screenio_multi_sweep).
        let body = |items: &[ScreenItem]| {
            let out = display_and_stop(items);
            let s = out.windows(4).position(|w| w == b"\x1b[2J").unwrap() + 4;
            let e = out.windows(PAUSE_PROMPT.len()).position(|w| w == PAUSE_PROMPT).unwrap();
            out[s..e].to_vec()
        };
        // (1,1)AA then (2,5)BB: row-change keeps the column (VPA) then space-fills; prompt below row 2.
        assert_eq!(
            body(&[ScreenItem { line: 1, column: 1, data: b"AA".to_vec(), attr: None },
                   ScreenItem { line: 2, column: 5, data: b"BB".to_vec(), attr: None }]),
            b"AA\x1b[2d  BB\r\x1b[3d"
        );
        // last field on a HIGHER row than the first: the prompt follows the last field (row 2 -> row 3).
        assert_eq!(
            body(&[ScreenItem { line: 5, column: 5, data: b"AA".to_vec(), attr: None },
                   ScreenItem { line: 2, column: 2, data: b"BB".to_vec(), attr: None }]),
            b"\x1b[5;5HAA\r\x1b[2d BB\r\x1b[3d"
        );
    }

    #[test]
    fn monochrome_attribute_wraps_field() {
        let body = |it: ScreenItem| {
            let out = display_and_stop(&[it]);
            let s = out.windows(4).position(|w| w == b"\x1b[2J").unwrap() + 4;
            let e = out.windows(PAUSE_PROMPT.len()).position(|w| w == PAUSE_PROMPT).unwrap();
            out[s..e].to_vec()
        };
        // HIGHLIGHT at (2,3): the field char is wrapped in the SGR on (`\e(B\e[0;1m...`) / off pair.
        assert_eq!(
            body(ScreenItem::with_attr(2, 3, b"X".to_vec(), ScreenAttr::Highlight)),
            b"\x1b[2d  \x1b(B\x1b[0;1m\x1b[39;49m\x1b[37m\x1b[40mX\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\r\x1b[3d"
        );
        // each attribute differs only in the SGR code.
        for (a, n) in [(ScreenAttr::Lowlight, b'2'), (ScreenAttr::Underline, b'4'),
                       (ScreenAttr::Blink, b'5'), (ScreenAttr::Reverse, b'7')] {
            let mut want = b"\x1b[2d  \x1b(B\x1b[0;".to_vec();
            want.push(n);
            want.extend_from_slice(b"m\x1b[39;49m\x1b[37m\x1b[40mX\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\r\x1b[3d");
            assert_eq!(body(ScreenItem::with_attr(2, 3, b"X".to_vec(), a)), want);
        }
    }

    #[test]
    fn cobol_to_curses_color_permutation() {
        // The low-3-bit reversal: COBOL (blue,green,red) bit order -> curses (red,green,blue).
        assert_eq!(curses_color(0), 0); // black
        assert_eq!(curses_color(1), 4); // blue  -> SGR 34/44
        assert_eq!(curses_color(2), 2); // green
        assert_eq!(curses_color(3), 6); // cyan
        assert_eq!(curses_color(4), 1); // red   -> SGR 31/41
        assert_eq!(curses_color(5), 5); // magenta
        assert_eq!(curses_color(6), 3); // brown/yellow
        assert_eq!(curses_color(7), 7); // white
    }

    /// Append `\e[<n><fin>` -- a tiny CSI helper local to the colour test (the module's `csi1` is
    /// private to non-test code; this keeps the test self-contained).
    fn csi1_str_test(out: &mut Vec<u8>, n: i32, fin: u8) {
        out.extend_from_slice(b"\x1b[");
        out.extend_from_slice(n.to_string().as_bytes());
        out.push(fin);
    }

    #[test]
    fn color_display_matches_oracle() {
        // Exact captures of `DISPLAY` of a colored SCREEN SECTION field then `STOP RUN`, taken from
        // the admitted oracle under a pty (TERM=xterm, ncurses 6.6). The vectors are the *body*
        // between the screen clear `\e[2J` and the pause prompt; the test reconstructs the full
        // stream (prologue + body + prompt + teardown) and asserts byte-identity. Validated
        // additionally against a 628-capture R>=2 x C x 8x8-color grid offline.
        let vectors: &[(&str, i32, i32, u8, u8, &[u8])] = &[
            ("\x1b[3d\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[J\x1b[H\x1b[K\x1b[2d  \x1b[32m\x1b[44mX\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[K\r\x1b[3d", 2, 3, 2, 1, b"X"),
            ("\x1b[4d\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[J\x1b[H\x1b[K\x1b[2d\x1b[K\x1b[3d\x1b[32m\x1b[44mX\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[K\x1b[4d\x08", 3, 1, 2, 1, b"X"),
            ("\x1b[4d\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[J\x1b[H\x1b[K\x1b[2d\x1b[K\x1b[3;5H\x1b[1K \x1b[32m\x1b[44mX\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[K\r\x1b[4d", 3, 6, 2, 1, b"X"),
            ("\x1b[6d\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[J\x1b[H\x1b[K\x1b[2d\x1b[K\x1b[3d\x1b[K\x1b[4d\x1b[K\x1b[5;9H\x1b[1K \x1b[32m\x1b[44mX\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[K\r\x1b[6d", 5, 10, 2, 1, b"X"),
            ("\x1b[5d\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[J\x1b[H\x1b[K\x1b[2d\x1b[K\x1b[3d\x1b[K\x1b[4;9H\x1b[1K \x1b[31m\x1b[43mHELLO\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[K\r\x1b[5d", 4, 10, 4, 6, b"HELLO"),
            ("\x1b[7d\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[J\x1b[H\x1b[K\x1b[2d\x1b[K\x1b[3d\x1b[K\x1b[4d\x1b[K\x1b[5d\x1b[K\x1b[6d \x1b[34m\x1b[45mQRS\x1b(B\x1b[m\x1b[39;49m\x1b[37m\x1b[40m\x1b[K\r\x1b[7d", 6, 2, 1, 5, b"QRS"),
        ];
        for (body_str, line, col, fg, bg, data) in vectors {
            let body = body_str.as_bytes();
            let mut want = Vec::new();
            // The colour prologue carries the field's colour-pair SGR before `\e[H\e[2J`.
            let hc = b"\x1b[H\x1b[2J".len();
            let split = INIT_PROLOGUE.len() - hc;
            want.extend_from_slice(&INIT_PROLOGUE[..split]);
            csi1_str_test(&mut want, 30 + curses_color(*fg) as i32, b'm');
            csi1_str_test(&mut want, 40 + curses_color(*bg) as i32, b'm');
            want.extend_from_slice(&INIT_PROLOGUE[split..]);
            want.extend_from_slice(body);
            want.extend_from_slice(PAUSE_PROMPT);
            want.extend_from_slice(TEARDOWN_EPILOGUE);
            let got = color_display_and_stop(*line, *col, data, *fg, *bg);
            assert_eq!(got, want, "color mismatch at L{} C{} fg{} bg{}", line, col, fg, bg);
        }
    }

    #[test]
    fn numeric_edited_display_matches_oracle() {
        // Exact captures of a `DISPLAY` of a numeric-edited SCREEN SECTION field `FROM` a numeric
        // source, then `STOP RUN`, from the admitted oracle under a pty (TERM=xterm, ncurses 6.6).
        // Each vector gives the FULL field image `edited` (leading/trailing blanks included, exactly
        // as the move/edit engine produces it) + the body between the screen clear and the pause
        // prompt; the test reconstructs the full stream and asserts byte-identity. Covers the
        // leading-blank skip, the all-blank field, the trailing CR/DB space-fill, and trailing signs.
        let vectors: &[(&[u8], i32, i32, &str)] = &[
            (b" 1,234.56", 2, 3, "\x1b[2;4H1,234.56\r\x1b[3d"),     // ZZ,ZZ9.99 of 1234.56
            (b"     7.00", 2, 3, "\x1b[2;8H7.00\r\x1b[3d"),         // ZZ,ZZ9.99 of 7.00
            (b"       ", 2, 3, "\x1b[2;10H\r\x1b[3d"),              // ZZZZ.ZZ of 0 -> all blank
            (b"    .07", 2, 3, "\x1b[2;7H.07\r\x1b[3d"),            // ZZZZ.ZZ of 0.07
            (b"0012.30  ", 2, 3, "\x1b[2d  0012.30  \r\x1b[3d"),    // 9(4).99CR of +12.30 (CR -> 2 spaces)
            (b"0012.30CR", 2, 3, "\x1b[2d  0012.30CR\r\x1b[3d"),    // 9(4).99CR of -12.30
            (b" 00012.30", 2, 3, "\x1b[2;4H00012.30\r\x1b[3d"),     // -9(5).99 of +12.30 (sign -> blank)
            (b"+0012.30", 2, 3, "\x1b[2d  +0012.30\r\x1b[3d"),      // +9(4).99 of +12.30
            (b"    88.10-", 3, 6, "\x1b[3;10H88.10-\r\x1b[4d"),     // ZZ,ZZ9.99- of -88.10
            (b" 1,234.56", 5, 10, "\x1b[5;11H1,234.56\r\x1b[6d"),   // ZZ,ZZ9.99 of 1234.56 @ (5,10)
        ];
        for (edited, line, col, body_str) in vectors {
            let mut want = Vec::new();
            want.extend_from_slice(INIT_PROLOGUE);
            want.extend_from_slice(body_str.as_bytes());
            want.extend_from_slice(PAUSE_PROMPT);
            want.extend_from_slice(TEARDOWN_EPILOGUE);
            let got = display_edited_and_stop(*line, *col, edited);
            assert_eq!(got, want, "numedit mismatch for {:?} at L{} C{}", edited, line, col);
        }
    }

    #[test]
    fn numeric_edited_composes_with_encode_edited() {
        // End-to-end: the sealed editor produces the field image, this court positions it. Proves the
        // composition `encode_edited` -> `display_edited_and_stop` yields the oracle stream for
        // `ZZ,ZZ9.99` of 7.00.
        use crate::edited::encode_edited;
        use crate::value::Decimal;
        let v = Decimal { negative: false, digits: vec![0, 0, 0, 7, 0, 0], scale: 2 };
        let edited = encode_edited("ZZ,ZZ9.99", &v).unwrap();
        assert_eq!(edited, b"     7.00");
        let got = display_edited_and_stop(2, 3, &edited);
        let mut want = Vec::new();
        want.extend_from_slice(INIT_PROLOGUE);
        want.extend_from_slice(b"\x1b[2;8H7.00\r\x1b[3d");
        want.extend_from_slice(PAUSE_PROMPT);
        want.extend_from_slice(TEARDOWN_EPILOGUE);
        assert_eq!(got, want);
    }

    #[test]
    fn accept_field_matches_oracle() {
        // Exact captures of a `SCREEN SECTION` `ACCEPT` of an alphanumeric field, input fed then EOF,
        // from the admitted oracle under a pty (TERM=xterm, ncurses 6.6). Verified additionally
        // byte-identical against GnuCOBOL 3.1.2 (the differential oracle). Each vector gives the body
        // between the screen clear and the teardown (there is NO pause prompt: EOF skips it). Covers
        // the prompt, the reposition (backspaces / HPA / CR), the echo, and the field-full backspace.
        let vectors: &[(i32, i32, i32, &[u8], &str)] = &[
            (2, 3, 5, b"HELLO", "\x1b[2d  _____\r  HELLO\x08"), // full -> field-full backspace
            (2, 3, 5, b"HI", "\x1b[2d  _____\r  HI"),           // partial
            (2, 3, 5, b"", "\x1b[2d  _____\r  "),               // empty input
            (2, 3, 3, b"AB", "\x1b[2d  ___\x08\x08\x08AB"),     // reposition by backspaces
            (4, 10, 5, b"XY", "\x1b[4;10H_____\x1b[10GXY"),     // reposition by HPA
            (2, 3, 1, b"A", "\x1b[2d  _\x08A\x08"),             // width-1, full
        ];
        for (line, col, width, typed, body_str) in vectors {
            let mut want = Vec::new();
            want.extend_from_slice(INIT_PROLOGUE);
            want.extend_from_slice(body_str.as_bytes());
            want.extend_from_slice(TEARDOWN_EPILOGUE);
            let got = accept_field_and_stop(*line, *col, *width, typed);
            assert_eq!(got, want, "accept mismatch L{} C{} W{} typed={:?}", line, col, width, typed);
        }
    }

    #[test]
    fn two_display_line_diff_matches_oracle() {
        // Exact captures of two same-row DISPLAYs then STOP RUN (TERM=xterm, ncurses 6.6), additionally
        // byte-identical against GnuCOBOL 3.1.2. Each vector covers a distinct doupdate branch: the
        // clr_eol trailing-erase, the leading-cell rewrite, the cursor-uncertainty CUP, backward HPA,
        // backspaces, a forward-gap, forward HPA, and the same-start small-column HPA. Validated
        // additionally against a 297-case grid (both oracles). The string is the FULL stream body.
        let vectors: &[((i32, i32, &[u8]), (i32, i32, &[u8]), &str)] = &[
            ((2, 3, b"ABCDEFGH"), (2, 3, b"XY"), "\x1b[2d  ABCDEFGH\r  XY\x1b[K\r\x1b[3d"),
            ((2, 3, b"ABCDEFGH"), (2, 4, b"WXYZ"), "\x1b[2d  ABCDEFGH\r  AWXYZ\x1b[K\r\x1b[3d"),
            ((2, 10, b"ABCDEFGH"), (2, 10, b"YY"), "\x1b[2;10HABCDEFGH\x1b[2;10HYY\x1b[K\r\x1b[3d"),
            ((2, 3, b"ABCDEFGH"), (2, 6, b"XY"), "\x1b[2d  ABCDEFGH\x1b[6GXY\x1b[K\r\x1b[3d"),
            ((2, 3, b"ABCDEFGH"), (2, 9, b"XY"), "\x1b[2d  ABCDEFGH\x08\x08XY\r\x1b[3d"),
            ((2, 3, b"ABCDEFGH"), (2, 13, b"XY"), "\x1b[2d  ABCDEFGH  XY\r\x1b[3d"),
            ((2, 3, b"ABCDEFGH"), (2, 16, b"YY"), "\x1b[2d  ABCDEFGH\x1b[16GYY\r\x1b[3d"),
            ((2, 5, b"MNOPQR"), (2, 5, b"YY"), "\x1b[2;5HMNOPQR\x1b[5GYY\x1b[K\r\x1b[3d"),
        ];
        for ((r1, c1, d1), (_r2, c2, d2), body) in vectors {
            let mut want = Vec::new();
            want.extend_from_slice(INIT_PROLOGUE);
            want.extend_from_slice(body.as_bytes());
            want.extend_from_slice(PAUSE_PROMPT);
            want.extend_from_slice(TEARDOWN_EPILOGUE);
            let got = two_display_line_and_stop(*r1, *c1, d1, *c2, d2);
            assert_eq!(got, want, "linediff mismatch ({},{},{:?})->({},{:?})", r1, c1, d1, c2, d2);
        }
    }

    #[test]
    fn accept_overflow_matches_oracle() {
        // GNURUST.SCREENIO.ACCEPT.2: typing past the field width. Each excess key rings the bell and
        // -- if it differs from the last-shown cell -- overwrites it then backspaces. Oracle-pinned
        // (TERM=xterm, ncurses 6.6), additionally byte-identical against GnuCOBOL 3.1.2. Bodies are
        // between the screen clear and the teardown.
        let vectors: &[(i32, i32, i32, &[u8], &str)] = &[
            (2, 3, 1, b"AB", "\x1b[2d  _\x08A\x08\x07B\x08"),         // 1 overflow, differs
            (2, 3, 1, b"ABC", "\x1b[2d  _\x08A\x08\x07B\x08\x07C\x08"), // 2 overflow, both differ
            (2, 3, 1, b"ZZZ", "\x1b[2d  _\x08Z\x08\x07\x07"),          // overflow == shown -> BEL only
            (2, 3, 3, b"ABCD", "\x1b[2d  ___\x08\x08\x08ABC\x08\x07D\x08"), // width-3, 1 overflow
            (2, 5, 2, b"WXYZ", "\x1b[2;5H__\x08\x08WX\x08\x07Y\x08\x07Z\x08"), // width-2, 2 overflow
        ];
        for (line, col, width, typed, body_str) in vectors {
            let mut want = Vec::new();
            want.extend_from_slice(INIT_PROLOGUE);
            want.extend_from_slice(body_str.as_bytes());
            want.extend_from_slice(TEARDOWN_EPILOGUE);
            let got = accept_field_and_stop(*line, *col, *width, typed);
            assert_eq!(got, want, "overflow mismatch L{} C{} W{} typed={:?}", line, col, width, typed);
        }
    }

    #[test]
    fn accept_reposition_cost_model() {
        // Same-row backward move: HPA `\e[<c>G`, backspaces, or CR+spaces -- shortest wins, HPA before
        // backspaces before CR on a tie.
        assert_eq!(accept_reposition(6, 3), b"\x08\x08\x08"); // back 3 from col6: 3 BS (3) < HPA \e[3G (4)
        assert_eq!(accept_reposition(8, 3), b"\r  "); // back 5 from col8: CR+2sp (3) < 5 BS, < HPA \e[3G (4)
        assert_eq!(accept_reposition(15, 10), b"\x1b[10G"); // back 5 from col15: HPA (5) ties 5 BS, HPA wins
        assert_eq!(accept_reposition(4, 3), b"\x08"); // back 1: single backspace
    }

    #[test]
    fn default_color_pair_falls_back_to_plain() {
        // FOREGROUND-COLOR 7 BACKGROUND-COLOR 0 is ncurses pair 0 -> no repaint; the stream is the
        // plain positioned write, identical to a no-color DISPLAY of the same field.
        let colored = color_display_and_stop(2, 3, b"X", DEFAULT_FG, DEFAULT_BG);
        let plain = display_and_stop(&[ScreenItem::plain(2, 3, b"X".to_vec())]);
        assert_eq!(colored, plain);
    }
}
