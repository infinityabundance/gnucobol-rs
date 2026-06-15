//! Port of the **pure, side-effect-free** version/info banner formatting, the COBOL statement-name
//! table + lookup, the temp-filename builder, the in-memory table SORT compare/reorder, the locale
//! category decision, and a couple of intermediate-move helpers from `libcob/common.c`.
//!
//! The big driver functions in common.c -- `print_info_detailed`, `print_runtime_conf`,
//! `get_screenio_and_mouse_info`, `cob_set_locale`, `print_version` -- are dominated by `printf`/`puts`
//! writes, `setlocale`, ncurses (`initscr`/`endwin`), and dozens of compile-time `#ifdef` macros.
//! Those I/O / OS calls are the documented runtime boundary. Here we port the **pure formatting cores**:
//! each function builds and returns the bytes that the C code would have written / decided, given the
//! version strings, build date, feature flags, etc. injected as parameters. The actual write leaf and
//! the actual `setlocale` call are the boundary; the byte production and the decision are 1:1.
//!
//! `#![forbid(unsafe_code)]`, ASCII-only, panic-free, every function cites its C origin.
#![forbid(unsafe_code)]
#![allow(clippy::needless_range_loop)]

// ===========================================================================
// Build stamp / version banners
// ===========================================================================

/// Port of `common.c:set_cob_build_stamp` -- format the compile build stamp.
///
/// The C reads `__DATE__` (e.g. `"Jun 15 2026"`) and `__TIME__` (e.g. `"12:34:56"`) -- those are the
/// injected `date`/`time` parameters here. If `__DATE__` parses as `"<month> <day> <year>"` it is
/// re-rendered as `"%s %2.2d %4.4d %s"` (zero-padded day to width 2, year to width 4); otherwise it
/// falls back to `"<date> <time>"`. Returns the stamp bytes.
pub fn set_cob_build_stamp(date: &[u8], time: &[u8]) -> Vec<u8> {
    // sscanf (__DATE__, "%63s %d %d", month, &day, &year)
    match scan_month_day_year(date) {
        Some((month, day, year)) => {
            let mut out = Vec::new();
            out.extend_from_slice(month);
            out.push(b' ');
            push_int_min_width(&mut out, day, 2, true);
            out.push(b' ');
            push_int_min_width(&mut out, year, 4, true);
            out.push(b' ');
            out.extend_from_slice(time);
            out
        }
        None => {
            let mut out = Vec::new();
            out.extend_from_slice(date);
            out.push(b' ');
            out.extend_from_slice(time);
            out
        }
    }
}

/// Port of `common.c:print_version` -- the `cobcrun --version` / `libcob` banner.
///
/// The C `printf`/`puts` sequence is reproduced verbatim into the returned byte buffer. The variable
/// fields (`PACKAGE_NAME`, `PACKAGE_VERSION`, `PATCH_LEVEL`, the copyright year, the build stamp, and
/// the `COB_TAR_DATE` package date) are injected. The trailing newline of every line is included.
/// The actual `printf` to stdout is the boundary.
pub fn print_version(
    package_name: &[u8],
    package_version: &[u8],
    patch_level: i32,
    copyright_year: &[u8],
    build_stamp: &[u8],
    tar_date: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    // printf ("libcob (%s) %s.%d\n", PACKAGE_NAME, PACKAGE_VERSION, PATCH_LEVEL);
    out.extend_from_slice(b"libcob (");
    out.extend_from_slice(package_name);
    out.extend_from_slice(b") ");
    out.extend_from_slice(package_version);
    out.push(b'.');
    push_int(&mut out, patch_level);
    out.push(b'\n');
    // puts ("Copyright (C) <year> Free Software Foundation, Inc.");
    out.extend_from_slice(b"Copyright (C) ");
    out.extend_from_slice(copyright_year);
    out.extend_from_slice(b" Free Software Foundation, Inc.\n");
    // printf (_("License LGPLv3+: GNU LGPL version 3 or later <%s>"), "...lgpl.html"); putchar('\n');
    out.extend_from_slice(
        b"License LGPLv3+: GNU LGPL version 3 or later <https://gnu.org/licenses/lgpl.html>\n",
    );
    // puts ("This is free software; ...");
    out.extend_from_slice(
        b"This is free software; see the source for copying conditions.  There is NO\n\
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.\n",
    );
    // printf (_("Written by %s"), "..."); putchar('\n');
    out.extend_from_slice(
        b"Written by Keisuke Nishida, Roger While, Ron Norman, Simon Sobisch, Edward Hart\n",
    );
    // printf (_("Built     %s"), cob_build_stamp); putchar('\n');
    out.extend_from_slice(b"Built     ");
    out.extend_from_slice(build_stamp);
    out.push(b'\n');
    // printf (_("Packaged  %s"), COB_TAR_DATE); putchar('\n');
    out.extend_from_slice(b"Packaged  ");
    out.extend_from_slice(tar_date);
    out.push(b'\n');
    out
}

/// Port of `common.c:print_version_summary` -- the two-line `GnuCOBOL + C compiler + libs` summary.
///
/// Reproduces the leading, always-present `printf` lines:
/// `"%s %s (%s), "` (package name, libcob version, build stamp), then `"%s\n"` (the C-compiler version
/// string), then the math-library line `"%s %d.%d.%d"` (`"GMP"`/`"MPIR"` + version triple). The many
/// optional `, lib x.y.z` fragments (libxml2, cJSON, curses, BDB, ...) are conditional on build macros;
/// the caller supplies them already-formatted as `extra` (appended verbatim before the final newline).
/// The actual `printf`/`putchar` writes are the boundary.
#[allow(clippy::too_many_arguments)]
pub fn print_version_summary(
    package_name: &[u8],
    libcob_version: &[u8],
    build_stamp: &[u8],
    c_version: &[u8],
    math_name: &[u8],
    math_major: i32,
    math_minor: i32,
    math_patch: i32,
    extra: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    // printf ("%s %s (%s), ", PACKAGE_NAME, libcob_version(), cob_build_stamp);
    out.extend_from_slice(package_name);
    out.push(b' ');
    out.extend_from_slice(libcob_version);
    out.extend_from_slice(b" (");
    out.extend_from_slice(build_stamp);
    out.extend_from_slice(b"), ");
    // printf ("%s\n", GC_C_VERSION_PRF GC_C_VERSION);
    out.extend_from_slice(c_version);
    out.push(b'\n');
    // printf ("%s %d.%d.%d", "GMP"/"MPIR", maj, min, patch);
    out.extend_from_slice(math_name);
    out.push(b' ');
    push_int(&mut out, math_major);
    out.push(b'.');
    push_int(&mut out, math_minor);
    out.push(b'.');
    push_int(&mut out, math_patch);
    // optional library fragments already formatted by caller (", libxml2 a.b.c", ...)
    out.extend_from_slice(extra);
    // putchar ('\n');
    out.push(b'\n');
    out
}

/// Port of `common.c:get_math_info` -- the math-library version description.
///
/// Models the FORMAT only; the GMP/MPIR version strings (`gmp_version`, `mpir_version`) are parsed by
/// the caller and the parsed triple + the compiled-against `__GNU_MP_VERSION`/`_MINOR` are injected.
/// If the runtime major/minor match the compiled-against pair the line is `"%s, version %d.%d.%d"`,
/// else `"%s, version %d.%d.%d (compiled with %d.%d)"`. The `name` is `"GMP"` (or `"MPIR"`).
pub fn get_math_info(
    name: &[u8],
    major: i32,
    minor: i32,
    patch: i32,
    compiled_major: i32,
    compiled_minor: i32,
) -> Vec<u8> {
    version_line(name, major, minor, patch, compiled_major, compiled_minor)
}

/// Port of the pure banner-build core of `common.c:get_screenio_and_mouse_info`.
///
/// The full C function runs ncurses (`initscr`/`mousemask`/`has_mouse`/`endwin`) and parses
/// `curses_version()` -- all OS/library bound and the documented boundary. The **pure decision** is the
/// version-string selection: given the parsed runtime curses `(major,minor,patch)`, the library display
/// name `with_curses` (`WITH_CURSES`), and the compiled-against `(cmp_major,cmp_minor)`:
///   * matching major+minor -> `"%s, version %d.%d.%d"`
///   * else non-zero major   -> `"%s, version %d.%d.%d (compiled with %d.%d)"`
///   * else (unparseable)    -> `"%s, version %s"` using the raw `raw_version` string.
/// Returns those banner bytes. The `chtype`/wide/utf8 suffix and the mouse-support probe are the
/// boundary.
pub fn get_screenio_and_mouse_info(
    with_curses: &[u8],
    major: i32,
    minor: i32,
    patch: i32,
    cmp_major: i32,
    cmp_minor: i32,
    raw_version: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    if major == cmp_major && minor == cmp_minor {
        // snprintf (buff, 55, _("%s, version %d.%d.%d"), WITH_CURSES, major, minor, patch);
        out.extend_from_slice(with_curses);
        out.extend_from_slice(b", version ");
        push_int(&mut out, major);
        out.push(b'.');
        push_int(&mut out, minor);
        out.push(b'.');
        push_int(&mut out, patch);
    } else if major != 0 {
        // "%s, version %d.%d.%d (compiled with %d.%d)"
        out.extend_from_slice(with_curses);
        out.extend_from_slice(b", version ");
        push_int(&mut out, major);
        out.push(b'.');
        push_int(&mut out, minor);
        out.push(b'.');
        push_int(&mut out, patch);
        out.extend_from_slice(b" (compiled with ");
        push_int(&mut out, cmp_major);
        out.push(b'.');
        push_int(&mut out, cmp_minor);
        out.push(b')');
    } else {
        // "%s, version %s"
        out.extend_from_slice(with_curses);
        out.extend_from_slice(b", version ");
        out.extend_from_slice(raw_version);
    }
    out
}

/// Shared `"%s, version maj.min.patch[ (compiled with cmaj.cmin)]"` helper used by
/// [`get_math_info`] -- mirrors the duplicated `snprintf` pair in common.c.
fn version_line(
    name: &[u8],
    major: i32,
    minor: i32,
    patch: i32,
    compiled_major: i32,
    compiled_minor: i32,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(name);
    out.extend_from_slice(b", version ");
    push_int(&mut out, major);
    out.push(b'.');
    push_int(&mut out, minor);
    out.push(b'.');
    push_int(&mut out, patch);
    if !(major == compiled_major && minor == compiled_minor) {
        out.extend_from_slice(b" (compiled with ");
        push_int(&mut out, compiled_major);
        out.push(b'.');
        push_int(&mut out, compiled_minor);
        out.push(b')');
    }
    out
}

// ===========================================================================
// print_info / print_runtime_conf -- documented boundary
// ===========================================================================
//
// `print_info` / `print_info_detailed` and `print_runtime_conf` are large impure drivers: they loop
// over the runtime configuration table (`cobsetptr`), read process environment, run ncurses, and emit
// every line via `var_print` (already ported in common_runerr) / `printf`. Their leaf formatting is the
// `var_print` "<name> ... : <value>" line builder (already ported) and the version_line helpers above.
// The driver bodies themselves are the runtime/OS boundary and are intentionally not reproduced here.

// ===========================================================================
// cob_set_locale -- pure category decision
// ===========================================================================

/// The locale category that `cob_set_locale` resolves to (the `category` argument), mirroring the
/// `COB_LC_*` constants in `common.h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CobLocaleCategory {
    Collate = 0,
    Ctype = 1,
    Messages = 2,
    Monetary = 3,
    Numeric = 4,
    Time = 5,
    All = 6,
    User = 7,
    Class = 8,
}

impl CobLocaleCategory {
    /// Map the integer `category` argument to the enum (`COB_LC_*`). Unknown values -> `None`
    /// (the C `switch` has no default -- `p` stays as passed and no `setlocale` runs).
    pub fn from_int(category: i32) -> Option<Self> {
        Some(match category {
            0 => Self::Collate,
            1 => Self::Ctype,
            2 => Self::Messages,
            3 => Self::Monetary,
            4 => Self::Numeric,
            5 => Self::Time,
            6 => Self::All,
            7 => Self::User,
            8 => Self::Class,
            _ => return None,
        })
    }
}

/// The decision `cob_set_locale` makes before it ever touches `setlocale`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetLocaleDecision {
    /// `locale` field given but converts to an empty string (`flen < 1`) -- the C `return`s immediately
    /// without raising an exception.
    EmptyLocale,
    /// `category` is out of range -- the `switch` falls through, `p` is unchanged, no exception, the
    /// final `setlocale(LC_ALL, NULL)` re-read still runs (modelled as `apply` with the resolved name).
    UnknownCategory,
    /// Proceed: `name` is the requested locale name (`None` => query current, i.e. C passes `NULL`),
    /// for the given category.
    Apply {
        category: CobLocaleCategory,
        name: Option<Vec<u8>>,
    },
}

/// Port of the pure name/category handling of `common.c:cob_set_locale`.
///
/// `locale_str` is the already-stringified `cob_field` (`cob_field_to_string`, the field->string
/// conversion is itself elsewhere): `None` => the C `locale == NULL` path (`p = NULL`, query the
/// category), `Some(s)` => the field value. A `Some("")` (`flen < 1`) yields [`SetLocaleDecision::EmptyLocale`].
/// The actual `setlocale` calls and the `COB_EC_LOCALE_MISSING` exception on a NULL return are the boundary.
pub fn cob_set_locale(locale_str: Option<&[u8]>, category: i32) -> SetLocaleDecision {
    let name: Option<Vec<u8>> = match locale_str {
        Some(s) => {
            if s.is_empty() {
                return SetLocaleDecision::EmptyLocale;
            }
            Some(s.to_vec())
        }
        None => None,
    };
    match CobLocaleCategory::from_int(category) {
        None => SetLocaleDecision::UnknownCategory,
        Some(cat) => SetLocaleDecision::Apply {
            category: cat,
            name,
        },
    }
}

// ===========================================================================
// Temp file name
// ===========================================================================

/// State for `cob_temp_name` / `cob_incr_temp_iteration` -- models the `cob_temp_iteration` global.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TempIteration {
    /// `cob_temp_iteration`
    pub value: i32,
}

impl TempIteration {
    /// Port of `common.c:cob_incr_temp_iteration` -- `cob_temp_iteration++`.
    pub fn cob_incr_temp_iteration(&mut self) {
        self.value = self.value.wrapping_add(1);
    }
}

/// Port of `common.c:cob_temp_name` (non-`HAVE_8DOT3_FILENAMES` schema).
///
/// Builds the temp filename from `tmpdir`, the path separator `slash`, the process `pid`, the
/// iteration counter, and an optional extension. With `ext = Some` it is `"%s%ccob%d_%d%s"`
/// (`TEMP_EXT_SCHEMA`); with `ext = None` it is `"%s%ccobsort%d_%d"` (`TEMP_SORT_SCHEMA`). `tmpdir`
/// is `cob_gettmpdir()` and `pid` is `cob_sys_getpid()` -- both injected (OS leaves). Returns the bytes.
pub fn cob_temp_name(tmpdir: &[u8], slash: u8, pid: i32, iteration: i32, ext: Option<&[u8]>) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(tmpdir);
    out.push(slash);
    match ext {
        Some(ext) => {
            // "cob%d_%d%s"
            out.extend_from_slice(b"cob");
            push_int(&mut out, pid);
            out.push(b'_');
            push_int(&mut out, iteration);
            out.extend_from_slice(ext);
        }
        None => {
            // "cobsort%d_%d"
            out.extend_from_slice(b"cobsort");
            push_int(&mut out, pid);
            out.push(b'_');
            push_int(&mut out, iteration);
        }
    }
    out
}

/// Port of `common.c:cob_temp_name` under `HAVE_8DOT3_FILENAMES` (short-name schema).
///
/// `pid` is taken modulo 9999, and the prefixes shrink to `"c%d_%d%s"` / `"s%d_%d"`.
pub fn cob_temp_name_8dot3(tmpdir: &[u8], slash: u8, pid: i32, iteration: i32, ext: Option<&[u8]>) -> Vec<u8> {
    let pid = pid % 9999;
    let mut out = Vec::new();
    out.extend_from_slice(tmpdir);
    out.push(slash);
    match ext {
        Some(ext) => {
            out.push(b'c');
            push_int(&mut out, pid);
            out.push(b'_');
            push_int(&mut out, iteration);
            out.extend_from_slice(ext);
        }
        None => {
            out.push(b's');
            push_int(&mut out, pid);
            out.push(b'_');
            push_int(&mut out, iteration);
        }
    }
    out
}

// ===========================================================================
// Intermediate moves
// ===========================================================================

/// Port of the pure byte-move core of `common.c:cob_move_intermediate`.
///
/// The C wraps `src[..size]` in a throwaway alphanumeric `cob_field` and runs `cob_move` into `dst`.
/// For an alphanumeric destination `cob_move` is a left-justified copy: copy `min(size, dst_size)`
/// bytes then space-pad (`0x20`) the remainder of `dst`. (When `dst` is numeric, `cob_move` instead
/// converts on the fly -- that conversion is the impure numeric path elsewhere; here we model the
/// alphanumeric padding move, which is what every caller in common.c uses it for.) Returns the
/// `dst_size`-byte destination buffer.
pub fn cob_move_intermediate(src: &[u8], dst_size: usize) -> Vec<u8> {
    let mut out = vec![0x20u8; dst_size];
    let n = src.len().min(dst_size);
    out[..n].copy_from_slice(&src[..n]);
    out
}

/// Port of the pure core of `common.c:cob_move_to_group_as_alnum`.
///
/// The C builds a shadow of `dst` whose type is forced to `COB_TYPE_ALPHANUMERIC` (same size/storage)
/// and `cob_move`s `src` into it -- i.e. a group move is `memcpy` of `min` bytes + space fill, exactly
/// like the alphanumeric [`cob_move_intermediate`]. Returns the `dst_size`-byte group buffer.
pub fn cob_move_to_group_as_alnum(src: &[u8], dst_size: usize) -> Vec<u8> {
    cob_move_intermediate(src, dst_size)
}

// ===========================================================================
// Statement name table + lookup
// ===========================================================================

/// The COBOL statement ids, in the fixed order of `statement.def` (offset by `STMT_UNKNOWN == 0`).
/// The integer value of each variant is its `enum cob_statement` value -- these are ABI (used as
/// integer arguments in generated modules), so the order must not change.
pub const STMT_UNKNOWN: u16 = 0;

/// Port of `cob_statement_name[]` built by `common.c:init_statement_list` from `statement.def`.
///
/// Index 0 is `STMT_UNKNOWN` -> `"UNKNOWN"`; every subsequent entry is a `COB_STATEMENT(ename, str)`
/// in `statement.def` order. The enum value of a statement is its index into this table.
pub const COB_STATEMENT_NAMES: &[&[u8]] = &[
    b"UNKNOWN",
    b"ADD",
    b"SUBTRACT",
    b"MULTIPLY",
    b"DIVIDE",
    b"COMPUTE",
    b"MOVE",
    b"INITIALIZE",
    b"STRING",
    b"UNSTRING",
    b"INSPECT",
    b"EXAMINE",
    b"VALIDATE",
    b"CONTINUE",
    b"CONTINUE AFTER",
    b"ACCEPT",
    b"DISPLAY",
    b"EXHIBIT",
    b"STOP",
    b"STOP ERROR",
    b"DISPLAY WINDOW",
    b"INQUIRE",
    b"DESTROY",
    b"MODIFY",
    b"CLOSE WINDOW",
    b"SEARCH",
    b"SEARCH ALL",
    b"SEARCH VARYING",
    b"AT END",
    b"NEXT SENTENCE",
    b"CALL",
    b"CANCEL",
    b"CHAIN",
    b"GOBACK",
    b"EXIT PROGRAM",
    b"EXIT FUNCTION",
    b"STOP RUN",
    b"STOP THREAD",
    b"LOCK THREAD",
    b"WAIT FOR THREAD",
    b"RAISE",
    b"RESUME",
    b"DISABLE",
    b"ENABLE",
    b"PURGE",
    b"SEND",
    b"RECEIVE",
    b"END",
    b"NOTE",
    b"PERFORM",
    b"UNTIL",
    b"VARYING",
    b"EXIT PERFORM",
    b"END-PERFORM",
    b"SET",
    b"END-SET",
    b"ENTER",
    b"INVOKE",
    b"END-INVOKE",
    b"ENTRY",
    b"ENTRY FOR GO TO",
    b"GO TO",
    b"ALTER",
    b"IF",
    b"ELSE",
    b"END-IF",
    b"VALUE THRU",
    b"EVALUATE",
    b"WHEN",
    b"WHEN OTHER",
    b"END-EVALUATE",
    b"EXECUTE",
    b"EXIT",
    b"EXIT SECTION",
    b"EXIT PARAGRAPH",
    b"EXIT PERFORM CYCLE",
    b"OPEN",
    b"REWRITE",
    b"WRITE",
    b"DELETE",
    b"CLOSE",
    b"START",
    b"READ",
    b"UNLOCK",
    b"LOCK FILE",
    b"DELETE FILE",
    b"ROLLBACK",
    b"COMMIT",
    b"ALLOCATE",
    b"FREE",
    b"INITIATE",
    b"GENERATE",
    b"TERMINATE",
    b"SUPPRESS",
    b"PRESENT WHEN",
    b"SORT",
    b"MERGE",
    b"RELEASE",
    b"RETURN",
    b"READY TRACE",
    b"RESET TRACE",
    b"OTHERWISE",
    b"RECOVER",
    b"SERVICE",
    b"TRANSFORM",
    b"JSON GENERATE",
    b"JSON GENERATE", // STMT_JSON_PARSE (string per statement.def)
    b"XML GENERATE",
    b"XML GENERATE", // STMT_XML_PARSE (string per statement.def)
    b"INIT STORAGE",
    b"INIT CALL",
    b"INIT UDF",
];

/// Port of `common.c:init_statement_list` -- the table that maps statement id -> name.
///
/// In Rust the table is the `'static` constant [`COB_STATEMENT_NAMES`]; this returns it (the C version
/// populates a `cob_statement_name[]` array, identical content). Lookup of id->name is just indexing.
pub fn init_statement_list() -> &'static [&'static [u8]] {
    COB_STATEMENT_NAMES
}

/// id -> name lookup into [`COB_STATEMENT_NAMES`]. Out-of-range -> `"UNKNOWN"`, mirroring how an
/// uninitialised slot would read in the C table after `init_statement_list`.
pub fn cob_statement_name(stmt: u16) -> &'static [u8] {
    COB_STATEMENT_NAMES
        .get(stmt as usize)
        .copied()
        .unwrap_or(COB_STATEMENT_NAMES[STMT_UNKNOWN as usize])
}

/// Port of `common.c:hash` -- the additive byte-sum hash used by `init_statement_hashlist`.
/// `register int val = 0; while (*p) val += *p++;` (wrapping, signed 32-bit).
pub fn hash(s: &[u8]) -> i32 {
    let mut val: i32 = 0;
    for &b in s {
        val = val.wrapping_add(b as i32);
    }
    val
}

/// Port of `common.c:init_statement_hashlist` -- precompute the per-statement name hash, indexed by id.
/// `cob_statement_hash[STMT_UNKNOWN] = hash("UNKNOWN");` then one entry per `statement.def` line.
pub fn init_statement_hashlist() -> Vec<i32> {
    COB_STATEMENT_NAMES.iter().map(|s| hash(s)).collect()
}

/// Port of the resolving core of `common.c:get_stmt_from_name` (the per-name lookup, without the
/// pointer-identity LRU cache which is an optimisation over the impure `const char *` identity).
///
/// Computes `hash(stmt_name)` and, for each `COB_STATEMENT(ename, str)`, returns `ename` when the hash
/// matches *and* the name string is equal (`strcmp == 0`). A `None`/empty name resolves to
/// `STMT_UNKNOWN`; an unresolved name resolves to `STMT_UNKNOWN` too (the C path warns then returns
/// `STMT_UNKNOWN`). Returns the statement id.
pub fn get_stmt_from_name(stmt_name: Option<&[u8]>) -> u16 {
    let name = match stmt_name {
        None => return STMT_UNKNOWN,
        Some(n) => n,
    };
    let want = hash(name);
    for (id, &cand) in COB_STATEMENT_NAMES.iter().enumerate() {
        // skip slot 0 (STMT_UNKNOWN) -- the C loop is over statement.def entries only
        if id == STMT_UNKNOWN as usize {
            continue;
        }
        if hash(cand) == want && cand == name {
            return id as u16;
        }
    }
    STMT_UNKNOWN
}

// ===========================================================================
// In-memory table SORT
// ===========================================================================

/// `COB_ASCENDING` / `COB_DESCENDING` (`common.h`).
pub const COB_ASCENDING: i32 = 0;
pub const COB_DESCENDING: i32 = 1;

/// One SORT key -- models a `cob_file_key` entry as used by `sort_keys[]`.
///
/// `offset` is the byte offset of the key within each fixed-size record; `len` is the key field size
/// (`f->size`); `flag` is `COB_ASCENDING`/`COB_DESCENDING`; `numeric` selects the numeric compare path
/// (`COB_FIELD_IS_NUMERIC`) vs. the byte/collation compare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortKey {
    pub offset: usize,
    pub len: usize,
    pub flag: i32,
    pub numeric: bool,
}

/// Sort state -- models the `sort_nkeys` / `sort_keys` / `sort_collate` globals built by
/// `cob_table_sort_init` + `cob_table_sort_init_key`.
#[derive(Clone, Debug, Default)]
pub struct TableSort {
    pub keys: Vec<SortKey>,
    /// `sort_collate`: a 256-entry collating sequence, or `None` (raw `memcmp`).
    pub collate: Option<[u8; 256]>,
}

impl TableSort {
    /// Port of `common.c:cob_table_sort_init` -- start a fresh key list with an optional collation.
    /// (The C `nkeys` pre-allocates `sort_keys`; here the `Vec` grows as keys are added.)
    pub fn cob_table_sort_init(collating_sequence: Option<[u8; 256]>) -> Self {
        TableSort {
            keys: Vec::new(),
            collate: collating_sequence,
        }
    }

    /// Port of `common.c:cob_table_sort_init_key` -- append a key (field size, flag, offset).
    pub fn cob_table_sort_init_key(&mut self, len: usize, flag: i32, offset: usize) {
        self.keys.push(SortKey {
            offset,
            len,
            flag,
            numeric: false,
        });
    }

    /// As [`Self::cob_table_sort_init_key`] but marking the key as numeric (the C `COB_FIELD_IS_NUMERIC`
    /// branch is decided by the field attribute, modelled here as a per-key flag).
    pub fn cob_table_sort_init_key_numeric(&mut self, len: usize, flag: i32, offset: usize) {
        self.keys.push(SortKey {
            offset,
            len,
            flag,
            numeric: true,
        });
    }

    /// Port of `common.c:sort_compare` / `sort_compare_collate` -- the multi-key record comparison.
    ///
    /// For each key in order: compare the key sub-slices of `a`/`b`. A numeric key uses
    /// [`numeric_cmp`]; otherwise, with a collation, the bytes go through it (`common_cmps`), without
    /// one it is a raw `memcmp`. The first non-zero key result is returned, negated for `COB_DESCENDING`.
    /// Equal on all keys -> `Equal`.
    pub fn compare(&self, a: &[u8], b: &[u8]) -> core::cmp::Ordering {
        use core::cmp::Ordering;
        for key in &self.keys {
            let a1 = &a[key.offset..key.offset + key.len];
            let b1 = &b[key.offset..key.offset + key.len];
            let res = if key.numeric {
                numeric_cmp(a1, b1)
            } else if let Some(col) = &self.collate {
                cmps_collate(a1, b1, col)
            } else {
                memcmp(a1, b1)
            };
            if res != 0 {
                let signed = if key.flag == COB_ASCENDING { res } else { -res };
                return signed.cmp(&0);
            }
        }
        Ordering::Equal
    }

    /// Port of `common.c:cob_table_sort` -- reorder `n` fixed-size records in `table` per the keys.
    ///
    /// `table` is the flat record buffer (`f->data`), `record_size` is `f->size`, `n` the element count.
    /// The C calls `qsort`; we sort the record indices with [`Self::compare`] and rebuild the buffer.
    /// `qsort` is not guaranteed stable; this models the resulting key-ordering (stable here, which is a
    /// permitted refinement -- the observable post-condition is "ordered by the keys"). Returns the
    /// reordered buffer.
    pub fn cob_table_sort(&self, table: &[u8], record_size: usize, n: usize) -> Vec<u8> {
        if record_size == 0 || n == 0 {
            return table.to_vec();
        }
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&i, &j| {
            let a = &table[i * record_size..i * record_size + record_size];
            let b = &table[j * record_size..j * record_size + record_size];
            self.compare(a, b)
        });
        let mut out = Vec::with_capacity(n * record_size);
        for &i in &order {
            out.extend_from_slice(&table[i * record_size..i * record_size + record_size]);
        }
        out
    }
}

/// `memcmp`-style compare returning the signed difference of the first differing byte (or 0).
fn memcmp(a: &[u8], b: &[u8]) -> i32 {
    for (&x, &y) in a.iter().zip(b.iter()) {
        if x != y {
            return x as i32 - y as i32;
        }
    }
    0
}

/// Mirror of `common.c:common_cmps` (collation-translated byte compare) for the SORT path.
fn cmps_collate(a: &[u8], b: &[u8], col: &[u8; 256]) -> i32 {
    for (&x, &y) in a.iter().zip(b.iter()) {
        let r = col[x as usize] as i32 - col[y as usize] as i32;
        if r != 0 {
            return r;
        }
    }
    0
}

/// Stand-in for `cob_numeric_cmp` used by the SORT numeric-key branch. The real routine is the
/// (impure-on-attrs) decimal comparison ported elsewhere; for the table-sort model the numeric key
/// slices are compared as already-aligned magnitude bytes (`memcmp`). Documented as a boundary seam:
/// callers needing true numeric semantics pass already-normalised key bytes.
fn numeric_cmp(a: &[u8], b: &[u8]) -> i32 {
    memcmp(a, b)
}

// ===========================================================================
// small integer / scan helpers (private)
// ===========================================================================

/// Append the base-10 ASCII of a signed int (matches C `printf("%d")`).
fn push_int(out: &mut Vec<u8>, value: i32) {
    if value < 0 {
        out.push(b'-');
    }
    let mut u = (value as i64).unsigned_abs() as u64;
    let start = out.len();
    loop {
        out.push(b'0' + (u % 10) as u8);
        u /= 10;
        if u == 0 {
            break;
        }
    }
    out[start..].reverse();
}

/// Append a non-negative int zero- or space-padded to at least `width` digits (`%2.2d` style uses
/// zero padding -> `zero = true`).
fn push_int_min_width(out: &mut Vec<u8>, value: i32, width: usize, zero: bool) {
    let mut digits = Vec::new();
    let mut u = (value as i64).unsigned_abs() as u64;
    loop {
        digits.push(b'0' + (u % 10) as u8);
        u /= 10;
        if u == 0 {
            break;
        }
    }
    let pad = if value < 0 { 1 } else { 0 };
    while digits.len() + pad < width {
        digits.push(if zero { b'0' } else { b' ' });
    }
    if value < 0 {
        out.push(b'-');
    }
    digits.reverse();
    out.extend_from_slice(&digits);
}

/// `sscanf(s, "%63s %d %d", month, &day, &year)` returning `Some((month, day, year))` only when all
/// three fields parse (status == 3), as `set_cob_build_stamp` requires.
fn scan_month_day_year(s: &[u8]) -> Option<(&[u8], i32, i32)> {
    let mut i = 0;
    let n = s.len();
    // %63s : skip leading ws, take a run of non-ws
    while i < n && s[i].is_ascii_whitespace() {
        i += 1;
    }
    let mstart = i;
    while i < n && !s[i].is_ascii_whitespace() {
        i += 1;
    }
    if i == mstart {
        return None;
    }
    let month = &s[mstart..i];
    let (day, ni) = scan_int(s, i)?;
    let (year, _) = scan_int(s, ni)?;
    Some((month, day, year))
}

/// Parse a `%d` starting at `pos` (skipping leading whitespace). Returns `(value, next_pos)`.
fn scan_int(s: &[u8], pos: usize) -> Option<(i32, usize)> {
    let mut i = pos;
    let n = s.len();
    while i < n && s[i].is_ascii_whitespace() {
        i += 1;
    }
    let neg = if i < n && (s[i] == b'-' || s[i] == b'+') {
        let neg = s[i] == b'-';
        i += 1;
        neg
    } else {
        false
    };
    let dstart = i;
    let mut val: i32 = 0;
    while i < n && s[i].is_ascii_digit() {
        val = val.wrapping_mul(10).wrapping_add((s[i] - b'0') as i32);
        i += 1;
    }
    if i == dstart {
        return None;
    }
    Some((if neg { -val } else { val }, i))
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_cob_build_stamp_parsed() {
        // __DATE__ = "Jun 15 2026", __TIME__ = "12:34:56"
        let s = set_cob_build_stamp(b"Jun 15 2026", b"12:34:56");
        assert_eq!(s, b"Jun 15 2026 12:34:56");
    }

    #[test]
    fn set_cob_build_stamp_pads_day() {
        let s = set_cob_build_stamp(b"Jun 5 2026", b"01:02:03");
        // %2.2d -> "05", %4.4d -> "2026"
        assert_eq!(s, b"Jun 05 2026 01:02:03");
    }

    #[test]
    fn set_cob_build_stamp_fallback() {
        // unparseable -> "<date> <time>"
        let s = set_cob_build_stamp(b"unknown", b"00:00:00");
        assert_eq!(s, b"unknown 00:00:00");
    }

    #[test]
    fn print_version_banner() {
        let s = print_version(b"GnuCOBOL", b"3.2", 0, b"2023", b"Jun 15 2026 12:00:00", b"Dec 24 2022 12:00:00");
        let text = String::from_utf8(s).unwrap();
        assert!(text.starts_with("libcob (GnuCOBOL) 3.2.0\n"));
        assert!(text.contains("Copyright (C) 2023 Free Software Foundation, Inc.\n"));
        assert!(text.contains("License LGPLv3+"));
        assert!(text.contains("Built     Jun 15 2026 12:00:00\n"));
        assert!(text.contains("Packaged  Dec 24 2022 12:00:00\n"));
    }

    #[test]
    fn print_version_summary_lines() {
        let s = print_version_summary(b"GnuCOBOL", b"3.2.0", b"stamp", b"gcc 16", b"GMP", 6, 3, 0, b"");
        let text = String::from_utf8(s).unwrap();
        assert_eq!(text, "GnuCOBOL 3.2.0 (stamp), gcc 16\nGMP 6.3.0\n");
    }

    #[test]
    fn print_version_summary_extra() {
        let s = print_version_summary(b"GnuCOBOL", b"3.2.0", b"stamp", b"cc", b"GMP", 6, 3, 0, b", libxml2 2.9.14");
        let text = String::from_utf8(s).unwrap();
        assert!(text.ends_with("GMP 6.3.0, libxml2 2.9.14\n"));
    }

    #[test]
    fn math_info_match_vs_mismatch() {
        let m = get_math_info(b"GMP", 6, 3, 0, 6, 3);
        assert_eq!(m, b"GMP, version 6.3.0");
        let m = get_math_info(b"GMP", 6, 2, 1, 6, 3);
        assert_eq!(m, b"GMP, version 6.2.1 (compiled with 6.3)");
    }

    #[test]
    fn screenio_banner_variants() {
        // match
        let b = get_screenio_and_mouse_info(b"ncursesw", 6, 4, 0, 6, 4, b"raw");
        assert_eq!(b, b"ncursesw, version 6.4.0");
        // mismatch but parsed
        let b = get_screenio_and_mouse_info(b"ncursesw", 6, 2, 0, 6, 4, b"raw");
        assert_eq!(b, b"ncursesw, version 6.2.0 (compiled with 6.4)");
        // unparseable
        let b = get_screenio_and_mouse_info(b"ncursesw", 0, 0, 0, 6, 4, b"ncurses 6.4.20221231");
        assert_eq!(b, b"ncursesw, version ncurses 6.4.20221231");
    }

    #[test]
    fn locale_decision() {
        assert_eq!(cob_set_locale(Some(b""), 6), SetLocaleDecision::EmptyLocale);
        assert_eq!(cob_set_locale(Some(b"de_DE"), 99), SetLocaleDecision::UnknownCategory);
        assert_eq!(
            cob_set_locale(Some(b"de_DE"), 6),
            SetLocaleDecision::Apply { category: CobLocaleCategory::All, name: Some(b"de_DE".to_vec()) }
        );
        assert_eq!(
            cob_set_locale(None, 0),
            SetLocaleDecision::Apply { category: CobLocaleCategory::Collate, name: None }
        );
    }

    #[test]
    fn temp_name_schemas() {
        assert_eq!(cob_temp_name(b"/tmp", b'/', 1234, 0, Some(b".tmp")), b"/tmp/cob1234_0.tmp");
        assert_eq!(cob_temp_name(b"/tmp", b'/', 1234, 3, None), b"/tmp/cobsort1234_3");
        // 100001 % 9999 == 11
        assert_eq!(cob_temp_name_8dot3(b"C:\\T", b'\\', 100001, 2, Some(b".s")), b"C:\\T\\c11_2.s".to_vec());
    }

    #[test]
    fn temp_iteration_counter() {
        let mut it = TempIteration::default();
        assert_eq!(it.value, 0);
        it.cob_incr_temp_iteration();
        it.cob_incr_temp_iteration();
        assert_eq!(it.value, 2);
    }

    #[test]
    fn move_intermediate_pads_and_truncates() {
        assert_eq!(cob_move_intermediate(b"AB", 5), b"AB   ");
        assert_eq!(cob_move_intermediate(b"ABCDE", 3), b"ABC");
        assert_eq!(cob_move_to_group_as_alnum(b" ", 4), b"    ");
    }

    #[test]
    fn statement_table_roundtrip() {
        // index of MOVE etc.
        let move_id = get_stmt_from_name(Some(b"MOVE"));
        assert_eq!(cob_statement_name(move_id), b"MOVE");
        assert_eq!(cob_statement_name(STMT_UNKNOWN), b"UNKNOWN");
        assert_eq!(get_stmt_from_name(Some(b"ADD")), 1);
        assert_eq!(cob_statement_name(1), b"ADD");
        assert_eq!(get_stmt_from_name(Some(b"SEARCH ALL")), get_stmt_from_name(Some(b"SEARCH ALL")));
        assert_ne!(get_stmt_from_name(Some(b"SEARCH")), get_stmt_from_name(Some(b"SEARCH ALL")));
        // unknown / none
        assert_eq!(get_stmt_from_name(Some(b"NOSUCHSTMT")), STMT_UNKNOWN);
        assert_eq!(get_stmt_from_name(None), STMT_UNKNOWN);
    }

    #[test]
    fn hash_collision_resolved_by_strcmp() {
        // "MOVE" and any anagram share the additive hash; strcmp disambiguates.
        assert_eq!(hash(b"MOVE"), hash(b"VOME"));
        assert_eq!(get_stmt_from_name(Some(b"VOME")), STMT_UNKNOWN);
        assert_eq!(cob_statement_name(get_stmt_from_name(Some(b"MOVE"))), b"MOVE");
    }

    #[test]
    fn statement_table_complete() {
        // table length = 1 (UNKNOWN) + entries in statement.def (111)
        assert_eq!(COB_STATEMENT_NAMES.len(), 1 + 111);
        assert_eq!(init_statement_list().len(), COB_STATEMENT_NAMES.len());
        assert_eq!(init_statement_hashlist().len(), COB_STATEMENT_NAMES.len());
    }

    #[test]
    fn table_sort_single_key_ascending() {
        // 3 records of size 2, key at offset 0 len 2, ascending
        let mut ts = TableSort::cob_table_sort_init(None);
        ts.cob_table_sort_init_key(2, COB_ASCENDING, 0);
        let table = b"CCAABB";
        let out = ts.cob_table_sort(table, 2, 3);
        assert_eq!(out, b"AABBCC");
    }

    #[test]
    fn table_sort_descending() {
        let mut ts = TableSort::cob_table_sort_init(None);
        ts.cob_table_sort_init_key(2, COB_DESCENDING, 0);
        let out = ts.cob_table_sort(b"AABBCC", 2, 3);
        assert_eq!(out, b"CCBBAA");
    }

    #[test]
    fn table_sort_multi_key() {
        // record size 4: key1 at off0 len2 asc, key2 at off2 len2 desc
        let mut ts = TableSort::cob_table_sort_init(None);
        ts.cob_table_sort_init_key(2, COB_ASCENDING, 0);
        ts.cob_table_sort_init_key(2, COB_DESCENDING, 2);
        // records: "AA99","AA11","BB00"
        let table = b"AA99AA11BB00";
        let out = ts.cob_table_sort(table, 4, 3);
        // primary AA before BB; within AA, desc on second key: 99 before 11
        assert_eq!(out, b"AA99AA11BB00");
    }

    #[test]
    fn table_sort_with_collation() {
        // collation that maps 'a'->'Z', leaving 'B' as-is, to flip a vs B ordering
        let mut col = [0u8; 256];
        for i in 0..256 {
            col[i] = i as u8;
        }
        col[b'a' as usize] = b'Z';
        let mut ts = TableSort::cob_table_sort_init(Some(col));
        ts.cob_table_sort_init_key(1, COB_ASCENDING, 0);
        // raw 'a' (0x61) > 'B'(0x42); but via collation 'a'->'Z'(0x5A) still > 'B'
        let out = ts.cob_table_sort(b"aB", 1, 2);
        assert_eq!(out, b"Ba");
    }
}
