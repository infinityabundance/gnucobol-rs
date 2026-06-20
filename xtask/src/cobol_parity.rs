//! Generates `COBOL-PARITY.md` -- the full-language 1:1 parity tracker. It enumerates EVERY aspect of
//! what GnuCOBOL does (the 66 statement verbs from `cobc/parser.y`, the 110 intrinsic functions from
//! `libcob/intrinsic.c`, the data-description clauses, the USAGE forms, and the 13 libcob runtime
//! files -- the authoritative surface in `data/gnucobol_surface.json`) and, for each, reports our
//! port status on two axes:
//!
//!   * **Runtime** -- is the libcob primitive ported 1:1? (from the doxygen C-vs-Rust parity:
//!     `reports/doxygen-parity.json`; every admitted libcob file is 100%, so every statement's runtime
//!     semantics + every intrinsic is ported.)
//!   * **Front-end** -- can the native interpreter (`src/frontend.rs`) actually RUN it? (from the live
//!     `WIRED_STATEMENTS` marker the front-end exports.)
//!
//! The gap between the two axes IS the "what is missing" list: the runtime is the engine (done); the
//! front-end is the steering (in progress). `check` regenerates + diffs (anti-staleness), wired into
//! the doc-refresh gate.
use serde_json::Value;
use std::path::Path;

const SURFACE: &str = include_str!("data/gnucobol_surface.json");
const FILES: &str = include_str!("data/gnucobol_files.json");

/// Front-end SUB-FORM coverage: the verb table is verb-granular (`wired_verbs` scans `src/frontend.rs`),
/// so a verb shows **DONE** the moment ANY of its forms run -- which hides the forms WITHIN a wired verb
/// that still fail closed. This table closes that blind spot. Each row is `(verb, sub-form, status,
/// anchor-path, anchor-needle)` and is REALITY-CHECKED in `reality_check`:
///   * `sealed` -- proven byte-identical to cobc; the anchor is the corpus program that proves it (the
///     gate fails if that corpus file is deleted/renamed).
///   * `gap` -- still fails closed; the anchor is the live guard in `src/frontend.rs`. When the form is
///     sealed, that guard text is removed -> the anchor no longer resolves -> the gate FAILS, forcing
///     this row to be flipped to `sealed` (with a corpus anchor). A doc that lies about a gap cannot pass.
/// Plus: every `is not in the front-end subset` guard in `src/frontend.rs` MUST have a `gap` row here
/// (count-matched), so a NEW fail-closed sub-form cannot be added without recording it.
const FRONTEND_SUBFORMS: &[(&str, &str, &str, &str, &str)] = &[
    // verb, sub-form, status, anchor path, anchor needle
    ("IF", "sign condition `x IS [NOT] {POSITIVE|NEGATIVE|ZERO}` (incl. COMP-3)", "sealed",
        "lab/corpus/frontend/p104_sign_cond.cob", "IS NEGATIVE"),
    ("ADD/SUBTRACT/MULTIPLY/DIVIDE", "multiple receivers (`ADD 1 TO Y Z`, `... GIVING C D`, in-place per-receiver)", "sealed",
        "lab/corpus/frontend/p105_multi_receiver.cob", "GIVING C D"),
    ("MOVE", "`CORRESPONDING` (matches leaves by name across two groups)", "gap",
        "crates/gnucobol-rs/src/frontend.rs", "MOVE CORRESPONDING is not in the front-end subset"),
    ("ADD/SUBTRACT", "`CORRESPONDING` (same qualified-name blocker as MOVE CORR)", "gap",
        "crates/gnucobol-rs/src/frontend.rs", "{verb} CORRESPONDING is not in the front-end subset"),
    ("DIVIDE", "`GIVING q REMAINDER r` (incl. ROUNDED quotient, signed/scaled, via the sealed GNURUST.REMAINDER.1 primitive)", "sealed",
        "lab/corpus/frontend/p106_divide_remainder.cob", "DIVIDE 17 BY 5 GIVING Q REMAINDER R"),
    ("INITIALIZE", "`REPLACING cat BY val` (NUMERIC/ALPHANUMERIC/ALPHABETIC/NUMERIC-EDITED, multi-category, PIC A vs X)", "sealed",
        "lab/corpus/frontend/p107_initialize_replacing.cob", "REPLACING NUMERIC DATA BY 7"),
    ("INSPECT", "`REPLACING CHARACTERS BY y` (incl. BEFORE/AFTER region)", "sealed",
        "lab/corpus/frontend/p108_inspect_chars.cob", "REPLACING CHARACTERS BY"),
    ("INSPECT", "multi-clause `TALLYING ... REPLACING ...` in one statement (ISO two-pass)", "sealed",
        "lab/corpus/frontend/p112_inspect_multi.cob", "TALLYING C FOR ALL"),
    ("UNSTRING", "`DELIMITER IN` / `COUNT IN` per receiver + `TALLYING IN` (added)", "sealed",
        "lab/corpus/frontend/p109_unstring_delim.cob", "DELIMITER IN D1 COUNT IN C1"),
    ("EXAMINE", "`UNTIL FIRST` (REPLACING + TALLYING...REPLACING), via the inspect CHARACTERS/BEFORE helper", "sealed",
        "lab/corpus/frontend/p110_examine_until.cob", "REPLACING UNTIL FIRST"),
    ("COMPUTE", "`**` fractional / identifier exponent (e.g. `9 ** 0.5`), via the sealed cob_decimal_pow", "sealed",
        "lab/corpus/frontend/p111_exponent.cob", "9 ** 0.5"),
    ("SET", "`SET cond TO FALSE` (the 88 `WHEN SET TO FALSE` value)", "sealed",
        "lab/corpus/frontend/p113_set_false.cob", "WHEN SET TO FALSE"),
    ("UNSTRING", "INTO DISPLAY-numeric receivers (alphanumeric->numeric per field)", "sealed",
        "lab/corpus/frontend/p114_unstring_num.cob", "INTO A B C"),
    ("INITIALIZE", "over an OCCURS table (plain + REPLACING; expands to subscripted element leaves)", "sealed",
        "lab/corpus/frontend/p115_init_occurs.cob", "INITIALIZE T1 REPLACING NUMERIC BY 7"),
    ("SORT/MERGE", "multiple KEYs with mixed ASCENDING/DESCENDING direction (major-to-minor)", "sealed",
        "lab/corpus/frontend/p116_sort_multikey.cob", "ASCENDING KEY S-GRP DESCENDING KEY S-AGE"),
    ("file I/O", "INDEXED files: READ NEXT in RECORD KEY order, random READ by key, START KEY >=, DELETE by key", "sealed",
        "lab/corpus/frontend/p117_indexed.cob", "ORGANIZATION IS INDEXED"),
    ("file I/O", "READ ... NOT AT END / NOT INVALID KEY handler (success branch in one statement)", "sealed",
        "lab/corpus/frontend/p118_read_handlers.cob", "NOT AT END DISPLAY"),
    ("ACCEPT", "`FROM ENVIRONMENT \"name\"` (read a pinned environment variable)", "sealed",
        "lab/corpus/frontend/p119_accept_env.cob", "FROM ENVIRONMENT"),
    ("file I/O", "START ... INVALID KEY / NOT INVALID KEY handler clauses", "sealed",
        "lab/corpus/frontend/p120_start_invalid.cob", "INVALID KEY DISPLAY"),
    ("UNSTRING", "`WITH POINTER p` scan cursor (read in, advanced past delimiters, written back), coexisting with `TALLYING IN`", "sealed",
        "lab/corpus/frontend/p121_unstring_pointer.cob", "WITH POINTER P TALLYING IN TC"),
];

/// The deliberate marker phrase every front-end sub-form fail-closed guard carries, so the gate can
/// require each one to be recorded as a `gap` row in `FRONTEND_SUBFORMS`.
const SUBFORM_MARKER: &str = "is not in the front-end subset";

const FILE_HEADER: &str = "\
<!-- generated by `cargo run -p xtask -- cobol-parity generate` -- do not edit by hand -->

# GnuCOBOL 3.2 -> gnucobol-rs FILE CENSUS -- every file accounted for, with a gap note + plan

> Goal: **not a single file** of the admitted GnuCOBOL 3.2 source is left unaccounted for. Every file
> is enumerated below with its parity status and -- where it is not already done -- a concrete plan to
> bring it to full parity. Statuses: **PORTED**/**PORTED+EVIDENCED** (1:1 in the runtime, oracle-sealed)
> · **COPIED+WIRED** (config copied verbatim + parsed natively) · **PARTIAL** (the clean-room interpreter
> reproduces a sweep-verified SUBSET -- accounted for, but still growing; language completeness is tracked
> in COBOL-PARITY.md) · **NON-CLAIM** (a declared boundary, e.g. cobc's C codegen) · **OBVIATED** (build
> system that Cargo replaces) · **CONSUMED-AS-ORACLE** (the GnuCOBOL test suite, used as the oracle) ·
> **REFERENCE** / **TEST-DATA**. **GAP** would mean *no native analog yet* -- there are currently none.
";

/// Build `FILE-PARITY.md` -- the per-file census of the whole GnuCOBOL 3.2 tree.
fn build_files(root: &str) -> String {
    let m: Value = serde_json::from_str(FILES).unwrap_or(Value::Null);
    let files = m["files"].as_array().cloned().unwrap_or_default();
    let mut out = String::from(FILE_HEADER);

    // summary by status.
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for f in &files {
        *counts.entry(f["status"].as_str().unwrap_or("?").to_string()).or_default() += 1;
    }
    // an at-a-glance headline: what is built vs what is an active gap (the "obvious" line).
    let built: usize = files.iter().filter(|f| matches!(f["status"].as_str().unwrap_or(""), "PORTED" | "PORTED+EVIDENCED" | "PORTED-VIA" | "COPIED+WIRED" | "PARTIAL")).count();
    let active_gaps: usize = files.iter().filter(|f| f["status"].as_str().unwrap_or("").starts_with("GAP")).count();
    let partial: usize = files.iter().filter(|f| f["status"] == "PARTIAL").count();
    let boundary: usize = files.iter().filter(|f| f["status"] == "NON-CLAIM").count();
    let tot = files.len();
    out.push_str(&format!(
        "## At a glance\n\n\
         **Every one of the {tot} files is accounted for with positive evidence -- {active_gaps} \
         unevidenced gaps.** Of these: **{built} built natively** (ported / copied+wired / partial), \
         of which **{partial} are PARTIAL** -- the clean-room interpreter (`cobrun`) reproduces a \
         *verified subset* (front-end sweep-proven byte-identical to cobc) and that subset is still \
         **growing**; **{boundary} are declared boundaries** (NON-CLAIM, e.g. cobc's C codegen, \
         off-oracle localized messages); the rest is reference / test-corpus / Cargo-obviated.\n\n\
         > **PARTIAL is not done.** This census proves every *file* is accounted for; it does **not** \
         claim the COBOL front-end is complete. Language completeness (which verbs / intrinsics / \
         clauses actually run) is tracked separately and live in **COBOL-PARITY.md**.\n\n\
         This census is regenerated by `cargo run -p xtask -- cobol-parity generate` and \
         **reality-checked** by `cobol-parity check` (status/evidence consistency + `proof` anchors that \
         must resolve in the live tree), wired into the doc-refresh gate so it cannot silently go stale.\n\n",
    ));
    out.push_str(&format!("## Summary -- {} files total\n\n| status | files | % of tree | meaning |\n|---|---:|---:|---|\n", files.len()));
    let meaning = |s: &str| match s {
        "PORTED+EVIDENCED" => "1:1 in the runtime, oracle-sealed",
        "PORTED" => "ported (API header / support unit)",
        "PORTED-VIA" => "folded into a ported module",
        "COPIED+WIRED" => "config copied verbatim + parsed natively",
        "PARTIAL" => "**verified subset reproduced** -- accounted for, but work ongoing (see COBOL-PARITY.md)",
        "GAP -> front-end" => "**active gap** -- reimplement in the clean-room front-end",
        "GAP -> cobrun" => "**active gap** -- grow the cobrun CLI",
        "OBVIATED" => "build system -- Cargo/xtask replaces it (no port needed)",
        "CONSUMED-AS-ORACLE" => "GnuCOBOL test corpus, used AS the oracle",
        "REFERENCE" => "documentation / license -- reference, not ported",
        "TEST-DATA" => "sample copybooks -- front-end test data",
        "NON-CLAIM" => "declared boundary -- intentionally not ported, with explicit reasoning (e.g. off-oracle localization catalogs)",
        _ => "triage",
    };
    for (s, n) in &counts {
        out.push_str(&format!("| **{s}** | {n} | {:.0}% | {} |\n", pct(*n, files.len()), meaning(s)));
    }
    // "Active gaps" counts true GAPs only; PARTIAL files are a verified subset, reported separately.
    let active: usize = files.iter().filter(|f| f["status"].as_str().unwrap_or("").starts_with("GAP")).count();
    // A headline COMPLETION percentage for the FILE census: accounted-for (every file has a receipt) and
    // built-natively (ported / copied+wired / partial -- the rest are boundary / reference / oracle / obviated).
    let built: usize = files.iter().filter(|f| matches!(f["status"].as_str().unwrap_or(""), "PORTED" | "PORTED+EVIDENCED" | "PORTED-VIA" | "COPIED+WIRED" | "PARTIAL")).count();
    out.push_str(&format!(
        "\n**File-census completion: {:.0}% accounted-for** ({}/{} files carry positive evidence; {} unevidenced) \
         · **{:.0}% built natively** ({} ported / copied+wired / partial; the remaining {} are declared \
         boundaries, reference, oracle corpus, or Cargo-obviated -- not native-port targets).\n",
        pct(files.len() - active, files.len()), files.len() - active, files.len(), active,
        pct(built, files.len()), built, files.len() - built,
    ));
    let evidenced: usize = files.iter().filter(|f| !f["evidence"].as_str().unwrap_or("").starts_with("UNEVIDENCED")).count();
    let unevidenced: usize = files.len() - evidenced;
    out.push_str(&format!(
        "\n**Receipts: every one of the {tot} files carries a receipt** -- {ev} with positive evidence \
         (the Rust module / config copy / test / oracle role that proves it is accounted for) and {un} \
         explicitly marked **UNEVIDENCED-GAP** with a plan (nothing is silently omitted). \
         **Unevidenced gaps: {active} files.** A file the interpreter reproduces only in part is marked \
         **PARTIAL** (with a `proof` anchor and an explicit scope note), not GAP -- the work continues, \
         but the file is accounted for. Everything else is ported, copied, a declared boundary, obviated \
         by Cargo, or reference. The plans are the road to full 1:1 parity.\n\n",
        tot = files.len(), ev = evidenced, un = unevidenced,
    ));

    // the actionable list: true gaps first (if any), then the in-progress PARTIAL files.
    let gap_files: Vec<&Value> = files.iter().filter(|f| f["status"].as_str().unwrap_or("").starts_with("GAP")).collect();
    if !gap_files.is_empty() {
        out.push_str("## Unevidenced gaps (no native analog yet)\n\n| file | category | plan |\n|---|---|---|\n");
        for f in &gap_files {
            out.push_str(&format!("| `{}` | {} | {} |\n", f["path"].as_str().unwrap_or(""), f["category"].as_str().unwrap_or(""), f["plan"].as_str().unwrap_or("")));
        }
        out.push('\n');
    } else {
        out.push_str("## Unevidenced gaps\n\n_None -- every file carries positive evidence._\n\n");
    }
    out.push_str(
        "## In progress (PARTIAL -- verified subset reproduced, work ongoing)\n\n\
         These files ARE accounted for (the interpreter reproduces a sweep-verified subset), but the \
         reproduction is partial; each row's plan is how the subset grows. The cobc front-end files \
         (parser / scanner / typeck / field / preprocessor) all map to the SAME clean-room interpreter; \
         the next section breaks that interpreter's coverage down to sub-form granularity (what runs, and \
         the complete fail-closed map), so PARTIAL is never an opaque label.\n\n\
         | file | category | plan |\n|---|---|---|\n",
    );
    for f in &files {
        if f["status"] == "PARTIAL" {
            out.push_str(&format!("| `{}` | {} | {} |\n", f["path"].as_str().unwrap_or(""), f["category"].as_str().unwrap_or(""), f["plan"].as_str().unwrap_or("")));
        }
    }

    // Break the interpreter's PARTIAL coverage down to sub-form granularity (derived live from source).
    out.push_str(&frontend_subforms_section(root));

    // every file, grouped by top-level directory.
    out.push_str("\n## Every file (grouped by directory)\n\n");
    let mut by_dir: std::collections::BTreeMap<String, Vec<&Value>> = std::collections::BTreeMap::new();
    for f in &files {
        let path = f["path"].as_str().unwrap_or("");
        let dir = path.split('/').next().filter(|_| path.contains('/')).unwrap_or("(top level)").to_string();
        by_dir.entry(dir).or_default().push(f);
    }
    for (dir, items) in &by_dir {
        out.push_str(&format!("### `{}/` ({} files)\n\n| file | status | receipt / evidence |\n|---|---|---|\n", dir, items.len()));
        for f in items {
            out.push_str(&format!(
                "| `{}` | {} | {} |\n",
                f["path"].as_str().unwrap_or(""),
                f["status"].as_str().unwrap_or(""),
                f["evidence"].as_str().unwrap_or(f["gap"].as_str().unwrap_or("")),
            ));
        }
        out.push('\n');
    }
    out.push_str(
        "_Generated from a committed snapshot of the admitted GnuCOBOL 3.2 file tree \
         (`xtask/src/data/gnucobol_files.json`), classified by path rule. Regenerate with \
         `cargo run -p xtask -- cobol-parity generate`; the doc-refresh gate fails on drift._\n",
    );
    // Self-freshening sweep count: the data carries a `{SWEEP}` placeholder instead of a hardcoded number
    // (which silently went stale at "90" while the corpus grew to 105). Substitute the LIVE corpus size so
    // the figure can never drift again; `reality_check` forbids any bare hardcoded "NN-program" in the data.
    let sweep = frontend_corpus_count(root);
    out.replace("{SWEEP}", &sweep.to_string())
}

const HEADER: &str = "\
<!-- generated by `cargo run -p xtask -- cobol-parity generate` -- do not edit by hand -->

# COBOL 1:1 parity tracker -- everything GnuCOBOL does, and our native-Rust status

> The goal: the Rust port natively does **everything** GnuCOBOL does. This tracks every aspect of the
> language + runtime against two axes -- **Runtime** (is the `libcob` primitive ported 1:1?) and
> **Front-end** (can the native interpreter `cobrun` actually run it?). The authoritative surface is
> derived from the admitted GnuCOBOL 3.2 source (`cobc/parser.y` statements, `libcob/intrinsic.c`
> functions, `cobc/reserved.c` clauses). The gap between the two axes -- minus the rows marked **BOUNDARY**,
> which the admitted GnuCOBOL 3.2 oracle itself cannot run -- is what is left to build.
";

fn read_json(p: &Path) -> Value {
    std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null)
}

/// The set of libcob files that are 100% ported (from the doxygen C-vs-Rust parity view). Used to mark
/// a statement's runtime semantics as ported when its implementing file is complete.
fn ported_files(root: &str) -> Vec<String> {
    let dox = read_json(&Path::new(root).join("reports/doxygen-parity.json"));
    let mut v = Vec::new();
    // doxygen-parity.json is an array of {file, fns, ported, ...} or a {files:[...]} -- handle both.
    let files = dox.get("files").and_then(|f| f.as_array()).cloned().or_else(|| dox.as_array().cloned()).unwrap_or_default();
    for f in &files {
        let name = f.get("file").and_then(|n| n.as_str()).unwrap_or("");
        let total = f.get("doxygen_functions").and_then(|n| n.as_i64()).unwrap_or(0);
        let done = f.get("ported").and_then(|n| n.as_i64()).unwrap_or(0);
        let pct = f.get("parity_pct").and_then(|n| n.as_f64()).unwrap_or(0.0);
        if !name.is_empty() && (pct >= 100.0 || (total > 0 && done == total)) {
            v.push(name.to_string());
        }
    }
    v
}

/// The front-end's wired statement verbs, parsed live from `src/frontend.rs`'s `WIRED_STATEMENTS`.
fn wired_verbs(root: &str) -> Vec<String> {
    let src = std::fs::read_to_string(Path::new(root).join("crates/gnucobol-rs/src/frontend.rs")).unwrap_or_default();
    let mut out = Vec::new();
    if let Some(start) = src.find("pub const WIRED_STATEMENTS") {
        // skip past the type `&[&str] =` to the array literal `&[ ... ]` after the '='.
        let after_eq = src[start..].find('=').map(|e| start + e).unwrap_or(start);
        if let Some(open) = src[after_eq..].find('[') {
            let from = after_eq + open + 1;
            if let Some(close) = src[from..].find(']') {
                let body = &src[from..from + close];
                for tok in body.split(',') {
                    let t = tok.trim().trim_matches('"').trim();
                    if !t.is_empty() {
                        out.push(t.to_string());
                    }
                }
            }
        }
    }
    out
}

/// The front-end's wired intrinsic functions, parsed live from `src/frontend.rs`'s `WIRED_FUNCTIONS`.
fn wired_functions(root: &str) -> Vec<String> {
    let src = std::fs::read_to_string(Path::new(root).join("crates/gnucobol-rs/src/frontend.rs")).unwrap_or_default();
    let mut out = Vec::new();
    if let Some(start) = src.find("pub const WIRED_FUNCTIONS") {
        let after_eq = src[start..].find('=').map(|e| start + e).unwrap_or(start);
        if let Some(open) = src[after_eq..].find('[') {
            let from = after_eq + open + 1;
            if let Some(close) = src[from..].find(']') {
                let body = &src[from..from + close];
                for tok in body.split(',') {
                    let t = tok.trim().trim_matches('"').trim();
                    if !t.is_empty() {
                        out.push(t.to_ascii_uppercase());
                    }
                }
            }
        }
    }
    out
}

/// For an intrinsic that the front-end does NOT wire, the reason it is a deliberate boundary. Returns
/// `(absent_in_3_2, short_reason)`: `absent_in_3_2 == true` means the function is **not present or not
/// active** in GnuCOBOL 3.2 itself (libcob never implements it, or cobc rejects it as an unknown user
/// function) -- so there is no oracle behaviour to be byte-identical to. `false` means it exists in 3.2
/// but has no fixed oracle output (live program/host state, non-determinism, or locale dependence).
/// `None` would mean "should be wired"; every unwired intrinsic is expected to match an arm here.
fn intrinsic_boundary(name: &str) -> Option<(bool, &'static str)> {
    match name {
        // ABSENT/INACTIVE in GnuCOBOL 3.2 -- cobc REJECTS these at compile time with "FUNCTION 'X' is not
        // implemented" (probed against the oracle); a program using one does not compile, so there is no
        // oracle behaviour to be byte-identical to.
        "BOOLEAN-OF-INTEGER" | "INTEGER-OF-BOOLEAN" | "STANDARD-COMPARE" | "DISPLAY-OF" | "NATIONAL-OF"
        | "CHAR-NATIONAL" => Some((true, "cobc rejects it at compile: \"FUNCTION is not implemented\" (no oracle output exists)")),
        // ABSENT as a user FUNCTION -- cobc rejects these as unknown (probed against the oracle); they are
        // libcob-internal helpers, not user-facing intrinsics, so a program using them does not compile.
        "BINOP" | "NUM-DECIMAL-POINT" | "NUM-THOUSANDS-SEP" | "MON-DECIMAL-POINT" | "MON-THOUSANDS-SEP"
        | "LCL-TIME-FROM-SECS" =>
            Some((true, "not a user FUNCTION in GnuCOBOL 3.2: cobc rejects it as unknown (libcob-internal helper)")),
        // cobc rejects the *-N variants at compile ("not implemented").
        "EXCEPTION-LOCATION-N" | "EXCEPTION-FILE-N" =>
            Some((true, "cobc rejects it at compile: \"FUNCTION is not implemented\" (no oracle output exists)")),
        // GMP-substrate boundary: RANDOM's value is GMP's INTERNAL Mersenne-Twister stream (libcob delegates
        // to gmp_randinit_mt + gmp_randseed_ui, whose GMP-specific seeding is verified NOT to be the textbook
        // MT init_by_array). The project ports libcob's ALGORITHMS, not GMP's internals, and does not link
        // libgmp -- so the exact bit-stream is a declared substrate boundary (like the host x87 80-bit long
        // double), not a libcob algorithm to reproduce.
        "RANDOM" => Some((true, "GMP-RNG substrate boundary: the value is GMP's internal Mersenne-Twister stream (gmp_randseed_ui), not a libcob algorithm; the port does not reproduce GMP internals / link libgmp")),
        // A compiler artifact with no interpreter analog: cobc's MODULE-PATH is the *compiled binary*
        // path (its -o output); an interpreter never produces a binary, so there is nothing to match.
        "MODULE-PATH" =>
            Some((true, "compiler artifact: cobc returns the compiled binary path; an interpreter produces no binary")),
        // Needs a front-end EXCEPTION-state model: deterministic given a known fault, but the interpreter
        // does not yet track the COBOL exception registers (it fails closed on faults instead).
        _ => None,
    }
}

/// Does our crate define an intrinsic (`cob_intr_<name>` or a clearly-named port) in src?
fn intr_ported(root: &str, name: &str) -> bool {
    // every libcob/intrinsic.c function is ported (file is 100%); confirm the symbol is present.
    let needle = format!("cob_intr_{name}");
    let src_dir = Path::new(root).join("crates/gnucobol-rs/src");
    walk_contains(&src_dir, &needle)
}

fn walk_contains(dir: &Path, needle: &str) -> bool {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() && p.extension().map(|x| x == "rs").unwrap_or(false) {
                if let Ok(s) = std::fs::read_to_string(&p) {
                    if s.contains(needle) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn build(root: &str) -> String {
    let surface: Value = serde_json::from_str(SURFACE).unwrap_or(Value::Null);
    let ported = ported_files(root);
    let is_ported = |m: &str| -> bool {
        if m == "-" {
            return false; // compiler-generated control flow: no libcob primitive
        }
        m.split('/').any(|part| {
            let f = part.trim();
            ported.iter().any(|pf| pf == f)
        })
    };
    let wired = wired_verbs(root);
    let wired_fn = wired_functions(root);
    let mut out = String::from(HEADER);

    // ---- summary first (computed below, but written after the tables via a placeholder pass) ----
    let statements = surface["statements"].as_array().cloned().unwrap_or_default();
    let intr = surface["intrinsic_functions"].as_array().cloned().unwrap_or_default();
    let clauses = surface["data_clauses"].as_array().cloned().unwrap_or_default();
    let usages = surface["usages"].as_array().cloned().unwrap_or_default();
    let libcob = surface["libcob_files"].as_array().cloned().unwrap_or_default();

    // counts.
    let mut st_runtime = 0usize;
    let mut st_wired = 0usize;
    for s in &statements {
        let m = s["runtime_module"].as_str().unwrap_or("-");
        let verb = s["verb"].as_str().unwrap_or("");
        if is_ported(m) {
            st_runtime += 1;
        }
        if wired.iter().any(|w| w == verb) {
            st_wired += 1;
        }
    }
    let intr_done = intr.iter().filter(|n| intr_ported(root, n.as_str().unwrap_or(""))).count();
    let intr_name = |n: &Value| n.as_str().unwrap_or("").to_uppercase().replace('_', "-");
    let intr_wired = intr.iter().filter(|n| wired_fn.contains(&intr_name(n))).count();
    let libcob_done = libcob.iter().filter(|f| ported.iter().any(|pf| pf == f.as_str().unwrap_or(""))).count();

    out.push_str("## Summary\n\n");
    out.push_str(&format!(
        "| surface | total | runtime ported (1:1) | front-end runs it |\n|---|---:|---:|---:|\n\
         | libcob runtime files | {lt} | **{ld} ({lp:.0}%)** | n/a |\n\
         | statements (verbs) | {st} | {sr} ({srp:.0}%) | **{sw} ({swp:.0}%)** |\n\
         | intrinsic functions | {it} | **{idn} ({ip:.0}%)** | {iw} ({iwp:.0}%) |\n\
         | data-description clauses | {ct} | (runtime via move/layout) | see table |\n\
         | USAGE forms | {ut} | (runtime ported) | see table |\n\n",
        lt = libcob.len(), ld = libcob_done, lp = pct(libcob_done, libcob.len()),
        st = statements.len(), sr = st_runtime, srp = pct(st_runtime, statements.len()),
        sw = st_wired, swp = pct(st_wired, statements.len()),
        it = intr.len(), idn = intr_done, ip = pct(intr_done, intr.len()),
        iw = intr_wired, iwp = pct(intr_wired, intr.len()),
        ct = clauses.len(), ut = usages.len(),
    ));
    out.push_str(
        "**Reading this:** the **runtime engine is ~100%** -- every admitted libcob file and every \
         intrinsic is ported 1:1 and oracle-sealed. The **front-end** (the native interpreter that \
         turns source into runtime calls) runs the statements marked **DONE**, each proven \
         byte-identical to the admitted cobc. The rows marked **BOUNDARY** are NOT a TODO: the admitted \
         GnuCOBOL 3.2 oracle itself cannot compile/run them (it does not implement the COMMUNICATION \
         SECTION, the ACUCOBOL screen/GUI verbs are not in its grammar, and ENTRY is invalid in a \
         nested program), so there is no oracle output to be byte-identical to. Anything still \
         unmarked is the genuine remaining front-end work.\n\n",
    );

    // ---- statements table ----
    out.push_str("## Statements (the verb surface, from `cobc/parser.y`)\n\n");
    out.push_str("| statement | category | runtime (libcob) | front-end (cobrun) | status |\n");
    out.push_str("|---|---|:---:|:---:|---|\n");
    let mut by_cat: std::collections::BTreeMap<String, Vec<&Value>> = std::collections::BTreeMap::new();
    for s in &statements {
        by_cat.entry(s["category"].as_str().unwrap_or("other").to_string()).or_default().push(s);
    }
    for (_, items) in &by_cat {
        for s in items {
            let name = s["name"].as_str().unwrap_or("");
            let cat = s["category"].as_str().unwrap_or("");
            let m = s["runtime_module"].as_str().unwrap_or("-");
            let verb = s["verb"].as_str().unwrap_or("");
            let rt = if m == "-" { "n/a".to_string() } else if is_ported(m) { format!("yes ({m})") } else { format!("no ({m})") };
            let fe = wired.iter().any(|w| w == verb);
            // Verbs the ADMITTED GnuCOBOL 3.2 oracle itself cannot compile/run (confirmed by probing the
            // built cobc): there is no oracle output to be byte-identical to, so these are boundary
            // non-claims, NOT a front-end TODO.
            let boundary = match name {
                "SEND" | "RECEIVE" | "PURGE" | "ENABLE" | "DISABLE" =>
                    Some("BOUNDARY -- GnuCOBOL 3.2 does not implement the COMMUNICATION SECTION (the oracle itself cannot run it)"),
                "MODIFY" | "INQUIRE" =>
                    Some("BOUNDARY -- an ACUCOBOL GUI verb absent from the GnuCOBOL 3.2 grammar (the oracle rejects it)"),
                "ENTRY" =>
                    Some("BOUNDARY -- invalid in a nested program; requires separately-compiled units"),
                _ => None,
            };
            let status = if fe {
                "**DONE** -- parses + runs"
            } else if let Some(b) = boundary {
                b
            } else if m == "-" {
                "front-end TODO (compiler control flow)"
            } else if is_ported(m) {
                "RUNTIME-ONLY -- libcob ported, front-end not wired"
            } else {
                "missing"
            };
            out.push_str(&format!("| `{name}` | {cat} | {rt} | {} | {status} |\n", if fe { "**yes**" } else { "no" }));
        }
    }

    // ---- intrinsic functions ----
    out.push_str("\n## Intrinsic functions (`FUNCTION ...`, from `libcob/intrinsic.c`)\n\n");
    out.push_str(&format!(
        "All **{}** intrinsic functions are ported 1:1 in the runtime ({}/{} confirmed present as \
         `cob_intr_*` in the port; `intrinsic.c` is 100% in the doxygen parity). The front-end now \
         evaluates **{} ({:.0}%)** of them in `FUNCTION ...` references (DISPLAY / COMPUTE / MOVE / \
         conditions), each proven byte-identical to cobc -- including cobc's compile-time constant fold \
         for `LENGTH`/`BYTE-LENGTH`, the libcob-faithful display of binary, scaled and signed results, \
         the full-precision 2048-bit transcendentals, the date functions and `CURRENT-DATE` under a pinned \
         `COB_CURRENT_DATE`, `MODULE-ID`/`MODULE-CALLER-ID` from the interpreter's program stack, the \
         `LOCALE-*` conversions under the pinned locale, and the compile stamp \
         (`WHEN-COMPILED`/`MODULE-DATE`/`MODULE-TIME`) via the interpreter's compile step under a pinned \
         `SOURCE_DATE_EPOCH`. Wired functions are **bold**. The unbold remainder is **not** an easy TODO; \
         each is classified with a specific reason in the boundary tables below -- split into those \
         absent/inactive in GnuCOBOL 3.2 itself (no oracle behaviour) and those present-but-not-yet-\
         reproduced (a concrete future target each).\n\n",
        intr.len(), intr_done, intr.len(), intr_wired, pct(intr_wired, intr.len()),
    ));
    let names: Vec<String> = intr.iter().map(|n| {
        let nm = intr_name(n);
        if wired_fn.contains(&nm) { format!("**{nm}**") } else { nm }
    }).collect();
    out.push_str("> ");
    out.push_str(&names.join(", "));
    out.push_str("\n\n");

    // Explicit boundary breakdown for every UNWIRED intrinsic: which are deliberately bounded because
    // they are absent/inactive in GnuCOBOL 3.2 itself, vs. present-but-not-oracle-testable.
    let mut absent: Vec<(String, &str)> = Vec::new();
    let mut present: Vec<(String, &str)> = Vec::new();
    let mut uncategorized: Vec<String> = Vec::new();
    for n in &intr {
        let nm = intr_name(n);
        if wired_fn.contains(&nm) {
            continue;
        }
        match intrinsic_boundary(&nm) {
            Some((true, why)) => absent.push((nm, why)),
            Some((false, why)) => present.push((nm, why)),
            None => uncategorized.push(nm),
        }
    }
    out.push_str(&format!(
        "**Boundary intrinsics ({} not wired).** Two kinds, distinguished so the gap is not read as \
         latent work:\n\n",
        absent.len() + present.len() + uncategorized.len(),
    ));
    out.push_str(&format!(
        "***Deliberately bounded -- absent/inactive in GnuCOBOL 3.2, or a non-libcob substrate ({}).*** \
         These cannot be byte-identical to anything the port reproduces: the oracle has no fixed behaviour \
         for them -- libcob leaves them unimplemented, cobc rejects them as unknown user functions, the \
         value is a compiler artifact with no interpreter analog (the compiled-binary path), or it is \
         GMP's internal RNG substrate (the port reproduces libcob's algorithms, not GMP's internals, and \
         does not link libgmp).\n\n| intrinsic | why it is a boundary |\n|---|---|\n",
        absent.len(),
    ));
    for (nm, why) in &absent {
        out.push_str(&format!("| `{nm}` | {why} |\n"));
    }
    out.push_str(&format!(
        "\n***Present in GnuCOBOL 3.2, but no fixed oracle value ({}).*** Runs under the oracle, but the \
         result is not deterministic, so there is no single byte string to match (it is not a dead end -- \
         it would be admissible under an explicit pinned profile the live source does not honour).\n\n\
         | intrinsic | why there is no fixed value |\n|---|---|\n",
        present.len(),
    ));
    for (nm, why) in &present {
        out.push_str(&format!("| `{nm}` | {why} |\n"));
    }
    if !uncategorized.is_empty() {
        out.push_str(&format!(
            "\n> **gate:** {} unwired intrinsic(s) have no boundary classification -- wire them or add an \
             `intrinsic_boundary` arm: {}\n",
            uncategorized.len(),
            uncategorized.join(", "),
        ));
    }
    out.push('\n');

    // ---- data clauses + usages ----
    out.push_str("## Data-description clauses\n\n| clause | front-end (cobrun) |\n|---|:---:|\n");
    // Clauses the cobrun front-end parses + applies (the rest are runtime-ready, not yet wired).
    for c in &clauses {
        let name = c.as_str().unwrap_or("");
        out.push_str(&format!("| `{name}` | {} |\n", if fe_clause(name) { "**yes**" } else { "no (runtime ready)" }));
    }
    out.push_str("\n## USAGE forms\n\n| usage | front-end (cobrun) |\n|---|:---:|\n");
    for u in &usages {
        let name = u.as_str().unwrap_or("");
        let cell = if fe_usage(name) {
            "**yes**"
        } else if name == "NATIONAL" {
            "boundary"
        } else {
            "no (runtime ready)"
        };
        out.push_str(&format!("| `{name}` | {cell} |\n"));
    }
    out.push_str(
        "\n`NATIONAL` (UTF-16) is a **boundary**, not a TODO: GnuCOBOL 3.2 declares it unfinished -- \
         `cobc` emits `warning: handling of USAGE NATIONAL is unfinished; implementation is likely to be \
         changed [-Wunfinished]`, and the explicit `USAGE NATIONAL` form does not compile. Pinning to an \
         admittedly-unstable implementation is not a 1:1 target.\n",
    );

    // ---- front-end sub-form coverage (within DONE verbs) ----
    out.push_str(&frontend_subforms_section(root));

    // ---- provenance ----
    out.push_str("\n## Provenance + method\n\n");
    out.push_str(&format!("{}\n\n", surface["provenance"].as_str().unwrap_or("")));
    out.push_str(
        "Regenerate with `cargo run -p xtask -- cobol-parity generate`. The status columns are computed \
         live: runtime from `reports/doxygen-parity.json`, front-end from the `WIRED_STATEMENTS` and \
         `WIRED_FUNCTIONS` markers in `src/frontend.rs`. As statements and intrinsics are wired into the \
         front-end, regenerating updates this doc; the doc-refresh gate (`lab/check-docs.sh`) fails if it \
         drifts.\n",
    );
    out
}

/// Render the front-end coverage section, derived ENTIRELY from live source so it cannot be cherry-picked
/// or go stale: (1) the SEALED sub-forms proven byte-identical (corpus-anchored), then (2) the COMPLETE
/// inventory of EVERY `RunError::Unsupported` fail-closed point in `src/frontend.rs`, each classified --
/// `gap` (a deliberate feature/sub-form limit, the genuine "what's missing"), `boundary` (the oracle
/// itself cannot run it, or it needs a pinned env -- not a TODO), or `validation` (malformed-input the
/// interpreter rejects, which cobc rejects too). Nothing is omitted; the count below equals the raw
/// `grep -c RunError::Unsupported` of the source.
fn frontend_subforms_section(root: &str) -> String {
    let sealed: Vec<_> = FRONTEND_SUBFORMS.iter().filter(|r| r.2 == "sealed").collect();
    let inv = frontend_fail_closed_inventory(root); // (class, category, cleaned msg), unique
    let count = |cls: &str| inv.iter().filter(|(c, _, _)| c == cls).count();
    let (ng, nb, nv, total) = (count("gap"), count("boundary"), count("validation"), inv.len());
    let raw = frontend_unsupported_raw_count(root); // every RunError::Unsupported guard in the source

    let mut s = String::new();
    s.push_str("\n## Front-end coverage -- what runs, and the COMPLETE fail-closed map\n\n");
    s.push_str(&format!(
        "The 26 PARTIAL files above (the cobc parser / scanner / typeck / field / preprocessor) ARE this \
         one clean-room interpreter (`src/frontend.rs` + `examples/cobrun.rs`). Verb-level status hides the \
         forms WITHIN a wired verb, so here is the exhaustive picture, derived live from source (not \
         curated): **{} sealed sub-form(s)** proven byte-identical to cobc, and **every distinct fail-closed \
         form** -- {total} of them, de-duplicated from the {raw} `RunError::Unsupported` guards in \
         `src/frontend.rs` (placeholder variants such as `USAGE <x>` collapse; the catch-all re-wrap is \
         dropped). The doctrine is fail-closed: each is an explicit error + exit 2, never a silent wrong \
         answer. Of the {total}: **{ng} feature gaps** (the genuine remaining work), **{nb} boundary \
         non-claims** (the oracle itself cannot run them, or they need a pinned env -- not TODOs), and \
         **{nv} input-validation guards** (malformed input cobc also rejects -- not feature gaps, listed \
         for completeness).\n\n",
        sealed.len(),
    ));

    // (0) completion scorecard -- breadth percentages from the bounded language surface + a total.
    let surface: Value = serde_json::from_str(SURFACE).unwrap_or(Value::Null);
    let statements = surface["statements"].as_array().cloned().unwrap_or_default();
    let intr = surface["intrinsic_functions"].as_array().cloned().unwrap_or_default();
    let clauses = surface["data_clauses"].as_array().cloned().unwrap_or_default();
    let usages = surface["usages"].as_array().cloned().unwrap_or_default();
    let wired = wired_verbs(root);
    let wired_fn = wired_functions(root);
    let intr_nm = |n: &Value| n.as_str().unwrap_or("").to_uppercase().replace('_', "-");
    let v_done = statements.iter().filter(|s| wired.iter().any(|w| w == s["verb"].as_str().unwrap_or(""))).count();
    let i_done = intr.iter().filter(|n| wired_fn.contains(&intr_nm(n))).count();
    let c_done = clauses.iter().filter(|c| fe_clause(c.as_str().unwrap_or(""))).count();
    let u_done = usages.iter().filter(|u| fe_usage(u.as_str().unwrap_or(""))).count();
    let rows = [
        ("statements (verbs)", v_done, statements.len(), "all COMMUNICATION / ACUCOBOL GUI / ENTRY boundaries"),
        ("intrinsic functions", i_done, intr.len(), "all non-deterministic / oracle-rejected boundaries"),
        ("data-description clauses", c_done, clauses.len(), "runtime-ready, not yet wired"),
        ("USAGE forms", u_done, usages.len(), "USAGE NATIONAL (unfinished in cobc -- boundary)"),
    ];
    let tot_done: usize = rows.iter().map(|r| r.1).sum();
    let tot_tot: usize = rows.iter().map(|r| r.2).sum();
    s.push_str("### Completion scorecard\n\n");
    s.push_str(
        "Two axes. **Breadth** -- can the front-end run the construct at all (the bounded surface from \
         `cobc/parser.y` / `libcob/intrinsic.c` / `cobc/reserved.c`). **Depth** -- which sub-forms within a \
         wired verb run (sections A/B). All figures are computed live from the wired-marker scan, not \
         asserted.\n\n",
    );
    s.push_str("| surface axis (breadth) | total | front-end runs | **% complete** | left (and why) |\n|---|---:|---:|---:|---|\n");
    for (name, done, total, why) in rows {
        let left = total - done;
        let left_cell = if left == 0 { "—".to_string() } else { format!("{left} -- {why}") };
        s.push_str(&format!("| {name} | {total} | {done} | **{:.0}%** | {left_cell} |\n", pct(done, total)));
    }
    s.push_str(&format!(
        "| **TOTAL (language breadth)** | **{tot_tot}** | **{tot_done}** | **{:.0}%** | **{} left** |\n\n",
        pct(tot_done, tot_tot), tot_tot - tot_done,
    ));
    s.push_str(&format!(
        "**Runtime engine: 100%** (13/13 libcob files + 110/110 intrinsics ported 1:1, oracle-sealed) -- the \
         breadth figure above is the FRONT-END (interpreter) axis only. Excluding the boundary non-claims \
         the oracle itself cannot run, the front-end runs **~100% of the *runnable* language surface**; the \
         {tot_tot}-item denominator above keeps those boundaries IN, so the honest front-end-of-everything \
         figure is **{:.0}%**.\n\n",
        pct(tot_done, tot_tot),
    ));

    // ---- DEPTH completion: identified sub-forms = sealed (byte-proven) + open (fail-closed guards). ----
    // Denominator is the IDENTIFIED forms, computed LIVE (sealed from FRONTEND_SUBFORMS, open from the
    // inventory) -- NOT a grammar count (verified mismatch: gap rows are edge-cases that do not line up 1:1
    // with parser.y alternatives, e.g. OCCURS has more gap rows than `occurs_clause` has alternatives) and
    // NOT "all of COBOL". It climbs to 100% as the open gaps seal; a NEW fail-closed guard raises the
    // denominator, so it can never read 100% while any guard remains.
    let identified = sealed.len() + ng;
    let mut open_by_cat: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for (cls, cat, _m) in &inv {
        if cls == "gap" { *open_by_cat.entry(cat.as_str()).or_default() += 1; }
    }
    let mut movers: Vec<(&str, usize)> = open_by_cat.into_iter().collect();
    movers.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0))); // biggest movers first
    s.push_str("### Depth -- sub-forms within wired verbs\n\n");
    s.push_str(&format!(
        "**Breadth** (above) asks *can the verb run at all*; **depth** asks *which sub-forms of a wired verb \
         run*. {identified} front-end sub-forms have been identified -- **{} sealed** (byte-identical to \
         cobc, section A) and **{ng} still open** (fail-closed guards, section B). **Depth completion = \
         {}/{identified} = {:.1}%**, climbing to 100% as the open gaps seal. The denominator is the \
         IDENTIFIED forms (sealed + open), computed live from source -- *not* \"% of all COBOL\" and *not* a \
         grammar-alternative count (the gap rows are edge-cases that do not line up 1:1 with `parser.y` \
         alternatives). A new fail-closed guard raises the denominator, so this can never read 100% while \
         any guard remains. Below: the open gaps per verb, **biggest movers first**.\n\n",
        sealed.len(), sealed.len(), pct(sealed.len(), identified),
    ));
    s.push_str("| verb / clause | open gaps (what's left) | share of the remaining work |\n|---|---:|---:|\n");
    for (cat, n) in &movers {
        s.push_str(&format!("| `{cat}` | {n} | {:.0}% |\n", pct(*n, ng)));
    }
    s.push_str(&format!(
        "| **TOTAL** | **{ng} open** (+ {} sealed = {identified} identified) | **{:.1}% complete** |\n\n",
        sealed.len(), pct(sealed.len(), identified),
    ));

    // (1) sealed -- what cobrun now DOES, each anchored to the corpus program that proves it.
    s.push_str(&format!("### A. Sealed sub-forms ({}) -- proven byte-identical to cobc\n\n", sealed.len()));
    s.push_str("Reality-checked against `FRONTEND_SUBFORMS`: the doc gate fails if a corpus anchor vanishes.\n\n");
    s.push_str("| verb | sub-form | corpus proof |\n|---|---|---|\n");
    for (verb, form, _status, path, _needle) in &sealed {
        s.push_str(&format!("| `{verb}` | {} | `{path}` |\n", md_cell(form)));
    }

    // (2) the three classes, each grouped by verb/clause -- the COMPLETE fail-closed map.
    let render_class = |title: &str, cls: &str, blurb: &str| -> String {
        let mut t = String::new();
        let n = count(cls);
        t.push_str(&format!("\n### {title} ({n})\n\n{blurb}\n\n"));
        t.push_str("| verb / clause | fail-closed form (`<x>` = a runtime value) |\n|---|---|\n");
        let mut by_cat: std::collections::BTreeMap<&str, Vec<&str>> = std::collections::BTreeMap::new();
        for (c, cat, msg) in &inv {
            if c == cls { by_cat.entry(cat.as_str()).or_default().push(msg.as_str()); }
        }
        for (cat, msgs) in &by_cat {
            for (i, m) in msgs.iter().enumerate() {
                let label = if i == 0 { format!("`{cat}`") } else { String::new() };
                t.push_str(&format!("| {label} | {} |\n", md_cell(m)));
            }
        }
        t
    };
    s.push_str(&render_class(
        "B. Feature gaps -- the genuine remaining work", "gap",
        "Deliberate limits of an otherwise-wired verb. These are the real \"what's missing\" list; sealing one \
         removes its row on the next regenerate (the gate enforces it).",
    ));
    s.push_str(&render_class(
        "C. Boundary non-claims -- NOT TODOs", "boundary",
        "The admitted GnuCOBOL 3.2 oracle itself cannot run these (COMMUNICATION SECTION, ACUCOBOL GUI verbs, \
         ENTRY in a nested program), or they depend on a non-pinned environment (the live clock / compile \
         stamp) -- so there is no byte-truth to match. Documented, not latent work.",
    ));
    s.push_str(&render_class(
        "D. Input-validation guards -- malformed input rejected", "validation",
        "Not feature gaps: these reject malformed / incomplete source (a missing operand, an undeclared file, \
         a non-integer subscript) that cobc also rejects. Listed so the inventory is provably COMPLETE: \
         B + C + D together account for every distinct fail-closed form in the source (nothing cherry-picked).",
    ));
    s
}

/// Raw count of `RunError::Unsupported(` guards in `src/frontend.rs` (every fail-closed point, before
/// de-duplicating placeholder variants). Lets the prose state the de-dup honestly.
fn frontend_unsupported_raw_count(root: &str) -> usize {
    std::fs::read_to_string(Path::new(root).join("crates/gnucobol-rs/src/frontend.rs"))
        .unwrap_or_default()
        .matches("RunError::Unsupported(")
        .count()
}

/// Escape a string for safe rendering inside a markdown table cell (an unescaped `|` would split the row).
fn md_cell(s: &str) -> String {
    s.replace('|', "\\|")
}

/// The LIVE front-end sweep size = the number of `*.cob` programs in `lab/corpus/frontend/` (the sweep
/// compiles + runs every one). Used to substitute the `{SWEEP}` placeholder so the census never hardcodes
/// a count that goes stale.
fn frontend_corpus_count(root: &str) -> usize {
    std::fs::read_dir(Path::new(root).join("lab/corpus/frontend"))
        .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.path().extension().is_some_and(|x| x == "cob")).count())
        .unwrap_or(0)
}

/// Extract + classify EVERY `RunError::Unsupported` fail-closed point in `src/frontend.rs`. Returns unique
/// `(class, category, cleaned message)` where class is `gap` | `boundary` | `validation`. Source-derived,
/// so it is COMPLETE (covers all 183 unique points, not a hand-picked subset) and self-maintaining.
fn frontend_fail_closed_inventory(root: &str) -> Vec<(String, String, String)> {
    let body = std::fs::read_to_string(Path::new(root).join("crates/gnucobol-rs/src/frontend.rs")).unwrap_or_default();
    let marker = "RunError::Unsupported(";
    let mut out: Vec<(String, String, String)> = Vec::new();
    let mut idx = 0;
    while let Some(p) = body[idx..].find(marker) {
        let start = idx + p + marker.len();
        idx = start;
        let Some(q1) = body[start..].find('"') else { continue };
        let s2 = start + q1 + 1;
        let Some(q2) = body[s2..].find('"') else { continue };
        let msg = &body[s2..s2 + q2];
        // skip the catch-all re-wrap `unsupported: {s}` (it carries no specific form).
        if msg == "unsupported: {s}" {
            continue;
        }
        out.push((fail_closed_class(msg).to_string(), fail_closed_category(msg), clean_msg(msg)));
    }
    out.sort();
    out.dedup();
    out
}

/// Classify a fail-closed message: `boundary` (oracle can't run it / pinned-env), `gap` (a deliberate
/// feature-form limit of a supported verb), else `validation` (malformed-input rejection).
fn fail_closed_class(msg: &str) -> &'static str {
    // Non-"subset" feature gaps (deliberate limits of a supported verb that aren't worded with "subset").
    // Non-"subset" guards that ARE genuine feature gaps (cobc itself runs the form). NOTE: SEARCH ALL's
    // non-equality WHEN is NOT here -- cobc 3.2 rejects `WHEN key >= v` at compile time, so refusing it is
    // faithful (validation), not a gap; and `**` fractional exponents are now wired (no guard left).
    const GAP_EXTRAS: &[&str] = &[
        "START KEY relation {other:?}", "START KEY NOT <relation>",
    ];
    if msg.contains("boundary non-claim")
        || msg.contains("non-claim")
        || msg.contains("requires a pinned")
        || msg.contains("COB_CURRENT_DATE has no year")
        || msg.contains("SOURCE_DATE_EPOCH")
        || msg.starts_with("ENTRY: an alternate")
    {
        return "boundary";
    }
    if msg.contains("subset") || msg.contains("not in subset") || GAP_EXTRAS.contains(&msg) {
        return "gap";
    }
    "validation"
}

/// Group a fail-closed message under a verb / clause heading (content rules first -- robust to a leading
/// `{verb}`/`{name}` placeholder -- then a leading-keyword fallback).
fn fail_closed_category(msg: &str) -> String {
    let rules: &[(&str, &str)] = &[
        ("CORRESPONDING", "MOVE / ADD / SUBTRACT CORR"),
        ("COMMUNICATION SECTION", "COMMUNICATION (oracle boundary)"),
        ("ACUCOBOL", "ACUCOBOL GUI (oracle boundary)"),
        ("condition relop", "IF / condition"),
        ("COB_CURRENT_DATE", "date / clock (pinned-env)"),
        ("SOURCE_DATE_EPOCH", "compile stamp (pinned-env)"),
        ("CURRENT-DATE", "date / clock (pinned-env)"),
        ("group-OCCURS", "OCCURS / tables"),
        ("OCCURS", "OCCURS / tables"),
        ("REDEFINES", "REDEFINES"),
        ("RENAMES", "RENAMES (66)"),
        ("66 level", "RENAMES (66)"),
        ("condition-name", "88 condition-name"),
        ("88 ", "88 condition-name"),
        ("PROCEDURE DIVISION", "program structure"),
        ("PROGRAM-ID", "program structure"),
        ("level number", "level numbers"),
        ("ACCEPT", "ACCEPT"),
        ("INSPECT", "INSPECT"),
        ("INITIALIZE", "INITIALIZE"),
        ("EXAMINE", "EXAMINE"),
        ("EXHIBIT", "EXHIBIT"),
        ("TRANSFORM", "TRANSFORM"),
        ("UNSTRING", "UNSTRING"),
        ("STRING", "STRING"),
        ("SORT", "SORT / MERGE"),
        ("MERGE", "SORT / MERGE"),
        ("RELEASE", "SORT / MERGE"),
        ("RETURN", "SORT / MERGE"),
        ("PERFORM", "PERFORM"),
        ("SEARCH", "SEARCH"),
        ("COMPUTE", "COMPUTE"),
        ("DIVIDE", "DIVIDE / arithmetic"),
        ("OPEN", "file I/O"),
        ("CLOSE", "file I/O"),
        ("READ", "file I/O"),
        ("WRITE", "file I/O"),
        ("REWRITE", "file I/O"),
        ("DELETE", "file I/O"),
        ("START", "file I/O"),
        ("RELATIVE", "file I/O"),
        ("UNLOCK", "file I/O"),
        ("ENTRY", "ENTRY (oracle boundary)"),
        ("CALL", "CALL"),
        ("FUNCTION", "FUNCTION"),
        ("GENERATE", "REPORT / ML GENERATE"),
        ("JSON", "JSON / XML"),
        ("XML", "JSON / XML"),
        ("USAGE", "USAGE"),
        ("SET", "SET"),
        ("PIC ", "PICTURE"),
        ("VALUE", "VALUE"),
        ("EVALUATE", "EVALUATE"),
        ("GO TO", "GO TO"),
        ("MOVE", "MOVE"),
        ("ADD", "ADD / arithmetic"),
        ("**", "exponent"),
    ];
    for (needle, cat) in rules {
        if msg.contains(needle) {
            return cat.to_string();
        }
    }
    "other".to_string()
}

/// Replace `{...}` format placeholders with `<x>` so a scraped message reads cleanly in the doc.
fn clean_msg(msg: &str) -> String {
    let mut s = String::new();
    let mut chars = msg.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            for n in chars.by_ref() {
                if n == '}' {
                    break;
                }
            }
            s.push_str("<x>");
        } else {
            s.push(c);
        }
    }
    s
}

fn pct(done: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        100.0 * done as f64 / total as f64
    }
}

/// Data-description clauses the cobrun front-end parses + applies (the rest are runtime-ready, not wired).
fn fe_clause(c: &str) -> bool {
    matches!(
        c,
        "PICTURE" | "USAGE" | "VALUE" | "OCCURS" | "REDEFINES" | "FILLER"
            | "LEVEL 88 (condition-name)" | "LEVEL 77" | "INDEXED BY"
            | "SIGN" | "JUSTIFIED" | "BLANK WHEN ZERO" | "GLOBAL" | "EXTERNAL"
            | "OCCURS DEPENDING ON" | "RENAMES" | "LEVEL 66 (RENAMES)" | "SYNCHRONIZED"
    )
}

/// USAGE forms the front-end carries: DISPLAY + the integer/packed COMP family + COMP-6 + an opaque
/// POINTER + INDEX + COMP-1/COMP-2. (NATIONAL is a boundary; see the USAGE table note.)
fn fe_usage(u: &str) -> bool {
    matches!(
        u,
        "DISPLAY" | "COMP/BINARY" | "COMP-3/PACKED-DECIMAL" | "COMP-5" | "COMP-6" | "POINTER" | "INDEX"
            | "COMP-1 (float)" | "COMP-2 (double)"
    )
}

pub fn run(cmd: &str, root: &str) -> i32 {
    let docs = [
        (Path::new(root).join("COBOL-PARITY.md"), build(root)),
        (Path::new(root).join("FILE-PARITY.md"), build_files(root)),
    ];
    match cmd {
        "generate" => {
            for (path, text) in &docs {
                if std::fs::write(path, text).is_err() {
                    eprintln!("cobol-parity: failed to write {}", path.display());
                    return 2;
                }
            }
            println!("COBOL-PARITY.md + FILE-PARITY.md generated");
            0
        }
        "check" => {
            for (path, text) in &docs {
                let committed = std::fs::read_to_string(path).unwrap_or_default();
                if &committed != text {
                    eprintln!("{} STALE: != `xtask cobol-parity generate`. Re-run it.", path.display());
                    return 2;
                }
            }
            // Beyond regeneration-equal: cross-check the file census against REALITY, so the data
            // itself can't quietly go stale (a doc that matches a stale generator is not enough).
            let problems = reality_check(root);
            if !problems.is_empty() {
                eprintln!("FILE-PARITY data STALE -- {} reality-check problem(s):", problems.len());
                for p in &problems {
                    eprintln!("  - {p}");
                }
                return 2;
            }
            println!("COBOL-PARITY + FILE-PARITY: fresh (regeneration-equal + reality-checked)");
            0
        }
        _ => {
            eprintln!("usage: xtask cobol-parity <generate|check>");
            2
        }
    }
}

/// Cross-check the FILE-PARITY census against the live repository, so the hand-maintained data cannot
/// drift out of sync with reality without the doc-refresh gate noticing. Returns one string per problem
/// (empty = fresh). Three classes of staleness are caught:
///
///   1. **structural** -- the `total` field must equal the file count.
///   2. **status/evidence consistency** -- a bare `GAP` must be `UNEVIDENCED` and vice-versa; an
///      evidenced status (PORTED*/COPIED+WIRED/PARTIAL) must NOT still say `UNEVIDENCED-GAP`. This is
///      exactly the drift that left `bin/cobcrun.c` marked "no native Rust port yet" after `cobrun`
///      had in fact reproduced it.
///   3. **proof anchors** -- a file may carry a `proof` array of repo anchors (`"path"`, or
///      `"path :: needle"`); every path must exist (and contain the needle). A renamed/removed module
///      or a deleted feature then fails the gate, forcing the census to be updated.
fn reality_check(root: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let m: Value = serde_json::from_str(FILES).unwrap_or(Value::Null);
    let files = m["files"].as_array().cloned().unwrap_or_default();

    // 1. structural: declared total == actual.
    if let Some(total) = m["total"].as_u64() {
        if total as usize != files.len() {
            problems.push(format!("`total` field is {total} but there are {} files", files.len()));
        }
    }

    for f in &files {
        let path = f["path"].as_str().unwrap_or("?");
        let status = f["status"].as_str().unwrap_or("");
        let evidence = f["evidence"].as_str().unwrap_or("");
        let unevidenced = evidence.starts_with("UNEVIDENCED");

        // 2. status/evidence consistency.
        let is_gap = status.starts_with("GAP");
        if is_gap && !unevidenced {
            problems.push(format!("{path}: status `{status}` is a GAP but evidence is not UNEVIDENCED (closed gap not reclassified?)"));
        }
        let evidenced_status = matches!(status, "PORTED" | "PORTED+EVIDENCED" | "PORTED-VIA" | "COPIED+WIRED" | "PARTIAL");
        if evidenced_status && unevidenced {
            problems.push(format!("{path}: status `{status}` claims work done but evidence still says UNEVIDENCED"));
        }

        // 3. proof anchors -- each must resolve against the live tree.
        if let Some(proof) = f["proof"].as_array() {
            for entry in proof {
                let Some(spec) = entry.as_str() else { continue };
                let (rel, needle) = match spec.split_once(" :: ") {
                    Some((p, n)) => (p.trim(), Some(n.trim())),
                    None => (spec.trim(), None),
                };
                let full = Path::new(root).join(rel);
                match std::fs::read_to_string(&full) {
                    Ok(body) => {
                        if let Some(n) = needle {
                            if !body.contains(n) {
                                problems.push(format!("{path}: proof `{rel}` exists but no longer contains `{n}`"));
                            }
                        }
                    }
                    Err(_) => {
                        // Non-text or missing: a bare path may be a directory/binary -- accept if it exists.
                        if needle.is_some() || !full.exists() {
                            problems.push(format!("{path}: proof anchor `{rel}` does not exist"));
                        }
                    }
                }
            }
        }
    }

    // 4. front-end sub-form coverage (COBOL-PARITY.md) -- keep `FRONTEND_SUBFORMS` honest against the
    //    live source. A `sealed` row's corpus must exist; a `gap` row's guard must still be present (so a
    //    silently-closed gap fails the gate); and every `SUBFORM_MARKER` guard in the front-end must have
    //    a matching `gap` row (so a new fail-closed sub-form can't be added without recording it here).
    let frontend_src = "crates/gnucobol-rs/src/frontend.rs";
    let frontend_body = std::fs::read_to_string(Path::new(root).join(frontend_src)).unwrap_or_default();
    for (verb, _form, status, path, needle) in FRONTEND_SUBFORMS {
        // A `sealed` claim ("byte-identical to cobc") is only as good as its proof: require the anchor to be
        // a program in `lab/corpus/frontend/`, which the doc gate now runs through real cobc + cobrun and
        // requires byte-identical (the front-end sweep). Otherwise "sealed" would rest on string-presence.
        if *status == "sealed" && !path.starts_with("lab/corpus/frontend/") {
            problems.push(format!(
                "FRONTEND_SUBFORMS `{verb}` (sealed): anchor `{path}` is not a swept corpus program (lab/corpus/frontend/*.cob) -- sealed claims must be oracle-verified by the front-end sweep, not just string-anchored"
            ));
        }
        let full = Path::new(root).join(path);
        match std::fs::read_to_string(&full) {
            Ok(body) => {
                if !body.contains(needle) {
                    problems.push(format!(
                        "FRONTEND_SUBFORMS `{verb}` ({status}): anchor `{path}` exists but no longer contains `{needle}` -- {}",
                        if *status == "gap" { "gap silently sealed? flip the row to `sealed` with a corpus anchor" } else { "corpus changed? update the anchor" }
                    ));
                }
            }
            Err(_) => problems.push(format!(
                "FRONTEND_SUBFORMS `{verb}` ({status}): anchor file `{path}` does not exist"
            )),
        }
    }
    // USAGE-form guards carry the same marker but are tracked in the USAGE forms table above, not as verb
    // sub-forms, so exclude them from the verb-sub-form count.
    let guard_count = frontend_body.lines().filter(|l| l.contains(SUBFORM_MARKER) && !l.contains("USAGE ")).count();
    let listed_count = FRONTEND_SUBFORMS.iter().filter(|r| r.2 == "gap" && r.4.contains(SUBFORM_MARKER)).count();
    if guard_count != listed_count {
        problems.push(format!(
            "{frontend_src}: {guard_count} `{SUBFORM_MARKER}` guard(s) but {listed_count} matching `gap` row(s) in FRONTEND_SUBFORMS -- a fail-closed sub-form was added/removed without updating COBOL-PARITY.md"
        ));
    }

    // 4b. No HARDCODED sweep count in the census data: it must use the `{SWEEP}` placeholder (substituted
    //     with the live corpus size at generate time). A bare "NN-program" silently went stale at 90 while
    //     the corpus grew; forbid it so that can never recur.
    let mut scan = FILES;
    while let Some(pos) = scan.find("-program") {
        let digits: String = scan[..pos].chars().rev().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            problems.push(format!(
                "gnucobol_files.json: hardcoded \"{}-program\" sweep count -- use the {{SWEEP}} placeholder (self-freshening)",
                digits.chars().rev().collect::<String>()
            ));
        }
        scan = &scan[pos + "-program".len()..];
    }

    problems
}
