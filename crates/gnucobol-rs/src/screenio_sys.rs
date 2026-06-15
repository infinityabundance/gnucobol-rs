//! Port of the **CBL system routines + settings + init/beep/status** layer of GnuCOBOL 3.2's
//! `libcob/screenio.c` -- the part that does *not* draw fields, but instead: rings the terminal bell
//! (`cob_beep`/`cob_speaker_beep`), maps an ACCEPT/DISPLAY return code to the CRT STATUS field
//! (`handle_status`/`get_crt3_status`/`cob_set_crt3_status`), reports the screen geometry
//! (`cob_screen_line_col`/`cob_get_scr_cols`/`cob_get_scr_lines`/`cob_sys_get_scr_size`), marshals the
//! cursor position to/from a CBL byte group (`cob_sys_get_csr_pos`/`cob_sys_set_csr_pos`), and drives the
//! screen lifecycle/settings (`cob_settings_screenio`/`cob_init_screenio`/`cob_screen_set_mode`/
//! `cob_exit_screen_from_signal`/`cob_sys_clear_screen`).
//!
//! ## Claim boundary (read this)
//!
//! Several of these routines bottom out in an **ncurses/OS leaf** that this port does not and cannot
//! reproduce as pure logic: `beep()`/`flash()`, `initscr`/`endwin`/`reset_prog_mode`,
//! `getyx`/`move`/`clear`, and the live `COLS`/`LINES` globals. Those calls are a **documented boundary**.
//! What is ported, faithfully and 1:1, is every piece of *decision logic* around that boundary:
//!
//! * the `COB_BELL` dispositions (`0` beep, `1` flash, `2` speaker, `9` silent) -- modelled as the
//!   [`BeepAction`] a real `cob_beep` would take;
//! * the speaker-beep byte (a single `\a` = `0x07` written to stdout);
//! * the exact CRT-3 three-byte status table of `get_crt3_status` (every documented `fret`);
//! * the numeric-vs-3-byte-vs-4-digit dispatch of `handle_status`;
//! * the field<->position byte marshaling of the cursor and screen-size CBL calls (the `getyx`/`move`
//!   themselves are the boundary; the 1-byte vs 2-byte packing is pure);
//! * the `WITHOUT EXTENDED` constant geometry (80x24) and the injected-state geometry for the extended
//!   path -- the screen dimensions are modelled as explicit state ([`ScreenState`]), never a global.
//!
//! Every routine is pure and panic-free: where C reads `cobsetptr`/`cobglobptr`/`COB_MODULE_PTR`
//! globals, this port takes the relevant value as an explicit argument or `&ScreenState`. Each function
//! below carries the **exact C name** from `screenio.c` and cites it.
#![forbid(unsafe_code)]

/// Which COBOL screen statement a status belongs to -- C `enum screen_statement` (`screenio.c:125`).
/// Selects which implicit exception `handle_status` raises (ACCEPT vs DISPLAY).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenStatement {
    /// `ACCEPT_STATEMENT` -- raises `COB_EC_IMP_ACCEPT` on a non-zero return.
    Accept,
    /// `DISPLAY_STATEMENT` -- raises `COB_EC_IMP_DISPLAY` on a non-zero return.
    Display,
}

/// The `COB_BELL` disposition (`cobsetptr->cob_beep_value`, runtime env `COB_BELL`) -- the value
/// `COB_BEEP_VALUE` switches on inside C `cob_beep` (`screenio.c:307`). The numeric encoding matches
/// libcob's `beepopts` enum (`common.c`): `0`=beep, `1`=flash, `2`=speaker (write `\a`), `9`=silent.
pub type BeepValue = u32;

/// The terminal action a faithful `cob_beep` produces for a given [`BeepValue`]. The actual emission
/// (`beep()`/`flash()`/`write`) is the documented ncurses/OS boundary; this enum *is* the pure decision
/// `cob_beep` makes -- the value that selects which boundary call fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeepAction {
    /// `default:` -- ncurses `beep()` (audible bell, terminfo `bel`).
    Beep,
    /// `case 1:` -- ncurses `flash()` (visual bell).
    Flash,
    /// `case 2:` -- `cob_speaker_beep()`: write a single `\a` byte to stdout.
    Speaker,
    /// `case 9:` -- do nothing (bell suppressed).
    Silent,
}

/// The single byte `cob_speaker_beep` writes to stdout: ASCII BEL (`\a`).
pub const COB_SPEAKER_BELL_BYTE: u8 = 0x07;

/// Default terminal geometry for the **non-extended** screenio build (`#else` branches throughout
/// `screenio.c`): 24 lines, 80 columns. These are the literal constants C hard-codes when
/// `WITH_EXTENDED_SCREENIO` is undefined.
pub const DEFAULT_LINES: i32 = 24;
/// Default terminal columns for the non-extended build (see [`DEFAULT_LINES`]).
pub const DEFAULT_COLS: i32 = 80;

/// Modelled screen geometry + cursor + insert mode. In C these live in ncurses globals (`LINES`/`COLS`),
/// curses cursor state (`getyx`), and `cobsetptr`/static `curr_setting_insert_mode`. This port makes that
/// state explicit so the geometry/cursor/settings logic is pure and testable; the curses I/O that *reads*
/// or *writes* this state in the real runtime is the documented boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenState {
    /// Modelled `LINES` -- terminal height in rows.
    pub lines: i32,
    /// Modelled `COLS` -- terminal width in columns.
    pub cols: i32,
    /// Modelled curses cursor row (`getyx` y / `cob_current_y`), 0-based.
    pub cursor_y: i32,
    /// Modelled curses cursor column (`getyx` x / `cob_current_x`), 0-based.
    pub cursor_x: i32,
    /// `cobglobptr->cob_screen_initialized` -- whether curses has been brought up.
    pub screen_initialized: bool,
    /// `COB_INSERT_MODE` (`cobsetptr->cob_insert_mode`): `0`=off (square cursor), `1`=on (bar cursor).
    pub insert_mode: u32,
    /// `cobsetptr->cob_extended_status` -- whether extended ACCEPT status returns are enabled.
    pub extended_status: u32,
    /// `cobsetptr->cob_use_esc` -- whether ESC terminates ACCEPT (forced off when no extended status).
    pub use_esc: u32,
}

impl Default for ScreenState {
    /// The library default before any resize/init: 80x24, cursor home, screen not yet initialized,
    /// insert off. Mirrors the constants the non-extended build hard-codes.
    fn default() -> Self {
        ScreenState {
            lines: DEFAULT_LINES,
            cols: DEFAULT_COLS,
            cursor_y: 0,
            cursor_x: 0,
            screen_initialized: false,
            insert_mode: 0,
            extended_status: 0,
            use_esc: 0,
        }
    }
}

/// Port of C `cob_speaker_beep` (`screenio.c:199`). The C routine writes a single `"\a"` byte to
/// `fileno(stdout)`. The actual `write(2)` is the OS boundary; the *content* it writes -- one BEL byte
/// -- is pure and returned here.
pub fn cob_speaker_beep() -> u8 {
    COB_SPEAKER_BELL_BYTE
}

/// Port of C `cob_beep` (`screenio.c:305`, the `WITH_EXTENDED_SCREENIO` definition). Maps the
/// `COB_BEEP_VALUE` (`beep_value`) to the terminal action. C dispatches `beep()`/`flash()`/
/// `cob_speaker_beep()`/nothing; this returns the [`BeepAction`] -- the pure switch result that selects
/// which (boundary) emitter the runtime invokes.
pub fn cob_beep(beep_value: BeepValue) -> BeepAction {
    match beep_value {
        1 => BeepAction::Flash,
        2 => BeepAction::Speaker,
        9 => BeepAction::Silent,
        _ => BeepAction::Beep,
    }
}

/// Port of C `get_crt3_status` (`screenio.c:226`). Computes the three CRT-3 status bytes a screen
/// ACCEPT/DISPLAY return code (`fret`) maps to, byte-for-byte per the C `switch`. Returns the three bytes
/// (`crtstat[0..3]`); C writes these into a 3-byte status field. ASCII `'0'`=`0x30`, `'1'`=`0x31`,
/// `'9'`=`0x39`. Note the function-key and exception-key ranges store a *raw small integer* (not a digit)
/// in byte 1, exactly as C casts `(unsigned char)(fret - 1000)` / `(fret - 2000)`.
pub fn get_crt3_status(fret: i32) -> [u8; 3] {
    // crtstat[0] = '0'; crtstat[1] = '\0'; crtstat[2] = '\0';
    let mut crtstat: [u8; 3] = [0x30, 0x00, 0x00];

    match fret {
        0 => {
            // OK
            crtstat[0] = 0x30; // '0'
            crtstat[1] = 0x30; // '0'
        }
        2005 => {
            // ESC
            crtstat[0] = 0x31; // '1'
            crtstat[1] = 0x00;
        }
        8000 | 9001 => {
            // NO_FIELD / MAX_FIELD
            crtstat[0] = 0x39; // '9'
            crtstat[1] = 0x00;
        }
        8001 => {
            // TIMEOUT
            crtstat[0] = 0x39; // '9'
            crtstat[1] = 0x01; // '\1'
        }
        _ => {
            if (1001..=1064).contains(&fret) {
                // function keys
                crtstat[0] = 0x31; // '1'
                crtstat[1] = (fret - 1000) as u8;
            } else if (2001..=2110).contains(&fret) {
                // exception keys
                crtstat[0] = 0x32; // '2'
                crtstat[1] = (fret - 2000) as u8;
            }
        }
    }
    crtstat
}

/// Port of C `cob_set_crt3_status` (`screenio.c:270`). C calls `get_crt3_status` then `memcpy`s the three
/// bytes into the 3-byte status field's data. Pure here: returns the bytes [`get_crt3_status`] computed;
/// the caller writes them into the field's 3-byte buffer.
pub fn cob_set_crt3_status(fret: i32) -> [u8; 3] {
    get_crt3_status(fret)
}

/// How `handle_status` renders the return code into the CRT STATUS field, given the field's shape. C
/// dispatches on `COB_FIELD_IS_NUMERIC` / `size == 3` / else (4-digit). This enum captures that pure
/// dispatch + the bytes to store, without owning the field memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrtStatusWrite {
    /// No `crt_status` field present (`COB_MODULE_PTR->crt_status` is null): store nothing.
    None,
    /// Numeric field: `cob_set_int(status_field, fret)` -- store the integer `fret`.
    Numeric(i32),
    /// 3-byte field: the CRT-3 status bytes from [`cob_set_crt3_status`].
    Crt3([u8; 3]),
    /// Any other size: a 4-char `"%4.4d"`-formatted `fret` (`memcpy` of 4 bytes), e.g. `0009`.
    Digits4([u8; 4]),
}

/// Describes the CRT STATUS field a `handle_status` call targets (mirrors the relevant bits of
/// `COB_MODULE_PTR->crt_status`). `None` means no status field is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrtStatusField {
    /// No status field present.
    None,
    /// A USAGE-numeric status field.
    Numeric,
    /// An alphanumeric status field of the given byte size.
    Alpha {
        /// Field byte size (the `status_field->size` C checks against 3).
        size: usize,
    },
}

/// The exception a `handle_status` raises for a non-zero `fret` (C `cob_set_exception`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenException {
    /// `COB_EC_IMP_ACCEPT`.
    ImpAccept,
    /// `COB_EC_IMP_DISPLAY`.
    ImpDisplay,
}

/// The full pure outcome of a `handle_status` call: which exception to raise (if any), the value to store
/// in `COB_ACCEPT_STATUS`, and how to write the CRT STATUS field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusOutcome {
    /// Exception to raise, or `None` when `fret == 0`.
    pub exception: Option<ScreenException>,
    /// The value assigned to `COB_ACCEPT_STATUS` (always `fret`).
    pub accept_status: i32,
    /// What to write into the CRT STATUS field.
    pub crt_write: CrtStatusWrite,
}

/// Render `fret` as the C `sprintf(buff, "%4.4d", fret)` 4-digit field would, returning the first 4 bytes
/// `memcpy`'d into the status field. Matches C for the realistic screen-return range (0..=9999): zero-
/// padded, right-justified, ASCII digits. For values whose decimal form exceeds 4 chars, C copies the
/// first 4 bytes of the longer string; this reproduces that by taking the leading 4 ASCII bytes.
fn format_status_digits4(fret: i32) -> [u8; 4] {
    let s = format!("{:04}", fret);
    let bytes = s.as_bytes();
    let mut out = [0x30u8; 4]; // unreachable default; format!("{:04}") yields >=4 bytes for any i32
    for (i, b) in bytes.iter().take(4).enumerate() {
        out[i] = *b;
    }
    out
}

/// Port of C `handle_status` (`screenio.c:279`). Given the ACCEPT/DISPLAY return code `fret`, the
/// statement kind, and the shape of the module's CRT STATUS field, computes the pure [`StatusOutcome`]:
/// the implicit exception (raised only when `fret != 0`), the `COB_ACCEPT_STATUS` value (always `fret`),
/// and the bytes/integer to store in the status field. The C function performs these side effects
/// (`cob_set_exception`, `COB_ACCEPT_STATUS = fret`, write field); this returns the data describing them.
pub fn handle_status(fret: i32, stmt: ScreenStatement, status_field: CrtStatusField) -> StatusOutcome {
    let exception = if fret != 0 {
        Some(match stmt {
            ScreenStatement::Accept => ScreenException::ImpAccept,
            ScreenStatement::Display => ScreenException::ImpDisplay,
        })
    } else {
        None
    };

    let crt_write = match status_field {
        CrtStatusField::None => CrtStatusWrite::None,
        CrtStatusField::Numeric => CrtStatusWrite::Numeric(fret),
        CrtStatusField::Alpha { size } => {
            if size == 3 {
                CrtStatusWrite::Crt3(cob_set_crt3_status(fret))
            } else {
                CrtStatusWrite::Digits4(format_status_digits4(fret))
            }
        }
    };

    StatusOutcome {
        exception,
        accept_status: fret,
        crt_write,
    }
}

/// Port of C `cob_screen_line_col` (`screenio.c:697`). Returns the value C stores via `cob_set_int(f, ..)`:
/// `COLS` when `get_columns` is true, else `LINES`. The extended build reads live `COLS`/`LINES` (here the
/// modelled `state`); the non-extended build hard-codes 80/24 ([`DEFAULT_COLS`]/[`DEFAULT_LINES`]). This
/// port returns the extended (state-driven) value; pass a default [`ScreenState`] to get the 80/24 path.
pub fn cob_screen_line_col(state: &ScreenState, get_columns: bool) -> i32 {
    if get_columns {
        state.cols
    } else {
        state.lines
    }
}

/// Port of C `cob_sys_sound_bell` (`screenio.c:716`). Returns the [`BeepAction`] the runtime performs for
/// the current `COB_BEEP_VALUE`: `9` returns silently (early `return 0`); otherwise the extended build
/// calls `cob_beep()`. (The non-extended build always speaker-beeps; the screen-init fallback that
/// speaker-beeps on init failure is part of the boundary.) Returns `Silent` for `9` and the
/// [`cob_beep`] action otherwise.
pub fn cob_sys_sound_bell(beep_value: BeepValue) -> BeepAction {
    if beep_value == 9 {
        return BeepAction::Silent;
    }
    cob_beep(beep_value)
}

/// Port of C `cob_sys_get_csr_pos` (`screenio.c:756`). Marshals the curses cursor (`getyx` -> `cline`,
/// `ccol`; here the modelled `state.cursor_y`/`cursor_x`) into the CBL byte group. C packs two ways by
/// field size: a 4-byte field gets two little-endian `cob_u16_t` (line then col); otherwise two single
/// bytes (MicroFocus layout). The non-extended build returns `[1, 1]`. `field_size` is the COBOL field's
/// byte size (`f->size`). Returns the bytes the C routine writes into the field.
pub fn cob_sys_get_csr_pos(state: &ScreenState, field_size: usize) -> Vec<u8> {
    if field_size == 4 {
        let bline = state.cursor_y as u16;
        let bcol = state.cursor_x as u16;
        let mut out = Vec::with_capacity(4);
        out.extend_from_slice(&bline.to_le_bytes());
        out.extend_from_slice(&bcol.to_le_bytes());
        out
    } else {
        vec![state.cursor_y as u8, state.cursor_x as u8]
    }
}

/// Port of C `cob_sys_get_char` (`screenio.c:797`). The actual key read (`cob_get_char` -> ncurses input)
/// is the boundary; the pure logic ported here is the **two-call function-key protocol**. Given whether
/// this is the second call (`got_sys_char`) and the current `COB_ACCEPT_STATUS`, it returns the byte to
/// store in `*fld` plus the new `got_sys_char` flag and the routine's return code.
///
/// * On the *first* call the byte comes from the (boundary) key read, so `first_char` carries the read
///   result: `> 255` becomes `0x00` and arms `got_sys_char`; otherwise the low byte is stored.
/// * On the *second* call (`got_sys_char` already set) it decodes `COB_ACCEPT_STATUS`: function keys
///   `1001..=1200` -> `status-1000`; exception keys `2001..=2055` -> `status-1800`; `0` recurses (modelled
///   here as a re-read request); anything else returns `-1`.
///
/// Returns `(fld_byte, got_sys_char_after, ret)` where `ret == 0` is success and `-1` is the error case.
pub fn cob_sys_get_char(
    got_sys_char: bool,
    first_char: i32,
    accept_status: i32,
) -> (u8, bool, i32) {
    if !got_sys_char {
        if first_char > 255 {
            (0, true, 0)
        } else {
            (first_char as u8, false, 0)
        }
    } else {
        // got_sys_char was set: decode the pending status, clearing the flag
        if accept_status == 0 {
            // C recurses into cob_get_char again; signal a re-read with byte 0, flag cleared
            (0, false, 0)
        } else if accept_status > 1000 && accept_status < 1201 {
            ((accept_status - 1000) as u8, false, 0)
        } else if accept_status > 2000 && accept_status < 2056 {
            ((accept_status - 1800) as u8, false, 0)
        } else {
            (0, false, -1)
        }
    }
}

/// Port of C `cob_sys_set_csr_pos` (`screenio.c:836`). Inverse of [`cob_sys_get_csr_pos`]: unpacks the CBL
/// byte group into a `(line, col)` cursor position. A 4-byte field is two little-endian `cob_u16_t`;
/// otherwise two single bytes. The subsequent `move(line, col)` is the boundary; this returns the decoded
/// coordinates (and the non-extended build is a no-op returning `0`, modelled by simply not moving).
/// Returns `None` if `fld` is too short for the requested layout.
pub fn cob_sys_set_csr_pos(fld: &[u8], field_size: usize) -> Option<(i32, i32)> {
    if field_size == 4 {
        if fld.len() < 4 {
            return None;
        }
        let bline = u16::from_le_bytes([fld[0], fld[1]]);
        let bcol = u16::from_le_bytes([fld[2], fld[3]]);
        Some((bline as i32, bcol as i32))
    } else {
        if fld.len() < 2 {
            return None;
        }
        Some((fld[0] as i32, fld[1] as i32))
    }
}

/// Port of C `cob_sys_get_scr_size` (`screenio.c:872`, `CBL_GET_SCR_SIZE`). Returns `(line, col, ret)`.
/// The extended build writes `(unsigned char)LINES`/`(unsigned char)COLS` and returns `0`; the
/// non-extended build writes `24`/`80`, raises `COB_EC_IMP_FEATURE_DISABLED`, and returns `-1`. This port
/// returns the extended (state-driven) values with `ret == 0`. The two bytes are the low byte of the
/// modelled `lines`/`cols` (matching the `(unsigned char)` truncation).
pub fn cob_sys_get_scr_size(state: &ScreenState) -> (u8, u8, i32) {
    (state.lines as u8, state.cols as u8, 0)
}

/// Port of C `cob_sys_set_scr_size` (`screenio.c:892`, `CBL_GC_SET_SCR_SIZE`). The extended+resize-capable
/// build calls `resize_term(line, col)` (the ncurses boundary) and returns its result; without
/// `HAVE_RESIZE_TERM` (or extended screenio) it raises `COB_EC_IMP_FEATURE_DISABLED` and returns `-1`.
/// This port performs the pure state update -- it returns the new geometry the runtime would resize to --
/// from the two request bytes. `(new_lines, new_cols)` are the requested dimensions.
pub fn cob_sys_set_scr_size(line: u8, col: u8) -> (i32, i32) {
    (line as i32, col as i32)
}

/// Port of C `cob_get_scr_cols` (`screenio.c:916`). Returns `COLS` (extended; modelled `state.cols`) or
/// `80` (non-extended). Returns the state-driven extended value; a default [`ScreenState`] gives 80.
pub fn cob_get_scr_cols(state: &ScreenState) -> i32 {
    state.cols
}

/// Port of C `cob_get_scr_lines` (`screenio.c:927`). Returns `LINES` (extended; modelled `state.lines`)
/// or `24` (non-extended). Returns the state-driven extended value; a default [`ScreenState`] gives 24.
pub fn cob_get_scr_lines(state: &ScreenState) -> i32 {
    state.lines
}

/// Port of C `cob_settings_screenio` (`screenio.c:939`). The C routine reconciles runtime settings with
/// curses cursor/mouse state when a screen is up. The mouse-mask plumbing and `curs_set` are the curses
/// boundary; the **pure settings logic** ported here is: when not initialized, do nothing; and the
/// extended-status rule -- `cob_extended_status == 0` forces `cob_use_esc = 0`. Mutates the modelled
/// `state` in place to reflect that reconciliation.
pub fn cob_settings_screenio(state: &mut ScreenState) {
    if !state.screen_initialized {
        return;
    }
    // Extended ACCEPT status returns: ESC only usable when extended status is on.
    if state.extended_status == 0 {
        state.use_esc = 0;
    }
    // (curs_set / mouseinterval / mousemask are the curses boundary.)
}

/// Port of C `cob_init_screenio` (`screenio.c:5028`). C stores the global/settings pointers, defaults the
/// "press a key to exit" message if unset, then calls `cob_settings_screenio`. The pointer storage and
/// `cob_strdup` are environment plumbing/the boundary; the pure parts ported here are the default exit
/// message and the settings reconciliation. Returns the exit-message string the runtime would install
/// (the C default when the incoming message is empty/unset) and reconciles `state`.
pub fn cob_init_screenio(state: &mut ScreenState, current_exit_msg: Option<&str>) -> String {
    let exit_msg = match current_exit_msg {
        Some(m) if !m.is_empty() => m.to_string(),
        _ => "end of program, please press a key to exit".to_string(),
    };
    cob_settings_screenio(state);
    exit_msg
}

/// Port of C `cob_exit_screen_from_signal` (`screenio.c:4543`/`4581`). A minimal, signal-safe curses exit:
/// if no global is set it does nothing; otherwise, when the screen is initialized, it calls `endwin()`
/// (the boundary). On older curses (`NCURSES_VERSION_MAJOR < 6`) a signal-safe-only (`ss_only`) request is
/// declined. This port returns whether `endwin()` should fire, given the modelled init flag, the
/// `signal_safe_only` request, and whether the host curses is safe for a signal-time exit
/// (`curses_signal_safe`, true for ncurses >= 6). It also marks the screen un-initialized when it exits.
pub fn cob_exit_screen_from_signal(
    state: &mut ScreenState,
    signal_safe_only: bool,
    curses_signal_safe: bool,
) -> bool {
    if signal_safe_only && !curses_signal_safe {
        return false;
    }
    if state.screen_initialized {
        state.screen_initialized = false;
        true // endwin()
    } else {
        false
    }
}

/// Port of C `cob_sys_clear_screen` (`screenio.c:4357`, `x'E4'`). The extended build calls `clear()` +
/// `refresh()` (the boundary) and resets `cob_current_y`/`cob_current_x` to 0; the non-extended build is a
/// no-op returning `0`. This port performs the pure state reset -- home the modelled cursor -- and returns
/// `0` like both C definitions. The actual terminal clear is the boundary.
pub fn cob_sys_clear_screen(state: &mut ScreenState) -> i32 {
    state.cursor_y = 0;
    state.cursor_x = 0;
    0
}

/// Port of C `cob_screen_set_mode` (`screenio.c:4372`/`4682`). Temporarily toggles "extended screen mode":
/// `smode == 0` suspends curses (`def_prog_mode`/`endwin`) if initialized; non-zero resumes
/// (`reset_prog_mode`) or brings curses up (`cob_screen_init`). It explicitly does *not* change
/// `cob_screen_initialized`. The curses calls are the boundary; the pure invariant ported here is the
/// `smode` -> action selection, returned as [`ScreenModeAction`]. The non-extended build is a no-op
/// ([`ScreenModeAction::NoOp`]).
pub fn cob_screen_set_mode(smode: u32, screen_initialized: bool) -> ScreenModeAction {
    if smode == 0 {
        if screen_initialized {
            ScreenModeAction::Suspend // refresh + def_prog_mode + endwin
        } else {
            ScreenModeAction::NoOp
        }
    } else if screen_initialized {
        ScreenModeAction::Resume // reset_prog_mode + refresh
    } else {
        ScreenModeAction::Init // cob_screen_init
    }
}

/// The action `cob_screen_set_mode` selects (the curses call it routes to is the boundary).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenModeAction {
    /// `smode == 0` with screen up: `refresh()` + `def_prog_mode()` + `endwin()` (suspend curses).
    Suspend,
    /// `smode != 0` with screen up: `reset_prog_mode()` + `refresh()` (resume curses).
    Resume,
    /// `smode != 0` with screen down: `cob_screen_init()` (bring curses up).
    Init,
    /// Nothing to do (`smode == 0` with screen down, or non-extended build).
    NoOp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cob_speaker_beep_emits_bel() {
        assert_eq!(cob_speaker_beep(), 0x07);
        assert_eq!(COB_SPEAKER_BELL_BYTE, 0x07);
    }

    #[test]
    fn cob_beep_dispositions() {
        assert_eq!(cob_beep(0), BeepAction::Beep);
        assert_eq!(cob_beep(1), BeepAction::Flash);
        assert_eq!(cob_beep(2), BeepAction::Speaker);
        assert_eq!(cob_beep(9), BeepAction::Silent);
        // any other value falls to default beep
        assert_eq!(cob_beep(3), BeepAction::Beep);
        assert_eq!(cob_beep(255), BeepAction::Beep);
    }

    #[test]
    fn get_crt3_status_table() {
        // OK -> "00"
        assert_eq!(get_crt3_status(0), [0x30, 0x30, 0x00]);
        // ESC
        assert_eq!(get_crt3_status(2005), [0x31, 0x00, 0x00]);
        // NO_FIELD / MAX_FIELD
        assert_eq!(get_crt3_status(8000), [0x39, 0x00, 0x00]);
        assert_eq!(get_crt3_status(9001), [0x39, 0x00, 0x00]);
        // TIMEOUT
        assert_eq!(get_crt3_status(8001), [0x39, 0x01, 0x00]);
        // default fall-through (unmapped)
        assert_eq!(get_crt3_status(42), [0x30, 0x00, 0x00]);
    }

    #[test]
    fn get_crt3_status_function_keys() {
        // F1 (1001) -> '1', raw 1
        assert_eq!(get_crt3_status(1001), [0x31, 1, 0x00]);
        // 1064 boundary -> '1', 64
        assert_eq!(get_crt3_status(1064), [0x31, 64, 0x00]);
        // 1000 and 1065 are out of the function-key range
        assert_eq!(get_crt3_status(1000), [0x30, 0x00, 0x00]);
        assert_eq!(get_crt3_status(1065), [0x30, 0x00, 0x00]);
    }

    #[test]
    fn get_crt3_status_exception_keys() {
        // 2001 -> '2', raw 1 ; 2005 is special-cased to ESC above, so test 2006
        assert_eq!(get_crt3_status(2001), [0x32, 1, 0x00]);
        assert_eq!(get_crt3_status(2006), [0x32, 6, 0x00]);
        assert_eq!(get_crt3_status(2110), [0x32, 110, 0x00]);
        // 2000 / 2111 out of range
        assert_eq!(get_crt3_status(2000), [0x30, 0x00, 0x00]);
        assert_eq!(get_crt3_status(2111), [0x30, 0x00, 0x00]);
    }

    #[test]
    fn cob_set_crt3_status_matches_get() {
        assert_eq!(cob_set_crt3_status(0), get_crt3_status(0));
        assert_eq!(cob_set_crt3_status(2005), get_crt3_status(2005));
    }

    #[test]
    fn handle_status_zero_no_exception() {
        let out = handle_status(0, ScreenStatement::Accept, CrtStatusField::Numeric);
        assert_eq!(out.exception, None);
        assert_eq!(out.accept_status, 0);
        assert_eq!(out.crt_write, CrtStatusWrite::Numeric(0));
    }

    #[test]
    fn handle_status_exception_kind() {
        let a = handle_status(9000, ScreenStatement::Accept, CrtStatusField::None);
        assert_eq!(a.exception, Some(ScreenException::ImpAccept));
        assert_eq!(a.crt_write, CrtStatusWrite::None);
        let d = handle_status(9000, ScreenStatement::Display, CrtStatusField::None);
        assert_eq!(d.exception, Some(ScreenException::ImpDisplay));
    }

    #[test]
    fn handle_status_crt3_field() {
        let out = handle_status(2005, ScreenStatement::Accept, CrtStatusField::Alpha { size: 3 });
        assert_eq!(out.crt_write, CrtStatusWrite::Crt3([0x31, 0x00, 0x00]));
    }

    #[test]
    fn handle_status_digits4_field() {
        let out = handle_status(9, ScreenStatement::Display, CrtStatusField::Alpha { size: 4 });
        // "%4.4d" of 9 -> "0009"
        assert_eq!(out.crt_write, CrtStatusWrite::Digits4(*b"0009"));
        let out2 = handle_status(1234, ScreenStatement::Display, CrtStatusField::Alpha { size: 6 });
        assert_eq!(out2.crt_write, CrtStatusWrite::Digits4(*b"1234"));
    }

    #[test]
    fn screen_line_col_reports_geometry() {
        let s = ScreenState::default();
        assert_eq!(cob_screen_line_col(&s, true), 80); // columns
        assert_eq!(cob_screen_line_col(&s, false), 24); // lines
        let big = ScreenState { lines: 50, cols: 132, ..ScreenState::default() };
        assert_eq!(cob_screen_line_col(&big, true), 132);
        assert_eq!(cob_screen_line_col(&big, false), 50);
    }

    #[test]
    fn sound_bell_honours_silence() {
        assert_eq!(cob_sys_sound_bell(9), BeepAction::Silent);
        assert_eq!(cob_sys_sound_bell(0), BeepAction::Beep);
        assert_eq!(cob_sys_sound_bell(1), BeepAction::Flash);
        assert_eq!(cob_sys_sound_bell(2), BeepAction::Speaker);
    }

    #[test]
    fn get_csr_pos_single_byte() {
        let s = ScreenState { cursor_y: 5, cursor_x: 12, ..ScreenState::default() };
        assert_eq!(cob_sys_get_csr_pos(&s, 2), vec![5, 12]);
    }

    #[test]
    fn get_csr_pos_four_byte_le() {
        let s = ScreenState { cursor_y: 300, cursor_x: 500, ..ScreenState::default() };
        let out = cob_sys_get_csr_pos(&s, 4);
        // 300 = 0x012C -> LE 2C 01 ; 500 = 0x01F4 -> LE F4 01
        assert_eq!(out, vec![0x2C, 0x01, 0xF4, 0x01]);
    }

    #[test]
    fn set_csr_pos_roundtrips_get() {
        let s = ScreenState { cursor_y: 7, cursor_x: 9, ..ScreenState::default() };
        let bytes = cob_sys_get_csr_pos(&s, 2);
        assert_eq!(cob_sys_set_csr_pos(&bytes, 2), Some((7, 9)));
        let s4 = ScreenState { cursor_y: 1000, cursor_x: 2000, ..ScreenState::default() };
        let b4 = cob_sys_get_csr_pos(&s4, 4);
        assert_eq!(cob_sys_set_csr_pos(&b4, 4), Some((1000, 2000)));
    }

    #[test]
    fn set_csr_pos_short_input() {
        assert_eq!(cob_sys_set_csr_pos(&[1], 2), None);
        assert_eq!(cob_sys_set_csr_pos(&[1, 2, 3], 4), None);
    }

    #[test]
    fn get_char_first_call() {
        // ordinary char
        assert_eq!(cob_sys_get_char(false, 65, 0), (65, false, 0));
        // function key (> 255): byte 0, arm got_sys_char
        assert_eq!(cob_sys_get_char(false, 1001, 0), (0, true, 0));
    }

    #[test]
    fn get_char_second_call_decodes_status() {
        // function key 1001 -> 1
        assert_eq!(cob_sys_get_char(true, 0, 1001), (1, false, 0));
        // exception key 2001 -> 2001-1800 = 201
        assert_eq!(cob_sys_get_char(true, 0, 2001), (201, false, 0));
        // status 0 -> recurse/re-read request
        assert_eq!(cob_sys_get_char(true, 0, 0), (0, false, 0));
        // out of range -> -1
        assert_eq!(cob_sys_get_char(true, 0, 5000), (0, false, -1));
    }

    #[test]
    fn scr_size_getters() {
        let s = ScreenState::default();
        assert_eq!(cob_sys_get_scr_size(&s), (24, 80, 0));
        assert_eq!(cob_get_scr_cols(&s), 80);
        assert_eq!(cob_get_scr_lines(&s), 24);
        assert_eq!(cob_sys_set_scr_size(50, 132), (50, 132));
    }

    #[test]
    fn settings_screenio_forces_esc_off() {
        // not initialized: no change
        let mut s = ScreenState { use_esc: 1, extended_status: 0, ..ScreenState::default() };
        cob_settings_screenio(&mut s);
        assert_eq!(s.use_esc, 1);
        // initialized + no extended status: ESC forced off
        s.screen_initialized = true;
        cob_settings_screenio(&mut s);
        assert_eq!(s.use_esc, 0);
        // initialized + extended status: ESC preserved
        let mut s2 = ScreenState {
            use_esc: 1,
            extended_status: 1,
            screen_initialized: true,
            ..ScreenState::default()
        };
        cob_settings_screenio(&mut s2);
        assert_eq!(s2.use_esc, 1);
    }

    #[test]
    fn init_screenio_defaults_exit_msg() {
        let mut s = ScreenState::default();
        let msg = cob_init_screenio(&mut s, None);
        assert_eq!(msg, "end of program, please press a key to exit");
        let msg2 = cob_init_screenio(&mut s, Some(""));
        assert_eq!(msg2, "end of program, please press a key to exit");
        let msg3 = cob_init_screenio(&mut s, Some("bye"));
        assert_eq!(msg3, "bye");
    }

    #[test]
    fn exit_screen_from_signal_logic() {
        // no global / not initialized -> no endwin
        let mut s = ScreenState { screen_initialized: false, ..ScreenState::default() };
        assert!(!cob_exit_screen_from_signal(&mut s, false, true));
        // initialized, safe curses -> endwin, marks uninitialized
        let mut s2 = ScreenState { screen_initialized: true, ..ScreenState::default() };
        assert!(cob_exit_screen_from_signal(&mut s2, true, true));
        assert!(!s2.screen_initialized);
        // signal-safe-only on unsafe curses -> declined
        let mut s3 = ScreenState { screen_initialized: true, ..ScreenState::default() };
        assert!(!cob_exit_screen_from_signal(&mut s3, true, false));
        assert!(s3.screen_initialized);
    }

    #[test]
    fn clear_screen_homes_cursor() {
        let mut s = ScreenState { cursor_y: 10, cursor_x: 20, ..ScreenState::default() };
        assert_eq!(cob_sys_clear_screen(&mut s), 0);
        assert_eq!((s.cursor_y, s.cursor_x), (0, 0));
    }

    #[test]
    fn screen_set_mode_actions() {
        assert_eq!(cob_screen_set_mode(0, true), ScreenModeAction::Suspend);
        assert_eq!(cob_screen_set_mode(0, false), ScreenModeAction::NoOp);
        assert_eq!(cob_screen_set_mode(1, true), ScreenModeAction::Resume);
        assert_eq!(cob_screen_set_mode(1, false), ScreenModeAction::Init);
    }
}
