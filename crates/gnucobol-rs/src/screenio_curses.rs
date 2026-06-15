//! Port of the **curses-window primitives** of GnuCOBOL 3.2 `libcob/screenio.c` -- the pure logic that
//! drives a modelled ncurses `stdscr`: writing characters into the window at the cursor and advancing it
//! (`cob_addnstr` / `cob_addnstr_graph` / `cob_addch` / `cob_addch_no_trunc_check` / `cob_addnch`), painting
//! a SCREEN SECTION field's data with its attribute (`cob_screen_puts`), moving the cursor
//! (`cob_set_cursor_pos` / `cob_move_to_beg_of_last_line` / `cob_screen_moveyx`), ordering input fields by
//! `(y,x)` (`compare_yx`), finalizing ACCEPT field data (`finalize_field_input` / `finalize_all_fields`),
//! and the insert-mode toggle (`cob_toggle_insert`).
//!
//! ## The boundary (read this)
//!
//! The actual terminal-byte emission -- ncurses `addstr` / `addch` / `move` / `refresh` / `wrefresh` -- is a
//! **documented boundary**: it is terminfo-optimized output reproduced byte-exactly in the sibling
//! [`crate::screenio`] native emitter, not in this module. Here we port the *pure logic* that mutates the
//! modelled [`Window`] (the cell grid + cursor + current attribute + bounds) and **decides what to emit**
//! (which character at which cell, whether truncation raises an exception condition, where the cursor lands).
//! Every function that ends in an `addch`/`addstr`/`move`/`refresh` in C records that intent into the
//! [`Window`] model and returns a [`Decision`] -- the byte stream itself is the boundary.
//!
//! The curses window state is modelled as an explicit [`Window`] struct (GnuCOBOL keeps it in the global
//! `stdscr`; we never use statics). All functions are pure and panic-free.
#![forbid(unsafe_code)]

/// `COB_CH_UL` -- the default prompt char `'_'` (screenio.c line 134).
pub const COB_CH_UL: u8 = 0x5F; // '_'
/// `COB_CH_SP` -- the space char `' '` (screenio.c line 135).
pub const COB_CH_SP: u8 = 0x20; // ' '
/// `COB_CH_AS` -- the SECURE masking char `'*'` (screenio.c line 136).
pub const COB_CH_AS: u8 = 0x2A; // '*'

/// `COB_SCREEN_SECURE` (`common.h` `(cob_flags_t)1 << 16`) -- mask input with `*`.
pub const COB_SCREEN_SECURE: u64 = 1 << 16;
/// `COB_SCREEN_INPUT` (`common.h` `(cob_flags_t)1 << 21`) -- the item is an ACCEPT input field.
pub const COB_SCREEN_INPUT: u64 = 1 << 21;
/// `COB_SCREEN_GRAPHICS` (`common.h` `(cob_flags_t)1 << 31`) -- CONTROL GRAPHICS (box-drawing) field.
pub const COB_SCREEN_GRAPHICS: u64 = 1 << 31;
/// `COB_SCREEN_LINE_PLUS` (`common.h` `1 << 0`) -- LINE PLUS relative move.
pub const COB_SCREEN_LINE_PLUS: u64 = 1 << 0;
/// `COB_SCREEN_LINE_MINUS` (`common.h` `1 << 1`) -- LINE MINUS relative move.
pub const COB_SCREEN_LINE_MINUS: u64 = 1 << 1;
/// `COB_SCREEN_COLUMN_PLUS` (`common.h` `1 << 2`) -- COLUMN PLUS relative move.
pub const COB_SCREEN_COLUMN_PLUS: u64 = 1 << 2;
/// `COB_SCREEN_COLUMN_MINUS` (`common.h` `1 << 3`) -- COLUMN MINUS relative move.
pub const COB_SCREEN_COLUMN_MINUS: u64 = 1 << 3;

/// `enum screen_statement` -- the statement context `cob_screen_puts` is invoked under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenStatement {
    /// `DISPLAY_STATEMENT`.
    Display,
    /// `ACCEPT_STATEMENT`.
    Accept,
}

/// `getexitstatus`-style verdict of a write primitive: whether the modelled write would have raised the
/// `COB_EC_SCREEN_ITEM_TRUNCATED` exception condition (the decision the boundary `addstr`/`addch` would
/// otherwise hide). The character emission itself is recorded into the [`Window`]; this captures the
/// *exception* the trunc-checking variants compute via `raise_ec_on_truncation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Decision {
    /// True when `x + item_size - 1 > max_x` held at write time (`raise_ec_on_truncation`, screenio.c
    /// line 1422) -- the `COB_EC_SCREEN_ITEM_TRUNCATED` exception condition was raised.
    pub truncated: bool,
}

/// One cell of the modelled `stdscr`: a byte plus the attribute that was current when it was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// The character byte written (or `' '` for an untouched cell). Box-drawing graphics from
    /// `cob_addnstr_graph` are recorded as their resolved ACS fallback byte (`+`/`-`/`|`/literal).
    pub ch: u8,
    /// The attribute flags current when this cell was written (the `attrset` state, modelled).
    pub attr: u64,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: COB_CH_SP, attr: 0 }
    }
}

/// The modelled ncurses `stdscr`: a `max_y` x `max_x` cell grid, the cursor `(y, x)`, the current
/// attribute (the `attrset` state), and the insert-mode flag (`COB_INSERT_MODE`). GnuCOBOL keeps this in
/// the curses global; we model it explicitly so every primitive is a pure mutation of this value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// Row count (`getmaxyx` `max_y`): valid rows are `0..max_y`.
    pub max_y: i32,
    /// Column count (`getmaxyx` `max_x`): valid columns are `0..max_x`.
    pub max_x: i32,
    /// Cursor row (`getyx` `y`).
    pub cur_y: i32,
    /// Cursor column (`getyx` `x`).
    pub cur_x: i32,
    /// Current attribute (the `attrset` state) stamped onto each written cell.
    pub attr: u64,
    /// The cell grid, row-major (`max_y * max_x` cells).
    pub cells: Vec<Cell>,
    /// `COB_INSERT_MODE` -- the global insert-mode flag toggled by `cob_toggle_insert`.
    pub insert_mode: i32,
}

impl Window {
    /// A fresh blank window of the given dimensions, cursor at `(0,0)`, no attribute, insert mode off.
    pub fn new(max_y: i32, max_x: i32) -> Self {
        let n = (max_y.max(0) as usize).saturating_mul(max_x.max(0) as usize);
        Window {
            max_y,
            max_x,
            cur_y: 0,
            cur_x: 0,
            attr: 0,
            cells: vec![Cell::default(); n],
            insert_mode: 0,
        }
    }

    /// Index of cell `(y, x)` in the row-major grid, if in bounds.
    fn idx(&self, y: i32, x: i32) -> Option<usize> {
        if y < 0 || x < 0 || y >= self.max_y || x >= self.max_x {
            return None;
        }
        Some(y as usize * self.max_x as usize + x as usize)
    }

    /// `raise_ec_on_truncation (item_size)` (screenio.c line 1411): does writing `item_size` cells from the
    /// current cursor `x` run past `max_x`? Returns the truncation decision; never mutates the window.
    fn raise_ec_on_truncation(&self, item_size: i32) -> bool {
        // C: if (x + item_size - 1 > max_x) -> COB_EC_SCREEN_ITEM_TRUNCATED.
        self.cur_x + item_size - 1 > self.max_x
    }

    /// Write one byte at the cursor with the current attribute and advance the cursor one column (the
    /// curses `addch` cell semantics, without the trunc check). Out-of-bounds writes are dropped (curses
    /// clips), but the cursor still advances, exactly mirroring the boundary `addch`.
    fn put_advance(&mut self, ch: u8) {
        let attr = self.attr;
        if let Some(i) = self.idx(self.cur_y, self.cur_x) {
            self.cells[i] = Cell { ch, attr };
        }
        self.cur_x += 1;
    }
}

/// Resolve the box-drawing fallback byte for a CONTROL GRAPHICS source char, matching the
/// `cob_addnstr_graph` switch's **non-wide** (`addch (ACS_*)` / ASCII fallback) arm. Since the ACS glyph a
/// terminal paints for `ACS_LRCORNER` etc. is a terminfo/`acsc` artifact (the boundary), we record the
/// portable ASCII fallback each `#else` arm spells: corners/tees/plus -> `+`, horizontal -> `-`, vertical
/// -> `|`, everything else -> the literal byte.
fn graph_fallback(c: u8) -> u8 {
    match c {
        b'j' | b'J' | b'k' | b'K' | b'm' | b'M' | b'l' | b'L' | b'n' | b'N' | b't' | b'T' | b'u'
        | b'U' | b'v' | b'V' | b'w' | b'W' => b'+', // corners, tees, plus
        b'q' | b'Q' => b'-', // horizontal line
        b'x' | b'X' => b'|', // vertical line
        other => other,
    }
}

/// `cob_addnstr (const char *data, const int size)` (screenio.c line 1430): raise the truncation EC for a
/// write of `size`, then `addnstr` -- write the `size` data bytes at the cursor with the current attribute,
/// advancing the cursor. The boundary `addnstr` is modelled as the cell writes; the EC is returned.
pub fn cob_addnstr(win: &mut Window, data: &[u8], size: i32) -> Decision {
    let truncated = win.raise_ec_on_truncation(size);
    let n = size.max(0) as usize;
    for k in 0..n {
        let ch = data.get(k).copied().unwrap_or(COB_CH_SP);
        win.put_advance(ch);
    }
    Decision { truncated }
}

/// `cob_addnstr_graph (const char *data, const int size)` (screenio.c line 1439): the CONTROL GRAPHICS
/// variant -- raise the truncation EC, then emit each char separately, replacing the box-drawing source
/// letters by their ACS symbol (we record the portable ASCII fallback, see [`graph_fallback`]). The
/// per-cell `addch`/`add_wch` is the boundary; the resolved byte + cursor advance is modelled.
pub fn cob_addnstr_graph(win: &mut Window, data: &[u8], size: i32) -> Decision {
    let truncated = win.raise_ec_on_truncation(size);
    let n = size.max(0) as usize;
    for k in 0..n {
        let c = data.get(k).copied().unwrap_or(COB_CH_SP);
        win.put_advance(graph_fallback(c));
    }
    Decision { truncated }
}

/// `cob_addch (const chtype c)` (screenio.c line 1630): raise the truncation EC for a 1-cell write, then
/// `addch` -- write one byte at the cursor with the current attribute and advance.
pub fn cob_addch(win: &mut Window, c: u8) -> Decision {
    let truncated = win.raise_ec_on_truncation(1);
    win.put_advance(c);
    Decision { truncated }
}

/// `cob_addch_no_trunc_check (const chtype c)` (screenio.c line 1638): write one byte at the cursor with
/// the current attribute and advance, **without** the trunc check (the caller raised it beforehand). Always
/// reports `truncated: false`.
pub fn cob_addch_no_trunc_check(win: &mut Window, c: u8) -> Decision {
    win.put_advance(c);
    Decision { truncated: false }
}

/// `cob_addnch (const int n, const chtype c)` (screenio.c line 1644): raise the truncation EC for an
/// `n`-cell write, then write `c` `n` times via `cob_addch_no_trunc_check` (advancing each time).
pub fn cob_addnch(win: &mut Window, n: i32, c: u8) -> Decision {
    let truncated = win.raise_ec_on_truncation(n);
    let count = n.max(0);
    for _ in 0..count {
        cob_addch_no_trunc_check(win, c);
    }
    Decision { truncated }
}

/// `cob_screen_puts (cob_screen *s, cob_field *f, is_input, stmt)` (screenio.c line 1785): paint a field's
/// `data` at `(line, column)` with the field's `attr`. The position/`move`/`refresh`/`cob_screen_attr`
/// calls are the boundary; here we port the **character-selection logic** (screenio.c lines 1804-1832):
///
/// * `COB_SCREEN_INPUT` set: emit `f->size` chars; each is `*` if `SECURE`, else (for ACCEPT) the prompt
///   char when the source byte `<= ' '`, else the literal source byte. `default_prompt_char` is the field's
///   `prompt` first byte, or `COB_CH_UL` (`_`) when no prompt is set.
/// * not input and `!is_input`: write the whole field (`cob_addnstr_graph` when `GRAPHICS`, else
///   `cob_addnstr`).
/// * else (update field, `is_input` legacy): no characters, the cursor just advances past the field.
///
/// Returns the aggregate truncation [`Decision`]. `prompt` is `None` to model an absent `s->prompt`.
pub fn cob_screen_puts(
    win: &mut Window,
    data: &[u8],
    size: i32,
    attr: u64,
    is_input: bool,
    prompt: Option<u8>,
    stmt: ScreenStatement,
) -> Decision {
    // C: cob_move_cursor (line, column) is the boundary -- the caller positions the cursor; here we
    // assume win.cur_x/cur_y already sit at the field origin (as cob_screen_puts does after its move).
    let mut truncated = false;
    if attr & COB_SCREEN_INPUT != 0 {
        // default_prompt_char = s->prompt ? s->prompt->data[0] : COB_CH_UL.
        let default_prompt_char = prompt.unwrap_or(COB_CH_UL);
        truncated = win.raise_ec_on_truncation(size);
        let n = size.max(0) as usize;
        for k in 0..n {
            let p = data.get(k).copied().unwrap_or(COB_CH_SP);
            if attr & COB_SCREEN_SECURE != 0 {
                cob_addch_no_trunc_check(win, COB_CH_AS);
            } else if p <= b' ' && stmt == ScreenStatement::Accept {
                cob_addch_no_trunc_check(win, default_prompt_char);
            } else {
                cob_addch_no_trunc_check(win, p);
            }
        }
    } else if !is_input {
        if attr & COB_SCREEN_GRAPHICS != 0 {
            truncated = cob_addnstr_graph(win, data, size).truncated;
        } else {
            truncated = cob_addnstr(win, data, size).truncated;
        }
    } else {
        // C: column += f->size; cob_move_cursor (line, column); -- advance past the field, no chars.
        win.cur_x += size;
    }
    Decision { truncated }
}

/// `refresh_field (cob_screen *s)` (screenio.c line 2023): refresh one ACCEPT field -- save the cursor
/// `(y,x)` (`getyx`), repaint the field via `cob_screen_puts (..ACCEPT_STATEMENT)`, then restore the cursor
/// to `(y,x)` (`cob_move_cursor`). The `wrefresh` is the boundary; the save/repaint/restore cursor state is
/// modelled. Returns the repaint's truncation [`Decision`].
pub fn refresh_field(
    win: &mut Window,
    data: &[u8],
    size: i32,
    attr: u64,
    is_input: bool,
    prompt: Option<u8>,
) -> Decision {
    // getyx (stdscr, y, x)
    let (y, x) = (win.cur_y, win.cur_x);
    let d = cob_screen_puts(win, data, size, attr, is_input, prompt, ScreenStatement::Accept);
    // cob_move_cursor (y, x) -- restore.
    win.cur_y = y;
    win.cur_x = x;
    d
}

/// `cob_set_cursor_pos (int line, int column)` (screenio.c line 348): set the cursor to `(line, column)`.
/// The `move` is the boundary; the cursor-state update is modelled (curses clips off-screen targets but
/// `cob_set_cursor_pos` ignores the status, so we still record the requested position).
pub fn cob_set_cursor_pos(win: &mut Window, line: i32, column: i32) {
    win.cur_y = line;
    win.cur_x = column;
}

/// `cob_move_to_beg_of_last_line (void)` (screenio.c line 356): `move (max_y, 0)` -- put the cursor at the
/// beginning of the last line. (`getmaxyx` reads `max_y`; the `move` is the boundary.) Note C uses `max_y`
/// directly as the row, matching that source faithfully.
pub fn cob_move_to_beg_of_last_line(win: &mut Window) {
    win.cur_y = win.max_y;
    win.cur_x = 0;
}

/// `cob_screen_moveyx (cob_screen *s)` (screenio.c line 2929): move the cursor for a screen item carrying
/// an explicit `line`/`column` or a relative `LINE/COLUMN PLUS/MINUS`. Ports the resolution logic
/// (screenio.c lines 2937-2980): when any of those are present, read `getyx` `(y,x)`, clamp negatives to 0,
/// decrement a non-zero `x` (the "column adjust"), resolve absolute line/column from `origin + value`
/// (falling back to the cursor when negative or absent), then apply the relative PLUS/MINUS deltas, and set
/// the cursor. The `move`/`refresh` is the boundary. Pass `line`/`column` as `Some(value)` when the item
/// has an explicit LINE/COLUMN clause (the `cob_get_int (s->line)` value), else `None`.
pub fn cob_screen_moveyx(
    win: &mut Window,
    line: Option<i32>,
    column: Option<i32>,
    attr: u64,
    origin_y: i32,
    origin_x: i32,
) {
    let rel = COB_SCREEN_LINE_PLUS | COB_SCREEN_LINE_MINUS | COB_SCREEN_COLUMN_PLUS | COB_SCREEN_COLUMN_MINUS;
    // C guard: if (s->line || s->column || s->attr & (relative moves)).
    if line.is_none() && column.is_none() && attr & rel == 0 {
        return;
    }
    // getyx (stdscr, y, x); negatives clamped to 0.
    let mut y = win.cur_y;
    let mut x = win.cur_x;
    if x < 0 || y < 0 {
        x = 0;
        y = 0;
    }
    // Column adjust: if (x != 0) x--.
    if x != 0 {
        x -= 1;
    }
    // Resolve line.
    let mut rline = match line {
        None => y,
        Some(v) => {
            let l = origin_y + v;
            if l < 0 {
                y
            } else {
                l
            }
        }
    };
    // Resolve column.
    let mut rcol = match column {
        None => x,
        Some(v) => {
            let c = origin_x + v;
            if c < 0 {
                x
            } else {
                c
            }
        }
    };
    // Relative deltas.
    if attr & COB_SCREEN_LINE_PLUS != 0 {
        rline = y + rline;
    } else if attr & COB_SCREEN_LINE_MINUS != 0 {
        rline = y - rline;
    }
    if attr & COB_SCREEN_COLUMN_PLUS != 0 {
        rcol = x + rcol;
    } else if attr & COB_SCREEN_COLUMN_MINUS != 0 {
        rcol = x - rcol;
    }
    // cob_move_cursor (line, column); refresh () [boundary]; cob_current_y/x = line/column.
    win.cur_y = rline;
    win.cur_x = rcol;
}

/// `compare_yx (const void *m1, const void *m2)` (screenio.c line 2906): the `qsort` comparator ordering
/// input fields by `(this_y, this_x)`. Returns `-1`/`0`/`1`. Pure.
pub fn compare_yx(m1: (i32, i32), m2: (i32, i32)) -> i32 {
    let (y1, x1) = m1;
    let (y2, x2) = m2;
    if y1 < y2 {
        return -1;
    }
    if y1 > y2 {
        return 1;
    }
    if x1 < x2 {
        return -1;
    }
    if x1 > x2 {
        return 1;
    }
    0
}

/// `finalize_field_input (cob_screen *s)` (screenio.c line 2062): finalize one ACCEPT field on leaving it.
/// Numeric / numeric-edited fields are validated and re-formatted (`valid_field_data` -> `format_field`,
/// which round-trips the bytes through `cob_intr_numval`/`numval_c` then `cob_move` to neaten the editing).
/// Then the FULL and REQUIRED clauses are checked. Returns `1` (non-zero) on a validation failure, `0` on
/// success -- the exact C return contract.
///
/// The numeric byte transformation and the FULL/REQUIRED satisfaction are the *decisions* this finalizer
/// makes; the heavy reformat (`cob_intr_numval` + `cob_move`) and the curses repaint (`refresh_field`) are
/// boundaries, so they are supplied as already-computed predicates:
///
/// * `is_numeric` -- the field is numeric or numeric-edited (`cob_field_is_numeric_or_numeric_edited`).
/// * `valid` -- `valid_field_data (s->field)` (only consulted for numeric fields).
/// * `full_ok` -- `satisfied_full_clause (s)`.
/// * `required_ok` -- `satisfied_required_clause (s)`.
pub fn finalize_field_input(is_numeric: bool, valid: bool, full_ok: bool, required_ok: bool) -> i32 {
    // C: if numeric/numeric-edited -> if (!valid_field_data) return 1; format_field (s).
    if is_numeric && !valid {
        return 1;
    }
    // (format_field is the boundary reformat; modelled as a no-op decision here.)
    // C: if (!satisfied_full_clause || !satisfied_required_clause) return 1.
    if !full_ok || !required_ok {
        return 1;
    }
    0
}

/// `finalize_all_fields (struct cob_inp_struct *sptr, const size_t total_idx)` (screenio.c line 2080):
/// finalize every input field in order; return `1` as soon as one fails, else `0`. Each entry is the
/// per-field tuple `(is_numeric, valid, full_ok, required_ok)` already evaluated for its `scr`.
pub fn finalize_all_fields(fields: &[(bool, bool, bool, bool)]) -> i32 {
    for &(is_numeric, valid, full_ok, required_ok) in fields {
        if finalize_field_input(is_numeric, valid, full_ok, required_ok) != 0 {
            return 1;
        }
    }
    0
}

/// `cob_toggle_insert ()` (screenio.c line 2099): flip `COB_INSERT_MODE` -- off (`0`) becomes on (`1`),
/// anything else becomes off (`0`). The cursor-shape change (`cob_settings_screenio`) is the boundary; the
/// flag flip is modelled on the [`Window`].
pub fn cob_toggle_insert(win: &mut Window) {
    if win.insert_mode == 0 {
        win.insert_mode = 1; // on
    } else {
        win.insert_mode = 0; // off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // cob_addnstr: writes the bytes, advances the cursor, no truncation within bounds.
    #[test]
    fn cob_addnstr_writes_and_advances() {
        let mut w = Window::new(5, 10);
        w.cur_y = 1;
        w.cur_x = 2;
        let d = cob_addnstr(&mut w, b"ABC", 3);
        assert!(!d.truncated);
        assert_eq!(w.cur_x, 5);
        assert_eq!(w.cells[w.idx(1, 2).unwrap()].ch, b'A');
        assert_eq!(w.cells[w.idx(1, 4).unwrap()].ch, b'C');
    }

    // cob_addnstr: truncation when x + size - 1 > max_x.
    #[test]
    fn cob_addnstr_truncation_decision() {
        let mut w = Window::new(2, 5); // max_x = 5
        w.cur_x = 4;
        // 4 + 3 - 1 = 6 > 5 -> truncated.
        assert!(cob_addnstr(&mut w, b"XYZ", 3).truncated);
        let mut w2 = Window::new(2, 5);
        w2.cur_x = 1;
        // 1 + 3 - 1 = 3 <= 5.
        assert!(!cob_addnstr(&mut w2, b"XYZ", 3).truncated);
    }

    // cob_addnstr_graph: box-drawing letters resolve to ASCII fallbacks; others pass through.
    #[test]
    fn cob_addnstr_graph_fallbacks() {
        let mut w = Window::new(1, 10);
        cob_addnstr_graph(&mut w, b"lqkA", 4);
        assert_eq!(w.cells[0].ch, b'+'); // l -> upper-left corner -> '+'
        assert_eq!(w.cells[1].ch, b'-'); // q -> horizontal -> '-'
        assert_eq!(w.cells[2].ch, b'+'); // k -> upper-right corner -> '+'
        assert_eq!(w.cells[3].ch, b'A'); // literal
    }

    // cob_addch / cob_addch_no_trunc_check.
    #[test]
    fn cob_addch_variants() {
        let mut w = Window::new(2, 3); // max_x = 3
        w.cur_x = 3;
        // 3 + 1 - 1 = 3 <= 3 -> not truncated.
        assert!(!cob_addch(&mut w, b'Q').truncated);
        let mut w2 = Window::new(2, 3);
        w2.cur_x = 4;
        // 4 + 1 - 1 = 4 > 3 -> truncated.
        assert!(cob_addch(&mut w2, b'Q').truncated);
        // no_trunc_check never reports truncation.
        let mut w3 = Window::new(2, 3);
        w3.cur_x = 9;
        assert!(!cob_addch_no_trunc_check(&mut w3, b'Z').truncated);
        assert_eq!(w3.cur_x, 10);
    }

    // cob_addnch: n copies of c.
    #[test]
    fn cob_addnch_repeats() {
        let mut w = Window::new(1, 10);
        let d = cob_addnch(&mut w, 4, b'*');
        assert!(!d.truncated);
        assert_eq!(w.cur_x, 4);
        for i in 0..4 {
            assert_eq!(w.cells[i].ch, b'*');
        }
        // n=0 is a no-op write.
        let mut w2 = Window::new(1, 10);
        assert!(!cob_addnch(&mut w2, 0, b'*').truncated);
        assert_eq!(w2.cur_x, 0);
    }

    // cob_screen_puts: non-input field writes the whole field.
    #[test]
    fn cob_screen_puts_non_input() {
        let mut w = Window::new(2, 20);
        let d = cob_screen_puts(&mut w, b"HELLO", 5, 0, false, None, ScreenStatement::Display);
        assert!(!d.truncated);
        assert_eq!(w.cells[0].ch, b'H');
        assert_eq!(w.cur_x, 5);
    }

    // cob_screen_puts: INPUT field, ACCEPT, prompt char replaces blanks; SECURE masks all.
    #[test]
    fn cob_screen_puts_input_prompt_and_secure() {
        // INPUT, no SECURE: blank (<= ' ') -> prompt char '_', non-blank passes through.
        let mut w = Window::new(2, 20);
        cob_screen_puts(&mut w, b"A B", 3, COB_SCREEN_INPUT, false, None, ScreenStatement::Accept);
        assert_eq!(w.cells[0].ch, b'A');
        assert_eq!(w.cells[1].ch, COB_CH_UL); // ' ' -> '_'
        assert_eq!(w.cells[2].ch, b'B');
        // explicit prompt char.
        let mut w2 = Window::new(2, 20);
        cob_screen_puts(&mut w2, b" ", 1, COB_SCREEN_INPUT, false, Some(b'.'), ScreenStatement::Accept);
        assert_eq!(w2.cells[0].ch, b'.');
        // SECURE masks every byte with '*'.
        let mut w3 = Window::new(2, 20);
        cob_screen_puts(&mut w3, b"AB", 2, COB_SCREEN_INPUT | COB_SCREEN_SECURE, false, None, ScreenStatement::Accept);
        assert_eq!(w3.cells[0].ch, COB_CH_AS);
        assert_eq!(w3.cells[1].ch, COB_CH_AS);
    }

    // cob_screen_puts: INPUT under DISPLAY does NOT prompt-replace blanks (stmt != ACCEPT).
    #[test]
    fn cob_screen_puts_input_display_keeps_blank() {
        let mut w = Window::new(2, 20);
        cob_screen_puts(&mut w, b" ", 1, COB_SCREEN_INPUT, false, None, ScreenStatement::Display);
        assert_eq!(w.cells[0].ch, b' '); // blank kept, not prompt char
    }

    // cob_screen_puts: GRAPHICS non-input uses the graph emitter.
    #[test]
    fn cob_screen_puts_graphics() {
        let mut w = Window::new(2, 20);
        cob_screen_puts(&mut w, b"q", 1, COB_SCREEN_GRAPHICS, false, None, ScreenStatement::Display);
        assert_eq!(w.cells[0].ch, b'-');
    }

    // cob_screen_puts: update field (is_input true, not INPUT) advances without writing.
    #[test]
    fn cob_screen_puts_update_advances_only() {
        let mut w = Window::new(2, 20);
        w.cur_x = 1;
        cob_screen_puts(&mut w, b"ZZZ", 3, 0, true, None, ScreenStatement::Accept);
        assert_eq!(w.cur_x, 4);
        assert_eq!(w.cells[1].ch, b' '); // untouched
    }

    // refresh_field: repaints but restores the cursor.
    #[test]
    fn refresh_field_restores_cursor() {
        let mut w = Window::new(3, 20);
        w.cur_y = 1;
        w.cur_x = 5;
        refresh_field(&mut w, b"XY", 2, 0, false, None);
        // field written at (1,5)/(1,6).
        assert_eq!(w.cells[w.idx(1, 5).unwrap()].ch, b'X');
        // cursor restored.
        assert_eq!((w.cur_y, w.cur_x), (1, 5));
    }

    // cob_set_cursor_pos.
    #[test]
    fn cob_set_cursor_pos_sets() {
        let mut w = Window::new(5, 5);
        cob_set_cursor_pos(&mut w, 3, 4);
        assert_eq!((w.cur_y, w.cur_x), (3, 4));
    }

    // cob_move_to_beg_of_last_line: move (max_y, 0).
    #[test]
    fn cob_move_to_beg_of_last_line_moves() {
        let mut w = Window::new(24, 80);
        w.cur_y = 5;
        w.cur_x = 9;
        cob_move_to_beg_of_last_line(&mut w);
        assert_eq!((w.cur_y, w.cur_x), (24, 0));
    }

    // cob_screen_moveyx: guard -- no clause, no relative attr -> no move.
    #[test]
    fn cob_screen_moveyx_guard() {
        let mut w = Window::new(24, 80);
        w.cur_y = 7;
        w.cur_x = 7;
        cob_screen_moveyx(&mut w, None, None, 0, 0, 0);
        assert_eq!((w.cur_y, w.cur_x), (7, 7));
    }

    // cob_screen_moveyx: absolute line/column from origin.
    #[test]
    fn cob_screen_moveyx_absolute() {
        let mut w = Window::new(24, 80);
        w.cur_y = 0;
        w.cur_x = 0;
        // origin_y=1, line=4 -> row 5; origin_x=2, column=3 -> col 5.
        cob_screen_moveyx(&mut w, Some(4), Some(3), 0, 1, 2);
        assert_eq!((w.cur_y, w.cur_x), (5, 5));
    }

    // cob_screen_moveyx: relative LINE PLUS / COLUMN MINUS, with the column-adjust decrement.
    #[test]
    fn cob_screen_moveyx_relative() {
        let mut w = Window::new(24, 80);
        w.cur_y = 10;
        w.cur_x = 6; // adjusted to x=5 inside
        // LINE PLUS with line=2: rline = y + (origin_y + 2). origin_y=0 -> 10 + 2 = 12.
        // COLUMN MINUS with column=1: rcol = x - (origin_x + 1). origin_x=0, x=5 -> 5 - 1 = 4.
        cob_screen_moveyx(
            &mut w,
            Some(2),
            Some(1),
            COB_SCREEN_LINE_PLUS | COB_SCREEN_COLUMN_MINUS,
            0,
            0,
        );
        assert_eq!((w.cur_y, w.cur_x), (12, 4));
    }

    // compare_yx: y dominates, then x; equal -> 0.
    #[test]
    fn compare_yx_orders() {
        assert_eq!(compare_yx((1, 5), (2, 0)), -1); // y1 < y2
        assert_eq!(compare_yx((3, 0), (2, 9)), 1); // y1 > y2
        assert_eq!(compare_yx((2, 1), (2, 4)), -1); // x1 < x2
        assert_eq!(compare_yx((2, 8), (2, 4)), 1); // x1 > x2
        assert_eq!(compare_yx((2, 2), (2, 2)), 0); // equal
    }

    // finalize_field_input: each return path.
    #[test]
    fn finalize_field_input_paths() {
        // numeric + invalid -> 1.
        assert_eq!(finalize_field_input(true, false, true, true), 1);
        // numeric + valid + clauses ok -> 0.
        assert_eq!(finalize_field_input(true, true, true, true), 0);
        // non-numeric (valid irrelevant) + clauses ok -> 0.
        assert_eq!(finalize_field_input(false, false, true, true), 0);
        // FULL not satisfied -> 1.
        assert_eq!(finalize_field_input(false, true, false, true), 1);
        // REQUIRED not satisfied -> 1.
        assert_eq!(finalize_field_input(false, true, true, false), 1);
    }

    // finalize_all_fields: stops at first failure.
    #[test]
    fn finalize_all_fields_short_circuits() {
        // all ok -> 0.
        assert_eq!(finalize_all_fields(&[(false, true, true, true), (true, true, true, true)]), 0);
        // second fails -> 1.
        assert_eq!(finalize_all_fields(&[(false, true, true, true), (true, false, true, true)]), 1);
        // empty -> 0.
        assert_eq!(finalize_all_fields(&[]), 0);
    }

    // cob_toggle_insert: off->on->off.
    #[test]
    fn cob_toggle_insert_flips() {
        let mut w = Window::new(2, 2);
        assert_eq!(w.insert_mode, 0);
        cob_toggle_insert(&mut w);
        assert_eq!(w.insert_mode, 1);
        cob_toggle_insert(&mut w);
        assert_eq!(w.insert_mode, 0);
        // any non-zero starting value goes to 0.
        w.insert_mode = 5;
        cob_toggle_insert(&mut w);
        assert_eq!(w.insert_mode, 0);
    }
}
