//! Port of `screenio.c` -- the `SCREEN SECTION` **screen-tree navigation + geometry** layer: the
//! functions that walk the `cob_screen` tree, compute a screen item's absolute `(line, column)`, resolve
//! `LINE`/`COLUMN` field operands (including the relative `+`/`-` handling), translate the
//! last-statement saved cursor, map raw curses key codes to COBOL key values, and map a mouse event mask
//! to a COBOL exception code.
//!
//! ## Claim boundary (read this)
//!
//! These functions are *pure tree/arithmetic kernels*. In GnuCOBOL they live inside `screenio.c` and read
//! the live `cob_screen` tree, the `cob_field` operands, and the curses `getyx`/`getmaxyx` window state.
//! This port models that state **explicitly** as plain structs ([`CobScreen`], [`PosField`],
//! [`SavedCursors`]) so the navigation/geometry math is reproduced exactly, byte-for-byte with the C, with
//! no I/O and no panics.
//!
//! The *boundary* of the port is the side effect the C performs at the edge:
//!
//!   * `raise_ec_on_invalid_line_or_col` / `raise_ec_on_truncation` in the C call `cob_set_exception(...)`.
//!     Here they instead **return** which exception(s) to raise ([`EcOutcome`]) -- the actual
//!     `cob_set_exception` is the caller's job. The decision logic is the faithful part.
//!   * `pass_cursor_to_program` / `get_cursor_from_program` in the C read/write a live curses cursor and
//!     `COB_MODULE_PTR->cursor_pos`. Here they operate on an explicit [`CursorField`] value, returning the
//!     bytes/ints to store, so the encoding (`*1000+col`, the `%4.4d`/`%6.6d` packing) is reproduced.
//!
//! Cited C function names are preserved **exactly** (`is_first_screen_item`, `get_last_child`,
//! `cob_convert_key`, ...) per the port-index match-by-exact-C-name rule.
#![forbid(unsafe_code)]

// --------------------------------------------------------------------------------------------------
// Screen attribute flags (subset used by the geometry layer) -- `common.h` `COB_SCREEN_*`.
// --------------------------------------------------------------------------------------------------

/// `COB_SCREEN_LINE_PLUS` (`common.h`): the `LINE PLUS n` relative form.
pub const COB_SCREEN_LINE_PLUS: i32 = 1 << 0;
/// `COB_SCREEN_LINE_MINUS` (`common.h`): the `LINE MINUS n` relative form.
pub const COB_SCREEN_LINE_MINUS: i32 = 1 << 1;
/// `COB_SCREEN_COLUMN_PLUS` (`common.h`): the `COLUMN PLUS n` relative form.
pub const COB_SCREEN_COLUMN_PLUS: i32 = 1 << 2;
/// `COB_SCREEN_COLUMN_MINUS` (`common.h`): the `COLUMN MINUS n` relative form.
pub const COB_SCREEN_COLUMN_MINUS: i32 = 1 << 3;

// --------------------------------------------------------------------------------------------------
// `enum screen_statement` -- `screenio.c:125`.
// --------------------------------------------------------------------------------------------------

/// `enum screen_statement` (`screenio.c`): which statement is resolving a position. Selects which saved
/// cursor (`display_*` vs `accept_*`) the relative/zero forms read back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenStatement {
    /// `ACCEPT_STATEMENT`.
    Accept,
    /// `DISPLAY_STATEMENT`.
    Display,
}

// --------------------------------------------------------------------------------------------------
// Explicit model of the curses window the C reads via getmaxyx/getyx, and the screen origin.
// --------------------------------------------------------------------------------------------------

/// The live curses window state the geometry layer reads (`getmaxyx(stdscr, ...)`, `getyx(stdscr, ...)`)
/// plus the `origin_y`/`origin_x` screen origin (`screenio.c` file-statics set by `screen_display`/
/// `screen_accept`). Modelled explicitly so the otherwise-pure math has no hidden global input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// Current cursor row (0-based, curses `getyx` `y`).
    pub y: i32,
    /// Current cursor column (0-based, curses `getyx` `x`).
    pub x: i32,
    /// `getmaxyx` `max_y` -- rows in the window.
    pub max_y: i32,
    /// `getmaxyx` `max_x` -- columns in the window.
    pub max_x: i32,
    /// `origin_y` -- the screen's base row added in `get_screen_item_line_and_col`.
    pub origin_y: i32,
    /// `origin_x` -- the screen's base column added in `get_screen_item_line_and_col`.
    pub origin_x: i32,
}

impl Window {
    /// A window with the given dimensions, cursor at home, origin at (0,0).
    pub fn new(max_y: i32, max_x: i32) -> Self {
        Window { y: 0, x: 0, max_y, max_x, origin_y: 0, origin_x: 0 }
    }
}

// --------------------------------------------------------------------------------------------------
// `cob_screen` -- `common.h:1196`. An arena-indexed model of the tree (next/prev/child/parent are
// indices into a `Vec<CobScreen>` rather than raw pointers; `None` == NULL).
// --------------------------------------------------------------------------------------------------

/// A `cob_field` operand attached to a screen item (`field`/`value`/`line`/`column`). The geometry layer
/// only needs the integer value the field holds (`cob_get_int`) and its byte size (`->size`), so that is
/// all this models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosField {
    /// `cob_get_int(field)` -- the integer the field evaluates to.
    pub int_val: i32,
    /// `field->size` -- the field's byte size.
    pub size: usize,
}

impl PosField {
    /// A position field holding `int_val` with byte size `size`.
    pub fn new(int_val: i32, size: usize) -> Self {
        PosField { int_val, size }
    }
}

/// `typedef struct __cob_screen` (`common.h:1196`). The tree links (`next`/`prev`/`child`/`parent`) are
/// modelled as `Option<usize>` indices into the owning [`ScreenTree`]'s arena (NULL == `None`); the
/// `field`/`value`/`line`/`column` `cob_field*` operands are modelled as `Option<PosField>` (NULL ==
/// `None`); `attr` carries the `COB_SCREEN_*` flag bits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobScreen {
    /// `->next` (arena index, NULL == `None`).
    pub next: Option<usize>,
    /// `->prev` (arena index, NULL == `None`).
    pub prev: Option<usize>,
    /// `->child` (arena index, NULL == `None`).
    pub child: Option<usize>,
    /// `->parent` (arena index, NULL == `None`).
    pub parent: Option<usize>,
    /// `->field` (`COB_SCREEN_TYPE_FIELD`).
    pub field: Option<PosField>,
    /// `->value` (`COB_SCREEN_TYPE_VALUE`).
    pub value: Option<PosField>,
    /// `->line` (`LINE` clause operand).
    pub line: Option<PosField>,
    /// `->column` (`COLUMN` clause operand).
    pub column: Option<PosField>,
    /// `->attr` (`COB_SCREEN_*` flag bits).
    pub attr: i32,
}

impl Default for CobScreen {
    fn default() -> Self {
        CobScreen {
            next: None,
            prev: None,
            child: None,
            parent: None,
            field: None,
            value: None,
            line: None,
            column: None,
            attr: 0,
        }
    }
}

/// An arena holding the `cob_screen` nodes; `next`/`prev`/`child`/`parent` are indices into `nodes`.
/// This is the explicit stand-in for the heap of `cob_screen` structs the C walks via raw pointers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScreenTree {
    /// The nodes; an index `i` is a `cob_screen *`, `None` is `NULL`.
    pub nodes: Vec<CobScreen>,
}

impl ScreenTree {
    /// An empty tree.
    pub fn new() -> Self {
        ScreenTree { nodes: Vec::new() }
    }
    /// Push a node, returning its index (its `cob_screen *`).
    pub fn push(&mut self, n: CobScreen) -> usize {
        self.nodes.push(n);
        self.nodes.len() - 1
    }
    /// Borrow node `i` if it exists.
    fn get(&self, i: usize) -> Option<&CobScreen> {
        self.nodes.get(i)
    }
}

// --------------------------------------------------------------------------------------------------
// is_first_screen_item -- screenio.c:1656
// --------------------------------------------------------------------------------------------------

/// Port of `is_first_screen_item` (`screenio.c:1656`).
///
/// ```c
/// do { if (s->prev) return 0; s = s->parent; } while (s);
/// return 1;
/// ```
///
/// Walks up the parent chain; an item is "first" iff neither it nor any ancestor has a `prev` sibling.
/// `s` is the arena index of the start node; a missing index (the C NULL deref guard) yields `true`.
pub fn is_first_screen_item(tree: &ScreenTree, s: usize) -> bool {
    let mut cur = Some(s);
    while let Some(i) = cur {
        let node = match tree.get(i) {
            Some(n) => n,
            None => return true,
        };
        if node.prev.is_some() {
            return false;
        }
        cur = node.parent;
    }
    true
}

// --------------------------------------------------------------------------------------------------
// get_last_child -- screenio.c:1669
// --------------------------------------------------------------------------------------------------

/// Port of `get_last_child` (`screenio.c:1669`).
///
/// ```c
/// for (child = s->child; child->next; child = child->next);
/// if (child->child) return get_last_child (child);
/// else return child;
/// ```
///
/// Descends to the deepest, last node under `s`: walk `child` to its last `next` sibling, and if that
/// node itself has a child, recurse. Returns the arena index of the last leaf, or `None` if the tree is
/// malformed (the C assumes `s->child` is non-NULL; here a missing child returns `None` rather than
/// dereferencing NULL).
pub fn get_last_child(tree: &ScreenTree, s: usize) -> Option<usize> {
    let mut child = tree.get(s)?.child?;
    loop {
        // for (; child->next; child = child->next)
        while let Some(node) = tree.get(child) {
            match node.next {
                Some(n) => child = n,
                None => break,
            }
        }
        match tree.get(child)?.child {
            Some(_) => {
                // return get_last_child (child);
                child = tree.get(child)?.child?;
            }
            None => return Some(child),
        }
    }
}

// --------------------------------------------------------------------------------------------------
// get_prev_screen_item -- screenio.c:1683
// --------------------------------------------------------------------------------------------------

/// Port of `get_prev_screen_item` (`screenio.c:1683`).
///
/// ```c
/// if (s->prev) {
///     if (s->prev->child) return get_last_child (s->prev);
///     else return s->prev;
/// } else if (s->parent) return s->parent;
/// else return NULL;
/// ```
///
/// The reverse-iteration step over the flattened screen tree: the previous sibling's deepest last child
/// if the previous sibling is a group, else the previous sibling, else the parent, else `None`.
pub fn get_prev_screen_item(tree: &ScreenTree, s: usize) -> Option<usize> {
    let node = tree.get(s)?;
    if let Some(prev) = node.prev {
        if tree.get(prev).and_then(|p| p.child).is_some() {
            get_last_child(tree, prev)
        } else {
            Some(prev)
        }
    } else if let Some(parent) = node.parent {
        Some(parent)
    } else {
        None
    }
}

// --------------------------------------------------------------------------------------------------
// get_size -- screenio.c:1719
// --------------------------------------------------------------------------------------------------

/// Port of `get_size` (`screenio.c:1719`).
///
/// ```c
/// if (s->field) return s->field->size; else return s->value->size;
/// ```
///
/// A screen item's display size: its `field`'s size if present, else its `value`'s size. Returns `0` if
/// neither is present (the C assumes one always is).
pub fn get_size(tree: &ScreenTree, s: usize) -> usize {
    match tree.get(s) {
        Some(node) => {
            if let Some(f) = &node.field {
                f.size
            } else if let Some(v) = &node.value {
                v.size
            } else {
                0
            }
        }
        None => 0,
    }
}

// --------------------------------------------------------------------------------------------------
// update_line / update_column -- screenio.c:1699 (the UPDATE_CLAUSE_FUNC macro)
// --------------------------------------------------------------------------------------------------

/// The `UPDATE_CLAUSE_FUNC` body (`screenio.c:1699`), instantiated for `LINE`/`COLUMN`. Given the item's
/// `attr` flags, the relative `+n`/`-n`/absolute operand value, and the `_PLUS`/`_MINUS` flag bits for
/// this clause, accumulate into `count` and report whether an absolute clause was found.
///
/// ```c
/// if (attr & _PLUS)  *count += n;
/// else if (attr & _MINUS) *count -= n;
/// else { *count += n - 1; *found_clause = 1; }
/// ```
fn update_clause(attr: i32, operand: i32, plus_flag: i32, minus_flag: i32, count: &mut i32, found: &mut bool) {
    if attr & plus_flag != 0 {
        *count += operand;
    } else if attr & minus_flag != 0 {
        *count -= operand;
    } else {
        *count += operand - 1;
        *found = true;
    }
}

// --------------------------------------------------------------------------------------------------
// get_screen_item_line_and_col -- screenio.c:1729
// --------------------------------------------------------------------------------------------------

/// Port of `get_screen_item_line_and_col` (`screenio.c:1729`).
///
/// Computes a screen item's absolute `(line, col)` by walking *backwards* over the flattened tree (via
/// [`get_prev_screen_item`]) from `s`, accumulating relative `LINE`/`COLUMN` adjustments until an
/// absolute clause is found for each axis, then adding the screen origin (`origin_y`/`origin_x`).
///
/// Mirrors the C exactly: `found_line`/`found_col` latch once an absolute clause is seen; for an
/// elementary (childless) item that is not the screen-to-display and whose column is still unresolved,
/// the previous item's size (`get_size - 1`) is added; a `LINE` without `COLUMN` latches the column; and
/// a non-first elementary item with no `COLUMN` gets the implicit `+1` column step.
pub fn get_screen_item_line_and_col(tree: &ScreenTree, win: &Window, s: usize) -> (i32, i32) {
    let mut line = 0;
    let mut col = 0;
    let mut found_line = false;
    let mut found_col = false;
    let mut is_screen_to_display = true;

    let mut cur = Some(s);
    while let Some(i) = cur {
        let node = match tree.get(i) {
            Some(n) => n,
            None => break,
        };

        if node.line.is_some() && !found_line {
            // update_line (s, line, &found_line)
            let operand = node.line.as_ref().map(|f| f.int_val).unwrap_or(0);
            update_clause(
                node.attr,
                operand,
                COB_SCREEN_LINE_PLUS,
                COB_SCREEN_LINE_MINUS,
                &mut line,
                &mut found_line,
            );
        }

        if !found_col {
            let is_elementary = node.child.is_none();

            if !is_screen_to_display && is_elementary {
                col += get_size(tree, i) as i32 - 1;
            }

            if node.column.is_some() {
                let operand = node.column.as_ref().map(|f| f.int_val).unwrap_or(0);
                update_clause(
                    node.attr,
                    operand,
                    COB_SCREEN_COLUMN_PLUS,
                    COB_SCREEN_COLUMN_MINUS,
                    &mut col,
                    &mut found_col,
                );
            }

            if node.line.is_some() && node.column.is_none() {
                found_col = true;
            }

            if !found_col && node.column.is_none() && is_elementary && !is_first_screen_item(tree, i) {
                col += 1;
            }
        }

        is_screen_to_display = false;
        cur = get_prev_screen_item(tree, i);
    }

    line += win.origin_y;
    col += win.origin_x;
    (line, col)
}

// --------------------------------------------------------------------------------------------------
// get_line_column -- screenio.c:3064
// --------------------------------------------------------------------------------------------------

/// Port of `get_line_column` (`screenio.c:3064`).
///
/// ```c
/// *line = fline ? cob_get_int(fline) : 1;
/// *col  = fcol  ? cob_get_int(fcol)  : 1;
/// ```
///
/// Resolves a `(LINE, COLUMN)` operand pair to `(line, col)`, defaulting a missing operand (NULL field)
/// to `1`.
pub fn get_line_column(fline: Option<&PosField>, fcol: Option<&PosField>) -> (i32, i32) {
    let line = fline.map(|f| f.int_val).unwrap_or(1);
    let col = fcol.map(|f| f.int_val).unwrap_or(1);
    (line, col)
}

// --------------------------------------------------------------------------------------------------
// Saved-cursor state: display_cursor_*/accept_cursor_* (screenio.c file-statics)
// + col_where_last_stmt_ended / line_where_last_stmt_ended (screenio.c:3081 / :3087)
// --------------------------------------------------------------------------------------------------

/// The saved cursor positions the last `DISPLAY`/`ACCEPT` ended at (`screenio.c` file-statics
/// `display_cursor_y/x`, `accept_cursor_y/x`). Modelled explicitly so the relative/zero position forms
/// have a definite input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SavedCursors {
    /// `display_cursor_y`.
    pub display_y: i32,
    /// `display_cursor_x`.
    pub display_x: i32,
    /// `accept_cursor_y`.
    pub accept_y: i32,
    /// `accept_cursor_x`.
    pub accept_x: i32,
}

/// Port of `col_where_last_stmt_ended` (`screenio.c:3081`).
///
/// ```c
/// return stmt == DISPLAY_STATEMENT ? display_cursor_x : accept_cursor_x;
/// ```
pub fn col_where_last_stmt_ended(saved: &SavedCursors, stmt: ScreenStatement) -> i32 {
    match stmt {
        ScreenStatement::Display => saved.display_x,
        ScreenStatement::Accept => saved.accept_x,
    }
}

/// Port of `line_where_last_stmt_ended` (`screenio.c:3087`).
///
/// ```c
/// return stmt == DISPLAY_STATEMENT ? display_cursor_y : accept_cursor_y;
/// ```
pub fn line_where_last_stmt_ended(saved: &SavedCursors, stmt: ScreenStatement) -> i32 {
    match stmt {
        ScreenStatement::Display => saved.display_y,
        ScreenStatement::Accept => saved.accept_y,
    }
}

// --------------------------------------------------------------------------------------------------
// get_line_and_col_from_field -- screenio.c:1353
// --------------------------------------------------------------------------------------------------

/// A `cob_field` carrying a `CURSOR`/position value: its byte size selects the packing (4 -> `line*100+col`,
/// 6 -> `line*1000+col`), `int_val` is `cob_get_int` of it, and `numeric` is `COB_FIELD_IS_NUMERIC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorField {
    /// `field->size`.
    pub size: usize,
    /// `cob_get_int(field)` (used when numeric).
    pub int_val: i32,
    /// The raw bytes (used when non-numeric, parsed with `sscanf("%d")`).
    pub data: Vec<u8>,
    /// `COB_FIELD_IS_NUMERIC(field)`.
    pub numeric: bool,
}

/// The outcome of [`get_line_and_col_from_field`]: the decoded `(line, column)` and the `max_line_column`
/// divisor the C returns (1 when the size is neither 4 nor 6 -- the C then `cob_fatal_error`s for the
/// non-numeric branch; here `Err(GeomFatal)` marks that "would-fatal" boundary).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineColFromField {
    /// `*line`.
    pub line: i32,
    /// `*column`.
    pub column: i32,
    /// the returned `max_line_column` divisor (100, 1000, or 1).
    pub max_line_column: i32,
}

/// The "would-`cob_fatal_error(COB_FERROR_CODEGEN)`" boundary -- a codegen invariant the C treats as
/// fatal (bad field size for a non-numeric position field, or unparsable digits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeomFatal;

/// Port of `get_line_and_col_from_field` (`screenio.c:1353`).
///
/// Decodes a packed position field into `(line, column)`: size 4 packs as `line*100 + col`, size 6 as
/// `line*1000 + col`, any other size yields `max_line_column == 1`. For a numeric field the value is
/// `cob_get_int`; for a non-numeric field the bytes are parsed as a decimal (`sscanf("%d")`). The
/// would-fatal paths (`max_line_column == 1` on the non-numeric branch, or unparsable digits) return
/// `Err(GeomFatal)`.
pub fn get_line_and_col_from_field(pos: &CursorField) -> Result<LineColFromField, GeomFatal> {
    let max_line_column = match pos.size {
        4 => 100,
        6 => 1000,
        _ => 1,
    };

    let pos_val = if pos.numeric {
        pos.int_val
    } else {
        // C: if (max_line_column == 1) fatal; then sscanf the bytes as %d.
        if max_line_column == 1 {
            return Err(GeomFatal);
        }
        match parse_leading_int(&pos.data, pos.size) {
            Some(v) => v,
            None => return Err(GeomFatal),
        }
    };

    Ok(LineColFromField {
        line: pos_val / max_line_column,
        column: pos_val % max_line_column,
        max_line_column,
    })
}

/// `sscanf(buff, "%d", &v)` semantics over the first `size` bytes: skip leading ASCII whitespace, an
/// optional sign, then decimal digits; succeeds iff at least one digit is consumed (C `sscanf` returns 1).
fn parse_leading_int(data: &[u8], size: usize) -> Option<i32> {
    let n = size.min(data.len());
    let bytes = &data[..n];
    let mut i = 0;
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r') {
        i += 1;
    }
    let mut neg = false;
    if i < bytes.len() && (bytes[i] == 0x2b || bytes[i] == 0x2d) {
        neg = bytes[i] == 0x2d;
        i += 1;
    }
    let start = i;
    let mut val: i64 = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        val = val * 10 + (bytes[i] - b'0') as i64;
        i += 1;
    }
    if i == start {
        return None; // no digits consumed -> sscanf returns 0
    }
    if neg {
        val = -val;
    }
    Some(val as i32)
}

// --------------------------------------------------------------------------------------------------
// get_cursor_from_program -- screenio.c:1393
// --------------------------------------------------------------------------------------------------

/// Port of `get_cursor_from_program` (`screenio.c:1393`).
///
/// ```c
/// if (COB_MODULE_PTR && COB_MODULE_PTR->cursor_pos) {
///     ret = get_line_and_col_from_field(cursor_pos, line, column);
///     if (ret == 1) fatal;
///     *line -= 1; *column -= 1;
/// } else { *line = *column = -1; }
/// ```
///
/// `cursor` is the program's `SPECIAL-NAMES CURSOR` field, or `None` if not provided. Returns the
/// zero-based `(line, column)`, or `(-1, -1)` when no cursor field is provided. A `max_line_column == 1`
/// result is the would-fatal boundary (`Err(GeomFatal)`).
pub fn get_cursor_from_program(cursor: Option<&CursorField>) -> Result<(i32, i32), GeomFatal> {
    match cursor {
        Some(f) => {
            let r = get_line_and_col_from_field(f)?;
            if r.max_line_column == 1 {
                return Err(GeomFatal);
            }
            Ok((r.line - 1, r.column - 1))
        }
        None => Ok((-1, -1)),
    }
}

// --------------------------------------------------------------------------------------------------
// pass_cursor_to_program -- screenio.c:1314
// --------------------------------------------------------------------------------------------------

/// What [`pass_cursor_to_program`] decides to write back into the program's `CURSOR` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursorWriteback {
    /// No `CURSOR` field (`!COB_MODULE_PTR || !cursor_pos`): nothing written.
    None,
    /// Numeric, non-`DISPLAY` field: store the int `line*1000 + col` via `cob_set_int`.
    SetInt(i32),
    /// Size-4 field: store the ASCII bytes of `%4.4d` of `line*100 + col`.
    Bytes4([u8; 4]),
    /// Size-6 field: store the ASCII bytes of `%6.6d` of `line*1000 + col`.
    Bytes6([u8; 6]),
    /// Any other display field size: the would-`cob_fatal_error(COB_FERROR_CODEGEN)` boundary.
    Fatal,
}

/// Port of `pass_cursor_to_program` (`screenio.c:1314`).
///
/// Encodes the current curses cursor `(sline, scolumn)` -- which the C reads via `getyx` and then makes
/// 1-based (`sline++; scolumn++`) -- into the program's `CURSOR` field. The encoding depends on the
/// field: a numeric non-`DISPLAY` field gets `line*1000+col` as an int; a size-4 display field gets
/// `%4.4d` of `line*100+col`; a size-6 display field gets `%6.6d` of `line*1000+col`; any other display
/// size is the codegen-fatal boundary.
///
/// `cursor` is the `CURSOR` field (or `None`); `(y, x)` is the **0-based** curses cursor (`getyx`).
/// `numeric_non_display` is `COB_FIELD_IS_NUMERIC && TYPE != NUMERIC_DISPLAY`.
pub fn pass_cursor_to_program(
    cursor: Option<&CursorField>,
    y: i32,
    x: i32,
    numeric_non_display: bool,
) -> CursorWriteback {
    let f = match cursor {
        Some(f) => f,
        None => return CursorWriteback::None,
    };
    let sline = y + 1; // zero-based in curses
    let scolumn = x + 1;

    if numeric_non_display {
        CursorWriteback::SetInt(sline * 1000 + scolumn)
    } else {
        match f.size {
            4 => {
                let v = sline * 100 + scolumn;
                CursorWriteback::Bytes4(fmt_fixed4(v))
            }
            6 => {
                let v = sline * 1000 + scolumn;
                CursorWriteback::Bytes6(fmt_fixed6(v))
            }
            _ => CursorWriteback::Fatal,
        }
    }
}

/// `sprintf(buff, "%4.4d", v)` -> 4 ASCII bytes. Mirrors C: a negative `v` would print a sign; positions
/// are always positive here, but the zero-padded decimal of `v.abs()` truncated/right-justified to 4 is
/// reproduced for the in-range case (line/col always fit).
fn fmt_fixed4(v: i32) -> [u8; 4] {
    let s = format!("{:04}", v);
    let b = s.as_bytes();
    let mut out = [b'0'; 4];
    // %4.4d is min-width 4 / min-digits 4; for the small positive values here it is exactly 4 bytes.
    let start = b.len().saturating_sub(4);
    let slice = &b[start..];
    let off = 4 - slice.len();
    out[off..].copy_from_slice(slice);
    out
}

/// `sprintf(buff, "%6.6d", v)` -> 6 ASCII bytes (see [`fmt_fixed4`]).
fn fmt_fixed6(v: i32) -> [u8; 6] {
    let s = format!("{:06}", v);
    let b = s.as_bytes();
    let mut out = [b'0'; 6];
    let start = b.len().saturating_sub(6);
    let slice = &b[start..];
    let off = 6 - slice.len();
    out[off..].copy_from_slice(slice);
    out
}

// --------------------------------------------------------------------------------------------------
// raise_ec_on_invalid_line_or_col -- screenio.c:322
// --------------------------------------------------------------------------------------------------

/// Which COBOL exception condition(s) a geometry check decided to raise. In the C these are
/// `cob_set_exception(COB_EC_SCREEN_*)` side effects; this port returns the decision and leaves the actual
/// raise to the caller (that `cob_set_exception` call is the module boundary).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EcOutcome {
    /// raise `COB_EC_SCREEN_LINE_NUMBER`.
    pub line_number: bool,
    /// raise `COB_EC_SCREEN_STARTING_COLUMN`.
    pub starting_column: bool,
    /// raise `COB_EC_SCREEN_ITEM_TRUNCATED`.
    pub item_truncated: bool,
}

/// Port of `raise_ec_on_invalid_line_or_col` (`screenio.c:322`).
///
/// ```c
/// getmaxyx(stdscr, max_y, max_x);
/// if (line < 0 || line >= max_y)  cob_set_exception(COB_EC_SCREEN_LINE_NUMBER);
/// if (column < 0 || column >= max_x) cob_set_exception(COB_EC_SCREEN_STARTING_COLUMN);
/// ```
///
/// Returns which screen-position exceptions to raise for a `(line, column)` against the window bounds.
/// The `cob_set_exception` call is the boundary (returned, not performed).
pub fn raise_ec_on_invalid_line_or_col(win: &Window, line: i32, column: i32) -> EcOutcome {
    let mut ec = EcOutcome::default();
    if line < 0 || line >= win.max_y {
        ec.line_number = true;
    }
    if column < 0 || column >= win.max_x {
        ec.starting_column = true;
    }
    ec
}

// --------------------------------------------------------------------------------------------------
// raise_ec_on_truncation -- screenio.c:1411
// --------------------------------------------------------------------------------------------------

/// Port of `raise_ec_on_truncation` (`screenio.c:1411`).
///
/// ```c
/// getyx(stdscr, y, x);
/// getmaxyx(stdscr, max_y, max_x);
/// if (x + item_size - 1 > max_x) cob_set_exception(COB_EC_SCREEN_ITEM_TRUNCATED);
/// ```
///
/// Returns whether writing an `item_size`-wide item at the current cursor would truncate against the
/// window's right edge. The `cob_set_exception` call is the boundary (returned, not performed).
pub fn raise_ec_on_truncation(win: &Window, item_size: i32) -> EcOutcome {
    let mut ec = EcOutcome::default();
    if win.x + item_size - 1 > win.max_x {
        ec.item_truncated = true;
    }
    ec
}

// --------------------------------------------------------------------------------------------------
// extract_line_and_col_vals -- screenio.c:3094
// --------------------------------------------------------------------------------------------------

/// The result of [`extract_line_and_col_vals`]: the resolved 0-based `(sline, scolumn)` plus the same
/// `EcOutcome` decision the C threads through `cob_set_exception` (here returned). `Fatal` marks the
/// would-`cob_fatal_error` codegen boundary in the `line->size >= 4` sub-path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractedPos {
    /// `*sline`.
    pub sline: i32,
    /// `*scolumn`.
    pub scolumn: i32,
    /// the exceptions the C would have raised.
    pub ec: EcOutcome,
}

/// Port of `extract_line_and_col_vals` (`screenio.c:3094`).
///
/// Resolves a `LINE`/`COLUMN` operand pair into the 0-based `(sline, scolumn)` used to position a screen
/// statement, handling every C combination:
///
///   * both operands NULL -> `(0, 0)`.
///   * only `LINE`, with `line->size < 4` -> treat as a bare `LINE n` (column defaults to 1).
///   * only `LINE`, with `line->size >= 4` -> the packed `line*K+col` form via
///     [`get_line_and_col_from_field`] (its `max_line_column == 1` is the codegen-fatal boundary).
///   * both present -> [`get_line_column`].
///
/// then maps the COBOL 1-based `(cobol_line, cobol_col)` to 0-based, with the `0` (relative-to-last)
/// special cases: when allowed, a 0 reads back the last statement's saved cursor
/// ([`line_where_last_stmt_ended`]/[`col_where_last_stmt_ended`], with `+1` on the line for the
/// "col present, line zero" case); when not allowed, it instead flags the corresponding
/// `COB_EC_SCREEN_*` exception and zeroes the axis.
///
/// `Err(GeomFatal)` is returned only on the packed-field codegen-fatal path.
pub fn extract_line_and_col_vals(
    line: Option<&PosField>,
    column: Option<&PosField>,
    line_packed: Option<&CursorField>,
    stmt: ScreenStatement,
    zero_line_col_allowed: bool,
    saved: &SavedCursors,
) -> Result<ExtractedPos, GeomFatal> {
    let cobol_line: i32;
    let cobol_col: i32;

    if column.is_none() {
        match line {
            None => {
                return Ok(ExtractedPos { sline: 0, scolumn: 0, ec: EcOutcome::default() });
            }
            Some(lf) => {
                if lf.size < 4 {
                    cobol_line = lf.int_val;
                    cobol_col = 1;
                } else {
                    // packed: line actually contains both line and field numbers.
                    let pf = line_packed.ok_or(GeomFatal)?;
                    let r = get_line_and_col_from_field(pf)?;
                    if r.max_line_column == 1 {
                        return Err(GeomFatal);
                    }
                    cobol_line = r.line;
                    cobol_col = r.column;
                }
            }
        }
    } else {
        let (l, c) = get_line_column(line, column);
        cobol_line = l;
        cobol_col = c;
    }

    let mut ec = EcOutcome::default();
    let sline: i32;
    let scolumn: i32;

    if cobol_line == 0 {
        if cobol_col == 0 {
            if zero_line_col_allowed {
                sline = line_where_last_stmt_ended(saved, stmt);
                scolumn = col_where_last_stmt_ended(saved, stmt);
            } else {
                ec.line_number = true;
                ec.starting_column = true;
                sline = 0;
                scolumn = 0;
            }
        } else {
            if zero_line_col_allowed {
                sline = line_where_last_stmt_ended(saved, stmt) + 1;
            } else {
                ec.line_number = true;
                sline = 0;
            }
            scolumn = cobol_col - 1;
        }
    } else if cobol_col == 0 {
        sline = cobol_line - 1;
        if zero_line_col_allowed {
            scolumn = col_where_last_stmt_ended(saved, stmt);
        } else {
            ec.starting_column = true;
            scolumn = 0;
        }
    } else {
        sline = cobol_line - 1;
        scolumn = cobol_col - 1;
    }

    Ok(ExtractedPos { sline, scolumn, ec })
}

// --------------------------------------------------------------------------------------------------
// curses key constants (ncurses 6.x) + cob_convert_key -- screenio.c:1186
// --------------------------------------------------------------------------------------------------

/// ncurses key constants (values pinned to ncurses 6.x `curses.h`) used by [`cob_convert_key`]. The COBOL
/// "new key" values (`ALT_DEL`/`ALT_LEFT`/`ALT_RIGHT`) follow the `COB_NEW_KEY(n) = KEY_MAX + n` branch
/// taken when ncurses defines `KEY_MAX`.
pub mod keys {
    /// `KEY_MIN`.
    pub const KEY_MIN: i32 = 0o401;
    /// `KEY_DOWN`.
    pub const KEY_DOWN: i32 = 0o402;
    /// `KEY_UP`.
    pub const KEY_UP: i32 = 0o403;
    /// `KEY_LEFT`.
    pub const KEY_LEFT: i32 = 0o404;
    /// `KEY_RIGHT`.
    pub const KEY_RIGHT: i32 = 0o405;
    /// `KEY_HOME`.
    pub const KEY_HOME: i32 = 0o406;
    /// `KEY_BACKSPACE`.
    pub const KEY_BACKSPACE: i32 = 0o407;
    /// `KEY_DC` (delete-character).
    pub const KEY_DC: i32 = 0o512;
    /// `KEY_IC` (insert-character).
    pub const KEY_IC: i32 = 0o513;
    /// `KEY_EOL` (clear-to-end-of-line).
    pub const KEY_EOL: i32 = 0o517;
    /// `KEY_NPAGE`.
    pub const KEY_NPAGE: i32 = 0o522;
    /// `KEY_PPAGE`.
    pub const KEY_PPAGE: i32 = 0o523;
    /// `KEY_STAB` (set-tab).
    pub const KEY_STAB: i32 = 0o524;
    /// `KEY_ENTER`.
    pub const KEY_ENTER: i32 = 0o527;
    /// `KEY_PRINT`.
    pub const KEY_PRINT: i32 = 0o532;
    /// `KEY_A1` (upper-left of keypad).
    pub const KEY_A1: i32 = 0o534;
    /// `KEY_A2` (upper-center of keypad) -- conditionally defined; the COBOL switch guards it.
    pub const KEY_A2: i32 = 0o535 - 1; // not in base ncurses table; value irrelevant if unused
    /// `KEY_A3` (upper-right of keypad).
    pub const KEY_A3: i32 = 0o535;
    /// `KEY_B1` (middle-left of keypad).
    pub const KEY_B1: i32 = 0o536 - 1; // conditionally defined
    /// `KEY_B3` (middle-right of keypad).
    pub const KEY_B3: i32 = 0o536;
    /// `KEY_C1` (lower-left of keypad).
    pub const KEY_C1: i32 = 0o537;
    /// `KEY_C2` (lower-center of keypad).
    pub const KEY_C2: i32 = 0o540 - 1; // conditionally defined
    /// `KEY_C3` (lower-right of keypad).
    pub const KEY_C3: i32 = 0o540;
    /// `KEY_CLOSE`.
    pub const KEY_CLOSE: i32 = 0o544;
    /// `KEY_END`.
    pub const KEY_END: i32 = 0o550;
    /// `KEY_PREVIOUS`.
    pub const KEY_PREVIOUS: i32 = 0o562;
    /// `KEY_MAX`.
    pub const KEY_MAX: i32 = 0o777;

    /// `COB_NEW_KEY(1)` = `ALT_DEL` (= `KEY_MAX + 1`).
    pub const ALT_DEL: i32 = KEY_MAX + 1;
    /// `COB_NEW_KEY(2)` = `ALT_LEFT` (= `KEY_MAX + 2`).
    pub const ALT_LEFT: i32 = KEY_MAX + 2;
    /// `COB_NEW_KEY(3)` = `ALT_RIGHT` (= `KEY_MAX + 3`).
    pub const ALT_RIGHT: i32 = KEY_MAX + 3;
}

/// Runtime flags the `cob_convert_key` ignore-pass consults (`COB_EXTENDED_STATUS`, `COB_USE_ESC`).
/// Modelled explicitly so the second switch is deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyConvertConfig {
    /// `COB_EXTENDED_STATUS`.
    pub extended_status: bool,
    /// `COB_USE_ESC`.
    pub use_esc: bool,
}

impl Default for KeyConvertConfig {
    fn default() -> Self {
        KeyConvertConfig { extended_status: false, use_esc: false }
    }
}

/// Port of `cob_convert_key` (`screenio.c:1186`).
///
/// Maps a raw curses key code to the COBOL key value. Two stages, exactly as the C:
///
///   1. **Translate** control bytes / keypad codes to canonical `KEY_*` (e.g. `\n`/`\r`/`^D`/`^Z` ->
///      `KEY_ENTER`, `\t` -> `KEY_STAB`, `\b`/`0177` -> `KEY_BACKSPACE`, `KEY_EOL` -> `ALT_DEL`,
///      `KEY_CLOSE`/`KEY_PREVIOUS` -> `ALT_LEFT`, keypad `A1/A3/C1/C3` -> `HOME/PPAGE/END/NPAGE`, and the
///      conditionally-present `A2/B1/B3/C2` -> `UP/LEFT/RIGHT/DOWN`).
///   2. **Ignore-filter**: zero out keys that should be swallowed unless the relevant runtime flag is set
///      (`ESC` unless `EXTENDED_STATUS && USE_ESC`; `PPAGE`/`NPAGE`/`PRINT` unless `EXTENDED_STATUS`;
///      `UP`/`DOWN` during a field accept unless `EXTENDED_STATUS`).
///
/// `field_accept` mirrors the C `field_accept` arg (nonzero during an in-field `ACCEPT`). Returns the
/// converted key value.
pub fn cob_convert_key(keyp: i32, field_accept: bool, cfg: &KeyConvertConfig) -> i32 {
    use keys::*;

    // Stage 1: map to KEY_xxx value.
    let mut k = match keyp {
        // '\n' | '\r' | '\004' | '\032'
        0x0a | 0x0d | 0x04 | 0x1a => KEY_ENTER,
        // '\t'
        0x09 => KEY_STAB,
        // '\b' | 0177
        0x08 | 0o177 => KEY_BACKSPACE,
        x if x == KEY_EOL => ALT_DEL,
        x if x == KEY_CLOSE => ALT_LEFT,
        x if x == KEY_PREVIOUS => ALT_LEFT,
        x if x == KEY_A1 => KEY_HOME,
        x if x == KEY_A3 => KEY_PPAGE,
        x if x == KEY_C1 => KEY_END,
        x if x == KEY_C3 => KEY_NPAGE,
        x if x == KEY_A2 => KEY_UP,
        x if x == KEY_B1 => KEY_LEFT,
        x if x == KEY_B3 => KEY_RIGHT,
        x if x == KEY_C2 => KEY_DOWN,
        other => other,
    };

    // Stage 2: check if key should be ignored.
    match k {
        // '\033' (ESC)
        0x1b => {
            if !cfg.extended_status || !cfg.use_esc {
                k = 0;
            }
        }
        x if x == KEY_PPAGE || x == KEY_NPAGE || x == KEY_PRINT => {
            if !cfg.extended_status {
                k = 0;
            }
        }
        x if x == KEY_UP || x == KEY_DOWN => {
            if field_accept && !cfg.extended_status {
                k = 0;
            }
        }
        _ => {}
    }

    k
}

// --------------------------------------------------------------------------------------------------
// mouse masks (ncurses 6.x, NCURSES_MOUSE_VERSION 2 -> 5-bit stride) + mouse_to_exception_code
// -- screenio.c:2150
// --------------------------------------------------------------------------------------------------

/// ncurses mouse-event mask bits, for `NCURSES_MOUSE_VERSION 2` where the per-button stride is 5 bits:
/// `NCURSES_MOUSE_MASK(b, m) = m << ((b-1)*5)`. The modifier bits (`SHIFT`/`CTRL`/`ALT`) sit in the
/// button-6 slot (`(6-1)*5 = 25`). `COB_HAS_MOUSEWHEEL` (buttons 4/5) is assumed defined, matching a
/// modern ncurses build.
pub mod mouse {
    /// `NCURSES_BUTTON_RELEASED` (001).
    pub const BUTTON_RELEASED: u64 = 0o1;
    /// `NCURSES_BUTTON_PRESSED` (002).
    pub const BUTTON_PRESSED: u64 = 0o2;
    /// `NCURSES_BUTTON_CLICKED` (004).
    pub const BUTTON_CLICKED: u64 = 0o4;
    /// `NCURSES_DOUBLE_CLICKED` (010).
    pub const DOUBLE_CLICKED: u64 = 0o10;
    /// `NCURSES_TRIPLE_CLICKED` (020).
    pub const TRIPLE_CLICKED: u64 = 0o20;

    /// `NCURSES_MOUSE_MASK(b, m)` for the version-2 (5-bit-stride) build.
    pub const fn mask(b: u32, m: u64) -> u64 {
        m << ((b - 1) * 5)
    }

    /// `BUTTON1_RELEASED`.
    pub const BUTTON1_RELEASED: u64 = mask(1, BUTTON_RELEASED);
    /// `BUTTON1_PRESSED`.
    pub const BUTTON1_PRESSED: u64 = mask(1, BUTTON_PRESSED);
    /// `BUTTON1_CLICKED`.
    pub const BUTTON1_CLICKED: u64 = mask(1, BUTTON_CLICKED);
    /// `BUTTON1_DOUBLE_CLICKED`.
    pub const BUTTON1_DOUBLE_CLICKED: u64 = mask(1, DOUBLE_CLICKED);
    /// `BUTTON1_TRIPLE_CLICKED`.
    pub const BUTTON1_TRIPLE_CLICKED: u64 = mask(1, TRIPLE_CLICKED);

    /// `BUTTON2_RELEASED`.
    pub const BUTTON2_RELEASED: u64 = mask(2, BUTTON_RELEASED);
    /// `BUTTON2_PRESSED`.
    pub const BUTTON2_PRESSED: u64 = mask(2, BUTTON_PRESSED);
    /// `BUTTON2_CLICKED`.
    pub const BUTTON2_CLICKED: u64 = mask(2, BUTTON_CLICKED);
    /// `BUTTON2_DOUBLE_CLICKED`.
    pub const BUTTON2_DOUBLE_CLICKED: u64 = mask(2, DOUBLE_CLICKED);
    /// `BUTTON2_TRIPLE_CLICKED`.
    pub const BUTTON2_TRIPLE_CLICKED: u64 = mask(2, TRIPLE_CLICKED);

    /// `BUTTON3_RELEASED`.
    pub const BUTTON3_RELEASED: u64 = mask(3, BUTTON_RELEASED);
    /// `BUTTON3_PRESSED`.
    pub const BUTTON3_PRESSED: u64 = mask(3, BUTTON_PRESSED);
    /// `BUTTON3_CLICKED`.
    pub const BUTTON3_CLICKED: u64 = mask(3, BUTTON_CLICKED);
    /// `BUTTON3_DOUBLE_CLICKED`.
    pub const BUTTON3_DOUBLE_CLICKED: u64 = mask(3, DOUBLE_CLICKED);
    /// `BUTTON3_TRIPLE_CLICKED`.
    pub const BUTTON3_TRIPLE_CLICKED: u64 = mask(3, TRIPLE_CLICKED);

    /// `BUTTON4_PRESSED` (wheel up).
    pub const BUTTON4_PRESSED: u64 = mask(4, BUTTON_PRESSED);
    /// `BUTTON5_PRESSED` (wheel down).
    pub const BUTTON5_PRESSED: u64 = mask(5, BUTTON_PRESSED);

    /// `BUTTON_CTRL` (button-6 slot, bit 0).
    pub const BUTTON_CTRL: u64 = mask(6, 0o1);
    /// `BUTTON_SHIFT` (button-6 slot, bit 1).
    pub const BUTTON_SHIFT: u64 = mask(6, 0o2);
    /// `BUTTON_ALT` (button-6 slot, bit 2).
    pub const BUTTON_ALT: u64 = mask(6, 0o4);
}

/// Port of `mouse_to_exception_code` (`screenio.c:2150`), built with `COB_HAS_MOUSEWHEEL` (the modern
/// ncurses build: buttons 4/5 active, and the wheel-aware modifier offsets).
///
/// Maps an ncurses mouse-event mask to the COBOL mouse exception code (`2040`..): each button state maps
/// to a base code (press/click 2041, release 2042, double/triple 2043 for button 1, etc.), defaulting to
/// `2040` (mouse-moved). Wheel up/down are `2080`/`2081`. A modifier then offsets the base: with
/// `COB_HAS_MOUSEWHEEL`, SHIFT `+10` (`+4` for wheel), CTRL `+20` (`+8`), ALT `+30` (`+12`).
pub fn mouse_to_exception_code(event_mask: u64) -> i32 {
    use mouse::*;

    let mut fret: i32;
    if event_mask & BUTTON1_PRESSED != 0 {
        fret = 2041;
    } else if event_mask & BUTTON1_CLICKED != 0 {
        fret = 2041;
    } else if event_mask & BUTTON1_RELEASED != 0 {
        fret = 2042;
    } else if event_mask & BUTTON1_DOUBLE_CLICKED != 0 {
        fret = 2043;
    } else if event_mask & BUTTON1_TRIPLE_CLICKED != 0 {
        fret = 2043;
    } else if event_mask & BUTTON2_PRESSED != 0 {
        fret = 2044;
    } else if event_mask & BUTTON2_CLICKED != 0 {
        fret = 2044;
    } else if event_mask & BUTTON2_RELEASED != 0 {
        fret = 2045;
    } else if event_mask & BUTTON2_DOUBLE_CLICKED != 0 {
        fret = 2046;
    } else if event_mask & BUTTON2_TRIPLE_CLICKED != 0 {
        fret = 2046;
    } else if event_mask & BUTTON3_PRESSED != 0 {
        fret = 2047;
    } else if event_mask & BUTTON3_CLICKED != 0 {
        fret = 2047;
    } else if event_mask & BUTTON3_RELEASED != 0 {
        fret = 2048;
    } else if event_mask & BUTTON3_DOUBLE_CLICKED != 0 {
        fret = 2049;
    } else if event_mask & BUTTON3_TRIPLE_CLICKED != 0 {
        fret = 2049;
    } else if event_mask & BUTTON4_PRESSED != 0 {
        fret = 2080;
    } else if event_mask & BUTTON5_PRESSED != 0 {
        fret = 2081;
    } else {
        fret = 2040; // mouse-moved (assumed)
    }

    // COB_HAS_MOUSEWHEEL modifier offsets.
    if event_mask & BUTTON_SHIFT != 0 {
        if fret < 2080 {
            fret += 10;
        } else {
            fret += 4;
        }
    } else if event_mask & BUTTON_CTRL != 0 {
        if fret < 2080 {
            fret += 20;
        } else {
            fret += 8;
        }
    } else if event_mask & BUTTON_ALT != 0 {
        if fret < 2080 {
            fret += 30;
        } else {
            fret += 12;
        }
    }

    fret
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- helpers ----------------------------------------------------------------------------

    /// Build a flat list of sibling field items under no parent (each elementary), linking prev/next.
    fn flat_fields(sizes: &[usize]) -> ScreenTree {
        let mut t = ScreenTree::new();
        for (i, &sz) in sizes.iter().enumerate() {
            let mut n = CobScreen::default();
            n.field = Some(PosField::new(0, sz));
            if i > 0 {
                n.prev = Some(i - 1);
            }
            let idx = t.push(n);
            if i > 0 {
                t.nodes[i - 1].next = Some(idx);
            }
        }
        t
    }

    // ---- is_first_screen_item ---------------------------------------------------------------

    #[test]
    fn is_first_screen_item_walks_parents() {
        let t = flat_fields(&[3, 3, 3]);
        assert!(is_first_screen_item(&t, 0));
        assert!(!is_first_screen_item(&t, 1));
        assert!(!is_first_screen_item(&t, 2));
        // child of a first parent with no prev is first; with a prev parent is not.
        let mut t2 = ScreenTree::new();
        let p = t2.push(CobScreen::default()); // parent, first
        let mut c = CobScreen::default();
        c.parent = Some(p);
        let ci = t2.push(c);
        assert!(is_first_screen_item(&t2, ci));
        // give the parent a prev sibling
        let prev = t2.push(CobScreen::default());
        t2.nodes[p].prev = Some(prev);
        assert!(!is_first_screen_item(&t2, ci));
    }

    // ---- get_last_child / get_prev_screen_item ----------------------------------------------

    #[test]
    fn get_last_child_descends_to_deepest_last() {
        // group g with children a,b ; b has children c,d -> last child is d.
        let mut t = ScreenTree::new();
        let g = t.push(CobScreen::default());
        let a = t.push(CobScreen { parent: Some(g), ..Default::default() });
        let b = t.push(CobScreen { parent: Some(g), prev: Some(a), ..Default::default() });
        t.nodes[a].next = Some(b);
        t.nodes[g].child = Some(a);
        let c = t.push(CobScreen { parent: Some(b), ..Default::default() });
        let d = t.push(CobScreen { parent: Some(b), prev: Some(c), ..Default::default() });
        t.nodes[c].next = Some(d);
        t.nodes[b].child = Some(c);
        assert_eq!(get_last_child(&t, g), Some(d));
    }

    #[test]
    fn get_prev_screen_item_reverse_iteration() {
        let t = flat_fields(&[3, 3, 3]);
        // item 2's prev is sibling 1 (no child) -> 1.
        assert_eq!(get_prev_screen_item(&t, 2), Some(1));
        assert_eq!(get_prev_screen_item(&t, 1), Some(0));
        // first item, no prev, no parent -> None.
        assert_eq!(get_prev_screen_item(&t, 0), None);

        // prev with a child -> last child of prev.
        let mut t2 = ScreenTree::new();
        let g = t2.push(CobScreen::default());
        let leaf = t2.push(CobScreen { parent: Some(g), ..Default::default() });
        t2.nodes[g].child = Some(leaf);
        let after = t2.push(CobScreen { prev: Some(g), ..Default::default() });
        t2.nodes[g].next = Some(after);
        assert_eq!(get_prev_screen_item(&t2, after), Some(leaf));

        // no prev but has parent -> parent.
        assert_eq!(get_prev_screen_item(&t2, leaf), Some(g));
    }

    // ---- get_size ---------------------------------------------------------------------------

    #[test]
    fn get_size_field_then_value() {
        let mut t = ScreenTree::new();
        let f = t.push(CobScreen { field: Some(PosField::new(0, 7)), ..Default::default() });
        let v = t.push(CobScreen { value: Some(PosField::new(0, 5)), ..Default::default() });
        let n = t.push(CobScreen::default());
        assert_eq!(get_size(&t, f), 7);
        assert_eq!(get_size(&t, v), 5);
        assert_eq!(get_size(&t, n), 0);
    }

    // ---- get_screen_item_line_and_col -------------------------------------------------------

    #[test]
    fn screen_item_line_col_implicit_plus_one() {
        // three elementary fields of size 3, no LINE/COLUMN clauses: standard COL+1 stepping.
        // item0 is first -> col 0; item1 adds prev size(3)-1=2 then +1 => 3; item2 => 3+ (3-1)+1 = 7.
        let t = flat_fields(&[3, 3, 3]);
        let win = Window::new(24, 80);
        assert_eq!(get_screen_item_line_and_col(&t, &win, 0), (0, 0));
        assert_eq!(get_screen_item_line_and_col(&t, &win, 1), (0, 3));
        assert_eq!(get_screen_item_line_and_col(&t, &win, 2), (0, 6));
    }

    #[test]
    fn screen_item_absolute_line_and_column() {
        // single field with absolute LINE 5 COLUMN 10 -> (4, 9) (1-based clause -> 0-based via n-1).
        let mut t = ScreenTree::new();
        let mut n = CobScreen::default();
        n.field = Some(PosField::new(0, 4));
        n.line = Some(PosField::new(5, 2));
        n.column = Some(PosField::new(10, 2));
        let i = t.push(n);
        let win = Window::new(24, 80);
        assert_eq!(get_screen_item_line_and_col(&t, &win, i), (4, 9));
    }

    #[test]
    fn screen_item_relative_line_plus_and_origin() {
        // LINE PLUS 2 COLUMN PLUS 3, with screen origin (10, 20).
        let mut t = ScreenTree::new();
        let mut n = CobScreen::default();
        n.field = Some(PosField::new(0, 4));
        n.line = Some(PosField::new(2, 2));
        n.column = Some(PosField::new(3, 2));
        n.attr = COB_SCREEN_LINE_PLUS | COB_SCREEN_COLUMN_PLUS;
        let i = t.push(n);
        let mut win = Window::new(24, 80);
        win.origin_y = 10;
        win.origin_x = 20;
        // line: +2 (relative) + origin 10 = 12 ; col: +3 + origin 20 = 23.
        assert_eq!(get_screen_item_line_and_col(&t, &win, i), (12, 23));
    }

    // ---- get_line_column --------------------------------------------------------------------

    #[test]
    fn get_line_column_defaults_and_values() {
        assert_eq!(get_line_column(None, None), (1, 1));
        assert_eq!(
            get_line_column(Some(&PosField::new(7, 2)), Some(&PosField::new(9, 2))),
            (7, 9)
        );
        assert_eq!(get_line_column(Some(&PosField::new(4, 2)), None), (4, 1));
    }

    // ---- saved cursors ----------------------------------------------------------------------

    #[test]
    fn last_stmt_cursor_selects_by_statement() {
        let s = SavedCursors { display_y: 3, display_x: 11, accept_y: 7, accept_x: 22 };
        assert_eq!(line_where_last_stmt_ended(&s, ScreenStatement::Display), 3);
        assert_eq!(col_where_last_stmt_ended(&s, ScreenStatement::Display), 11);
        assert_eq!(line_where_last_stmt_ended(&s, ScreenStatement::Accept), 7);
        assert_eq!(col_where_last_stmt_ended(&s, ScreenStatement::Accept), 22);
    }

    // ---- get_line_and_col_from_field --------------------------------------------------------

    #[test]
    fn line_col_from_field_size4_and_6_numeric() {
        // size 4: max=100, 0512 (decimal 512) -> line 5, col 12.
        let f4 = CursorField { size: 4, int_val: 512, data: vec![], numeric: true };
        let r = get_line_and_col_from_field(&f4).unwrap();
        assert_eq!((r.line, r.column, r.max_line_column), (5, 12, 100));
        // size 6: max=1000, 005034 -> line 5, col 34.
        let f6 = CursorField { size: 6, int_val: 5034, data: vec![], numeric: true };
        let r = get_line_and_col_from_field(&f6).unwrap();
        assert_eq!((r.line, r.column, r.max_line_column), (5, 34, 1000));
    }

    #[test]
    fn line_col_from_field_nonnumeric_parsed() {
        // non-numeric size-6 "005034" parsed as 5034.
        let f = CursorField { size: 6, int_val: 0, data: b"005034".to_vec(), numeric: false };
        let r = get_line_and_col_from_field(&f).unwrap();
        assert_eq!((r.line, r.column), (5, 34));
        // non-numeric with bad size (max_line_column == 1) -> fatal boundary.
        let bad = CursorField { size: 5, int_val: 0, data: b"00050".to_vec(), numeric: false };
        assert_eq!(get_line_and_col_from_field(&bad), Err(GeomFatal));
    }

    // ---- get_cursor_from_program ------------------------------------------------------------

    #[test]
    fn cursor_from_program_zero_based_and_absent() {
        let f = CursorField { size: 4, int_val: 512, data: vec![], numeric: true };
        // line 5, col 12 -> zero-based (4, 11).
        assert_eq!(get_cursor_from_program(Some(&f)), Ok((4, 11)));
        // no cursor field -> (-1, -1).
        assert_eq!(get_cursor_from_program(None), Ok((-1, -1)));
    }

    // ---- pass_cursor_to_program -------------------------------------------------------------

    #[test]
    fn pass_cursor_encodings() {
        // no field
        assert_eq!(pass_cursor_to_program(None, 0, 0, false), CursorWriteback::None);
        // numeric non-display: (y=4,x=11) -> 1-based (5,12) -> 5*1000+12 = 5012.
        let f = CursorField { size: 6, int_val: 0, data: vec![], numeric: true };
        assert_eq!(
            pass_cursor_to_program(Some(&f), 4, 11, true),
            CursorWriteback::SetInt(5012)
        );
        // size 4 display: 5*100+12 = 512 -> "0512".
        let f4 = CursorField { size: 4, int_val: 0, data: vec![], numeric: false };
        assert_eq!(
            pass_cursor_to_program(Some(&f4), 4, 11, false),
            CursorWriteback::Bytes4(*b"0512")
        );
        // size 6 display: 5*1000+12 = 5012 -> "005012".
        let f6 = CursorField { size: 6, int_val: 0, data: vec![], numeric: false };
        assert_eq!(
            pass_cursor_to_program(Some(&f6), 4, 11, false),
            CursorWriteback::Bytes6(*b"005012")
        );
        // other size -> fatal.
        let f5 = CursorField { size: 5, int_val: 0, data: vec![], numeric: false };
        assert_eq!(
            pass_cursor_to_program(Some(&f5), 4, 11, false),
            CursorWriteback::Fatal
        );
    }

    // ---- raise_ec_on_* ----------------------------------------------------------------------

    #[test]
    fn ec_invalid_line_or_col() {
        let win = Window::new(24, 80);
        assert_eq!(raise_ec_on_invalid_line_or_col(&win, 5, 10), EcOutcome::default());
        let e = raise_ec_on_invalid_line_or_col(&win, -1, 80);
        assert!(e.line_number && e.starting_column);
        let e2 = raise_ec_on_invalid_line_or_col(&win, 23, 79);
        assert!(!e2.line_number && !e2.starting_column);
        let e3 = raise_ec_on_invalid_line_or_col(&win, 24, 79);
        assert!(e3.line_number && !e3.starting_column);
    }

    #[test]
    fn ec_truncation() {
        let mut win = Window::new(24, 80);
        win.x = 78;
        // x + size - 1 > max_x : 78 + 3 - 1 = 80 > 80? no.
        assert!(!raise_ec_on_truncation(&win, 3).item_truncated);
        // 78 + 4 - 1 = 81 > 80 -> truncate.
        assert!(raise_ec_on_truncation(&win, 4).item_truncated);
    }

    // ---- extract_line_and_col_vals ----------------------------------------------------------

    #[test]
    fn extract_both_null_is_origin() {
        let saved = SavedCursors::default();
        let r = extract_line_and_col_vals(None, None, None, ScreenStatement::Display, true, &saved)
            .unwrap();
        assert_eq!((r.sline, r.scolumn), (0, 0));
    }

    #[test]
    fn extract_line_only_small_size() {
        // only LINE, size < 4: bare LINE n, col defaults 1 -> sline = n-1, scol = 0.
        let saved = SavedCursors::default();
        let line = PosField::new(5, 2);
        let r = extract_line_and_col_vals(Some(&line), None, None, ScreenStatement::Display, false, &saved)
            .unwrap();
        assert_eq!((r.sline, r.scolumn), (4, 0));
    }

    #[test]
    fn extract_line_packed() {
        // only LINE with size >= 4: packed line*100+col. size 4 value 512 -> line5 col12 -> (4, 11).
        let saved = SavedCursors::default();
        let line = PosField::new(512, 4);
        let packed = CursorField { size: 4, int_val: 512, data: vec![], numeric: true };
        let r = extract_line_and_col_vals(
            Some(&line),
            None,
            Some(&packed),
            ScreenStatement::Display,
            false,
            &saved,
        )
        .unwrap();
        assert_eq!((r.sline, r.scolumn), (4, 11));
    }

    #[test]
    fn extract_both_present() {
        let saved = SavedCursors::default();
        let line = PosField::new(3, 2);
        let col = PosField::new(7, 2);
        let r = extract_line_and_col_vals(
            Some(&line),
            Some(&col),
            None,
            ScreenStatement::Display,
            false,
            &saved,
        )
        .unwrap();
        assert_eq!((r.sline, r.scolumn), (2, 6));
    }

    #[test]
    fn extract_zero_line_col_relative() {
        // line 0 col 0, allowed -> saved cursor; not allowed -> exceptions + (0,0).
        let saved = SavedCursors { display_y: 9, display_x: 4, accept_y: 0, accept_x: 0 };
        let line = PosField::new(0, 2);
        let col = PosField::new(0, 2);
        let r = extract_line_and_col_vals(
            Some(&line),
            Some(&col),
            None,
            ScreenStatement::Display,
            true,
            &saved,
        )
        .unwrap();
        assert_eq!((r.sline, r.scolumn), (9, 4));
        let r2 = extract_line_and_col_vals(
            Some(&line),
            Some(&col),
            None,
            ScreenStatement::Display,
            false,
            &saved,
        )
        .unwrap();
        assert_eq!((r2.sline, r2.scolumn), (0, 0));
        assert!(r2.ec.line_number && r2.ec.starting_column);
    }

    #[test]
    fn extract_line_zero_col_present_relative() {
        // line 0, col 5, allowed -> sline = last_line + 1, scol = col - 1.
        let saved = SavedCursors { display_y: 2, display_x: 0, accept_y: 0, accept_x: 0 };
        let line = PosField::new(0, 2);
        let col = PosField::new(5, 2);
        let r = extract_line_and_col_vals(
            Some(&line),
            Some(&col),
            None,
            ScreenStatement::Display,
            true,
            &saved,
        )
        .unwrap();
        assert_eq!((r.sline, r.scolumn), (3, 4));
    }

    #[test]
    fn extract_col_zero_relative() {
        // line 4, col 0, allowed -> sline = 3, scol = saved col.
        let saved = SavedCursors { display_y: 0, display_x: 17, accept_y: 0, accept_x: 0 };
        let line = PosField::new(4, 2);
        let col = PosField::new(0, 2);
        let r = extract_line_and_col_vals(
            Some(&line),
            Some(&col),
            None,
            ScreenStatement::Display,
            true,
            &saved,
        )
        .unwrap();
        assert_eq!((r.sline, r.scolumn), (3, 17));
    }

    // ---- cob_convert_key --------------------------------------------------------------------

    #[test]
    fn convert_key_control_bytes() {
        let cfg = KeyConvertConfig::default();
        for &b in &[0x0a, 0x0d, 0x04, 0x1a] {
            assert_eq!(cob_convert_key(b, false, &cfg), keys::KEY_ENTER);
        }
        assert_eq!(cob_convert_key(0x09, false, &cfg), keys::KEY_STAB);
        assert_eq!(cob_convert_key(0x08, false, &cfg), keys::KEY_BACKSPACE);
        assert_eq!(cob_convert_key(0o177, false, &cfg), keys::KEY_BACKSPACE);
    }

    #[test]
    fn convert_key_keypad_and_alt() {
        let cfg = KeyConvertConfig::default();
        assert_eq!(cob_convert_key(keys::KEY_EOL, false, &cfg), keys::ALT_DEL);
        assert_eq!(cob_convert_key(keys::KEY_CLOSE, false, &cfg), keys::ALT_LEFT);
        assert_eq!(cob_convert_key(keys::KEY_PREVIOUS, false, &cfg), keys::ALT_LEFT);
        assert_eq!(cob_convert_key(keys::KEY_A1, false, &cfg), keys::KEY_HOME);
        assert_eq!(cob_convert_key(keys::KEY_C1, false, &cfg), keys::KEY_END);
    }

    #[test]
    fn convert_key_ignore_filter() {
        let off = KeyConvertConfig { extended_status: false, use_esc: false };
        let on = KeyConvertConfig { extended_status: true, use_esc: true };
        // ESC ignored unless extended_status && use_esc.
        assert_eq!(cob_convert_key(0x1b, false, &off), 0);
        assert_eq!(cob_convert_key(0x1b, false, &on), 0x1b);
        // PPAGE/NPAGE/PRINT ignored unless extended_status.
        assert_eq!(cob_convert_key(keys::KEY_PPAGE, false, &off), 0);
        assert_eq!(cob_convert_key(keys::KEY_PPAGE, false, &on), keys::KEY_PPAGE);
        // A3 maps to PPAGE then gets filtered when not extended.
        assert_eq!(cob_convert_key(keys::KEY_A3, false, &off), 0);
        // UP/DOWN ignored during field accept unless extended.
        assert_eq!(cob_convert_key(keys::KEY_UP, true, &off), 0);
        assert_eq!(cob_convert_key(keys::KEY_UP, false, &off), keys::KEY_UP);
        assert_eq!(cob_convert_key(keys::KEY_UP, true, &on), keys::KEY_UP);
    }

    // ---- mouse_to_exception_code ------------------------------------------------------------

    #[test]
    fn mouse_codes_base() {
        use mouse::*;
        assert_eq!(mouse_to_exception_code(BUTTON1_PRESSED), 2041);
        assert_eq!(mouse_to_exception_code(BUTTON1_CLICKED), 2041);
        assert_eq!(mouse_to_exception_code(BUTTON1_RELEASED), 2042);
        assert_eq!(mouse_to_exception_code(BUTTON1_DOUBLE_CLICKED), 2043);
        assert_eq!(mouse_to_exception_code(BUTTON2_PRESSED), 2044);
        assert_eq!(mouse_to_exception_code(BUTTON3_RELEASED), 2048);
        assert_eq!(mouse_to_exception_code(BUTTON4_PRESSED), 2080);
        assert_eq!(mouse_to_exception_code(BUTTON5_PRESSED), 2081);
        assert_eq!(mouse_to_exception_code(0), 2040); // moved
    }

    #[test]
    fn mouse_codes_modifiers() {
        use mouse::*;
        // button1 press (2041) + shift -> +10 = 2051.
        assert_eq!(mouse_to_exception_code(BUTTON1_PRESSED | BUTTON_SHIFT), 2051);
        // + ctrl -> +20.
        assert_eq!(mouse_to_exception_code(BUTTON1_PRESSED | BUTTON_CTRL), 2061);
        // + alt -> +30.
        assert_eq!(mouse_to_exception_code(BUTTON1_PRESSED | BUTTON_ALT), 2071);
        // wheel (>=2080) shift -> +4.
        assert_eq!(mouse_to_exception_code(BUTTON4_PRESSED | BUTTON_SHIFT), 2084);
        assert_eq!(mouse_to_exception_code(BUTTON4_PRESSED | BUTTON_CTRL), 2088);
        assert_eq!(mouse_to_exception_code(BUTTON5_PRESSED | BUTTON_ALT), 2093);
    }
}
