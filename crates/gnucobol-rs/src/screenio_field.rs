//! Port of the `screenio.c` **field-validation + numeric-insert helpers** -- the predicates GnuCOBOL's
//! `ACCEPT` engine runs over a `SCREEN SECTION` field's data bytes and its `PIC` while a field is being
//! entered or finalized. They answer "is this field empty / zero / full?", "does the `FULL`/`REQUIRED`
//! clause hold?", "is the data valid for its type?", and -- for numeric *insert* mode -- "where does the
//! decimal point / a `PIC` symbol sit, and where should the cursor land?".
//!
//! ## Modeling boundary (read this)
//!
//! In libcob these helpers reach through a `cob_screen *`/`cob_field *` into the runtime field descriptor
//! (`f->attr->type`, `f->attr->pic[]`, `f->data`, `f->size`) and global module state
//! (`COB_MODULE_PTR->decimal_point` / `numeric_separator`, `cobsetptr->cob_insert_mode`). A pure,
//! panic-free port models exactly those touched members as plain owned data: [`CobField`] (type + flags +
//! pic + data), [`CobScreen`] (a field plus the `FULL`/`REQUIRED` attribute bits), and [`ModuleState`]
//! (the decimal-point / separator chars + insert-mode flag). No I/O, no ncurses, no globals.
//!
//! `valid_field_data` calls libcob's `cob_check_numval` (the full NUMVAL byte parser, which lives in the
//! intrinsic bucket); here it is supplied as an injected `numval_check` result so the *control flow* of
//! `valid_field_data` (its NUMERIC-EDITED all-spaces -> single `'0'` special case) is reproduced exactly
//! without re-porting the parser. `shift_left`/`shift_right` (screen-mutating) are deliberately **not**
//! ported here -- they belong to the data-shift bucket.
//!
//! Every `pub fn` carries the exact C name from `libcob/screenio.c`; the doc comment cites it.
#![forbid(unsafe_code)]

// ASCII char constants (the C literals appear as raw chars; we spell them numerically per convention).
const CH_SPACE: u8 = 0x20; // ' '
const CH_ZERO: u8 = 0x30; // '0'
const CH_NINE: u8 = 0x39; // '9'
const CH_Z: u8 = 0x5A; // 'Z' (zero-suppression)
const CH_STAR: u8 = 0x2A; // '*' (check-protect / zero-suppression)
const CH_B: u8 = 0x42; // 'B' (insertion blank)
const CH_SLASH: u8 = 0x2F; // '/' (insertion slash)

// Field type codes mirrored from `common.h` (only the ones these helpers test).
/// `COB_TYPE_NUMERIC` base bit (`0x10`). `COB_FIELD_IS_NUMERIC` is `type & this`.
pub const COB_TYPE_NUMERIC: u8 = 0x10;
/// `COB_TYPE_NUMERIC_EDITED` (`0x24`).
pub const COB_TYPE_NUMERIC_EDITED: u8 = 0x24;
/// `COB_TYPE_ALPHANUMERIC` (`0x21`) -- a representative non-numeric type for tests.
pub const COB_TYPE_ALPHANUMERIC: u8 = 0x21;

// Screen attribute bits mirrored from `common.h` (only `FULL`/`REQUIRED`, which these helpers test).
/// `COB_SCREEN_FULL` (`1 << 11`).
pub const COB_SCREEN_FULL: u64 = 1 << 11;
/// `COB_SCREEN_REQUIRED` (`1 << 14`).
pub const COB_SCREEN_REQUIRED: u64 = 1 << 14;

// Field flag bits (only the two that move `COB_FIELD_DATA`/`COB_FIELD_SIZE`).
/// `COB_FLAG_SIGN_SEPARATE` (`1 << 1`).
pub const COB_FLAG_SIGN_SEPARATE: u32 = 1 << 1;
/// `COB_FLAG_SIGN_LEADING` (`1 << 2`).
pub const COB_FLAG_SIGN_LEADING: u32 = 1 << 2;

/// A single `PIC` symbol cell, mirroring `cob_pic_symbol { char symbol; int times_repeated; }`. The C
/// `pic[]` array is `'\0'`-terminated; here the terminator is the end of the [`Vec`] (so a loop
/// `while pic[i].symbol != '\0'` becomes a slice iteration).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PicSymbol {
    /// The PICTURE character (`'9'`, `'Z'`, `'*'`, the decimal point, `'+'`, `'-'`, `'S'`, ...).
    pub symbol: u8,
    /// How many consecutive positions this symbol covers (`times_repeated`).
    pub times_repeated: i32,
}

impl PicSymbol {
    /// Convenience constructor.
    pub fn new(symbol: u8, times_repeated: i32) -> Self {
        PicSymbol { symbol, times_repeated }
    }
}

/// The members of a `cob_field` (and its `attr`) these helpers actually read: the runtime type, the field
/// flags (for the sign-separate/leading data+size adjustment), the `'\0'`-terminated `pic[]` (modeled as a
/// non-terminated [`Vec`]), and the raw data bytes. `size` is `data.len()` plus any separate-sign byte, so
/// it is stored explicitly to mirror `f->size` exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobField {
    /// `f->attr->type`.
    pub type_: u8,
    /// `f->attr->flags`.
    pub flags: u32,
    /// `f->attr->pic` (the `'\0'`-terminated symbol array, terminator omitted).
    pub pic: Vec<PicSymbol>,
    /// `f->data` (the raw on-screen bytes).
    pub data: Vec<u8>,
}

impl CobField {
    /// `COB_FIELD_TYPE (f)` -- `f->attr->type`.
    pub fn field_type(&self) -> u8 {
        self.type_
    }

    /// `COB_FIELD_IS_NUMERIC (f)` -- `COB_FIELD_TYPE (f) & COB_TYPE_NUMERIC`.
    pub fn is_numeric(&self) -> bool {
        (self.type_ & COB_TYPE_NUMERIC) != 0
    }

    /// `f->size` -- the full byte length of the field (data plus any separate-sign byte). Modeled as the
    /// data length (the test fixtures store the on-screen bytes directly in `data`).
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// `COB_FIELD_SIGN_SEPARATE (f)` -- `flags & COB_FLAG_SIGN_SEPARATE`.
    fn sign_separate(&self) -> bool {
        (self.flags & COB_FLAG_SIGN_SEPARATE) != 0
    }

    /// `COB_FIELD_SIGN_LEADSEP (f)` -- both `SIGN_SEPARATE` and `SIGN_LEADING` set.
    fn sign_leadsep(&self) -> bool {
        const LEADSEP: u32 = COB_FLAG_SIGN_SEPARATE | COB_FLAG_SIGN_LEADING;
        (self.flags & LEADSEP) == LEADSEP
    }

    /// `COB_FIELD_DATA (f)` -- `f->data + (leadsep ? 1 : 0)` (the numeric digits, past a leading sign byte).
    fn field_data(&self) -> &[u8] {
        if self.sign_leadsep() {
            &self.data[1.min(self.data.len())..]
        } else {
            &self.data
        }
    }

    /// `COB_FIELD_SIZE (f)` -- `sign_separate ? size - 1 : size`.
    fn field_size(&self) -> usize {
        if self.sign_separate() {
            self.size().saturating_sub(1)
        } else {
            self.size()
        }
    }
}

/// The members of a `cob_screen` these helpers read: its field plus the attribute bitmask carrying the
/// `FULL`/`REQUIRED` clause flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobScreen {
    /// `s->field`.
    pub field: CobField,
    /// `s->attr` (the screen-item attribute bitmask).
    pub attr: u64,
}

/// One entry of the `ACCEPT` input-field array (`struct cob_inp_struct`): the field plus its 1-based screen
/// origin `this_y`/`this_x`. `find_field_by_pos` searches a slice of these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CobInpStruct {
    /// `structure->scr` -- the screen item (only its field's `size` is read, for `right_pos`).
    pub scr: CobScreen,
    /// `structure->this_y` -- the field's screen line.
    pub this_y: i32,
    /// `structure->this_x` -- the field's screen start column.
    pub this_x: i32,
}

/// The `COB_MODULE_PTR` / `cobsetptr` globals these helpers consult: the runtime decimal point and numeric
/// separator characters, and the insert-mode flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleState {
    /// `COB_MODULE_PTR->decimal_point`.
    pub decimal_point: u8,
    /// `COB_MODULE_PTR->numeric_separator`.
    pub numeric_separator: u8,
    /// `cobsetptr->cob_insert_mode` (`COB_INSERT_MODE`).
    pub insert_mode: bool,
}

impl Default for ModuleState {
    /// The common US default: `'.'` decimal point, `','` separator, insert mode off.
    fn default() -> Self {
        ModuleState { decimal_point: b'.', numeric_separator: b',', insert_mode: false }
    }
}

/// C `isspace` for the byte values libcob's screen data carries (ASCII whitespace).
fn isspace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// C `isdigit`.
fn isdigit(c: u8) -> bool {
    c.is_ascii_digit()
}

/// `cob_field_is_numeric_or_numeric_edited` (`screenio.c`): numeric *or* numeric-edited.
///
/// `COB_FIELD_IS_NUMERIC (field) || COB_FIELD_TYPE (field) == COB_TYPE_NUMERIC_EDITED`.
pub fn cob_field_is_numeric_or_numeric_edited(field: &CobField) -> bool {
    field.is_numeric() || field.field_type() == COB_TYPE_NUMERIC_EDITED
}

/// `field_is_empty` (`screenio.c`): every byte of `s->field` is whitespace.
pub fn field_is_empty(s: &CobScreen) -> bool {
    let data = &s.field.data;
    for &b in data.iter() {
        if !isspace(b) {
            return false;
        }
    }
    true
}

/// `field_is_zero` (`screenio.c`): every byte is whitespace, `'0'`, or the runtime decimal point.
pub fn field_is_zero(s: &CobScreen, module: &ModuleState) -> bool {
    let data = &s.field.data;
    let decimal_point = module.decimal_point;
    for &b in data.iter() {
        if !(isspace(b) || b == CH_ZERO || b == decimal_point) {
            return false;
        }
    }
    true
}

/// `pic_has_zero_suppression` (`screenio.c`): the `PIC` contains a `Z` or `*` (zero-suppression) symbol.
///
/// `+`/`-` are floating-insertion editing chars, not zero-suppression, so they do not count.
pub fn pic_has_zero_suppression(pic: &[PicSymbol]) -> bool {
    for sym in pic.iter() {
        if sym.symbol == CH_Z || sym.symbol == CH_STAR {
            return true;
        }
    }
    false
}

/// `get_num_int_digits_for_no_zero_sup` (`screenio.c`): count the integer digit positions (`9`/`Z`/`*`,
/// summing `times_repeated`); a non-separator/non-`B`/`0`/`/` symbol *after* digits were seen ends the
/// integer part.
pub fn get_num_int_digits_for_no_zero_sup(pic: &[PicSymbol], module: &ModuleState) -> i32 {
    let mut num_digits: i32 = 0;
    let numeric_separator = module.numeric_separator;

    for sym in pic.iter() {
        if sym.symbol == CH_NINE || sym.symbol == CH_Z || sym.symbol == CH_STAR {
            num_digits += sym.times_repeated;
        } else if !(sym.symbol == numeric_separator
            || sym.symbol == CH_B
            || sym.symbol == CH_ZERO
            || sym.symbol == CH_SLASH)
            && num_digits != 0
        {
            break;
        }
    }

    num_digits
}

/// `field_is_zero_or_no_zero_suppression` (`screenio.c`): for a numeric-edited field, either the data is
/// zero or the `PIC` has no zero-suppression, OR enough non-zero-suppressed integer digits are present to
/// fill the integer part.
pub fn field_is_zero_or_no_zero_suppression(s: &CobScreen, module: &ModuleState) -> bool {
    let pic = &s.field.pic;
    let size = s.field.field_size();
    let data = s.field.field_data();
    let mut num_digits_seen: i32 = 0;

    if field_is_zero(s, module) || !pic_has_zero_suppression(pic) {
        return true;
    }

    let num_integer_digits = get_num_int_digits_for_no_zero_sup(pic, module);

    // Verify there are sufficient non-zero digits before a decimal point / the end to fill the integer
    // part of the field.
    for i in 0..size {
        let b = data.get(i).copied().unwrap_or(0);
        if isdigit(b) {
            if b != CH_ZERO || num_digits_seen != 0 {
                num_digits_seen += 1;
            }
        } else if !isspace(b) && num_digits_seen != 0 {
            break;
        }
    }

    num_digits_seen >= num_integer_digits
}

/// `field_is_full` (`screenio.c`): per the standard only the first and last chars of an alphanumeric field
/// need be non-space.
pub fn field_is_full(s: &CobScreen) -> bool {
    let data = &s.field.data;
    let size = data.len();
    if size == 0 {
        // C dereferences `*data` and `*(data+size-1)`; an empty field never reaches here in libcob.
        // We treat an empty field as not full (both ends "space").
        return false;
    }
    !isspace(data[0]) && !isspace(data[size - 1])
}

/// `satisfied_full_clause` (`screenio.c`): the `FULL` clause holds (trivially true if not set).
///
/// Numeric: data is non-zero. Numeric-edited: zero-or-no-zero-suppression. Alphanumeric: full or empty.
pub fn satisfied_full_clause(s: &CobScreen, module: &ModuleState) -> bool {
    if (s.attr & COB_SCREEN_FULL) == 0 {
        return true;
    }

    if s.field.is_numeric() {
        !field_is_zero(s, module)
    } else if s.field.field_type() == COB_TYPE_NUMERIC_EDITED {
        field_is_zero_or_no_zero_suppression(s, module)
    } else {
        // field is alphanumeric
        field_is_full(s) || field_is_empty(s)
    }
}

/// `satisfied_required_clause` (`screenio.c`): the `REQUIRED` clause holds (trivially true if not set).
///
/// Numeric/numeric-edited: data is non-zero. Alphanumeric: data is non-empty.
pub fn satisfied_required_clause(s: &CobScreen, module: &ModuleState) -> bool {
    if (s.attr & COB_SCREEN_REQUIRED) == 0 {
        return true;
    }

    if cob_field_is_numeric_or_numeric_edited(&s.field) {
        !field_is_zero(s, module)
    } else {
        // field is alphanumeric
        !field_is_empty(s)
    }
}

/// `valid_field_data` (`screenio.c`): the field's data is valid for its type.
///
/// Numeric: `cob_check_numval(field, NULL, 0, 0) == 0`. Numeric-edited: `cob_check_numval(field, NULL, 1,
/// 0)`; the all-spaces special case (`num_check == field->size + 1`) is treated valid and the first byte is
/// set to `'0'`. Non-numeric: always valid.
///
/// `cob_check_numval` (the full NUMVAL parser) lives in the intrinsic bucket; its result is injected here
/// as `numval_check` -- a function `(field, chkcurr) -> i32` returning 0 for valid or `bad_offset + 1`.
/// This reproduces `valid_field_data`'s control flow exactly without re-porting the parser. The field is
/// taken by value and the possibly-mutated copy is returned alongside the validity verdict (the C code
/// writes `field->data[0] = '0'` in place; the caller observes that mutation in the returned field).
pub fn valid_field_data<F>(mut field: CobField, mut numval_check: F) -> (bool, CobField)
where
    F: FnMut(&CobField, i32) -> i32,
{
    if field.is_numeric() {
        let valid = numval_check(&field, 0) == 0;
        (valid, field)
    } else if field.field_type() == COB_TYPE_NUMERIC_EDITED {
        let num_check = numval_check(&field, 1);
        // Test for all spaces, which is valid in this case, and change to a single zero instead.
        if num_check == field.size() as i32 + 1 {
            if !field.data.is_empty() {
                field.data[0] = CH_ZERO;
            }
            return (true, field);
        }
        let valid = numval_check(&field, 1) == 0;
        (valid, field)
    } else {
        (true, field)
    }
}

/// `find_field_by_pos` (`screenio.c`): search the input-field array from `initial_curs` for the field whose
/// screen rectangle contains `(line, column)`; returns its index or `-1`.
///
/// `right_pos = scolumn + field.size - 1` (the `SET_FLD_REFS` macro). Match requires `line == sline &&
/// scolumn <= column <= right_pos`.
pub fn find_field_by_pos(
    fields: &[CobInpStruct],
    initial_curs: i32,
    line: i32,
    column: i32,
) -> i32 {
    let start = if initial_curs < 0 { 0usize } else { initial_curs as usize };
    for idx in start..fields.len() {
        let structure = &fields[idx];
        let sline = structure.this_y;
        let scolumn = structure.this_x;
        let right_pos = scolumn + structure.scr.field.size() as i32 - 1;
        if line == sline && column >= scolumn && column <= right_pos {
            return idx as i32;
        }
    }
    -1
}

/// `is_number_with_pic_symbol` (`screenio.c`): the field is numeric-edited and its `PIC` contains `symbol`.
pub fn is_number_with_pic_symbol(f: &CobField, symbol: u8) -> bool {
    if f.field_type() != COB_TYPE_NUMERIC_EDITED {
        return false;
    }
    for sym in f.pic.iter() {
        if sym.symbol == symbol {
            return true;
        }
    }
    false
}

/// `has_decimal_point` (`screenio.c`): the numeric-edited `PIC` contains the runtime decimal point symbol.
pub fn has_decimal_point(f: &CobField, module: &ModuleState) -> bool {
    is_number_with_pic_symbol(f, module.decimal_point)
}

/// `get_pic_symbol_offset` (`screenio.c`): the data-column offset of `symbol` within the field -- the sum
/// of `times_repeated` for all `PIC` cells up to and including the matching one, minus 1.
///
/// Mirrors the C `do { offset += pic[i].times_repeated; } while (pic[i++].symbol != symbol)` exactly: it
/// always consumes at least the first cell, and stops *after* the cell whose symbol matches. If no cell
/// matches it would walk off the `'\0'` terminator in C; here we stop at the end of the slice (returning
/// the running offset minus 1) to stay panic-free -- callers only invoke this when the symbol is known
/// present (guarded by `has_decimal_point` / `is_number_with_pic_symbol`).
pub fn get_pic_symbol_offset(f: &CobField, symbol: u8) -> i32 {
    let mut offset: i32 = 0;
    let mut i = 0usize;
    loop {
        if i >= f.pic.len() {
            break;
        }
        offset += f.pic[i].times_repeated;
        let matched = f.pic[i].symbol == symbol;
        i += 1;
        if matched {
            break;
        }
    }
    offset - 1
}

/// `get_decimal_point_offset` (`screenio.c`): the data-column offset of the decimal point symbol.
pub fn get_decimal_point_offset(f: &CobField, module: &ModuleState) -> i32 {
    get_pic_symbol_offset(f, module.decimal_point)
}

/// `at_offset_from_decimal_point` (`screenio.c`): the cursor column `ccolumn` sits exactly `offset`
/// positions from the decimal point (relative to the field start `scolumn`), and the field has one.
pub fn at_offset_from_decimal_point(
    f: &CobField,
    scolumn: i32,
    ccolumn: i32,
    offset: i32,
    module: &ModuleState,
) -> bool {
    has_decimal_point(f, module)
        && ccolumn == offset + scolumn + get_decimal_point_offset(f, module)
}

/// The result of `move_to_initial_field_pos`: the data-pointer offset into `f->data` (`*data_ptr`
/// advance) and the target cursor `(line, column)` that the C `cob_move_cursor` is called with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitialFieldPos {
    /// How many bytes `*data_ptr` was advanced from `f->data`.
    pub data_offset: i32,
    /// The cursor line passed to `cob_move_cursor` (always `sline`).
    pub cursor_line: i32,
    /// The cursor column passed to `cob_move_cursor`.
    pub cursor_column: i32,
}

/// `move_to_initial_field_pos` (`screenio.c`): compute the data-pointer offset and target cursor position
/// when entering a field.
///
/// Insert mode + numeric/numeric-edited: with a decimal point, land just right of it (leftmost decimal
/// digit) when `to_right_side`, else just left of it (rightmost integer digit); without a decimal point,
/// land at the rightmost integer digit (`right_pos`). Otherwise land at the start (`scolumn`) or end
/// (`right_pos`) of the field. The C function returns `cob_move_cursor`'s int result and writes `*data_ptr`
/// by side effect; this returns both as an [`InitialFieldPos`].
pub fn move_to_initial_field_pos(
    f: &CobField,
    sline: i32,
    scolumn: i32,
    right_pos: i32,
    to_right_side: bool,
    module: &ModuleState,
) -> InitialFieldPos {
    if module.insert_mode && cob_field_is_numeric_or_numeric_edited(f) {
        if has_decimal_point(f, module) {
            let offset = if to_right_side {
                // At leftmost decimal digit.
                get_decimal_point_offset(f, module) + 1
            } else {
                // At rightmost integer digit.
                get_decimal_point_offset(f, module) - 1
            };
            InitialFieldPos {
                data_offset: offset,
                cursor_line: sline,
                cursor_column: scolumn + offset,
            }
        } else {
            // At rightmost integer digit.
            InitialFieldPos {
                data_offset: right_pos - scolumn,
                cursor_line: sline,
                cursor_column: right_pos,
            }
        }
    } else {
        // At start/end of field.
        if to_right_side {
            InitialFieldPos {
                data_offset: right_pos - scolumn,
                cursor_line: sline,
                cursor_column: right_pos,
            }
        } else {
            InitialFieldPos {
                data_offset: 0,
                cursor_line: sline,
                cursor_column: scolumn,
            }
        }
    }
}

/// `can_insert_left` (`screenio.c`): whether a left-insert is allowed at cursor column `ccolumn` given the
/// current leftmost field char.
///
/// Numeric/numeric-edited: with a decimal point, only when the cursor is at or left of it (`ccolumn <=
/// decimal_point_offset`), and then only when the leftmost char is `'0'` or space; otherwise blocked.
/// Non-numeric: only when the leftmost char is a space.
pub fn can_insert_left(
    f: &CobField,
    leftmost_char: u8,
    ccolumn: i32,
    module: &ModuleState,
) -> bool {
    if cob_field_is_numeric_or_numeric_edited(f) {
        if has_decimal_point(f, module) && ccolumn > get_decimal_point_offset(f, module) {
            false
        } else {
            leftmost_char == CH_ZERO || leftmost_char == CH_SPACE
        }
    } else {
        leftmost_char == CH_SPACE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alnum(data: &[u8]) -> CobField {
        CobField { type_: COB_TYPE_ALPHANUMERIC, flags: 0, pic: vec![], data: data.to_vec() }
    }
    fn numeric(data: &[u8]) -> CobField {
        CobField { type_: COB_TYPE_NUMERIC, flags: 0, pic: vec![], data: data.to_vec() }
    }
    fn numedit(pic: Vec<PicSymbol>, data: &[u8]) -> CobField {
        CobField { type_: COB_TYPE_NUMERIC_EDITED, flags: 0, pic, data: data.to_vec() }
    }
    fn scr(field: CobField, attr: u64) -> CobScreen {
        CobScreen { field, attr }
    }

    #[test]
    fn numeric_or_numeric_edited() {
        assert!(cob_field_is_numeric_or_numeric_edited(&numeric(b"0")));
        assert!(cob_field_is_numeric_or_numeric_edited(&numedit(vec![], b"0")));
        assert!(!cob_field_is_numeric_or_numeric_edited(&alnum(b"A")));
    }

    #[test]
    fn empty_and_zero() {
        let m = ModuleState::default();
        assert!(field_is_empty(&scr(alnum(b"   "), 0)));
        assert!(!field_is_empty(&scr(alnum(b" A "), 0)));
        // zero: spaces, '0', and decimal point all count as zero.
        assert!(field_is_zero(&scr(numeric(b" 00.0 "), 0), &m));
        assert!(!field_is_zero(&scr(numeric(b" 01 "), 0), &m));
    }

    #[test]
    fn pic_zero_suppression() {
        assert!(pic_has_zero_suppression(&[PicSymbol::new(CH_Z, 3)]));
        assert!(pic_has_zero_suppression(&[PicSymbol::new(CH_STAR, 2)]));
        // '+'/'-' are floating-insertion, not zero-suppression.
        assert!(!pic_has_zero_suppression(&[PicSymbol::new(b'+', 1), PicSymbol::new(CH_NINE, 4)]));
        assert!(!pic_has_zero_suppression(&[PicSymbol::new(CH_NINE, 4)]));
    }

    #[test]
    fn num_int_digits_for_no_zero_sup() {
        let m = ModuleState::default();
        // ZZ9 -> 3 integer digit positions.
        let pic = vec![PicSymbol::new(CH_Z, 2), PicSymbol::new(CH_NINE, 1)];
        assert_eq!(get_num_int_digits_for_no_zero_sup(&pic, &m), 3);
        // ZZ9.99 -> the decimal point ends the integer part after digits seen (3 integer digits).
        let pic = vec![
            PicSymbol::new(CH_Z, 2),
            PicSymbol::new(CH_NINE, 1),
            PicSymbol::new(b'.', 1),
            PicSymbol::new(CH_NINE, 2),
        ];
        assert_eq!(get_num_int_digits_for_no_zero_sup(&pic, &m), 3);
        // a leading separator/'B'/'0'/'/' before any digit does not terminate.
        let pic = vec![PicSymbol::new(CH_B, 1), PicSymbol::new(CH_Z, 2)];
        assert_eq!(get_num_int_digits_for_no_zero_sup(&pic, &m), 2);
    }

    #[test]
    fn zero_or_no_zero_suppression() {
        let m = ModuleState::default();
        // ZZ9 with full digits "123": enough integer digits -> satisfied.
        let pic = vec![PicSymbol::new(CH_Z, 2), PicSymbol::new(CH_NINE, 1)];
        let s = scr(numedit(pic.clone(), b"123"), 0);
        assert!(field_is_zero_or_no_zero_suppression(&s, &m));
        // ZZ9 with "  7": only one integer digit seen, needs 3 -> NOT satisfied.
        let s = scr(numedit(pic.clone(), b"  7"), 0);
        assert!(!field_is_zero_or_no_zero_suppression(&s, &m));
        // all-zero data is trivially satisfied.
        let s = scr(numedit(pic, b"  0"), 0);
        assert!(field_is_zero_or_no_zero_suppression(&s, &m));
        // no zero-suppression in PIC -> always satisfied.
        let s = scr(numedit(vec![PicSymbol::new(CH_NINE, 3)], b"  7"), 0);
        assert!(field_is_zero_or_no_zero_suppression(&s, &m));
    }

    #[test]
    fn full_predicate() {
        assert!(field_is_full(&scr(alnum(b"AbC"), 0))); // ends non-space
        assert!(!field_is_full(&scr(alnum(b" bC"), 0))); // leading space
        assert!(!field_is_full(&scr(alnum(b"Ab "), 0))); // trailing space
        assert!(field_is_full(&scr(alnum(b"A C"), 0))); // interior space is fine
    }

    #[test]
    fn full_clause() {
        let m = ModuleState::default();
        // not set -> always satisfied.
        assert!(satisfied_full_clause(&scr(numeric(b"000"), 0), &m));
        // numeric + FULL: non-zero required.
        assert!(satisfied_full_clause(&scr(numeric(b"012"), COB_SCREEN_FULL), &m));
        assert!(!satisfied_full_clause(&scr(numeric(b"000"), COB_SCREEN_FULL), &m));
        // alphanumeric + FULL: full or empty.
        assert!(satisfied_full_clause(&scr(alnum(b"AbC"), COB_SCREEN_FULL), &m));
        assert!(satisfied_full_clause(&scr(alnum(b"   "), COB_SCREEN_FULL), &m));
        assert!(!satisfied_full_clause(&scr(alnum(b" b "), COB_SCREEN_FULL), &m));
    }

    #[test]
    fn required_clause() {
        let m = ModuleState::default();
        assert!(satisfied_required_clause(&scr(numeric(b"000"), 0), &m)); // not set
        assert!(satisfied_required_clause(&scr(numeric(b"012"), COB_SCREEN_REQUIRED), &m));
        assert!(!satisfied_required_clause(&scr(numeric(b"000"), COB_SCREEN_REQUIRED), &m));
        assert!(satisfied_required_clause(&scr(alnum(b" A "), COB_SCREEN_REQUIRED), &m));
        assert!(!satisfied_required_clause(&scr(alnum(b"   "), COB_SCREEN_REQUIRED), &m));
    }

    #[test]
    fn valid_field_data_paths() {
        // non-numeric: always valid, no check called.
        let (ok, _) = valid_field_data(alnum(b"xyz"), |_, _| panic!("should not be called"));
        assert!(ok);
        // numeric valid (check returns 0).
        let (ok, _) = valid_field_data(numeric(b"123"), |_, _| 0);
        assert!(ok);
        // numeric invalid (check returns bad offset+1).
        let (ok, _) = valid_field_data(numeric(b"1a3"), |_, _| 2);
        assert!(!ok);
        // numeric-edited all-spaces special case: check returns size+1, data[0] set to '0', valid.
        let f = numedit(vec![PicSymbol::new(CH_NINE, 3)], b"   ");
        let sz = f.size() as i32;
        let (ok, out) = valid_field_data(f, move |_, _| sz + 1);
        assert!(ok);
        assert_eq!(out.data[0], CH_ZERO);
        // numeric-edited otherwise: second check decides.
        let (ok, _) = valid_field_data(numedit(vec![PicSymbol::new(CH_NINE, 3)], b"012"), |_, _| 0);
        assert!(ok);
    }

    #[test]
    fn find_by_pos() {
        let mk = |y, x, size: usize| CobInpStruct {
            scr: scr(alnum(&vec![b' '; size]), 0),
            this_y: y,
            this_x: x,
        };
        let fields = vec![mk(1, 1, 3), mk(2, 5, 4), mk(3, 1, 2)];
        // (2, 6) is inside field 1 (cols 5..=8).
        assert_eq!(find_field_by_pos(&fields, 0, 2, 6), 1);
        // (1, 1) is inside field 0.
        assert_eq!(find_field_by_pos(&fields, 0, 1, 1), 0);
        // (2, 9) is past field 1's right edge -> not found.
        assert_eq!(find_field_by_pos(&fields, 0, 2, 9), -1);
        // initial_curs skips earlier fields.
        assert_eq!(find_field_by_pos(&fields, 1, 1, 1), -1);
    }

    #[test]
    fn pic_symbol_inspection() {
        let m = ModuleState::default();
        // 999.99 numeric-edited: has '.', not present in a plain numeric.
        let pic = vec![PicSymbol::new(CH_NINE, 3), PicSymbol::new(b'.', 1), PicSymbol::new(CH_NINE, 2)];
        let f = numedit(pic, b"123.45");
        assert!(is_number_with_pic_symbol(&f, b'.'));
        assert!(!is_number_with_pic_symbol(&f, b'+'));
        assert!(has_decimal_point(&f, &m));
        // a plain numeric is never "number with pic symbol".
        assert!(!is_number_with_pic_symbol(&numeric(b"123"), b'.'));
    }

    #[test]
    fn pic_symbol_offsets() {
        let m = ModuleState::default();
        // 999.99: '.' is after 3 digit positions -> offset = 3+1 - 1 = 3.
        let pic = vec![PicSymbol::new(CH_NINE, 3), PicSymbol::new(b'.', 1), PicSymbol::new(CH_NINE, 2)];
        let f = numedit(pic, b"123.45");
        assert_eq!(get_decimal_point_offset(&f, &m), 3);
        assert_eq!(get_pic_symbol_offset(&f, b'.'), 3);
        // ZZ9V99 style with leading symbol: '9' at index 0 -> offset 0.
        let pic2 = vec![PicSymbol::new(CH_NINE, 1), PicSymbol::new(b'.', 1)];
        let f2 = numedit(pic2, b"1.");
        assert_eq!(get_pic_symbol_offset(&f2, CH_NINE), 0);
    }

    #[test]
    fn offset_from_decimal_point() {
        let m = ModuleState::default();
        let pic = vec![PicSymbol::new(CH_NINE, 3), PicSymbol::new(b'.', 1), PicSymbol::new(CH_NINE, 2)];
        let f = numedit(pic, b"123.45");
        // scolumn=10, dp offset=3 -> dp at column 13. offset 0 from dp => ccolumn 13.
        assert!(at_offset_from_decimal_point(&f, 10, 13, 0, &m));
        // offset 1 from dp (leftmost decimal digit) => ccolumn 14.
        assert!(at_offset_from_decimal_point(&f, 10, 14, 1, &m));
        assert!(!at_offset_from_decimal_point(&f, 10, 12, 0, &m));
        // no decimal point -> always false.
        assert!(!at_offset_from_decimal_point(&numeric(b"123"), 10, 13, 0, &m));
    }

    #[test]
    fn initial_field_pos() {
        let mut m = ModuleState::default();
        let pic = vec![PicSymbol::new(CH_NINE, 3), PicSymbol::new(b'.', 1), PicSymbol::new(CH_NINE, 2)];
        let f = numedit(pic, b"123.45");
        // insert mode off: to_right_side=false -> start of field.
        let p = move_to_initial_field_pos(&f, 5, 10, 15, false, &m);
        assert_eq!(p, InitialFieldPos { data_offset: 0, cursor_line: 5, cursor_column: 10 });
        // insert mode off: to_right_side -> end of field (right_pos).
        let p = move_to_initial_field_pos(&f, 5, 10, 15, true, &m);
        assert_eq!(p, InitialFieldPos { data_offset: 5, cursor_line: 5, cursor_column: 15 });
        // insert mode on, has decimal point (offset 3), to right side -> leftmost decimal digit (offset+1=4).
        m.insert_mode = true;
        let p = move_to_initial_field_pos(&f, 5, 10, 15, true, &m);
        assert_eq!(p, InitialFieldPos { data_offset: 4, cursor_line: 5, cursor_column: 14 });
        // insert mode on, has dp, not right side -> rightmost integer digit (offset-1=2).
        let p = move_to_initial_field_pos(&f, 5, 10, 15, false, &m);
        assert_eq!(p, InitialFieldPos { data_offset: 2, cursor_line: 5, cursor_column: 12 });
        // insert mode on, numeric without decimal point -> rightmost integer digit (right_pos).
        let p = move_to_initial_field_pos(&numeric(b"123"), 5, 10, 12, false, &m);
        assert_eq!(p, InitialFieldPos { data_offset: 2, cursor_line: 5, cursor_column: 12 });
    }

    #[test]
    fn insert_left() {
        let m = ModuleState::default();
        let pic = vec![PicSymbol::new(CH_NINE, 3), PicSymbol::new(b'.', 1), PicSymbol::new(CH_NINE, 2)];
        let f = numedit(pic, b"123.45");
        // numeric-edited with dp (offset 3): ccolumn past dp -> blocked.
        assert!(!can_insert_left(&f, b'0', 4, &m));
        // at/left of dp with leftmost '0' -> allowed.
        assert!(can_insert_left(&f, b'0', 2, &m));
        // at/left of dp with leftmost ' ' -> allowed.
        assert!(can_insert_left(&f, b' ', 2, &m));
        // at/left of dp with leftmost non-zero digit -> blocked.
        assert!(!can_insert_left(&f, b'5', 2, &m));
        // non-numeric: only a leading space allows insert.
        assert!(can_insert_left(&alnum(b" bc"), b' ', 1, &m));
        assert!(!can_insert_left(&alnum(b"abc"), b'a', 1, &m));
    }
}
