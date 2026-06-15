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
        out.extend_from_slice(&it.data);
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
            ScreenItem { line: l1, column: c1, data: vec![a] },
            ScreenItem { line: l2, column: c2, data: vec![b] },
        ]);
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

    /// The cursor-movement bytes between the screen clear and the pause prompt, for one positioned field
    /// -- exercising each branch of the mvcur cost model against the oracle-pinned encodings.
    fn move_and_field(line: i32, column: i32) -> Vec<u8> {
        let out = display_and_stop(&[ScreenItem { line, column, data: b"Z".to_vec() }]);
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
            body(&[ScreenItem { line: 1, column: 1, data: b"AA".to_vec() },
                   ScreenItem { line: 2, column: 5, data: b"BB".to_vec() }]),
            b"AA\x1b[2d  BB\r\x1b[3d"
        );
        // last field on a HIGHER row than the first: the prompt follows the last field (row 2 -> row 3).
        assert_eq!(
            body(&[ScreenItem { line: 5, column: 5, data: b"AA".to_vec() },
                   ScreenItem { line: 2, column: 2, data: b"BB".to_vec() }]),
            b"\x1b[5;5HAA\r\x1b[2d BB\r\x1b[3d"
        );
    }
}
