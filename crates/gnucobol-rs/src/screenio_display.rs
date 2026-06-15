//! Port of the `DISPLAY`-statement drivers of GnuCOBOL 3.2 `libcob/screenio.c`.
//!
//! These are the functions that turn a `DISPLAY` of a `SCREEN SECTION` item or a stand-alone
//! `DISPLAY ... AT/WITH` field into positioned terminal writes: `screen_display`, `field_display`,
//! `field_display_at_curpos`, `cob_screen_display`, `cob_field_display`, `cob_display_field`, plus the
//! plain-text helpers `cob_display_text`, `cob_display_formatted_text`, and the retrieval `cob_get_text`.
//!
//! ## Boundary (read this)
//!
//! The leaf operations that actually paint glyphs -- ncurses `move`/`addstr`/`addnstr`/`addch`/`scrl`/
//! `refresh` -- are a DOCUMENTED BOUNDARY (the literal terminal bytes ncurses emits live in
//! `screenio.rs`). This module ports the *pure* layer above that leaf: the dispatch (entry point ->
//! worker), the position decisions (`extract_line_and_col_vals`, `cob_move_cursor`), the SIZE IS /
//! figurative-constant-WITH-SIZE / GRAPHICS / trailing-blank field-byte handling of `field_display`, and
//! the `EMULATE_NL`, `display_cursor_y`/`display_cursor_x` state. Instead of calling ncurses, each ported
//! function records what *would* be drawn into an explicit [`Screen`] state and returns a [`DisplayAction`]
//! (the positioned field bytes) -- so the decision logic is testable without a terminal.
//!
//! Every function carries the exact C name of the function it ports.
#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// COB_SCREEN_* flags (common.h) -- only the subset the DISPLAY drivers read.
// ---------------------------------------------------------------------------

/// `COB_SCREEN_SCROLL_DOWN` (common.h `1 << 22`): with a `SCROLL` clause, scroll down instead of up.
pub const COB_SCREEN_SCROLL_DOWN: u64 = 1 << 22;
/// `COB_SCREEN_NO_DISP` (common.h `1 << 26`): `SECURE`/`NO-DISPLAY` -- the field is not painted.
pub const COB_SCREEN_NO_DISP: u64 = 1 << 26;
/// `COB_SCREEN_EMULATE_NL` (common.h `1 << 27`): after the field, emulate a newline (advance one line, col 0).
pub const COB_SCREEN_EMULATE_NL: u64 = 1 << 27;
/// `COB_SCREEN_GRAPHICS` (common.h `1 << 31`): draw line-graphics translation of the bytes.
pub const COB_SCREEN_GRAPHICS: u64 = 1 << 31;
/// `COB_SCREEN_INITIAL` (common.h `1 << 23`).
pub const COB_SCREEN_INITIAL: u64 = 1 << 23;

/// `COB_TYPE_ALPHANUMERIC` (common.h `0x21`).
pub const COB_TYPE_ALPHANUMERIC: u32 = 0x21;
/// `COB_TYPE_ALPHANUMERIC_ALL` (common.h `0x22`) -- a figurative-constant `ALL` field.
pub const COB_TYPE_ALPHANUMERIC_ALL: u32 = 0x22;

/// `COB_CH_SP` -- the screen space fill byte (`' '`, `0x20`).
pub const COB_CH_SP: u8 = b' ';

/// Default ncurses `LINES` (terminal height) used by `EMULATE_NL` wrap. The real value comes from the
/// terminal at init; 24 is the admitted default (matches `screenio.rs` `SCREEN_ROWS`).
pub const LINES: i32 = 24;

/// `COB_ACCEPT_STATUS` -- the keyboard status `cob_get_text`/the ACCEPT helpers return. Modeled as the
/// "no key / normal" status; the real value lives in a runtime global the leaf ACCEPT path sets.
pub const COB_ACCEPT_STATUS: i32 = 0;

// ---------------------------------------------------------------------------
// Minimal local models of cob_field / cob_screen (no dependency on other modules).
// ---------------------------------------------------------------------------

/// Minimal model of `cob_field_attr` -- only the `type` the DISPLAY path inspects (to detect a
/// figurative-constant `ALL` field for the WITH SIZE repeat).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobFieldAttr {
    /// `attr->type` (e.g. [`COB_TYPE_ALPHANUMERIC`], [`COB_TYPE_ALPHANUMERIC_ALL`]).
    pub r#type: u32,
}

impl CobFieldAttr {
    /// `COB_ATTR_INIT (COB_TYPE_ALPHANUMERIC, 0, 0, 0, NULL)` -- the plain-text attr the helpers build.
    pub fn alphanumeric() -> Self {
        CobFieldAttr { r#type: COB_TYPE_ALPHANUMERIC }
    }
}

/// Minimal model of `cob_field` for the DISPLAY path: the data bytes and the attribute. `f->size` is the
/// byte length and `f->data` is `data`; the rest of the runtime field is not consulted here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobField {
    /// `f->data` -- the field's bytes (length is `f->size`).
    pub data: Vec<u8>,
    /// `f->attr`.
    pub attr: CobFieldAttr,
}

impl CobField {
    /// `COB_FIELD_INIT (len, data, &attr)` with an alphanumeric attr -- the field the text helpers build.
    pub fn new_alpha(data: Vec<u8>) -> Self {
        CobField { data, attr: CobFieldAttr::alphanumeric() }
    }
    /// `f->size` -- the field byte length.
    pub fn size(&self) -> i32 {
        self.data.len() as i32
    }
}

/// `COB_SCREEN_TYPE_*` (common.h).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenType {
    /// `COB_SCREEN_TYPE_GROUP` (0).
    Group,
    /// `COB_SCREEN_TYPE_FIELD` (1).
    Field,
    /// `COB_SCREEN_TYPE_VALUE` (2).
    Value,
    /// `COB_SCREEN_TYPE_ATTRIBUTE` (3).
    Attribute,
}

/// Minimal model of `cob_screen`: a tree node with `child`/`next` links, an optional field/value, and the
/// per-item `LINE`/`COLUMN` and `attr` flags. The runtime uses pointers; here children are owned `Vec`s so
/// the iteration is a pure tree walk.
#[derive(Debug, Clone)]
pub struct CobScreen {
    /// `s->type`.
    pub r#type: ScreenType,
    /// `s->field` (for `COB_SCREEN_TYPE_FIELD`).
    pub field: Option<CobField>,
    /// `s->value` (for `COB_SCREEN_TYPE_VALUE`).
    pub value: Option<CobField>,
    /// `s->line` -- the LINE clause value (already resolved to an int here for the pure model).
    pub line: Option<i32>,
    /// `s->column` -- the COLUMN clause value.
    pub column: Option<i32>,
    /// `s->attr` -- the `COB_SCREEN_*` flag bits for this item.
    pub attr: u64,
    /// `s->child` -- the children of a group (owned; the runtime threads a `child`/`next` list).
    pub children: Vec<CobScreen>,
}

impl CobScreen {
    /// A leaf `COB_SCREEN_TYPE_FIELD` item.
    pub fn field_item(field: CobField, line: i32, column: i32, attr: u64) -> Self {
        CobScreen {
            r#type: ScreenType::Field,
            field: Some(field),
            value: None,
            line: Some(line),
            column: Some(column),
            attr,
            children: Vec::new(),
        }
    }
    /// A `COB_SCREEN_TYPE_GROUP` item with children.
    pub fn group(children: Vec<CobScreen>) -> Self {
        CobScreen {
            r#type: ScreenType::Group,
            field: None,
            value: None,
            line: None,
            column: None,
            attr: 0,
            children,
        }
    }
}

// ---------------------------------------------------------------------------
// Explicit display state (replaces the screenio.c file-scope statics).
// ---------------------------------------------------------------------------

/// The explicit display state that `screenio.c` keeps in file-scope statics. Modeling it as a struct (never
/// a static) lets the ported functions stay pure: each takes `&mut Screen`, mutates the cursor / records the
/// drawn actions, and returns the same [`DisplayAction`]s the leaf `addstr` calls would have painted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Screen {
    /// `origin_y` -- the LINE the current statement started at.
    pub origin_y: i32,
    /// `origin_x` -- the COLUMN the current statement started at.
    pub origin_x: i32,
    /// The cursor row (the ncurses `move` target / `getyx` `cline`), 0-based as the runtime keeps it.
    pub cursor_y: i32,
    /// The cursor column (the ncurses `move` target / `getyx` `ccolumn`), 0-based.
    pub cursor_x: i32,
    /// `display_cursor_y` -- where the last `field_display` left the logical display cursor (row).
    pub display_cursor_y: i32,
    /// `display_cursor_x` -- the logical display cursor column (`scolumn + size_display`).
    pub display_cursor_x: i32,
    /// `pending_accept` -- set when a successful `cob_move_cursor` precedes a pending DISPLAY/ACCEPT.
    pub pending_accept: bool,
    /// The accumulated draw actions (what the leaf `addstr`/`addch`/`addnstr` would have painted), in order.
    pub actions: Vec<DisplayAction>,
}

/// `ERR` -- the ncurses error sentinel that `move`/`cob_move_cursor` return when the target is off-screen.
pub const ERR: i32 = -1;
/// `OK` -- the ncurses success sentinel.
pub const OK: i32 = 0;

/// One paint the leaf would perform: positioned bytes written to the screen, plus whether the field was
/// suppressed (`COB_SCREEN_NO_DISP`). This is the testable stand-in for the `move(y,x); addnstr(...)` leaf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayAction {
    /// The row the bytes were drawn at (`sline`, the `cob_move_cursor` target).
    pub line: i32,
    /// The column the bytes were drawn at (`scolumn`).
    pub column: i32,
    /// The exact bytes that would be painted at `(line, column)` (after SIZE/repeat/graphics handling).
    pub bytes: Vec<u8>,
    /// True when the item was `NO_DISP` and nothing was painted (the action records position only).
    pub suppressed: bool,
}

/// `cob_min_int (a, b)` (screenio.c helper): the smaller of two ints.
fn cob_min_int(a: i32, b: i32) -> i32 {
    if a < b {
        a
    } else {
        b
    }
}

impl Screen {
    /// `cob_move_cursor (line, column)` (screenio.c ~338). Moves the ncurses cursor to `(line, column)`,
    /// returning `OK`/`ERR`. Here the cursor is the explicit state; an off-screen target (negative, or
    /// beyond [`LINES`]) yields `ERR` exactly as ncurses `move` does -- the only branch `field_display`
    /// reads (it sets `pending_accept` on success). Pure and panic-free.
    pub fn cob_move_cursor(&mut self, line: i32, column: i32) -> i32 {
        if line < 0 || column < 0 || line >= LINES {
            return ERR;
        }
        self.cursor_y = line;
        self.cursor_x = column;
        OK
    }
}

// ---------------------------------------------------------------------------
// extract_line_and_col_vals (screenio.c ~3095) -- pure position resolution.
// ---------------------------------------------------------------------------

/// `enum screen_statement` (screenio.c): which statement's "last ended" position to fall back on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenStatement {
    /// `DISPLAY_STATEMENT`.
    Display,
    /// `ACCEPT_STATEMENT`.
    Accept,
}

/// Where the last statement ended, for the LINE/COL 0 fallback (`line_where_last_stmt_ended` /
/// `col_where_last_stmt_ended`). Modeled explicitly instead of the runtime's per-statement statics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LastStmtPos {
    /// The line the last statement of this kind ended on (0-based, as the runtime stores).
    pub line: i32,
    /// The column the last statement of this kind ended on.
    pub col: i32,
}

/// `extract_line_and_col_vals (line, column, stmt, zero_line_col_allowed, &sline, &scolumn)`
/// (screenio.c ~3095). Resolves the COBOL 1-based `LINE`/`COLUMN` clause values into the 0-based ncurses
/// `(sline, scolumn)`. Modeled with the clauses pre-resolved to optional ints (`None` == the C `NULL`
/// field). Returns `(sline, scolumn)`. The `get_line_and_col_from_field` packed-LINE form and the
/// exception raising are the runtime's; here we reproduce the arithmetic and the `zero_line_col_allowed`
/// last-position fallback. Pure and panic-free.
pub fn extract_line_and_col_vals(
    line: Option<i32>,
    column: Option<i32>,
    zero_line_col_allowed: bool,
    last: LastStmtPos,
) -> (i32, i32) {
    let (cobol_line, cobol_col) = match (line, column) {
        (None, None) => return (0, 0),
        // Only a LINE clause (no AT): column defaults to 1.
        (Some(l), None) => (l, 1),
        // get_line_column when both clauses present (or AT POS).
        (l, Some(c)) => (l.unwrap_or(0), c),
    };

    let mut sline;
    let mut scolumn;
    if cobol_line == 0 {
        if cobol_col == 0 {
            if zero_line_col_allowed {
                sline = last.line;
                scolumn = last.col;
            } else {
                sline = 0;
                scolumn = 0;
            }
        } else {
            if zero_line_col_allowed {
                sline = last.line + 1;
            } else {
                sline = 0;
            }
            scolumn = cobol_col - 1;
        }
    } else if cobol_col == 0 {
        sline = cobol_line - 1;
        if zero_line_col_allowed {
            scolumn = last.col;
        } else {
            scolumn = 0;
        }
    } else {
        sline = cobol_line - 1;
        scolumn = cobol_col - 1;
    }
    // silence the "assigned but possibly overwritten" lint cleanly:
    let _ = &mut sline;
    let _ = &mut scolumn;
    (sline, scolumn)
}

// ---------------------------------------------------------------------------
// screen_display (screenio.c ~3171) + the SCREEN SECTION iteration leaf it drives.
// ---------------------------------------------------------------------------

/// `cob_screen_iterate (s)` (screenio.c) -- here the pure tree walk a `screen_display` drives: visit each
/// `COB_SCREEN_TYPE_FIELD`/`VALUE` leaf in `child`/`next` order, painting its field bytes at its
/// `LINE`/`COLUMN` via [`field_display`]. Groups recurse. Items without a field/value paint nothing.
fn cob_screen_iterate(scr: &Screen, s: &CobScreen, line: i32, column: i32) -> Vec<DisplayAction> {
    let mut out = Vec::new();
    match s.r#type {
        ScreenType::Group => {
            for child in &s.children {
                out.extend(cob_screen_iterate(scr, child, line, column));
            }
        }
        ScreenType::Field | ScreenType::Value => {
            let f = s.field.as_ref().or(s.value.as_ref());
            if let Some(f) = f {
                let sline = s.line.map(|l| l - 1).unwrap_or(line);
                let scolumn = s.column.map(|c| c - 1).unwrap_or(column);
                // A fresh sub-state so the worker's cursor bookkeeping does not disturb the caller's.
                let mut local = scr.clone();
                if let Some(action) = field_display(&mut local, f, s.attr, sline, scolumn, None) {
                    out.push(action);
                }
            }
        }
        ScreenType::Attribute => {}
    }
    out
}

/// `screen_display (cob_screen *s, const int line, const int column)` (screenio.c ~3171). Records the
/// statement origin, moves the cursor to `(line, column)` (setting `pending_accept` on success), then
/// iterates the screen tree painting each leaf field. Returns the painted [`DisplayAction`]s (the
/// `refresh()` is the boundary). Pure over the explicit [`Screen`]; panic-free.
pub fn screen_display(scr: &mut Screen, s: &CobScreen, line: i32, column: i32) -> Vec<DisplayAction> {
    scr.origin_y = line;
    scr.origin_x = column;

    let status = scr.cob_move_cursor(line, column);
    if status != ERR {
        scr.pending_accept = true;
    }
    let actions = cob_screen_iterate(scr, s, line, column);
    scr.actions.extend(actions.iter().cloned());
    // refresh ();  <-- ncurses leaf (boundary)
    actions
}

// ---------------------------------------------------------------------------
// field_display (screenio.c ~3292) -- the big one.
// ---------------------------------------------------------------------------

/// `field_display (f, fattr, line, column, fgc, bgc, fscroll, size_is, control, color)`
/// (screenio.c ~3292). Displays one field at `(line, column)` honouring `SIZE IS`, the
/// figurative-constant-`ALL` WITH SIZE repeat, `GRAPHICS`, trailing-space fill when SIZE > field, the
/// `NO_DISP` suppression, and the `EMULATE_NL` post-advance; updates `display_cursor_y`/`display_cursor_x`.
///
/// Models the optional cob_field clauses as resolved ints: `fscroll` (`Some(n)` == a `SCROLL` of `n`
/// lines, negated under `COB_SCREEN_SCROLL_DOWN`) and `size_is` (`Some(n)` == `SIZE IS n`, with `SIZE 0`
/// ignored exactly as the C does). `fgc`/`bgc`/`control`/`color` feed the attribute leaf
/// (`cob_screen_attr`) which is the color boundary; the only attribute bit the field-byte logic reads is
/// `COB_SCREEN_NO_DISP`. The `scrl`/`addnstr`/`refresh` leaves are the boundary -- the painted bytes are
/// returned as a [`DisplayAction`]. Pure; panic-free.
#[allow(clippy::too_many_arguments)]
pub fn field_display(
    scr: &mut Screen,
    f: &CobField,
    fattr: u64,
    line: i32,
    column: i32,
    size_is: Option<i32>,
) -> Option<DisplayAction> {
    field_display_full(scr, f, fattr, line, column, None, size_is)
}

/// The full `field_display` including the `SCROLL` clause. `field_display` is the common entry (no scroll);
/// this carries the extra `fscroll` argument so both the no-scroll callers and the scroll-capable ones map
/// to the exact C body.
#[allow(clippy::too_many_arguments)]
pub fn field_display_full(
    scr: &mut Screen,
    f: &CobField,
    mut fattr: u64,
    line: i32,
    column: i32,
    fscroll: Option<i32>,
    size_is: Option<i32>,
) -> Option<DisplayAction> {
    // origin_y = origin_x = 0;
    scr.origin_y = 0;
    scr.origin_x = 0;

    let fsize = f.size();
    // SIZE IS handling, with SIZE ZERO ignored.
    let size_display: i32 = if let Some(s) = size_is {
        if s == 0 {
            fsize
        } else {
            s
        }
    } else if fattr & COB_SCREEN_NO_DISP != 0 {
        0
    } else {
        fsize
    };

    // SCROLL clause -> scrl() leaf (boundary). We honour the sign decision but the scroll itself is the leaf.
    if let Some(mut sl) = fscroll {
        if fattr & COB_SCREEN_SCROLL_DOWN != 0 {
            sl = -sl;
        }
        let _ = sl; // scrollok/scrl/refresh -- boundary
    }

    let sline = line;
    let scolumn = column;
    let status = scr.cob_move_cursor(sline, scolumn);
    if status != ERR {
        scr.pending_accept = true;
    }

    // cob_screen_attr (fgc, bgc, fattr, control, color, DISPLAY_STATEMENT) -- color leaf (boundary).
    // It returns the (possibly adjusted) attribute flags; the field-byte logic below only reads NO_DISP,
    // which it does not clear, so we leave fattr as-is.
    fattr = cob_screen_attr(fattr);

    // Build the bytes the leaf addnstr/addnch calls would paint.
    let mut bytes: Vec<u8> = Vec::new();
    let suppressed = fattr & COB_SCREEN_NO_DISP != 0;
    if !suppressed {
        if size_is.is_some() && f.attr.r#type == COB_TYPE_ALPHANUMERIC_ALL {
            // figurative constant WITH SIZE repeats the literal.
            if fsize == 1 {
                let fig_const = f.data.first().copied().unwrap_or(COB_CH_SP);
                // cob_addnch (size_display, fig_const)
                for _ in 0..size_display.max(0) {
                    bytes.push(fig_const);
                }
            } else if fsize > 0 {
                // size_display / fsize whole copies, then a partial of size_display % fsize.
                let copies = size_display / fsize;
                for _ in 0..copies.max(0) {
                    bytes.extend_from_slice(&f.data[..fsize as usize]);
                }
                let rem = (size_display % fsize).max(0) as usize;
                bytes.extend_from_slice(&f.data[..rem.min(f.data.len())]);
            }
        } else {
            let n = cob_min_int(size_display, fsize).max(0) as usize;
            if fattr & COB_SCREEN_GRAPHICS != 0 {
                // cob_addnstr_graph -- the line-graphics translation is the leaf; the byte slice is the same.
                bytes.extend_from_slice(&f.data[..n.min(f.data.len())]);
            } else {
                bytes.extend_from_slice(&f.data[..n.min(f.data.len())]);
            }
            if size_display > fsize {
                // WITH SIZE larger than field: trailing spaces.
                for _ in 0..(size_display - fsize) {
                    bytes.push(COB_CH_SP);
                }
            }
        }
    }

    scr.display_cursor_y = sline;
    scr.display_cursor_x = scolumn + size_display;

    // EMULATE_NL: advance one line (wrapping at LINES), move to column 0.
    if fattr & COB_SCREEN_EMULATE_NL != 0 {
        let mut nl = sline + 1;
        if nl >= LINES {
            nl = 0;
        }
        scr.cob_move_cursor(nl, 0);
    }
    // refresh ();  <-- boundary

    let action = DisplayAction {
        line: sline,
        column: scolumn,
        bytes,
        suppressed,
    };
    scr.actions.push(action.clone());
    Some(action)
}

/// `cob_screen_attr (fgc, bgc, attr, control, color, stmt)` (screenio.c ~895) -- the color/attribute leaf.
/// The full body resolves color pairs and emits ncurses `attrset`/`color_set` (the BOUNDARY). For the
/// DISPLAY field-byte path the only consumed result is the returned flag word, which it does not alter for
/// the bits this module reads, so the pure stand-in is the identity on the flags.
fn cob_screen_attr(attr: u64) -> u64 {
    attr
}

// ---------------------------------------------------------------------------
// field_display_at_curpos (screenio.c ~4122)
// ---------------------------------------------------------------------------

/// `field_display_at_curpos (f, fgc, bgc, fscroll, size_is, fattr)` (screenio.c ~4122). Reads the current
/// cursor (`getyx`) and displays the field there via [`field_display_full`]. The plain-text helpers
/// (`cob_display_text` / `cob_display_formatted_text`) funnel through here. Pure; panic-free.
pub fn field_display_at_curpos(
    scr: &mut Screen,
    f: &CobField,
    fscroll: Option<i32>,
    size_is: Option<i32>,
    fattr: u64,
) -> Option<DisplayAction> {
    // getyx (stdscr, cline, ccolumn);
    let cline = scr.cursor_y;
    let ccolumn = scr.cursor_x;
    field_display_full(scr, f, fattr, cline, ccolumn, fscroll, size_is)
}

// ---------------------------------------------------------------------------
// Entry points.
// ---------------------------------------------------------------------------

/// `cob_screen_display (cob_screen *s, cob_field *line, cob_field *column, zero_line_col_allowed)`
/// (screenio.c ~4140) -- the `DISPLAY scr-name` entry. Extracts `(sline, scolumn)` from the LINE/COLUMN
/// clauses and calls [`screen_display`]. The `init_cob_screen_if_needed` leaf is the boundary.
pub fn cob_screen_display(
    scr: &mut Screen,
    s: &CobScreen,
    line: Option<i32>,
    column: Option<i32>,
    zero_line_col_allowed: bool,
    last: LastStmtPos,
) -> Vec<DisplayAction> {
    // init_cob_screen_if_needed ();  <-- boundary
    let (sline, scolumn) = extract_line_and_col_vals(line, column, zero_line_col_allowed, last);
    screen_display(scr, s, sline, scolumn)
}

/// `cob_display_field (cob_field *f, const cob_flags_t fattr, const char *parms, ...)`
/// (screenio.c ~4167) -- the modern `DISPLAY scr-item WITH/AT` entry. The C reads a `parms` format string +
/// varargs into the optional clauses; here the clauses are passed explicitly via [`DisplayParms`]. LINE/COL
/// 0 is always allowed here (`zero_line_col_allowed = 1`), then it dispatches to [`field_display_full`].
pub fn cob_display_field(
    scr: &mut Screen,
    f: &CobField,
    fattr: u64,
    parms: &DisplayParms,
    last: LastStmtPos,
) -> Option<DisplayAction> {
    let zero_line_col_allowed = true;
    // init_cob_screen_if_needed ();  <-- boundary
    let (sline, scolumn) =
        extract_line_and_col_vals(parms.line, parms.column, zero_line_col_allowed, last);
    field_display_full(scr, f, fattr, sline, scolumn, parms.fscroll, parms.size_is)
}

/// The clauses the `cob_display_field` varargs `parms` string carries (`'l'`/`'p'` LINE, `'c'` COLUMN,
/// `'f'`/`'b'` colors, `'s'` SCROLL, `'S'` SIZE, `'C'`/`'L'` control/color). The color/control clauses feed
/// the attribute boundary only, so the pure dispatch needs the position + scroll + size values resolved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DisplayParms {
    /// `'l'`/`'p'` AT LINE / AT POS.
    pub line: Option<i32>,
    /// `'c'` AT COLUMN.
    pub column: Option<i32>,
    /// `'s'` SCROLL UP|DOWN (line count; sign decided by `COB_SCREEN_SCROLL_DOWN`).
    pub fscroll: Option<i32>,
    /// `'S'` SIZE IS.
    pub size_is: Option<i32>,
}

/// `cob_field_display (cob_field *f, cob_field *line, cob_field *column, fgc, bgc, fscroll, size_is, fattr)`
/// (screenio.c ~4321) -- the pre-3.2 compat `DISPLAY scr-item WITH/AT` entry. Same shape as
/// [`cob_display_field`] but the clauses arrive as explicit cob_field pointers (here resolved ints) rather
/// than a varargs string. `zero_line_col_allowed = 1`. Dispatches to [`field_display_full`].
#[allow(clippy::too_many_arguments)]
pub fn cob_field_display(
    scr: &mut Screen,
    f: &CobField,
    line: Option<i32>,
    column: Option<i32>,
    fscroll: Option<i32>,
    size_is: Option<i32>,
    fattr: u64,
    last: LastStmtPos,
) -> Option<DisplayAction> {
    let zero_line_col_allowed = true;
    // init_cob_screen_if_needed ();  <-- boundary
    let (sline, scolumn) = extract_line_and_col_vals(line, column, zero_line_col_allowed, last);
    field_display_full(scr, f, fattr, sline, scolumn, fscroll, size_is)
}

// ---------------------------------------------------------------------------
// Plain-text display + retrieval helpers.
// ---------------------------------------------------------------------------

/// `cob_display_text (const char *text)` (screenio.c ~4394). Displays a C string at the current cursor with
/// no auto-newline and no attributes (`fattr == 0`). An empty string is a no-op (returns 0). Builds an
/// alphanumeric field of `strlen(text)` bytes and calls [`field_display_at_curpos`]. Returns `0`.
pub fn cob_display_text(scr: &mut Screen, text: &[u8]) -> i32 {
    // init_cob_screen_if_needed ();  <-- boundary
    // text[0] == 0  ->  empty C string (no leading NUL): nothing to display.
    if text.is_empty() || text[0] == 0 {
        return 0;
    }
    // COB_FIELD_INIT (strlen(text), text, &attr) -- strlen stops at the first NUL.
    let len = text.iter().position(|&b| b == 0).unwrap_or(text.len());
    let field = CobField::new_alpha(text[..len].to_vec());
    field_display_at_curpos(scr, &field, None, None, 0);
    0
}

/// `cob_display_formatted_text (const char *fmt, ...)` (screenio.c ~4462). Formats `fmt` + varargs into a
/// buffer then displays it at the current cursor with no auto-newline. The `vsnprintf` is the caller's; here
/// the already-formatted bytes arrive in `formatted`. A negative format result (`size < 0`) returns `-1`; an
/// empty result returns `0`. The displayed length is capped at `COB_NORMAL_MAX` (modeled via `max_len`).
pub fn cob_display_formatted_text(scr: &mut Screen, formatted: &[u8], max_len: i32) -> i32 {
    // init_cob_screen_if_needed ();  <-- boundary
    // size = vsnprintf(...) is done by the caller; we receive the bytes. (A real negative size is the
    // caller's to report; here a non-empty slice is a successful format.)
    if formatted.is_empty() || formatted[0] == 0 {
        return 0;
    }
    let len = formatted.iter().position(|&b| b == 0).unwrap_or(formatted.len());
    let size = cob_min_int(len as i32, max_len.max(0)).max(0) as usize;
    let field = CobField::new_alpha(formatted[..size].to_vec());
    field_display_at_curpos(scr, &field, None, None, 0);
    0
}

/// `cob_get_text (char *text, int size)` (screenio.c ~4442). The retrieval counterpart: an ACCEPT of up to
/// `size` bytes from the keyboard into `text`, returning the keyboard status (`COB_ACCEPT_STATUS`). The
/// keyboard read is the ACCEPT leaf (BOUNDARY); the pure part is the dispatch -- `size > 0` accepts into the
/// caller's buffer, `size <= 0` is an `ACCEPT OMITTED`-style read with no field. Returns the status.
pub fn cob_get_text(text: &mut [u8], size: i32) -> i32 {
    // init_cob_screen_if_needed ();  <-- boundary
    if size > 0 {
        // COB_FIELD_INIT (size, text, &attr); field_accept_from_curpos(...) -- keyboard leaf fills `text`.
        // With no keystrokes available in the pure model, the buffer is left as-is (spaces under the runtime).
        let n = cob_min_int(size, text.len() as i32).max(0) as usize;
        let _ = &text[..n];
    } else {
        // field_accept (NULL, ...) -- ACCEPT OMITTED: no receiving field.
    }
    COB_ACCEPT_STATUS
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- extract_line_and_col_vals ---------------------------------------

    #[test]
    fn extract_both_null_is_zero_zero() {
        assert_eq!(
            extract_line_and_col_vals(None, None, true, LastStmtPos::default()),
            (0, 0)
        );
    }

    #[test]
    fn extract_line_only_defaults_col_to_one() {
        // LINE 5 only -> (sline=4, scolumn=0).
        assert_eq!(
            extract_line_and_col_vals(Some(5), None, false, LastStmtPos::default()),
            (4, 0)
        );
    }

    #[test]
    fn extract_one_based_to_zero_based() {
        // LINE 3 COL 7 -> (2, 6).
        assert_eq!(
            extract_line_and_col_vals(Some(3), Some(7), false, LastStmtPos::default()),
            (2, 6)
        );
    }

    #[test]
    fn extract_zero_line_zero_col_uses_last_when_allowed() {
        let last = LastStmtPos { line: 9, col: 4 };
        assert_eq!(extract_line_and_col_vals(Some(0), Some(0), true, last), (9, 4));
        // not allowed -> (0,0)
        assert_eq!(extract_line_and_col_vals(Some(0), Some(0), false, last), (0, 0));
    }

    #[test]
    fn extract_zero_line_nonzero_col() {
        let last = LastStmtPos { line: 9, col: 4 };
        // LINE 0 COL 5, allowed -> sline = last+1 = 10, scolumn = 5-1 = 4.
        assert_eq!(extract_line_and_col_vals(Some(0), Some(5), true, last), (10, 4));
        // not allowed -> sline=0, scolumn=4.
        assert_eq!(extract_line_and_col_vals(Some(0), Some(5), false, last), (0, 4));
    }

    #[test]
    fn extract_nonzero_line_zero_col() {
        let last = LastStmtPos { line: 9, col: 4 };
        // LINE 6 COL 0, allowed -> sline=5, scolumn=last.col=4.
        assert_eq!(extract_line_and_col_vals(Some(6), Some(0), true, last), (5, 4));
    }

    // --- cob_move_cursor / ERR -------------------------------------------

    #[test]
    fn move_cursor_off_screen_is_err() {
        let mut scr = Screen::default();
        assert_eq!(scr.cob_move_cursor(-1, 0), ERR);
        assert_eq!(scr.cob_move_cursor(0, -1), ERR);
        assert_eq!(scr.cob_move_cursor(LINES, 0), ERR);
        assert_eq!(scr.cob_move_cursor(2, 3), OK);
        assert_eq!((scr.cursor_y, scr.cursor_x), (2, 3));
    }

    // --- field_display: plain field --------------------------------------

    #[test]
    fn field_display_plain_paints_field_bytes() {
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"HELLO".to_vec());
        let a = field_display(&mut scr, &f, 0, 2, 3, None).unwrap();
        assert_eq!(a.line, 2);
        assert_eq!(a.column, 3);
        assert_eq!(a.bytes, b"HELLO");
        assert!(!a.suppressed);
        // display cursor advanced by size_display (= fsize = 5).
        assert_eq!((scr.display_cursor_y, scr.display_cursor_x), (2, 8));
        assert!(scr.pending_accept);
    }

    #[test]
    fn field_display_no_disp_suppresses() {
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"SECRET".to_vec());
        let a = field_display(&mut scr, &f, COB_SCREEN_NO_DISP, 1, 1, None).unwrap();
        assert!(a.suppressed);
        assert!(a.bytes.is_empty());
        // size_display is 0 for NO_DISP without SIZE IS -> display cursor x unchanged from column.
        assert_eq!(scr.display_cursor_x, 1);
    }

    #[test]
    fn field_display_size_is_truncates() {
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"ABCDEFGH".to_vec());
        // SIZE IS 3 -> only first 3 bytes.
        let a = field_display(&mut scr, &f, 0, 1, 1, Some(3)).unwrap();
        assert_eq!(a.bytes, b"ABC");
        assert_eq!(scr.display_cursor_x, 1 + 3);
    }

    #[test]
    fn field_display_size_zero_ignored() {
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"ABCD".to_vec());
        // SIZE IS 0 -> ignored, falls back to fsize.
        let a = field_display(&mut scr, &f, 0, 1, 1, Some(0)).unwrap();
        assert_eq!(a.bytes, b"ABCD");
    }

    #[test]
    fn field_display_size_larger_than_field_pads_spaces() {
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"AB".to_vec());
        // SIZE IS 5 > fsize 2 -> "AB" + 3 trailing spaces.
        let a = field_display(&mut scr, &f, 0, 1, 1, Some(5)).unwrap();
        assert_eq!(a.bytes, b"AB   ");
        assert_eq!(scr.display_cursor_x, 1 + 5);
    }

    #[test]
    fn field_display_figurative_all_single_char_repeats() {
        let mut scr = Screen::default();
        let mut f = CobField::new_alpha(b"*".to_vec());
        f.attr.r#type = COB_TYPE_ALPHANUMERIC_ALL;
        // ALL "*" WITH SIZE 4 -> "****".
        let a = field_display(&mut scr, &f, 0, 1, 1, Some(4)).unwrap();
        assert_eq!(a.bytes, b"****");
    }

    #[test]
    fn field_display_figurative_all_multichar_repeats_and_partial() {
        let mut scr = Screen::default();
        let mut f = CobField::new_alpha(b"AB".to_vec());
        f.attr.r#type = COB_TYPE_ALPHANUMERIC_ALL;
        // ALL "AB" (fsize 2) WITH SIZE 5 -> 2 whole copies "ABAB" + 1 partial "A" = "ABABA".
        let a = field_display(&mut scr, &f, 0, 1, 1, Some(5)).unwrap();
        assert_eq!(a.bytes, b"ABABA");
    }

    #[test]
    fn field_display_emulate_nl_advances_line() {
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"X".to_vec());
        field_display(&mut scr, &f, COB_SCREEN_EMULATE_NL, 3, 5, None);
        // after the field, cursor moved to next line, column 0.
        assert_eq!((scr.cursor_y, scr.cursor_x), (4, 0));
    }

    #[test]
    fn field_display_emulate_nl_wraps_at_last_line() {
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"X".to_vec());
        // on the last addressable line (LINES-1), EMULATE_NL wraps to line 0.
        field_display(&mut scr, &f, COB_SCREEN_EMULATE_NL, LINES - 1, 0, None);
        assert_eq!(scr.cursor_y, 0);
    }

    #[test]
    fn field_display_scroll_down_negates() {
        // Just exercises the SCROLL sign branch (the scrl itself is the boundary): no panic, field painted.
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"Y".to_vec());
        let a = field_display_full(&mut scr, &f, COB_SCREEN_SCROLL_DOWN, 1, 1, Some(2), None).unwrap();
        assert_eq!(a.bytes, b"Y");
    }

    // --- field_display_at_curpos -----------------------------------------

    #[test]
    fn field_display_at_curpos_uses_current_cursor() {
        let mut scr = Screen::default();
        scr.cob_move_cursor(4, 7);
        let f = CobField::new_alpha(b"HI".to_vec());
        let a = field_display_at_curpos(&mut scr, &f, None, None, 0).unwrap();
        assert_eq!((a.line, a.column), (4, 7));
        assert_eq!(a.bytes, b"HI");
    }

    // --- screen_display ---------------------------------------------------

    #[test]
    fn screen_display_iterates_leaf_fields() {
        let mut scr = Screen::default();
        let s = CobScreen::group(vec![
            CobScreen::field_item(CobField::new_alpha(b"AA".to_vec()), 1, 1, 0),
            CobScreen::field_item(CobField::new_alpha(b"BB".to_vec()), 2, 5, 0),
        ]);
        let actions = screen_display(&mut scr, &s, 0, 0);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].bytes, b"AA");
        assert_eq!((actions[0].line, actions[0].column), (0, 0));
        assert_eq!(actions[1].bytes, b"BB");
        assert_eq!((actions[1].line, actions[1].column), (1, 4));
        assert!(scr.pending_accept);
    }

    // --- entry points -----------------------------------------------------

    #[test]
    fn cob_screen_display_extracts_position() {
        let mut scr = Screen::default();
        let s = CobScreen::field_item(CobField::new_alpha(b"Z".to_vec()), 0, 0, 0);
        // DISPLAY scr AT LINE 3 COL 4 -> screen origin (2,3).
        cob_screen_display(&mut scr, &s, Some(3), Some(4), true, LastStmtPos::default());
        assert_eq!((scr.origin_y, scr.origin_x), (2, 3));
    }

    #[test]
    fn cob_display_field_dispatches() {
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"DATA".to_vec());
        let parms = DisplayParms { line: Some(2), column: Some(3), fscroll: None, size_is: Some(2) };
        let a = cob_display_field(&mut scr, &f, 0, &parms, LastStmtPos::default()).unwrap();
        assert_eq!((a.line, a.column), (1, 2));
        assert_eq!(a.bytes, b"DA"); // SIZE IS 2
    }

    #[test]
    fn cob_field_display_compat_dispatches() {
        let mut scr = Screen::default();
        let f = CobField::new_alpha(b"WX".to_vec());
        let a = cob_field_display(&mut scr, &f, Some(5), Some(6), None, None, 0, LastStmtPos::default())
            .unwrap();
        assert_eq!((a.line, a.column), (4, 5));
        assert_eq!(a.bytes, b"WX");
    }

    // --- plain text helpers ----------------------------------------------

    #[test]
    fn cob_display_text_empty_is_noop() {
        let mut scr = Screen::default();
        assert_eq!(cob_display_text(&mut scr, b""), 0);
        assert_eq!(cob_display_text(&mut scr, b"\0rest"), 0);
        assert!(scr.actions.is_empty());
    }

    #[test]
    fn cob_display_text_paints_at_cursor() {
        let mut scr = Screen::default();
        scr.cob_move_cursor(1, 2);
        assert_eq!(cob_display_text(&mut scr, b"hi"), 0);
        assert_eq!(scr.actions.len(), 1);
        assert_eq!(scr.actions[0].bytes, b"hi");
        assert_eq!((scr.actions[0].line, scr.actions[0].column), (1, 2));
    }

    #[test]
    fn cob_display_text_stops_at_nul() {
        let mut scr = Screen::default();
        assert_eq!(cob_display_text(&mut scr, b"ab\0cd"), 0);
        assert_eq!(scr.actions[0].bytes, b"ab");
    }

    #[test]
    fn cob_display_formatted_text_caps_at_max() {
        let mut scr = Screen::default();
        // formatted "HELLO", cap 3 -> "HEL".
        assert_eq!(cob_display_formatted_text(&mut scr, b"HELLO", 3), 0);
        assert_eq!(scr.actions[0].bytes, b"HEL");
    }

    #[test]
    fn cob_display_formatted_text_empty_is_noop() {
        let mut scr = Screen::default();
        assert_eq!(cob_display_formatted_text(&mut scr, b"", 80), 0);
        assert!(scr.actions.is_empty());
    }

    #[test]
    fn cob_get_text_returns_status() {
        let mut buf = [b' '; 8];
        assert_eq!(cob_get_text(&mut buf, 8), COB_ACCEPT_STATUS);
        // size <= 0: ACCEPT OMITTED, still returns status.
        assert_eq!(cob_get_text(&mut buf, 0), COB_ACCEPT_STATUS);
    }
}
