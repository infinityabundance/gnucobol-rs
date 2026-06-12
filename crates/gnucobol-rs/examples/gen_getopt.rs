//! Generator for the cobgetopt.c differential (`cob_getopt_long_long`). Emits scenarios — one per line,
//! tab-separated `LABEL\tlong_only\toptstring\tlongspec\targs` — fed identically to the libcob oracle
//! (`getopt_harness.c`) and the Rust port (`getopt_rows`). longspec is `-` or
//! `name:has_arg:val|...` (flag always NULL, as in every GnuCOBOL long_options table). Covers short
//! options (required/optional/missing args, clustering, unknown), the leading `+`/`-`/`:` ordering
//! modes, the `--` terminator + PERMUTE reordering, long options (exact/abbrev/ambiguous/`=arg`),
//! `getopt_long_only`, and the `-W foo` convenience form.

fn emit(label: &str, long_only: i32, optstring: &str, longspec: &str, args: &str) {
    println!("{label}\t{long_only}\t{optstring}\t{longspec}\t{args}");
}

fn main() {
    // --- short options ------------------------------------------------------------------------
    emit("short_simple", 0, "ab", "-", "-a -b");
    emit("short_cluster", 0, "abc", "-", "-abc");
    emit("short_unknown", 0, "ab", "-", "-x");
    emit("short_reqarg_sep", 0, "a:b", "-", "-a foo -b");
    emit("short_reqarg_join", 0, "a:b", "-", "-afoo");
    emit("short_reqarg_missing", 0, "a:", "-", "-a");
    emit("short_reqarg_missing_colon", 0, ":a:", "-", "-a");
    emit("short_optarg_present", 0, "a::b", "-", "-axyz -b");
    emit("short_optarg_absent", 0, "a::b", "-", "-a -b");
    emit("short_only_nonopt", 0, "ab", "-", "x y");
    emit("short_dashdash", 0, "ab", "-", "-a -- -b");
    emit("short_lonedash", 0, "ab", "-", "-a - -b");

    // --- ordering modes -----------------------------------------------------------------------
    emit("permute_default", 0, "ab", "-", "x -a y -b z");
    emit("require_order_plus", 0, "+ab", "-", "x -a -b");
    emit("return_in_order_minus", 0, "-ab", "-", "x -a y -b");
    emit("permute_arg", 0, "a:b", "-", "one -a val two -b three");

    // --- long options -------------------------------------------------------------------------
    emit("long_exact", 0, "ab", "foo:0:1|bar:0:2", "--foo --bar");
    emit("long_eqarg", 0, "ab", "foo:1:1", "--foo=hello");
    emit("long_sep_arg", 0, "ab", "foo:1:1", "--foo hello");
    emit("long_abbrev_unique", 0, "ab", "foo:0:1|zap:0:2", "--fo");
    emit("long_ambiguous", 0, "ab", "foo:0:1|fob:0:2", "--fo");
    emit("long_unknown", 0, "ab", "foo:0:1", "--xyz");
    emit("long_noarg_with_eq", 0, "ab", "foo:0:1", "--foo=oops");
    emit("long_optarg_eq", 0, "ab", "foo:2:1", "--foo=val");
    emit("long_optarg_none", 0, "ab", "foo:2:1", "--foo");
    emit("long_reqarg_missing", 0, "ab", "foo:1:1", "--foo");
    emit("long_then_short", 0, "ab", "foo:0:1", "--foo -a -b");
    emit("long_mixed_nonopt", 0, "ab", "foo:1:9", "x --foo v y -a");

    // --- getopt_long_only ---------------------------------------------------------------------
    emit(
        "longonly_single_dash",
        1,
        "ab",
        "foo:0:1|bar:1:2",
        "-foo -bar v",
    );
    emit("longonly_shortfallback", 1, "abf", "foo:0:1", "-f");
    emit("longonly_abbrev", 1, "ab", "verbose:0:5", "-verb");

    // --- -W convenience -----------------------------------------------------------------------
    emit("w_long", 0, "W;ab", "foo:1:1", "-W foo=bar");
    emit("w_long_sep", 0, "W;ab", "foo:1:1", "-W foo bar");

    // --- edge: no args, only program name -----------------------------------------------------
    emit("empty", 0, "ab", "-", "");
    emit("only_dashdash", 0, "ab", "-", "--");
}
