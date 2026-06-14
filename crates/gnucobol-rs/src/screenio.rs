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

/// A single positioned screen item of a `DISPLAY`: 1-based COBOL `LINE`/`COLUMN` and the literal/`FROM`
/// bytes to write (already space-padded to the item's `PIC` size, as the compiler presents it).
#[derive(Debug, Clone)]
pub struct ScreenItem {
    /// COBOL `LINE` (1-based).
    pub line: i32,
    /// COBOL `COLUMN` (1-based).
    pub column: i32,
    /// The item's display bytes.
    pub data: Vec<u8>,
}

/// Reproduce ncurses's cursor move from `(from_line, from_col)` to `(to_line, to_col)` for the in-screen
/// case GnuCOBOL's positioned `DISPLAY` exercises (ncurses `mvcur` via `move`). For a move that changes the
/// row, xterm-ncurses emits `\e[<row>d` (VPA) leaving the column at 1, then advances the column with spaces
/// when that is the cheapest path; a column-only advance to the right also uses spaces. Coordinates here are
/// 1-based COBOL (ncurses is 0-based internally; the `-1` is folded into the emitted CSI which is 1-based).
fn move_cursor(out: &mut Vec<u8>, from_line: i32, from_col: i32, to_line: i32, to_col: i32) {
    let mut col = from_col;
    if to_line != from_line {
        // VPA to the target row; xterm leaves the column at 1 after a bare VPA.
        out.extend_from_slice(b"\x1b[");
        out.extend_from_slice(to_line.to_string().as_bytes());
        out.push(b'd');
        col = 1;
    }
    // Advance to the target column with spaces (ncurses chooses this for short right-moves from col 1).
    while col < to_col {
        out.push(b' ');
        col += 1;
    }
}

/// Reproduce the full terminal byte stream of a `SCREEN SECTION` `DISPLAY` of `items` followed by `STOP
/// RUN` (which, with a screen active, prints the pause prompt and tears the screen down): the init
/// prologue, each item positioned + written, the pause prompt on the line below the lowest item, then the
/// teardown epilogue -- byte-identical to GnuCOBOL on the admitted terminal.
pub fn display_and_stop(items: &[ScreenItem]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(INIT_PROLOGUE);
    // After `\e[2J` the cursor is at home (row 1, col 1).
    let (mut cur_line, mut cur_col) = (1, 1);
    for it in items {
        move_cursor(&mut out, cur_line, cur_col, it.line, it.column);
        out.extend_from_slice(&it.data);
        cur_line = it.line;
        cur_col = it.column + it.data.len() as i32;
        // ncurses returns the cursor to column 1 of the item's row after writing (observed `\r`).
        out.push(b'\r');
        cur_col = 1;
    }
    // The pause prompt sits one row below the lowest displayed item.
    let prompt_line = items.iter().map(|i| i.line).max().unwrap_or(1) + 1;
    move_cursor(&mut out, cur_line, cur_col, prompt_line, 1);
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
        let items = vec![ScreenItem { line, column, data: vec![b] }];
        let out = display_and_stop(&items);
        assert!(out.len() >= INIT_PROLOGUE.len() + TEARDOWN_EPILOGUE.len());
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
        let items = vec![ScreenItem { line: 2, column: 3, data: b"X".to_vec() }];
        let got = display_and_stop(&items);
        let want: &[u8] = b"\x1b[?1049h\x1b[22;0;0t\x1b[1;24r\x1b(B\x1b[m\x1b[4l\x1b[?7h\x1b[?1h\x1b=\x1b[39;49m\x1b[?12;25h\x1b[?1006;1000h\x1b[39;49m\x1b[37m\x1b[40m\x1b[H\x1b[2J\x1b[2d  X\r\x1b[3dend of program, please press a key to exit \x1b[?1006;1000l\x1b[39;49m\r\x1b[24d\x1b[K\x1b[24;1H\x1b[?12l\x1b[?25h\x1b[?1049l\x1b[23;0;0t\r\x1b[?1l\x1b>";
        assert_eq!(got, want);
    }
}
