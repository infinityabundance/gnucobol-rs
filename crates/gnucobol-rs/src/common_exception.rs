//! Port of the runtime **EXCEPTION-state machinery** of `libcob/common.c` -- the `cob_*_exception`
//! family that records, queries and names the last-raised COBOL exception condition (the `EC-*`
//! conditions, e.g. `EC-BOUND-SUBSCRIPT`).
//!
//! In libcob this state is two pieces of global mutable storage:
//!   * `cobglobptr->cob_exception_code` -- the *current* exception bitmask (reset to 0 at the start of
//!     each statement that monitors exceptions, see `common.c:5776` / `5826`), and
//!   * the file-static `last_exception_code` -- the *last* raised exception (only reset on
//!     `SET LAST EXCEPTION TO OFF`).
//! plus the two generated tables `cob_exception_tab_code[]` / `cob_exception_tab_name[]` (built from
//! `exception.def` via the `COB_EXCEPTION(code, tag, name, critical)` X-macro). Following the panel
//! guidance and the `MlModule` precedent, the global state is modelled here **explicitly** as the
//! [`ExceptionState`] struct; the tables are pure `const` data; every function is the faithful 1:1 of
//! its `common.c` original. No global statics, no `unsafe`, panic-free (every table access is
//! bounds-guarded).
//!
//! The exception `code` is a 16-bit-ish bitmask whose high byte is the category (e.g. `0x02xx` =
//! `EC-BOUND`) and whose low byte distinguishes the specific condition. A "category" code such as
//! `EC-BOUND` (`0x0200`) matches any of its members under `code & tab_code == tab_code`, which is how
//! [`cob_last_exception_is`] / [`cob_get_last_exception_name`] resolve the most specific name.

#![forbid(unsafe_code)]

/// The generated exception table -- the `(code, name)` pairs of `cob_exception_tab_code[]` /
/// `cob_exception_tab_name[]`, built from `libcob/exception.def`. Index `0` is `COB_EC_ZERO`
/// (`{0, "None"}`) and the final index is `COB_EC_MAX` (`{0, "Invalid"}`); the entries between are the
/// `COB_EXCEPTION(...)` rows of `exception.def`, in file order, so the array index equals the
/// `enum cob_exception_id` value passed as `id` to [`cob_add_exception`] / [`cob_set_exception`].
pub const EXCEPTION_TAB: &[(i32, &str)] = &[
    (0x0000, "None"),
    (0xFFFF, "EC-ALL"),
    (0x0100, "EC-ARGUMENT"),
    (0x0101, "EC-ARGUMENT-FUNCTION"),
    (0x0102, "EC-ARGUMENT-IMP"),
    (0x0200, "EC-BOUND"),
    (0x0209, "EC-BOUND-FUNC-RET-VALUE"),
    (0x0201, "EC-BOUND-IMP"),
    (0x0202, "EC-BOUND-ODO"),
    (0x0203, "EC-BOUND-OVERFLOW"),
    (0x0204, "EC-BOUND-PTR"),
    (0x0205, "EC-BOUND-REF-MOD"),
    (0x0206, "EC-BOUND-SET"),
    (0x0207, "EC-BOUND-SUBSCRIPT"),
    (0x0208, "EC-BOUND-TABLE-LIMIT"),
    (0x1800, "EC-CONTINUE"),
    (0x1801, "EC-CONTINUE-IMP"),
    (0x1802, "EC-CONTINUE-LESS-THAN-ZERO"),
    (0x0300, "EC-DATA"),
    (0x0301, "EC-DATA-CONVERSION"),
    (0x0302, "EC-DATA-IMP"),
    (0x0303, "EC-DATA-INCOMPATIBLE"),
    (0x0304, "EC-DATA-NOT-FINITE"),
    (0x0305, "EC-DATA-OVERFLOW"),
    (0x0306, "EC-DATA-PTR-NULL"),
    (0x1900, "EC-EXTERNAL"),
    (0x1901, "EC-EXTERNAL-DATA-MISMATCH"),
    (0x1902, "EC-EXTERNAL-FILE-MISMATCH"),
    (0x1903, "EC-EXTERNAL-FORMAT-CONFLICT"),
    (0x1904, "EC-EXTERNAL-IMP"),
    (0x0400, "EC-FLOW"),
    (0x040A, "EC-FLOW-APPLY-COMMIT"),
    (0x040B, "EC-FLOW-COMMIT"),
    (0x0401, "EC-FLOW-GLOBAL-EXIT"),
    (0x0402, "EC-FLOW-GLOBAL-GOBACK"),
    (0x0403, "EC-FLOW-IMP"),
    (0x0404, "EC-FLOW-RELEASE"),
    (0x0405, "EC-FLOW-REPORT"),
    (0x0406, "EC-FLOW-RETURN"),
    (0x040C, "EC-FLOW-ROLLBACK"),
    (0x0407, "EC-FLOW-SEARCH"),
    (0x0408, "EC-FLOW-USE"),
    (0x1500, "EC-FUNCTION"),
    (0x1503, "EC-FUNCTION-ARG-OMITTED"),
    (0x1503, "EC-FUNCTION-IMP"),
    (0x1504, "EC-FUNCTION-NOT-FOUND"),
    (0x1501, "EC-FUNCTION-PTR-INVALID"),
    (0x1502, "EC-FUNCTION-PTR-NULL"),
    (0x0600, "EC-IMP"),
    (0x0601, "EC-IMP-ACCEPT"),
    (0x0602, "EC-IMP-DISPLAY"),
    (0x0603, "EC-IMP-UTC-UNKNOWN"),
    (0x0604, "EC-IMP-FEATURE-DISABLED"),
    (0x0605, "EC-IMP-FEATURE-MISSING"),
    (0x0700, "EC-LOCALE"),
    (0x0701, "EC-LOCALE-IMP"),
    (0x0702, "EC-LOCALE-INCOMPATIBLE"),
    (0x0703, "EC-LOCALE-INVALID"),
    (0x0704, "EC-LOCALE-INVALID-PTR"),
    (0x0705, "EC-LOCALE-MISSING"),
    (0x0706, "EC-LOCALE-SIZE"),
    (0x1A00, "EC-MCS"),
    (0x1A01, "EC-MCS-ABNORMAL-TERMINATION"),
    (0x1A02, "EC-MCS-IMP"),
    (0x1A03, "EC-MCS-INVALID-TAG"),
    (0x1A04, "EC-MCS-MESSAGE-LENGTH"),
    (0x1A05, "EC-MCS-NO-REQUESTER"),
    (0x1A06, "EC-MCS-NO-SERVER"),
    (0x1A07, "EC-MCS-NORMAL-TERMINATION"),
    (0x1A08, "EC-MCS-REQUESTOR-FAILED"),
    (0x0800, "EC-OO"),
    (0x0803, "EC-OO-ARG-OMITTED"),
    (0x0801, "EC-OO-CONFORMANCE"),
    (0x0802, "EC-OO-EXCEPTION"),
    (0x0804, "EC-OO-IMP"),
    (0x0805, "EC-OO-METHOD"),
    (0x0806, "EC-OO-NULL"),
    (0x0807, "EC-OO-RESOURCE"),
    (0x0808, "EC-OO-UNIVERSAL"),
    (0x0900, "EC-ORDER"),
    (0x0901, "EC-ORDER-IMP"),
    (0x0902, "EC-ORDER-NOT-SUPPORTED"),
    (0x0A00, "EC-OVERFLOW"),
    (0x0A01, "EC-OVERFLOW-IMP"),
    (0x0A02, "EC-OVERFLOW-STRING"),
    (0x0A03, "EC-OVERFLOW-UNSTRING"),
    (0x0B00, "EC-PROGRAM"),
    (0x0B01, "EC-PROGRAM-ARG-MISMATCH"),
    (0x0B02, "EC-PROGRAM-ARG-OMITTED"),
    (0x0B03, "EC-PROGRAM-CANCEL-ACTIVE"),
    (0x0B04, "EC-PROGRAM-IMP"),
    (0x0B05, "EC-PROGRAM-NOT-FOUND"),
    (0x0B06, "EC-PROGRAM-PTR-NULL"),
    (0x0B07, "EC-PROGRAM-RECURSIVE-CALL"),
    (0x0B08, "EC-PROGRAM-RESOURCES"),
    (0x0C00, "EC-RAISING"),
    (0x0C01, "EC-RAISING-IMP"),
    (0x0C02, "EC-RAISING-NOT-SPECIFIED"),
    (0x0D00, "EC-RANGE"),
    (0x0D01, "EC-RANGE-IMP"),
    (0x0D02, "EC-RANGE-INDEX"),
    (0x0D03, "EC-RANGE-INSPECT-SIZE"),
    (0x0D04, "EC-RANGE-INVALID"),
    (0x0D05, "EC-RANGE-PERFORM-VARYING"),
    (0x0D06, "EC-RANGE-PTR"),
    (0x0D07, "EC-RANGE-SEARCH-INDEX"),
    (0x0D08, "EC-RANGE-SEARCH-NO-MATCH"),
    (0x0E00, "EC-REPORT"),
    (0x0E01, "EC-REPORT-ACTIVE"),
    (0x0E02, "EC-REPORT-COLUMN-OVERLAP"),
    (0x0E03, "EC-REPORT-FILE-MODE"),
    (0x0E04, "EC-REPORT-IMP"),
    (0x0E05, "EC-REPORT-INACTIVE"),
    (0x0E06, "EC-REPORT-LINE-OVERLAP"),
    (0x0E08, "EC-REPORT-NOT-TERMINATED"),
    (0x0E09, "EC-REPORT-PAGE-LIMIT"),
    (0x0E0A, "EC-REPORT-PAGE-WIDTH"),
    (0x0E0B, "EC-REPORT-SUM-SIZE"),
    (0x0E0C, "EC-REPORT-VARYING"),
    (0x0F00, "EC-SCREEN"),
    (0x0F01, "EC-SCREEN-FIELD-OVERLAP"),
    (0x0F02, "EC-SCREEN-IMP"),
    (0x0F03, "EC-SCREEN-ITEM-TRUNCATED"),
    (0x0F04, "EC-SCREEN-LINE-NUMBER"),
    (0x0F05, "EC-SCREEN-STARTING-COLUMN"),
    (0x1000, "EC-SIZE"),
    (0x1001, "EC-SIZE-ADDRESS"),
    (0x1002, "EC-SIZE-EXPONENTIATION"),
    (0x1003, "EC-SIZE-IMP"),
    (0x1004, "EC-SIZE-OVERFLOW"),
    (0x1005, "EC-SIZE-TRUNCATION"),
    (0x1006, "EC-SIZE-UNDERFLOW"),
    (0x1007, "EC-SIZE-ZERO-DIVIDE"),
    (0x1100, "EC-SORT-MERGE"),
    (0x1101, "EC-SORT-MERGE-ACTIVE"),
    (0x1102, "EC-SORT-MERGE-FILE-OPEN"),
    (0x1103, "EC-SORT-MERGE-IMP"),
    (0x1104, "EC-SORT-MERGE-RELEASE"),
    (0x1105, "EC-SORT-MERGE-RETURN"),
    (0x1106, "EC-SORT-MERGE-SEQUENCE"),
    (0x1200, "EC-STORAGE"),
    (0x1201, "EC-STORAGE-IMP"),
    (0x1202, "EC-STORAGE-NOT-ALLOC"),
    (0x1203, "EC-STORAGE-NOT-AVAIL"),
    (0x1300, "EC-USER"),
    (0x1400, "EC-VALIDATE"),
    (0x1401, "EC-VALIDATE-CONTENT"),
    (0x1402, "EC-VALIDATE-FORMAT"),
    (0x1403, "EC-VALIDATE-IMP"),
    (0x1404, "EC-VALIDATE-RELATION"),
    (0x1405, "EC-VALIDATE-VARYING"),
    (0x1600, "EC-XML"),
    (0x1601, "EC-XML-CODESET"),
    (0x1602, "EC-XML-CODESET-CONVERSION"),
    (0x1603, "EC-XML-COUNT"),
    (0x1604, "EC-XML-DOCUMENT-TYPE"),
    (0x1605, "EC-XML-IMPLICIT-CLOSE"),
    (0x1606, "EC-XML-INVALID"),
    (0x1607, "EC-XML-NAMESPACE"),
    (0x1608, "EC-XML-STACKED-OPEN"),
    (0x1609, "EC-XML-RANGE"),
    (0x1610, "EC-XML-IMP"),
    (0x1700, "EC-JSON"),
    (0x1710, "EC-JSON-IMP"),
    (0x0000, "Invalid"),
];

/// `common.c:EXCEPTION_TAB_SIZE` -- the number of entries in [`EXCEPTION_TAB`]
/// (`sizeof cob_exception_tab_code / sizeof(int)`).
pub const EXCEPTION_TAB_SIZE: usize = EXCEPTION_TAB.len();

// Selected `enum cob_exception_id` indices used by name resolution / status translation. These are the
// array indices into EXCEPTION_TAB (== the enum values), taken from exception.def's file order.
/// `COB_EC_IMP_FEATURE_DISABLED` index.
pub const COB_EC_IMP_FEATURE_DISABLED: usize = 52;
/// `COB_EC_IMP_FEATURE_MISSING` index.
pub const COB_EC_IMP_FEATURE_MISSING: usize = 53;
/// `COB_EC_PROGRAM` index.
pub const COB_EC_PROGRAM: usize = 86;
/// `COB_EC_PROGRAM_NOT_FOUND` index.
pub const COB_EC_PROGRAM_NOT_FOUND: usize = 91;
/// `COB_EC_PROGRAM_RESOURCES` index.
pub const COB_EC_PROGRAM_RESOURCES: usize = 94;

/// The bitmask code (`cob_exception_tab_code[id]`) for an `enum cob_exception_id` index, bounds-guarded
/// (out-of-range ids yield `0`, mirroring the absence of any matching bit; C would index out of bounds).
pub fn tab_code(id: usize) -> i32 {
    match EXCEPTION_TAB.get(id) {
        Some(&(code, _)) => code,
        None => 0,
    }
}

/// The name (`cob_exception_tab_name[id]`) for an `enum cob_exception_id` index, bounds-guarded.
pub fn tab_name(id: usize) -> &'static str {
    match EXCEPTION_TAB.get(id) {
        Some(&(_, name)) => name,
        None => "Invalid",
    }
}

/// The runtime exception state of `libcob/common.c`, modelled explicitly (the C globals
/// `cobglobptr->cob_exception_code` and the file-static `last_exception_code`). Constructed empty
/// (`ExceptionState::default()`), mutated by [`cob_set_exception`] / [`cob_add_exception`], queried by
/// the `cob_*_exception*` accessors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExceptionState {
    /// `cobglobptr->cob_exception_code` -- the current statement's exception bitmask (reset to 0 at the
    /// start of each exception-monitoring statement in libcob).
    pub cob_exception_code: i32,
    /// The file-static `last_exception_code` -- the last raised exception (only reset on
    /// `SET LAST EXCEPTION TO OFF`).
    pub last_exception_code: i32,
    /// `cobglobptr->cob_got_exception` -- set when a (nonzero) exception is raised.
    pub cob_got_exception: i32,
}

impl ExceptionState {
    /// A fresh, empty exception state (all codes `0`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Port of `common.c:cob_set_exception` -- set the last/current exception to the single condition
    /// `id` (overwriting, not OR-ing). `cob_got_exception` is `1` for a nonzero `id`, `0` for
    /// `COB_EC_ZERO` (`id == 0`). The location-tracking fields (`last_exception_line`, statement,
    /// module/section/paragraph names) are libcob's `cob_module`-walking side effects, outside this
    /// pure-state port; only the exception codes and the got-exception flag are modelled.
    pub fn cob_set_exception(&mut self, id: usize) {
        self.cob_exception_code = tab_code(id);
        self.last_exception_code = self.cob_exception_code;
        if id != 0 {
            self.cob_got_exception = 1;
        } else {
            self.cob_got_exception = 0;
        }
    }

    /// Port of `common.c:cob_add_exception` -- add the condition `id` to the current/last exception
    /// state, OR-ing its bit in; if the current code is empty this is exactly [`Self::cob_set_exception`].
    pub fn cob_add_exception(&mut self, id: usize) {
        if self.cob_exception_code == 0 {
            self.cob_set_exception(id);
            return;
        }
        self.cob_exception_code |= tab_code(id);
        self.last_exception_code |= tab_code(id);
    }

    /// Port of `common.c:cob_get_last_exception_code` -- the last raised exception code (`0` if none
    /// active).
    pub fn cob_get_last_exception_code(&self) -> i32 {
        self.last_exception_code
    }

    /// Port of `common.c:cob_last_exception_is` -- is the last exception set and does it include the
    /// condition at index `exception_to_check`? (`last_exception_code & tab_code == tab_code`). Returns
    /// `1`/`0`, as the C `int`.
    pub fn cob_last_exception_is(&self, exception_to_check: usize) -> i32 {
        let code = tab_code(exception_to_check);
        if (self.last_exception_code & code) == code {
            1
        } else {
            0
        }
    }

    /// Port of `common.c:cob_get_last_exception_name` -- the name of the last raised exception. First a
    /// direct equality match against every table code; then, preferring the most specific, the
    /// missing/disabled feature conditions, then a high-to-low bitmask (`&`) match (the C returns
    /// `NULL` only if nothing matches). Because the final table entry `COB_EC_MAX` has code `0`, the
    /// `(code & 0) == 0` bitmask test always succeeds against it, so an empty / unmatched state yields
    /// `"Invalid"` (faithful to the C, which then returns `cob_exception_tab_name[COB_EC_MAX]`); `None`
    /// is therefore effectively unreachable, kept to mirror the C's `return NULL;`.
    pub fn cob_get_last_exception_name(&self) -> Option<&'static str> {
        // direct match (skip index 0)
        for n in 1..EXCEPTION_TAB_SIZE {
            if self.last_exception_code == EXCEPTION_TAB[n].0 {
                return Some(EXCEPTION_TAB[n].1);
            }
        }
        // any match (most specific first; missing/disabled checked first)
        if self.cob_last_exception_is(COB_EC_IMP_FEATURE_MISSING) != 0 {
            return Some(tab_name(COB_EC_IMP_FEATURE_MISSING));
        }
        if self.cob_last_exception_is(COB_EC_IMP_FEATURE_DISABLED) != 0 {
            return Some(tab_name(COB_EC_IMP_FEATURE_DISABLED));
        }
        let mut n = EXCEPTION_TAB_SIZE - 1;
        while n != 0 {
            let code = EXCEPTION_TAB[n].0;
            if (self.last_exception_code & code) == code {
                return Some(EXCEPTION_TAB[n].1);
            }
            n -= 1;
        }
        None
    }

    /// Port of `common.c:cob_accept_exception_status` -- the value `ACCEPT ... FROM EXCEPTION STATUS`
    /// returns. For a 3-byte USAGE DISPLAY numeric receiving field libcob performs a Micro Focus
    /// translation (`COB_EC_PROGRAM_RESOURCES -> 1`, `COB_EC_PROGRAM_NOT_FOUND -> 2`, else `128`);
    /// otherwise the raw `last_exception_code` is returned. The receiving field is modelled abstractly
    /// by `(field_size, is_numeric_display)`; the C then `cob_set_int`s the result into the field --
    /// here that integer is the return value (the caller stores it).
    ///
    /// FAITHFULNESS NOTE: the C's third branch is `else if (exception || cob_exception_tab_code[COB_EC_PROGRAM])`
    /// -- a `||` against a compile-time-nonzero table value, so the branch is always taken once the
    /// earlier two have not. This is almost certainly an upstream bug (a `&` / membership test was
    /// intended), but it is ported exactly: any other nonzero exception maps to `128`.
    pub fn cob_accept_exception_status(&self, field_size: usize, is_numeric_display: bool) -> i32 {
        let mut exception = self.last_exception_code;
        if exception != 0 && field_size == 3 && is_numeric_display {
            if exception == tab_code(COB_EC_PROGRAM_RESOURCES) {
                exception = 1;
            } else if exception == tab_code(COB_EC_PROGRAM_NOT_FOUND) {
                exception = 2;
            } else if exception != 0 || tab_code(COB_EC_PROGRAM) != 0 {
                exception = 128;
            }
        }
        exception
    }
}

// Free-function forms matching the exact C names, operating on an explicit &mut/& ExceptionState (the
// `cobglobptr`/`last_exception_code` the C reaches through globals). The port-index matches by name.

/// Port of `common.c:cob_set_exception` -- see [`ExceptionState::cob_set_exception`].
pub fn cob_set_exception(state: &mut ExceptionState, id: usize) {
    state.cob_set_exception(id);
}

/// Port of `common.c:cob_add_exception` -- see [`ExceptionState::cob_add_exception`].
pub fn cob_add_exception(state: &mut ExceptionState, id: usize) {
    state.cob_add_exception(id);
}

/// Port of `common.c:cob_get_last_exception_code` -- see [`ExceptionState::cob_get_last_exception_code`].
pub fn cob_get_last_exception_code(state: &ExceptionState) -> i32 {
    state.cob_get_last_exception_code()
}

/// Port of `common.c:cob_get_last_exception_name` -- see [`ExceptionState::cob_get_last_exception_name`].
pub fn cob_get_last_exception_name(state: &ExceptionState) -> Option<&'static str> {
    state.cob_get_last_exception_name()
}

/// Port of `common.c:cob_last_exception_is` -- see [`ExceptionState::cob_last_exception_is`].
pub fn cob_last_exception_is(state: &ExceptionState, exception_to_check: usize) -> i32 {
    state.cob_last_exception_is(exception_to_check)
}

/// Port of `common.c:cob_accept_exception_status` -- see [`ExceptionState::cob_accept_exception_status`].
pub fn cob_accept_exception_status(state: &ExceptionState, field_size: usize, is_numeric_display: bool) -> i32 {
    state.cob_accept_exception_status(field_size, is_numeric_display)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Index helper: find the table index of a name (for tests; tab order is the enum order).
    fn idx(name: &str) -> usize {
        EXCEPTION_TAB.iter().position(|&(_, n)| n == name).unwrap()
    }

    #[test]
    fn table_shape_and_anchors() {
        // 0 = COB_EC_ZERO, last = COB_EC_MAX
        assert_eq!(EXCEPTION_TAB[0], (0x0000, "None"));
        assert_eq!(EXCEPTION_TAB[EXCEPTION_TAB_SIZE - 1], (0x0000, "Invalid"));
        // EXCEPTION_TAB_SIZE = COB_EC_ZERO + 163 def rows + COB_EC_MAX
        assert_eq!(EXCEPTION_TAB_SIZE, 165);
        // a few well-known codes
        assert_eq!(tab_code(idx("EC-BOUND-SUBSCRIPT")), 0x0207);
        assert_eq!(tab_code(idx("EC-BOUND")), 0x0200);
        assert_eq!(tab_code(idx("EC-ALL")), 0xFFFF);
        // pinned index constants line up with the named entries
        assert_eq!(tab_name(COB_EC_PROGRAM), "EC-PROGRAM");
        assert_eq!(tab_name(COB_EC_PROGRAM_NOT_FOUND), "EC-PROGRAM-NOT-FOUND");
        assert_eq!(tab_name(COB_EC_PROGRAM_RESOURCES), "EC-PROGRAM-RESOURCES");
        assert_eq!(tab_name(COB_EC_IMP_FEATURE_MISSING), "EC-IMP-FEATURE-MISSING");
        assert_eq!(tab_name(COB_EC_IMP_FEATURE_DISABLED), "EC-IMP-FEATURE-DISABLED");
    }

    #[test]
    fn tab_accessors_bounds_guard() {
        assert_eq!(tab_code(99999), 0);
        assert_eq!(tab_name(99999), "Invalid");
    }

    #[test]
    fn set_and_get_last_code() {
        let mut s = ExceptionState::new();
        assert_eq!(cob_get_last_exception_code(&s), 0);
        cob_set_exception(&mut s, idx("EC-BOUND-SUBSCRIPT"));
        assert_eq!(cob_get_last_exception_code(&s), 0x0207);
        assert_eq!(s.cob_exception_code, 0x0207);
        assert_eq!(s.cob_got_exception, 1);
        // COB_EC_ZERO clears got-exception
        cob_set_exception(&mut s, 0);
        assert_eq!(s.cob_got_exception, 0);
        assert_eq!(cob_get_last_exception_code(&s), 0);
    }

    #[test]
    fn add_exception_or_and_set_when_empty() {
        let mut s = ExceptionState::new();
        // empty -> behaves as set
        cob_add_exception(&mut s, idx("EC-BOUND-SUBSCRIPT"));
        assert_eq!(s.cob_exception_code, 0x0207);
        // OR a second condition in
        cob_add_exception(&mut s, idx("EC-SIZE-OVERFLOW")); // 0x1004
        assert_eq!(s.cob_exception_code, 0x0207 | 0x1004);
        assert_eq!(s.last_exception_code, 0x0207 | 0x1004);
    }

    #[test]
    fn last_exception_is_category_and_specific() {
        let mut s = ExceptionState::new();
        cob_set_exception(&mut s, idx("EC-BOUND-SUBSCRIPT")); // 0x0207
        // matches itself
        assert_eq!(cob_last_exception_is(&s, idx("EC-BOUND-SUBSCRIPT")), 1);
        // matches its category EC-BOUND (0x0200): 0x0207 & 0x0200 == 0x0200
        assert_eq!(cob_last_exception_is(&s, idx("EC-BOUND")), 1);
        // does not match a different category
        assert_eq!(cob_last_exception_is(&s, idx("EC-SIZE")), 0);
        // empty state matches nothing nonzero
        let e = ExceptionState::new();
        assert_eq!(cob_last_exception_is(&e, idx("EC-BOUND")), 0);
    }

    #[test]
    fn get_last_exception_name_direct_and_bitmask() {
        let mut s = ExceptionState::new();
        // direct equality match
        cob_set_exception(&mut s, idx("EC-BOUND-SUBSCRIPT"));
        assert_eq!(cob_get_last_exception_name(&s), Some("EC-BOUND-SUBSCRIPT"));
        // category code resolves to its own name
        cob_set_exception(&mut s, idx("EC-BOUND"));
        assert_eq!(cob_get_last_exception_name(&s), Some("EC-BOUND"));
        // empty state: the direct loop skips index 0, the feature checks fail, then the high-to-low
        // bitmask sweep hits the final entry first (COB_EC_MAX, code 0): (0 & 0) == 0 -> "Invalid".
        // This is faithful to the C (it returns cob_exception_tab_name[COB_EC_MAX] = "Invalid", not NULL).
        let e = ExceptionState::new();
        assert_eq!(cob_get_last_exception_name(&e), Some("Invalid"));
    }

    #[test]
    fn get_last_exception_name_feature_priority() {
        // last code carries the missing-feature bit plus extra bits with no exact table entry:
        // direct match fails, and the missing-feature check fires before the generic bitmask sweep.
        let mut s = ExceptionState::new();
        s.last_exception_code = tab_code(COB_EC_IMP_FEATURE_MISSING) | 0x0010; // 0x0615: no exact table entry
        assert_eq!(cob_get_last_exception_name(&s), Some("EC-IMP-FEATURE-MISSING"));
    }

    #[test]
    fn accept_exception_status_translation() {
        let mut s = ExceptionState::new();
        // PROGRAM_RESOURCES -> 1 for a 3-byte DISPLAY numeric field
        cob_set_exception(&mut s, COB_EC_PROGRAM_RESOURCES);
        assert_eq!(cob_accept_exception_status(&s, 3, true), 1);
        // PROGRAM_NOT_FOUND -> 2
        cob_set_exception(&mut s, COB_EC_PROGRAM_NOT_FOUND);
        assert_eq!(cob_accept_exception_status(&s, 3, true), 2);
        // any other nonzero exception -> 128 (the always-true || branch, ported faithfully)
        cob_set_exception(&mut s, idx("EC-BOUND-SUBSCRIPT"));
        assert_eq!(cob_accept_exception_status(&s, 3, true), 128);
        // non-3-byte / non-DISPLAY field returns the raw code
        assert_eq!(cob_accept_exception_status(&s, 5, true), 0x0207);
        assert_eq!(cob_accept_exception_status(&s, 3, false), 0x0207);
        // zero exception returns 0 untranslated
        let e = ExceptionState::new();
        assert_eq!(cob_accept_exception_status(&e, 3, true), 0);
    }
}
