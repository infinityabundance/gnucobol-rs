//! Port of the `screenio.c` **`ACCEPT`** machinery -- the interactive screen-input engine of
//! GnuCOBOL 3.2 (`libcob/screenio.c`). The native engine drives **ncurses** (`getch`/`move`/
//! `addch`/`refresh`); the blocking key read and the screen redraw are an explicit DOCUMENTED
//! BOUNDARY. This module ports the *pure, deterministic* parts faithfully:
//!
//! * the **input-field array preparation** from the screen tree (`cob_prep_input`,
//!   `cob_screen_iterate`),
//! * the **timeout computation** (`get_accept_timeout`),
//! * the **up/down navigation index wiring** plus field sort (`screen_accept`),
//! * the **per-key DECISION logic** of the input loop (`cob_screen_get_all`, `field_accept`):
//!   given a translated key and the current cursor/field state, *which action* does it take --
//!   navigate, backspace-shift, delete-shift, store-a-character, auto-skip, return -- and the
//!   field-data transformations those actions perform, and
//! * the **entry points** (`cob_screen_accept`, `cob_field_accept`, `cob_accept_field`) plus the
//!   escape-key writeback (`cob_accept_escape_key`).
//!
//! ## The boundary
//!
//! `getch()`/`wgetch()` (the blocking interactive key read), `move`/`refresh`/`addch` (terminal
//! output), and ncurses key translation are NOT ported -- they are the interactive terminal leaf.
//! The blocking read is modelled as an **injected key stream**: a key is fed in, the pure
//! key-dispatch step transforms field state and yields an [`Action`] (and possibly a return code),
//! and the driver replays the stream. This is exactly GnuCOBOL's structure with the
//! `keyp = getch()` line replaced by "take the next injected key".
//!
//! Ported C symbols (exact names): `cob_screen_get_all`, `cob_prep_input`, `cob_screen_iterate`,
//! `get_accept_timeout`, `screen_accept`, `field_accept`, `field_accept_from_curpos`,
//! `cob_screen_accept`, `cob_field_accept`, `cob_accept_field`, `cob_accept_escape_key`.
#![forbid(unsafe_code)]

// ----------------------------------------------------------------------------------------------
// Constants (from libcob/common.h and libcob/screenio.c) -- exact values.
// ----------------------------------------------------------------------------------------------

/// `COB_INP_FLD_MAX` (`screenio.c`): maximum number of input fields in one `ACCEPT screen`.
pub const COB_INP_FLD_MAX: usize = 512;

/// `COB_TIMEOUT_SCALE` (`screenio.c`): factor applied to a `TIME-OUT` field's integer value.
/// GnuCOBOL ties this to its `getch` timeout units (milliseconds, hundredths scaled). The value
/// used by the runtime is `1000` (seconds -> ms is the conceptual unit before module-scale).
pub const COB_TIMEOUT_SCALE: i32 = 1000;

// `COB_CH_*` printable defaults.
/// `COB_CH_UL` -- the default prompt fill character `'_'` (`0x5F`).
pub const COB_CH_UL: u8 = 0x5F;
/// `COB_CH_SP` -- a space `' '` (`0x20`).
pub const COB_CH_SP: u8 = 0x20;
/// `COB_CH_AS` -- an asterisk `'*'` (`0x2A`), used for `SECURE` echo.
pub const COB_CH_AS: u8 = 0x2A;

// `COB_SCREEN_TYPE_*` (common.h).
/// `COB_SCREEN_TYPE_GROUP`.
pub const COB_SCREEN_TYPE_GROUP: i32 = 0;
/// `COB_SCREEN_TYPE_FIELD`.
pub const COB_SCREEN_TYPE_FIELD: i32 = 1;
/// `COB_SCREEN_TYPE_VALUE`.
pub const COB_SCREEN_TYPE_VALUE: i32 = 2;
/// `COB_SCREEN_TYPE_ATTRIBUTE`.
pub const COB_SCREEN_TYPE_ATTRIBUTE: i32 = 3;

// `COB_SCREEN_*` attribute flag bits (common.h) -- the ones this machinery reads.
/// `COB_SCREEN_AUTO` (`1 << 4`): auto-skip to next field when the last position is filled.
pub const COB_SCREEN_AUTO: u64 = 1 << 4;
/// `COB_SCREEN_SECURE` (`1 << 16`): echo `'*'`.
pub const COB_SCREEN_SECURE: u64 = 1 << 16;
/// `COB_SCREEN_PROMPT` (`1 << 19`): show the prompt-fill character.
pub const COB_SCREEN_PROMPT: u64 = 1 << 19;
/// `COB_SCREEN_UPDATE` (`1 << 20`): pre-load the field's current value.
pub const COB_SCREEN_UPDATE: u64 = 1 << 20;
/// `COB_SCREEN_INPUT` (`1 << 21`): the screen item accepts input.
pub const COB_SCREEN_INPUT: u64 = 1 << 21;
/// `COB_SCREEN_INITIAL` (`1 << 23`): the field the cursor starts on.
pub const COB_SCREEN_INITIAL: u64 = 1 << 23;
/// `COB_SCREEN_NO_ECHO` (`1 << 24`): echo a space, do not show typed character.
pub const COB_SCREEN_NO_ECHO: u64 = 1 << 24;
/// `COB_SCREEN_UPPER` (`1 << 28`): uppercase typed characters.
pub const COB_SCREEN_UPPER: u64 = 1 << 28;
/// `COB_SCREEN_LOWER` (`1 << 29`): lowercase typed characters.
pub const COB_SCREEN_LOWER: u64 = 1 << 29;

// ncurses `KEY_*` codes (curses.h, octal) -- the input-loop switch labels.
/// `KEY_DOWN` (`0402`).
pub const KEY_DOWN: i32 = 0o402;
/// `KEY_UP` (`0403`).
pub const KEY_UP: i32 = 0o403;
/// `KEY_LEFT` (`0404`).
pub const KEY_LEFT: i32 = 0o404;
/// `KEY_RIGHT` (`0405`).
pub const KEY_RIGHT: i32 = 0o405;
/// `KEY_HOME` (`0406`).
pub const KEY_HOME: i32 = 0o406;
/// `KEY_BACKSPACE` (`0407`).
pub const KEY_BACKSPACE: i32 = 0o407;
/// `KEY_F0` (`0410`); function keys are `KEY_F0 + n`.
pub const KEY_F0: i32 = 0o410;
/// `KEY_DC` (`0512`): delete-character.
pub const KEY_DC: i32 = 0o512;
/// `KEY_IC` (`0513`): insert-character (toggles insert mode).
pub const KEY_IC: i32 = 0o513;
/// `KEY_NPAGE` (`0522`).
pub const KEY_NPAGE: i32 = 0o522;
/// `KEY_PPAGE` (`0523`).
pub const KEY_PPAGE: i32 = 0o523;
/// `KEY_STAB` (`0524`): tab (forward).
pub const KEY_STAB: i32 = 0o524;
/// `KEY_ENTER` (`0527`).
pub const KEY_ENTER: i32 = 0o527;
/// `KEY_PRINT` (`0532`).
pub const KEY_PRINT: i32 = 0o532;
/// `KEY_BTAB` (`0541`): back-tab.
pub const KEY_BTAB: i32 = 0o541;
/// `KEY_END` (`0550`).
pub const KEY_END: i32 = 0o550;

/// `KEY_F(n)` (`curses.h`): the keycode of function key `n`.
pub fn key_f(n: i32) -> i32 {
    KEY_F0 + n
}

// ----------------------------------------------------------------------------------------------
// cob_field / cob_screen / cob_inp_struct -- the explicit state these functions touch.
// ----------------------------------------------------------------------------------------------

/// The slice of `cob_field` the ACCEPT machinery touches: a mutable data buffer, its size, and
/// whether the picture is numeric / numeric-edited (drives the `'0'`-vs-`' '` fill and the
/// digit-only input check). Faithful to `cob_field` for these purposes only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobField {
    /// `f->data` -- the field's storage.
    pub data: Vec<u8>,
    /// Whether `COB_FIELD_IS_NUMERIC(f)` holds (PIC 9 / numeric-display).
    pub is_numeric: bool,
    /// Whether `cob_field_is_numeric_or_numeric_edited(f)` holds (the input-validity check).
    pub is_numeric_or_edited: bool,
}

impl CobField {
    /// An alphanumeric field of `size` spaces.
    pub fn alpha(size: usize) -> Self {
        CobField { data: vec![b' '; size], is_numeric: false, is_numeric_or_edited: false }
    }
    /// A numeric (`PIC 9`) field of `size` zeros.
    pub fn numeric(size: usize) -> Self {
        CobField { data: vec![b'0'; size], is_numeric: true, is_numeric_or_edited: true }
    }
    /// `f->size`.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// The slice of the `cob_screen` tree node the ACCEPT machinery touches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobScreen {
    /// `s->type` (`COB_SCREEN_TYPE_*`).
    pub type_: i32,
    /// `s->attr` -- the `COB_SCREEN_*` flag set.
    pub attr: u64,
    /// `s->occurs` -- repeat count for a `VALUE` literal (0 = none).
    pub occurs: i32,
    /// `s->field` -- the input/display field for `COB_SCREEN_TYPE_FIELD` (index into a field
    /// pool; `None` for groups/values).
    pub field: Option<usize>,
    /// `s->child` for `COB_SCREEN_TYPE_GROUP` -- child node indices, in `next` order.
    pub children: Vec<usize>,
    /// `s->prompt`: the prompt-fill character if a `PROMPT` clause gives one.
    pub prompt: Option<u8>,
    /// The screen position this item ends up at (what `cob_current_y/x` becomes after the
    /// `cob_screen_puts`/`cob_screen_moveyx` -- modelled directly so the pure walk is testable).
    pub at_y: i32,
    /// The screen column this item ends up at.
    pub at_x: i32,
}

impl CobScreen {
    /// A leaf `COB_SCREEN_TYPE_FIELD` node at `(y, x)` referencing field index `field`.
    pub fn field_node(field: usize, attr: u64, y: i32, x: i32) -> Self {
        CobScreen {
            type_: COB_SCREEN_TYPE_FIELD,
            attr,
            occurs: 0,
            field: Some(field),
            children: Vec::new(),
            prompt: None,
            at_y: y,
            at_x: x,
        }
    }
    /// A `COB_SCREEN_TYPE_GROUP` node with the given child indices.
    pub fn group(children: Vec<usize>) -> Self {
        CobScreen {
            type_: COB_SCREEN_TYPE_GROUP,
            attr: 0,
            occurs: 0,
            field: None,
            children,
            prompt: None,
            at_y: 0,
            at_x: 0,
        }
    }
}

/// `struct cob_inp_struct` (`screenio.c`): one prepared input field. `scr` is the index of the
/// owning screen node; `up_index`/`down_index` are the navigation targets computed by
/// `screen_accept`; `this_y`/`this_x` are the sorted screen position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CobInpStruct {
    /// `sptr->scr` -- index of the screen node.
    pub scr: usize,
    /// `sptr->up_index`.
    pub up_index: usize,
    /// `sptr->down_index`.
    pub down_index: usize,
    /// `sptr->this_y`.
    pub this_y: i32,
    /// `sptr->this_x`.
    pub this_x: i32,
}

// ----------------------------------------------------------------------------------------------
// get_accept_timeout -- pure.
// ----------------------------------------------------------------------------------------------

/// Port of `get_accept_timeout` (`screenio.c`): the curses `timeout()` argument. When a
/// `TIME-OUT` field is present its integer value is scaled by [`COB_TIMEOUT_SCALE`]; otherwise
/// `-1` (block forever).
pub fn get_accept_timeout(ftimeout: Option<i32>) -> i32 {
    match ftimeout {
        Some(v) => v.wrapping_mul(COB_TIMEOUT_SCALE),
        None => -1,
    }
}

// ----------------------------------------------------------------------------------------------
// cob_screen_iterate / cob_prep_input -- the pure screen-tree walks.
// ----------------------------------------------------------------------------------------------

/// One emitted display operation from a screen-tree walk. The actual byte emission is the ncurses
/// boundary; this records *what* the walk decided to display, in order, so the recursion is
/// testable. (`cob_screen_puts`/`cob_screen_attr` are the boundary leaves.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenOp {
    /// `cob_screen_puts(s, s->field, ...)` for a `FIELD` node (node index).
    PutsField(usize),
    /// `cob_screen_puts(s, s->value, ...)` for a `VALUE` node (node index, repetition #).
    PutsValue(usize, i32),
    /// `cob_screen_attr(...)` for an `ATTRIBUTE` node (node index).
    Attr(usize),
    /// `cob_screen_moveyx(s)` for a `GROUP` node (node index).
    MoveYx(usize),
}

/// Port of `cob_screen_iterate` (`screenio.c`): recurse the screen tree, emitting the
/// `DISPLAY`-statement display operations in tree order. Pure: returns the op sequence (the byte
/// emission is the boundary). `nodes` is the node pool; `root` is the node to walk.
pub fn cob_screen_iterate(nodes: &[CobScreen], root: usize, out: &mut Vec<ScreenOp>) {
    let s = &nodes[root];
    match s.type_ {
        COB_SCREEN_TYPE_GROUP => {
            out.push(ScreenOp::MoveYx(root));
            for &c in &s.children {
                cob_screen_iterate(nodes, c, out);
            }
        }
        COB_SCREEN_TYPE_FIELD => {
            out.push(ScreenOp::PutsField(root));
        }
        COB_SCREEN_TYPE_VALUE => {
            out.push(ScreenOp::PutsValue(root, 0));
            if s.occurs != 0 {
                let mut n = 1;
                while n < s.occurs {
                    out.push(ScreenOp::PutsValue(root, n));
                    n += 1;
                }
            }
        }
        COB_SCREEN_TYPE_ATTRIBUTE => {
            out.push(ScreenOp::Attr(root));
        }
        _ => {}
    }
}

/// Result of `cob_prep_input`: the prepared input array (in tree order, before the sort), the
/// emitted display ops, and the C return value (`1` if `COB_INP_FLD_MAX` was exceeded, else `0`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepInput {
    /// The `cob_base_inp[0..totl_index]` entries, in the order built.
    pub inp: Vec<CobInpStruct>,
    /// The display operations performed (boundary record).
    pub ops: Vec<ScreenOp>,
    /// The C return value: `1` on overflow, `0` otherwise.
    pub overflow: bool,
}

/// Port of `cob_prep_input` (`screenio.c`): walk the screen tree, displaying every item (the
/// `cob_screen_puts`/`cob_screen_moveyx` boundary, recorded as ops) and, for every `FIELD` node
/// flagged `COB_SCREEN_INPUT`, append a `cob_inp_struct` capturing its screen position. Returns
/// `overflow = true` exactly when `totl_index` would reach `COB_INP_FLD_MAX` (the C `return 1`).
pub fn cob_prep_input(nodes: &[CobScreen], root: usize) -> PrepInput {
    let mut st = PrepInput { inp: Vec::new(), ops: Vec::new(), overflow: false };
    prep_input_rec(nodes, root, &mut st);
    st
}

fn prep_input_rec(nodes: &[CobScreen], root: usize, st: &mut PrepInput) -> bool {
    if st.overflow {
        return true;
    }
    let s = &nodes[root];
    match s.type_ {
        COB_SCREEN_TYPE_GROUP => {
            st.ops.push(ScreenOp::MoveYx(root));
            for &c in &s.children {
                if prep_input_rec(nodes, c, st) {
                    return true;
                }
            }
        }
        COB_SCREEN_TYPE_FIELD => {
            st.ops.push(ScreenOp::PutsField(root));
            if s.attr & COB_SCREEN_INPUT != 0 {
                if st.inp.len() >= COB_INP_FLD_MAX {
                    st.overflow = true;
                    return true;
                }
                st.inp.push(CobInpStruct {
                    scr: root,
                    up_index: 0,
                    down_index: 0,
                    this_y: s.at_y,
                    this_x: s.at_x,
                });
            }
        }
        COB_SCREEN_TYPE_VALUE => {
            st.ops.push(ScreenOp::PutsValue(root, 0));
            if s.occurs != 0 {
                let mut n = 1;
                while n < s.occurs {
                    st.ops.push(ScreenOp::PutsValue(root, n));
                    n += 1;
                }
            }
        }
        // COB_SCREEN_TYPE_ATTRIBUTE: the cob_screen_attr call is `#if 0`-ed out -- no op.
        _ => {}
    }
    false
}

// ----------------------------------------------------------------------------------------------
// screen_accept -- the field sort + up/down navigation wiring.
// ----------------------------------------------------------------------------------------------

/// `compare_yx` (`screenio.c`): order input fields by `(this_y, this_x)`.
fn compare_yx(a: &CobInpStruct, b: &CobInpStruct) -> core::cmp::Ordering {
    a.this_y.cmp(&b.this_y).then(a.this_x.cmp(&b.this_x))
}

/// Result of the `screen_accept` setup (everything up to the `cob_screen_get_all` call): the
/// sorted input array with `up_index`/`down_index` wired, and the `initial_curs` index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenAcceptSetup {
    /// The sorted, navigation-wired `cob_base_inp[0..totl_index]`.
    pub inp: Vec<CobInpStruct>,
    /// `initial_curs` passed to `cob_screen_get_all`.
    pub initial_curs: usize,
    /// The error status, if `cob_prep_input` overflowed (9001) or no input field (8000).
    pub error: Option<i32>,
}

/// Port of `screen_accept` (`screenio.c`) -- the deterministic setup: prepare the input array
/// (via [`cob_prep_input`]), bail with 9001 (max fields) / 8000 (no field) on the documented
/// error paths, sort the fields on `(y, x)`, then wire the cursor UP/DOWN `up_index`/`down_index`
/// arrays and pick `initial_curs` (the first `COB_SCREEN_INITIAL` field, else 0). The interactive
/// `cob_screen_get_all` is the boundary and is *not* called here -- the caller drives it with a
/// key stream.
pub fn screen_accept(nodes: &[CobScreen], root: usize) -> ScreenAcceptSetup {
    let prep = cob_prep_input(nodes, root);
    if prep.overflow {
        return ScreenAcceptSetup { inp: Vec::new(), initial_curs: 0, error: Some(9001) };
    }
    let mut inp = prep.inp;
    let totl_index = inp.len();
    if totl_index == 0 {
        return ScreenAcceptSetup { inp, initial_curs: 0, error: Some(8000) };
    }

    // qsort (cob_base_inp, totl_index, ..., compare_yx);
    inp.sort_by(compare_yx);

    // Set up array for Cursor UP/DOWN (faithful translation of the C loop).
    let mut posu: usize = 0;
    let mut posd: usize = 0;
    let mut prevy: usize = 0;
    let mut firsty: usize = 0;
    let mut starty: i32 = inp[0].this_y;
    let mut initial_curs: i32 = -1;

    for n in 0..totl_index {
        if (nodes[inp[n].scr].attr & COB_SCREEN_INITIAL != 0) && initial_curs < 0 {
            initial_curs = n as i32;
        }
        if inp[n].this_y > starty {
            if firsty == 0 {
                firsty = n;
            }
            starty = inp[n].this_y;
            for idx in posd..n {
                inp[idx].down_index = n;
            }
            posu = prevy;
            prevy = n;
            posd = n;
        }
        inp[n].up_index = posu;
    }
    for entry in inp.iter_mut().take(firsty) {
        entry.up_index = posd;
    }

    let initial_curs = if initial_curs < 0 { 0 } else { initial_curs as usize };
    ScreenAcceptSetup { inp, initial_curs, error: None }
}

// ----------------------------------------------------------------------------------------------
// cob_convert_key -- the key-mapping decision (the pure half of ncurses key translation).
// ----------------------------------------------------------------------------------------------

/// Port of the deterministic half of `cob_convert_key` (`screenio.c`): map a raw key to its
/// canonical `KEY_*` value, then zero keys that should be ignored. The ncurses raw keycodes
/// (KEY_A1 etc.) are the boundary; the ASCII control mapping and the ignore-policy are ported.
///
/// * `field_accept` -- the `field_accept` flag (1 for `field_accept`, 0 for `cob_screen_get_all`).
/// * `extended_status` -- `COB_EXTENDED_STATUS` runtime flag (allows PgUp/PgDn/Print/arrows).
/// * `use_esc` -- `COB_USE_ESC` runtime flag (allows the ESC key through).
pub fn cob_convert_key(
    keyp: i32,
    field_accept: bool,
    extended_status: bool,
    use_esc: bool,
) -> i32 {
    // Map key to KEY_xxx value.
    let mut k = match keyp {
        // '\n' | '\r' | '\004' | '\032'
        0x0A | 0x0D | 0x04 | 0x1A => KEY_ENTER,
        // '\t'
        0x09 => KEY_STAB,
        // '\b' | 0177
        0x08 | 0o177 => KEY_BACKSPACE,
        _ => keyp,
    };

    // Check if key should be ignored.
    match k {
        0x1B => {
            // '\033' ESC
            if !extended_status || !use_esc {
                k = 0;
            }
        }
        KEY_PPAGE | KEY_NPAGE | KEY_PRINT => {
            if !extended_status {
                k = 0;
            }
        }
        KEY_UP | KEY_DOWN => {
            if field_accept && !extended_status {
                k = 0;
            }
        }
        _ => {}
    }
    k
}

// ----------------------------------------------------------------------------------------------
// The per-key DECISION of the input loop -- the heart of cob_screen_get_all / field_accept.
// ----------------------------------------------------------------------------------------------

/// The decision a single key produces inside the input loop. The ncurses redraw (`cob_addch`,
/// `cob_move_cursor`, `refresh`) is the boundary; this names the *control-flow* + state outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Beep and ignore the key (`cob_beep(); continue;`).
    Beep,
    /// Consume the key, no field/index change beyond what the step already applied (`continue;`).
    Continue,
    /// Move to a different field index (`SET_FLD_AND_DATA_REFS`) -- the new `curr_index`.
    GoField(usize),
    /// Re-feed a synthetic key into the stream (`ungetch`) -- e.g. LEFT at column 0 -> BTAB.
    Ungetch(i32),
    /// Finalize all fields and return from the ACCEPT (`goto screen_return`) with this status.
    Return(i32),
    /// Toggle insert mode (`cob_toggle_insert`).
    ToggleInsert,
    /// A character was stored into the field at the cursor (the byte actually written).
    Stored(u8),
}

/// The minimal cursor + field state the per-key decision reads/writes inside the loop, for one
/// field. `scolumn`..=`right_pos` is the field's screen span; `ccolumn` is the cursor column;
/// `field` is the live editable buffer (`s->field`). `insert_mode` is `COB_INSERT_MODE`.
/// `at_eof` tracks the "cursor sat on the last char" beep state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopState {
    /// `scolumn` -- the field's first screen column.
    pub scolumn: i32,
    /// `right_pos` -- the field's last screen column (`scolumn + size - 1`).
    pub right_pos: i32,
    /// `ccolumn` -- the current cursor column.
    pub ccolumn: i32,
    /// `s->attr` -- the field's `COB_SCREEN_*` flags.
    pub attr: u64,
    /// Whether the field is numeric (`COB_FIELD_IS_NUMERIC`).
    pub is_numeric: bool,
    /// Whether the field is numeric or numeric-edited (input-validity check).
    pub is_numeric_or_edited: bool,
    /// `COB_INSERT_MODE`.
    pub insert_mode: bool,
    /// `at_eof` flag.
    pub at_eof: bool,
    /// The live field buffer (`s->field->data`, length = field size).
    pub field: Vec<u8>,
    /// `curr_index` -- the current input-field index.
    pub curr_index: usize,
    /// `totl_index` -- total number of input fields.
    pub totl_index: usize,
    /// `up_index` for the current field.
    pub up_index: usize,
    /// `down_index` for the current field.
    pub down_index: usize,
}

/// The default prompt fill character: `s->prompt->data[0]` if a `PROMPT` clause gave one, else
/// `COB_CH_UL` (`screenio.c`, top of the `cob_screen_get_all` loop).
pub fn default_prompt_char(prompt: Option<u8>) -> u8 {
    prompt.unwrap_or(COB_CH_UL)
}

/// Port of one iteration of the **`cob_screen_get_all`** input loop's key dispatch (the body of
/// the big `switch (keyp)` after `cob_convert_key`). `keyp` is the *already converted* key. This
/// is the screen-ACCEPT variant (multi-field navigation). Returns the [`Action`]; mutates `st`
/// (field bytes / cursor column / at_eof) exactly as the C does. The `getch`/`addch`/`refresh`
/// calls are the boundary.
///
/// Cites `cob_screen_get_all` (`screenio.c` ~2375).
pub fn cob_screen_get_all(st: &mut LoopState, keyp: i32, prompt: Option<u8>) -> Action {
    let _ = default_prompt_char(prompt); // mirrors the per-iteration prompt-char setup

    // keyp <= 0 -> beep (post cob_convert_key).
    if keyp <= 0 {
        return Action::Beep;
    }
    // Function keys F1..F64.
    if keyp > KEY_F0 && keyp < key_f(65) {
        return Action::Return(1000 + keyp - KEY_F0);
    }

    match keyp {
        KEY_ENTER => Action::Return(0),
        KEY_PPAGE => Action::Return(2001),
        KEY_NPAGE => Action::Return(2002),
        KEY_PRINT => Action::Return(2006),
        0x1B => Action::Return(2005),
        KEY_STAB => {
            // forward field
            let next = if st.curr_index < st.totl_index - 1 {
                st.curr_index + 1
            } else {
                0
            };
            Action::GoField(next)
        }
        KEY_BTAB => {
            let prev = if st.curr_index > 0 {
                st.curr_index - 1
            } else {
                st.totl_index - 1
            };
            Action::GoField(prev)
        }
        KEY_UP => Action::GoField(st.up_index),
        KEY_DOWN => Action::GoField(st.down_index),
        KEY_HOME => Action::GoField(0),
        KEY_END => Action::GoField(st.totl_index - 1),
        KEY_BACKSPACE => backspace_shift(st),
        KEY_LEFT => {
            if st.ccolumn > st.scolumn {
                st.ccolumn -= 1;
                Action::Continue
            } else {
                Action::Ungetch(KEY_BTAB)
            }
        }
        KEY_RIGHT => {
            if st.ccolumn < st.right_pos {
                st.ccolumn += 1;
                Action::Continue
            } else {
                Action::Ungetch(0x09) // '\t'
            }
        }
        KEY_IC => Action::ToggleInsert,
        KEY_DC => delete_shift(st),
        _ => {
            // Handle printable character.
            if keyp > 0o37 && keyp <= 0xFF {
                return store_char(st, keyp);
            }
            Action::Beep
        }
    }
}

/// Port of the printable-character branch of `cob_screen_get_all`: numeric-field validation,
/// UPPER/LOWER folding, then store the byte at the cursor and apply the auto-skip / advance.
/// Returns [`Action::Stored`] with the byte actually written, or [`Action::Beep`] on rejection,
/// or [`Action::Return`]/`Ungetch` when auto-skip fires at end of field.
fn store_char(st: &mut LoopState, keyp: i32) -> Action {
    // Numeric field check.
    if st.is_numeric_or_edited
        && (keyp != b'+' as i32
            && keyp != b'-' as i32
            && (keyp < b'0' as i32 || keyp > b'9' as i32))
    {
        return Action::Beep;
    }

    // Handle UPPER/LOWER.
    let mut k = keyp as u8;
    if st.attr & COB_SCREEN_UPPER != 0 {
        k = k.to_ascii_uppercase();
    } else if st.attr & COB_SCREEN_LOWER != 0 {
        k = k.to_ascii_lowercase();
    }

    let off = (st.ccolumn - st.scolumn) as usize;
    if off < st.field.len() {
        st.field[off] = k;
    }

    if st.ccolumn == st.right_pos {
        // Auto-skip at end of field.
        if st.attr & COB_SCREEN_AUTO != 0 {
            if st.curr_index == st.totl_index - 1 {
                return Action::Return(0);
            } else {
                return Action::Ungetch(0x09); // '\t'
            }
        }
        if st.at_eof {
            return Action::Beep; // beep but key already stored
        } else {
            st.at_eof = !st.insert_mode;
        }
    } else {
        st.ccolumn += 1;
    }
    Action::Stored(k)
}

/// Port of the `KEY_BACKSPACE` branch: shift the remainder of the field left over the cursor,
/// blank the rightmost position (`'0'` for numeric, `' '` otherwise), move the cursor left.
/// Beeps if the cursor is already at the field start.
fn backspace_shift(st: &mut LoopState) -> Action {
    if st.ccolumn > st.scolumn {
        st.at_eof = false;
        // Shift remainder left with cursor.
        let mut count = st.ccolumn;
        while count < st.right_pos + 1 {
            let from = (count - st.scolumn) as usize;
            let to = (count - st.scolumn - 1) as usize;
            if from < st.field.len() {
                let mc = st.field[from];
                st.field[to] = mc;
            }
            count += 1;
        }
        // Put space (or '0') as the right-most character.
        let last = st.field.len().saturating_sub(1);
        st.field[last] = if st.is_numeric { b'0' } else { b' ' };
        st.ccolumn -= 1;
        Action::Continue
    } else {
        Action::Beep
    }
}

/// Port of the `KEY_DC` branch: delete the character under the cursor, shifting the remainder
/// left, and blank the rightmost position. (The decimal-point guard is the numeric-edited
/// refinement; the base shift is ported here.)
fn delete_shift(st: &mut LoopState) -> Action {
    let mut count = st.ccolumn;
    while count < st.right_pos {
        let from = (count - st.scolumn + 1) as usize;
        let to = (count - st.scolumn) as usize;
        if from < st.field.len() {
            let mc = st.field[from];
            st.field[to] = mc;
        }
        count += 1;
    }
    let last = st.field.len().saturating_sub(1);
    st.field[last] = if st.is_numeric { b'0' } else { b' ' };
    Action::Continue
}

/// Port of one iteration of the **`field_accept`** key dispatch (the single-field `ACCEPT`
/// variant). The special-key handling differs from `cob_screen_get_all`: arrows/tab/page return
/// distinct exception codes rather than navigating. `keyp` is the already-converted key.
///
/// Cites `field_accept` (`screenio.c` ~3384).
pub fn field_accept(st: &mut LoopState, keyp: i32, omitted: bool) -> Action {
    if keyp <= 0 {
        return Action::Beep;
    }
    if keyp > KEY_F0 && keyp < key_f(65) {
        return Action::Return(1000 + keyp - KEY_F0);
    }

    // Return special keys (same for normal + OMITTED).
    match keyp {
        KEY_ENTER => return Action::Return(0),
        KEY_PPAGE => return Action::Return(2001),
        KEY_NPAGE => return Action::Return(2002),
        KEY_UP => return Action::Return(2003),
        KEY_DOWN => return Action::Return(2004),
        KEY_PRINT => return Action::Return(2006),
        0x1B => return Action::Return(2005),
        KEY_STAB => return Action::Return(2007),
        KEY_BTAB => return Action::Return(2008),
        _ => {}
    }

    // ACCEPT OMITTED: positioning keys return distinct exception codes.
    if omitted {
        return match keyp {
            KEY_LEFT => Action::Return(2009),
            KEY_RIGHT => Action::Return(2010),
            KEY_IC => Action::Return(2011),
            KEY_DC => Action::Return(2012),
            KEY_BACKSPACE => Action::Return(2013),
            KEY_HOME => Action::Return(2014),
            KEY_END => Action::Return(2015),
            _ => Action::Beep,
        };
    }

    // Positioning keys (field present).
    match keyp {
        KEY_BACKSPACE => {
            let a = backspace_shift(st);
            st.at_eof = false;
            a
        }
        KEY_LEFT => {
            if st.ccolumn > st.scolumn {
                st.ccolumn -= 1;
            }
            Action::Continue
        }
        KEY_RIGHT => {
            if st.ccolumn < st.right_pos {
                st.ccolumn += 1;
            }
            Action::Continue
        }
        KEY_IC => Action::ToggleInsert,
        KEY_DC => delete_shift(st),
        _ => {
            if keyp > 0o37 && keyp <= 0xFF {
                return store_char(st, keyp);
            }
            Action::Beep
        }
    }
}

/// Port of `field_accept_from_curpos` (`screenio.c`): `getyx`-then-`field_accept`. The `getyx`
/// is the boundary (it reads the live cursor); here the current `(line, column)` is injected and
/// the call delegates to [`field_accept`]. Returns the dispatch for `keyp`.
pub fn field_accept_from_curpos(st: &mut LoopState, keyp: i32, omitted: bool) -> Action {
    // getyx (stdscr, cline, ccolumn) -> already reflected in st by the caller.
    field_accept(st, keyp, omitted)
}

// ----------------------------------------------------------------------------------------------
// Entry points -- cob_screen_accept / cob_field_accept / cob_accept_field.
// ----------------------------------------------------------------------------------------------

/// Resolved `LINE`/`COLUMN` for an ACCEPT entry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptPos {
    /// 1-based COBOL `LINE` (defaults to 1 when no field given).
    pub line: i32,
    /// 1-based COBOL `COLUMN` (defaults to 1 when no field given).
    pub column: i32,
}

/// `get_line_column` (`screenio.c`): resolve `(line, column)` from optional fields, defaulting
/// each to 1 when absent.
fn get_line_column(line: Option<i32>, col: Option<i32>) -> AcceptPos {
    AcceptPos { line: line.unwrap_or(1), column: col.unwrap_or(1) }
}

/// Port of `cob_screen_accept` (`screenio.c`): the `ACCEPT scr-name` entry point. Resolves the
/// position, then runs the [`screen_accept`] setup. The interactive `cob_screen_get_all` is the
/// boundary; the returned [`ScreenAcceptSetup`] is everything the entry point computes before it.
pub fn cob_screen_accept(
    nodes: &[CobScreen],
    root: usize,
    line: Option<i32>,
    column: Option<i32>,
) -> (AcceptPos, ScreenAcceptSetup) {
    let pos = get_line_column(line, column);
    let setup = screen_accept(nodes, root);
    (pos, setup)
}

/// Port of `cob_field_accept` (`screenio.c`) -- the `< 3.2` compat `ACCEPT scr-item` entry. It
/// resolves the position and delegates to `field_accept`. Returns the resolved position and the
/// initial `LoopState` for the field; the interactive loop is the boundary.
pub fn cob_field_accept(
    field: &CobField,
    fattr: u64,
    line: Option<i32>,
    column: Option<i32>,
    size_is: Option<usize>,
) -> (AcceptPos, LoopState) {
    let pos = get_line_column(line, column);
    let st = build_field_loop_state(field, fattr, pos, size_is);
    (pos, st)
}

/// Port of `cob_accept_field` (`screenio.c`) -- the 3.2 `ACCEPT scr-item WITH/AT` entry. The
/// varargs `parms` string is parsed by the caller; this takes the already-extracted clauses and
/// delegates to `field_accept`. Returns the resolved position and the initial `LoopState`.
pub fn cob_accept_field(
    field: &CobField,
    fattr: u64,
    line: Option<i32>,
    column: Option<i32>,
    size_is: Option<usize>,
) -> (AcceptPos, LoopState) {
    let pos = get_line_column(line, column);
    let st = build_field_loop_state(field, fattr, pos, size_is);
    (pos, st)
}

/// Build the initial `LoopState` for a single-field ACCEPT, mirroring `field_accept`'s setup:
/// `size_accept` (SIZE IS, ignoring 0), `right_pos = scolumn + size_accept - 1`, the buffer
/// pre-loaded from the field on `UPDATE` else blanked.
fn build_field_loop_state(
    field: &CobField,
    fattr: u64,
    pos: AcceptPos,
    size_is: Option<usize>,
) -> LoopState {
    let scolumn = pos.column - 1; // curses is 0-based
    let size_accept = match size_is {
        Some(s) if s >= 1 => s,
        _ => field.size(),
    };
    // temp_field / COB_TERM_BUFF setup.
    let mut buf = if fattr & COB_SCREEN_UPDATE != 0 {
        let mut v = field.data.clone();
        v.resize(size_accept, b' ');
        v
    } else {
        vec![b' '; size_accept]
    };
    buf.resize(size_accept, b' ');
    LoopState {
        scolumn,
        right_pos: scolumn + size_accept as i32 - 1,
        ccolumn: scolumn,
        attr: fattr,
        is_numeric: field.is_numeric,
        is_numeric_or_edited: field.is_numeric_or_edited,
        insert_mode: false,
        at_eof: false,
        field: buf,
        curr_index: 0,
        totl_index: 1,
        up_index: 0,
        down_index: 0,
    }
}

// ----------------------------------------------------------------------------------------------
// cob_accept_escape_key -- write the termination key into a field.
// ----------------------------------------------------------------------------------------------

/// Port of `get_crt3_status` (`screenio.c`): translate an ACCEPT return code into the 3-byte CRT
/// status. Byte 2 is always `'\0'`. Returns `[c0, c1, c2]`.
pub fn get_crt3_status(fret: i32) -> [u8; 3] {
    let mut s = [b'0', 0u8, 0u8];
    match fret {
        0 => {
            s[0] = b'0';
            s[1] = b'0';
        }
        2005 => {
            s[0] = b'1';
            s[1] = 0;
        }
        8000 | 9001 => {
            s[0] = b'9';
            s[1] = 0;
        }
        8001 => {
            s[0] = b'9';
            s[1] = 1;
        }
        _ => {
            if (1001..=1064).contains(&fret) {
                s[0] = b'1';
                s[1] = (fret - 1000) as u8;
            } else if (2001..=2110).contains(&fret) {
                s[0] = b'2';
                s[1] = (fret - 2000) as u8;
            }
        }
    }
    s
}

/// Port of `cob_accept_escape_key` (`screenio.c`): write the last ACCEPT termination key
/// (`COB_ACCEPT_STATUS`, passed in as `status`) into the field `f`. For a 2-byte field with a
/// non-zero status the MF/ACU 9(2) translation via [`get_crt3_status`] is used (first two CRT
/// bytes); otherwise `cob_set_int` stores the integer status (modelled here by overwriting the
/// field bytes with the value's setting -- the integer store is the numeric-field boundary, so we
/// return the would-be set value for the caller to apply).
pub fn cob_accept_escape_key(f: &mut CobField, status: i32) -> EscapeKeyResult {
    if f.size() == 2 && status != 0 {
        let crtstat = get_crt3_status(status);
        f.data[0] = crtstat[0];
        f.data[1] = crtstat[1];
        EscapeKeyResult::Crt3([crtstat[0], crtstat[1]])
    } else {
        // cob_set_int (f, status) -- numeric store is the boundary; report the value.
        EscapeKeyResult::SetInt(status)
    }
}

/// The outcome of [`cob_accept_escape_key`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeKeyResult {
    /// The 2-byte CRT-status translation was written into the field (the two bytes).
    Crt3([u8; 2]),
    /// `cob_set_int(f, status)` would store this integer into the (numeric) field.
    SetInt(i32),
}

// ----------------------------------------------------------------------------------------------
// Tests.
// ----------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_accept_timeout_scales_and_defaults() {
        assert_eq!(get_accept_timeout(None), -1);
        assert_eq!(get_accept_timeout(Some(0)), 0);
        assert_eq!(get_accept_timeout(Some(5)), 5 * COB_TIMEOUT_SCALE);
        assert_eq!(get_accept_timeout(Some(-1)), -COB_TIMEOUT_SCALE);
    }

    fn two_field_tree() -> (Vec<CobScreen>, usize) {
        // group { field0 @ (3,5) INPUT, field1 @ (1,2) INPUT }
        let f0 = CobScreen::field_node(0, COB_SCREEN_INPUT, 3, 5);
        let f1 = CobScreen::field_node(1, COB_SCREEN_INPUT, 1, 2);
        let group = CobScreen::group(vec![1, 2]);
        (vec![group, f0, f1], 0)
    }

    #[test]
    fn cob_screen_iterate_walks_group_in_order() {
        let (nodes, root) = two_field_tree();
        let mut ops = Vec::new();
        cob_screen_iterate(&nodes, root, &mut ops);
        assert_eq!(
            ops,
            vec![ScreenOp::MoveYx(0), ScreenOp::PutsField(1), ScreenOp::PutsField(2)]
        );
    }

    #[test]
    fn cob_screen_iterate_value_occurs() {
        let mut v = CobScreen::field_node(0, 0, 1, 1);
        v.type_ = COB_SCREEN_TYPE_VALUE;
        v.field = None;
        v.occurs = 3;
        let nodes = vec![v];
        let mut ops = Vec::new();
        cob_screen_iterate(&nodes, 0, &mut ops);
        assert_eq!(
            ops,
            vec![
                ScreenOp::PutsValue(0, 0),
                ScreenOp::PutsValue(0, 1),
                ScreenOp::PutsValue(0, 2)
            ]
        );
    }

    #[test]
    fn cob_prep_input_collects_input_fields() {
        let (nodes, root) = two_field_tree();
        let prep = cob_prep_input(&nodes, root);
        assert!(!prep.overflow);
        assert_eq!(prep.inp.len(), 2);
        assert_eq!(prep.inp[0].this_y, 3);
        assert_eq!(prep.inp[1].this_y, 1);
        assert_eq!(prep.inp[0].scr, 1);
    }

    #[test]
    fn cob_prep_input_skips_non_input_field() {
        let f = CobScreen::field_node(0, 0, 1, 1); // no COB_SCREEN_INPUT
        let nodes = vec![CobScreen::group(vec![1]), f];
        let prep = cob_prep_input(&nodes, 0);
        assert_eq!(prep.inp.len(), 0);
        assert_eq!(prep.ops, vec![ScreenOp::MoveYx(0), ScreenOp::PutsField(1)]);
    }

    #[test]
    fn cob_prep_input_overflow() {
        // build a group with COB_INP_FLD_MAX + 1 input children
        let mut nodes = vec![CobScreen::group((1..=COB_INP_FLD_MAX + 1).collect())];
        for i in 0..COB_INP_FLD_MAX + 1 {
            nodes.push(CobScreen::field_node(i, COB_SCREEN_INPUT, i as i32, 1));
        }
        let prep = cob_prep_input(&nodes, 0);
        assert!(prep.overflow);
    }

    #[test]
    fn screen_accept_sorts_and_wires_navigation() {
        let (nodes, root) = two_field_tree();
        let setup = screen_accept(&nodes, root);
        assert!(setup.error.is_none());
        // sorted: (1,2) then (3,5)
        assert_eq!(setup.inp[0].this_y, 1);
        assert_eq!(setup.inp[1].this_y, 3);
        // two rows: top row's down_index -> 1, second row's up_index -> 0
        assert_eq!(setup.inp[0].down_index, 1);
        assert_eq!(setup.inp[1].up_index, 0);
        assert_eq!(setup.initial_curs, 0);
    }

    #[test]
    fn screen_accept_no_field_is_8000() {
        let nodes = vec![CobScreen::group(vec![])];
        let setup = screen_accept(&nodes, 0);
        assert_eq!(setup.error, Some(8000));
    }

    #[test]
    fn screen_accept_initial_field_picks_initial_curs() {
        let f0 = CobScreen::field_node(0, COB_SCREEN_INPUT, 1, 1);
        let f1 = CobScreen::field_node(1, COB_SCREEN_INPUT | COB_SCREEN_INITIAL, 2, 1);
        let nodes = vec![CobScreen::group(vec![1, 2]), f0, f1];
        let setup = screen_accept(&nodes, 0);
        assert_eq!(setup.initial_curs, 1);
    }

    fn alpha_state(size: usize) -> LoopState {
        LoopState {
            scolumn: 0,
            right_pos: size as i32 - 1,
            ccolumn: 0,
            attr: 0,
            is_numeric: false,
            is_numeric_or_edited: false,
            insert_mode: false,
            at_eof: false,
            field: vec![b' '; size],
            curr_index: 0,
            totl_index: 3,
            up_index: 0,
            down_index: 0,
        }
    }

    #[test]
    fn convert_key_maps_controls() {
        assert_eq!(cob_convert_key(0x0A, false, true, true), KEY_ENTER);
        assert_eq!(cob_convert_key(0x09, false, true, true), KEY_STAB);
        assert_eq!(cob_convert_key(0x08, false, true, true), KEY_BACKSPACE);
        assert_eq!(cob_convert_key(0o177, false, true, true), KEY_BACKSPACE);
    }

    #[test]
    fn convert_key_ignores_esc_without_flags() {
        assert_eq!(cob_convert_key(0x1B, false, false, false), 0);
        assert_eq!(cob_convert_key(0x1B, false, true, true), 0x1B);
    }

    #[test]
    fn convert_key_arrows_ignored_in_field_accept() {
        assert_eq!(cob_convert_key(KEY_UP, true, false, false), 0);
        assert_eq!(cob_convert_key(KEY_UP, false, false, false), KEY_UP);
        assert_eq!(cob_convert_key(KEY_UP, true, true, false), KEY_UP);
    }

    #[test]
    fn get_all_stores_printable_and_advances() {
        let mut st = alpha_state(4);
        let a = cob_screen_get_all(&mut st, b'A' as i32, None);
        assert_eq!(a, Action::Stored(b'A'));
        assert_eq!(st.field[0], b'A');
        assert_eq!(st.ccolumn, 1);
    }

    #[test]
    fn get_all_uppercases() {
        let mut st = alpha_state(4);
        st.attr = COB_SCREEN_UPPER;
        let a = cob_screen_get_all(&mut st, b'a' as i32, None);
        assert_eq!(a, Action::Stored(b'A'));
    }

    #[test]
    fn get_all_numeric_rejects_alpha() {
        let mut st = alpha_state(4);
        st.is_numeric = true;
        st.is_numeric_or_edited = true;
        st.field = vec![b'0'; 4];
        let a = cob_screen_get_all(&mut st, b'A' as i32, None);
        assert_eq!(a, Action::Beep);
    }

    #[test]
    fn get_all_enter_returns_zero() {
        let mut st = alpha_state(4);
        assert_eq!(cob_screen_get_all(&mut st, KEY_ENTER, None), Action::Return(0));
    }

    #[test]
    fn get_all_tab_navigates_wrap() {
        let mut st = alpha_state(4);
        st.curr_index = 2; // last
        assert_eq!(cob_screen_get_all(&mut st, KEY_STAB, None), Action::GoField(0));
        st.curr_index = 0;
        assert_eq!(cob_screen_get_all(&mut st, KEY_STAB, None), Action::GoField(1));
    }

    #[test]
    fn get_all_btab_navigates_wrap() {
        let mut st = alpha_state(4);
        st.curr_index = 0;
        assert_eq!(cob_screen_get_all(&mut st, KEY_BTAB, None), Action::GoField(2));
    }

    #[test]
    fn get_all_left_at_start_ungetches_btab() {
        let mut st = alpha_state(4);
        st.ccolumn = 0; // == scolumn
        assert_eq!(cob_screen_get_all(&mut st, KEY_LEFT, None), Action::Ungetch(KEY_BTAB));
    }

    #[test]
    fn get_all_right_at_end_ungetches_tab() {
        let mut st = alpha_state(4);
        st.ccolumn = st.right_pos;
        assert_eq!(cob_screen_get_all(&mut st, KEY_RIGHT, None), Action::Ungetch(0x09));
    }

    #[test]
    fn get_all_backspace_shifts_left() {
        let mut st = alpha_state(4);
        st.field = b"ABCD".to_vec();
        st.ccolumn = 2; // cursor over 'C'
        let a = cob_screen_get_all(&mut st, KEY_BACKSPACE, None);
        assert_eq!(a, Action::Continue);
        // 'B' deleted: ABCD -> ACD + blank
        assert_eq!(&st.field, b"ACD ");
        assert_eq!(st.ccolumn, 1);
    }

    #[test]
    fn get_all_backspace_at_start_beeps() {
        let mut st = alpha_state(4);
        st.ccolumn = 0;
        assert_eq!(cob_screen_get_all(&mut st, KEY_BACKSPACE, None), Action::Beep);
    }

    #[test]
    fn get_all_delete_shifts() {
        let mut st = alpha_state(5);
        st.field = b"ABCDE".to_vec();
        st.ccolumn = 1; // delete 'B'
        let a = cob_screen_get_all(&mut st, KEY_DC, None);
        assert_eq!(a, Action::Continue);
        assert_eq!(&st.field, b"ACDE ");
    }

    #[test]
    fn get_all_insert_toggle() {
        let mut st = alpha_state(4);
        assert_eq!(cob_screen_get_all(&mut st, KEY_IC, None), Action::ToggleInsert);
    }

    #[test]
    fn get_all_auto_skip_last_field_returns() {
        let mut st = alpha_state(2);
        st.attr = COB_SCREEN_AUTO;
        st.curr_index = 2; // last (totl_index 3)
        st.ccolumn = st.right_pos;
        let a = cob_screen_get_all(&mut st, b'Z' as i32, None);
        assert_eq!(a, Action::Return(0));
        assert_eq!(st.field[st.right_pos as usize], b'Z');
    }

    #[test]
    fn get_all_auto_skip_ungetches_tab() {
        let mut st = alpha_state(2);
        st.attr = COB_SCREEN_AUTO;
        st.curr_index = 0;
        st.ccolumn = st.right_pos;
        assert_eq!(cob_screen_get_all(&mut st, b'Z' as i32, None), Action::Ungetch(0x09));
    }

    #[test]
    fn field_accept_arrow_returns_codes() {
        let mut st = alpha_state(4);
        assert_eq!(field_accept(&mut st, KEY_UP, false), Action::Return(2003));
        assert_eq!(field_accept(&mut st, KEY_DOWN, false), Action::Return(2004));
        assert_eq!(field_accept(&mut st, KEY_STAB, false), Action::Return(2007));
        assert_eq!(field_accept(&mut st, KEY_BTAB, false), Action::Return(2008));
    }

    #[test]
    fn field_accept_omitted_positioning_codes() {
        let mut st = alpha_state(4);
        assert_eq!(field_accept(&mut st, KEY_LEFT, true), Action::Return(2009));
        assert_eq!(field_accept(&mut st, KEY_RIGHT, true), Action::Return(2010));
        assert_eq!(field_accept(&mut st, KEY_BACKSPACE, true), Action::Return(2013));
        assert_eq!(field_accept(&mut st, KEY_HOME, true), Action::Return(2014));
    }

    #[test]
    fn field_accept_stores_char() {
        let mut st = alpha_state(4);
        assert_eq!(field_accept(&mut st, b'Q' as i32, false), Action::Stored(b'Q'));
        assert_eq!(st.field[0], b'Q');
    }

    #[test]
    fn field_accept_from_curpos_delegates() {
        let mut st = alpha_state(4);
        assert_eq!(field_accept_from_curpos(&mut st, KEY_ENTER, false), Action::Return(0));
    }

    #[test]
    fn function_keys_return_codes() {
        let mut st = alpha_state(4);
        assert_eq!(cob_screen_get_all(&mut st, key_f(1), None), Action::Return(1001));
        assert_eq!(field_accept(&mut st, key_f(5), false), Action::Return(1005));
    }

    #[test]
    fn entry_points_build_loop_state() {
        let f = CobField::alpha(6);
        let (pos, st) = cob_field_accept(&f, 0, Some(2), Some(10), None);
        assert_eq!(pos, AcceptPos { line: 2, column: 10 });
        assert_eq!(st.scolumn, 9);
        assert_eq!(st.right_pos, 9 + 6 - 1);
        assert_eq!(st.field.len(), 6);

        let (_p2, st2) = cob_accept_field(&f, COB_SCREEN_UPDATE, None, None, None);
        // UPDATE pre-loads the field (spaces here)
        assert_eq!(st2.field, f.data);
    }

    #[test]
    fn cob_screen_accept_entry() {
        let (nodes, root) = two_field_tree();
        let (pos, setup) = cob_screen_accept(&nodes, root, Some(1), Some(1));
        assert_eq!(pos, AcceptPos { line: 1, column: 1 });
        assert!(setup.error.is_none());
        assert_eq!(setup.inp.len(), 2);
    }

    #[test]
    fn get_crt3_status_codes() {
        assert_eq!(get_crt3_status(0), [b'0', b'0', 0]);
        assert_eq!(get_crt3_status(2005), [b'1', 0, 0]);
        assert_eq!(get_crt3_status(8000), [b'9', 0, 0]);
        assert_eq!(get_crt3_status(8001), [b'9', 1, 0]);
        assert_eq!(get_crt3_status(1001), [b'1', 1, 0]);
        assert_eq!(get_crt3_status(2005 + 5), [b'2', 10, 0]); // 2010 -> exception 10
    }

    #[test]
    fn accept_escape_key_two_byte_crt3() {
        let mut f = CobField::alpha(2);
        let r = cob_accept_escape_key(&mut f, 2005);
        assert_eq!(r, EscapeKeyResult::Crt3([b'1', 0]));
        assert_eq!(f.data[0], b'1');
    }

    #[test]
    fn accept_escape_key_set_int() {
        let mut f = CobField::numeric(4);
        let r = cob_accept_escape_key(&mut f, 2005);
        assert_eq!(r, EscapeKeyResult::SetInt(2005));
    }
}
