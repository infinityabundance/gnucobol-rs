//! Port of the remaining `screenio.c` functions: the lazy screen init/teardown, the ACCEPT field
//! reformat, and the numeric-insert data shifts. The ncurses leaf (`initscr`/`endwin`/`move`/`addch`) and
//! the numeric reformat engine (`cob_intr_numval` + `cob_move`, sealed in the intrinsic/move courts) are
//! the declared boundaries; the pure state-transition + byte-shift logic is reproduced here.
#![forbid(unsafe_code)]

// COB_SCREEN_* attribute flag bits (common.h) the shift/echo logic reads.
const COB_SCREEN_NO_ECHO: i64 = 1 << 15;
const COB_SCREEN_SECURE: i64 = 1 << 16;
/// `COB_CH_SP` (space) / `COB_CH_AS` (asterisk) -- the echo substitutes for NO-ECHO / SECURE fields.
const COB_CH_SP: u8 = 0x20;
const COB_CH_AS: u8 = 0x2a;

/// The libcob screen-runtime state `cob_screen_init` resets (`cobglobptr`/file-statics it zeroes). Modelled
/// explicitly so the init is a testable state transition; the ncurses `initscr`/`start_color` is the leaf.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScreenInitState {
    /// `cobglobptr->cob_screen_initialized`.
    pub initialized: bool,
    /// `cob_has_color`.
    pub has_color: bool,
    /// `cob_current_y` / `cob_current_x`.
    pub current_y: i32,
    pub current_x: i32,
    /// `display_cursor_y/x`, `accept_cursor_y/x`.
    pub display_cursor_y: i32,
    pub display_cursor_x: i32,
    pub accept_cursor_y: i32,
    pub accept_cursor_x: i32,
    /// `pending_accept`.
    pub pending_accept: bool,
    /// `fore_color` / `back_color`.
    pub fore_color: i32,
    pub back_color: i32,
    /// `totl_index`.
    pub totl_index: usize,
}

/// Port of `screenio.c:cob_screen_init` -- initialize the screen runtime: when already initialized return
/// `0` unchanged; otherwise zero the runtime cursor/colour/index state and (the boundary) call ncurses
/// `initscr`/`start_color`/`keypad`/`raw`/`noecho`. Returns `0` on success (this port always succeeds; the
/// real `initscr` failure path is the boundary). After a successful init `initialized` is set by the caller
/// (`cob_screen_initialized = 1`) -- modelled here by leaving `initialized` true.
pub fn cob_screen_init(state: &mut ScreenInitState) -> i32 {
    if state.initialized {
        return 0;
    }
    state.totl_index = 0;
    state.has_color = false;
    state.current_y = 0;
    state.current_x = 0;
    state.fore_color = 0;
    state.back_color = 0;
    state.display_cursor_y = 0;
    state.display_cursor_x = 0;
    state.accept_cursor_y = 0;
    state.accept_cursor_x = 0;
    state.pending_accept = false;
    // boundary: fflush(stdout/stderr); initscr(); start_color(); keypad(); raw(); noecho(); ...
    state.initialized = true;
    0
}

/// Port of `screenio.c:init_cob_screen_if_needed` -- lazily initialize the screen on first use. Returns
/// `true` when init ran (false when already initialized). The C `cob_fatal_error(COB_FERROR_INITIALIZED)`
/// for a missing runtime, and `cob_hard_failure()` on a non-zero init return, are the declared abort
/// boundaries (this port's `cob_screen_init` never fails).
pub fn init_cob_screen_if_needed(state: &mut ScreenInitState) -> bool {
    if !state.initialized {
        let _ = cob_screen_init(state);
        true
    } else {
        false
    }
}

/// Port of `screenio.c:cob_exit_screen` -- screen teardown. When a DISPLAY left a pending ACCEPT and
/// `cob_exit_wait` is set, the runtime shows the exit message (`"\n<msg> "` or `" "`) and waits for a key;
/// then (the boundary) `endwin()`. Returns the exit-message bytes that would be displayed (`None` when no
/// pending wait), modelling the pure part; `initialized` is cleared.
pub fn cob_exit_screen(state: &mut ScreenInitState, exit_wait: bool, exit_msg: &[u8]) -> Option<Vec<u8>> {
    if !state.initialized {
        return None;
    }
    let shown = if state.pending_accept && exit_wait {
        let mut m = Vec::new();
        if !exit_msg.is_empty() && exit_msg[0] != 0 {
            m.push(b'\n');
            // up to the first NUL (snprintf "%s")
            let end = exit_msg.iter().position(|&b| b == 0).unwrap_or(exit_msg.len());
            m.extend_from_slice(&exit_msg[..end]);
            m.push(b' ');
        } else {
            m.push(b' ');
        }
        Some(m)
    } else {
        None
    };
    // boundary: the ACCEPT OMITTED wait + endwin().
    state.initialized = false;
    state.pending_accept = false;
    shown
}

/// The numeric class of an ACCEPT field for [`format_field`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatFieldKind {
    /// `COB_FIELD_IS_NUMERIC` -- reformat via `cob_intr_numval`.
    Numeric,
    /// `COB_TYPE_NUMERIC_EDITED` -- reformat via `cob_intr_numval_c`.
    NumericEdited,
    /// Neither -- no reformat.
    Other,
}

/// Port of `screenio.c:format_field` -- after ACCEPT, reformat a numeric field's typed-in data neatly: a
/// NUMERIC field is run through `cob_intr_numval` and moved back, a NUMERIC EDITED field through
/// `cob_intr_numval_c`, others are unchanged. The numval parse + `cob_move` (sealed in the intrinsic / move
/// courts) and the trailing `refresh_field` are the declared boundary; this returns which reformat applies.
pub fn format_field(kind: FormatFieldKind) -> FormatFieldKind {
    kind
}

/// Port of `screenio.c:shift_left` -- numeric-insert DELETE: shift the field bytes left by one from the
/// cursor column, pulling each `data[offset+1]` into `data[offset]` for `offset in 0..(ccolumn-scolumn)`.
/// Mutates `data` in place and returns the echo bytes that would be written (a non-space moved char is
/// echoed -- as a space for NO-ECHO, `*` for SECURE, else itself; `right_pos` is unused, as in the C).
pub fn shift_left(data: &mut [u8], attr: i64, cline: i32, ccolumn: i32, _right_pos: i32, scolumn: i32) -> Vec<(i32, i32, u8)> {
    let mut echoes = Vec::new();
    let n = ccolumn - scolumn;
    for offset in 0..n.max(0) {
        let o = offset as usize;
        if o + 1 >= data.len() {
            break;
        }
        let move_char = data[o + 1];
        data[o] = move_char;
        if move_char != b' ' {
            let ch = if attr & COB_SCREEN_NO_ECHO != 0 {
                COB_CH_SP
            } else if attr & COB_SCREEN_SECURE != 0 {
                COB_CH_AS
            } else {
                move_char
            };
            echoes.push((cline, offset + scolumn, ch));
        }
    }
    echoes
}

/// Port of `screenio.c:shift_right` -- numeric-insert INSERT: shift the field bytes right by one toward the
/// cursor, pulling each `data[offset-1]` into `data[offset]` for `offset` from `right_pos-scolumn` down to
/// just above `ccolumn-scolumn`. Mutates `data` in place and returns the echo bytes (same NO-ECHO/SECURE
/// substitution as [`shift_left`]).
pub fn shift_right(data: &mut [u8], attr: i64, cline: i32, ccolumn: i32, right_pos: i32, scolumn: i32) -> Vec<(i32, i32, u8)> {
    let mut echoes = Vec::new();
    let mut offset = right_pos - scolumn;
    while offset > ccolumn - scolumn {
        let o = offset as usize;
        if o == 0 || o >= data.len() {
            offset -= 1;
            continue;
        }
        let move_char = data[o - 1];
        data[o] = move_char;
        if move_char != b' ' {
            let ch = if attr & COB_SCREEN_NO_ECHO != 0 {
                COB_CH_SP
            } else if attr & COB_SCREEN_SECURE != 0 {
                COB_CH_AS
            } else {
                move_char
            };
            echoes.push((cline, offset + scolumn, ch));
        }
        offset -= 1;
    }
    echoes
}

/// Port of `screenio.c:cob_get_char` -- read a single character from the screen (an `ACCEPT` of a 1-byte
/// `AUTO NO-ECHO` field at the current cursor). The blocking key read is the boundary; given the read byte
/// `c` and the resulting `COB_ACCEPT_STATUS`, this returns the C result: `COB_ACCEPT_STATUS` when `c` is a
/// space, `-1` on EOF, else the character value.
pub fn cob_get_char(c: u8, accept_status: i32, is_eof: bool) -> i32 {
    if c == b' ' {
        accept_status
    } else if is_eof {
        -1
    } else {
        c as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_exit_state() {
        let mut s = ScreenInitState::default();
        assert!(init_cob_screen_if_needed(&mut s)); // ran
        assert!(s.initialized);
        assert!(!init_cob_screen_if_needed(&mut s)); // already
        assert_eq!(cob_screen_init(&mut s), 0); // idempotent
        // exit: no pending accept -> no message, clears initialized
        assert_eq!(cob_exit_screen(&mut s, true, b""), None);
        assert!(!s.initialized);
    }

    #[test]
    fn exit_screen_message() {
        let mut s = ScreenInitState { initialized: true, pending_accept: true, ..Default::default() };
        assert_eq!(cob_exit_screen(&mut s, true, b"bye\0junk"), Some(b"\nbye ".to_vec()));
        let mut s2 = ScreenInitState { initialized: true, pending_accept: true, ..Default::default() };
        assert_eq!(cob_exit_screen(&mut s2, true, b""), Some(b" ".to_vec()));
        let mut s3 = ScreenInitState { initialized: true, pending_accept: false, ..Default::default() };
        assert_eq!(cob_exit_screen(&mut s3, true, b"x"), None); // no pending -> none
    }

    #[test]
    fn format_field_dispatch() {
        assert_eq!(format_field(FormatFieldKind::Numeric), FormatFieldKind::Numeric);
        assert_eq!(format_field(FormatFieldKind::NumericEdited), FormatFieldKind::NumericEdited);
        assert_eq!(format_field(FormatFieldKind::Other), FormatFieldKind::Other);
    }

    #[test]
    fn shift_left_pulls_in_from_right() {
        // field "12345", scolumn 1, ccolumn 3 -> shift left over offsets 0,1: data[0]=data[1], data[1]=data[2]
        let mut d = *b"12345";
        let echoes = shift_left(&mut d, 0, 1, 3, 5, 1);
        assert_eq!(&d, b"23345");
        assert_eq!(echoes, vec![(1, 1, b'2'), (1, 2, b'3')]);
        // SECURE echoes '*'
        let mut d2 = *b"12345";
        let e2 = shift_left(&mut d2, COB_SCREEN_SECURE, 1, 2, 5, 1);
        assert_eq!(e2, vec![(1, 1, b'*')]);
    }

    #[test]
    fn shift_right_pushes_toward_cursor() {
        // field "12345", scolumn 1, ccolumn 2, right_pos 5 -> offsets 4,3,2: each data[o]=data[o-1]
        let mut d = *b"12345";
        let echoes = shift_right(&mut d, 0, 1, 2, 5, 1);
        assert_eq!(&d, b"12234"); // [4]<-[3]=4, [3]<-[2]=3, [2]<-[1]=2
        assert_eq!(echoes, vec![(1, 5, b'4'), (1, 4, b'3'), (1, 3, b'2')]);
    }

    #[test]
    fn get_char_result() {
        assert_eq!(cob_get_char(b' ', 7, false), 7); // space -> accept status
        assert_eq!(cob_get_char(b'A', 0, false), b'A' as i32);
        assert_eq!(cob_get_char(0xff, 0, true), -1); // eof
    }
}
