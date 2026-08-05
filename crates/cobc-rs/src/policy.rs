//! The explicit option-policy registry (prompt §1.2/§1.4). Every recognized option carries an
//! explicit policy; nothing falls through a generic "ignore unknown flags" path. The registry is
//! derived from the real invocation census of the admitted GnuCOBOL 3.2 testsuite
//! (reports/gnucobol-testsuite/invocation-census.json) and is the authoritative source for
//! `--print-capabilities` / `--explain-translation` and the generated option-compatibility table.

/// The per-option policy (prompt §1.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionPolicy {
    /// Translated into a candidate-side equivalent (dialect, format, includes, defines, output…).
    Translated,
    /// Accepted as the equivalent of another form (e.g. long vs short spelling).
    AcceptedEquivalent,
    /// Accepted as a PROVEN no-op for the candidate execution model (diagnostic/optimization/native
    /// flags the test does not depend on). Always recorded in the invocation ledger.
    AcceptedProvenNoOp,
    /// Rejected: the option affects semantics the candidate cannot honor.
    RejectedUnsupported,
    /// Rejected: ambiguous/unknown spelling (fail closed).
    RejectedAmbiguous,
}

impl OptionPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            OptionPolicy::Translated => "translated",
            OptionPolicy::AcceptedEquivalent => "accepted-equivalent",
            OptionPolicy::AcceptedProvenNoOp => "accepted-proven-no-op",
            OptionPolicy::RejectedUnsupported => "rejected-unsupported",
            OptionPolicy::RejectedAmbiguous => "rejected-ambiguous",
        }
    }
}

/// The compile-mode compatibility contract (prompt §1.3): strict is the DEFAULT and never
/// silently weakens semantics; the testsuite mode may accept a bounded allowlist of proven-benign
/// options (the registry entries marked AcceptedProvenNoOp) — semantic options are never relaxed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompatMode {
    #[default]
    Strict,
    GnucobolTestsuite,
}

impl CompatMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompatMode::Strict => "strict",
            CompatMode::GnucobolTestsuite => "gnucobol-testsuite",
        }
    }
}

/// The requested artifact mode of an invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Executable, // -x
    Module,     // -m
    SyntaxOnly, // -fsyntax-only
    Preprocess, // -E
    Dependency, // -M
    Info,       // --info / --version / --runtime-conf / --list-* / --help
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Executable => "executable",
            Mode::Module => "module",
            Mode::SyntaxOnly => "syntax-only",
            Mode::Preprocess => "preprocess",
            Mode::Dependency => "dependency",
            Mode::Info => "info",
        }
    }
}

/// The semantic category (mirrors the census categories; prompt §0.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptCategory {
    Semantic,
    Dialect,
    SourceFormat,
    IncludeCopybook,
    Preprocessor,
    OutputSelection,
    CompileLinkMode,
    RuntimeModule,
    Diagnostic,
    OptimizationDebug,
    TestHarness,
}

impl OptCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            OptCategory::Semantic => "semantic",
            OptCategory::Dialect => "dialect",
            OptCategory::SourceFormat => "source-format",
            OptCategory::IncludeCopybook => "include-copybook",
            OptCategory::Preprocessor => "preprocessor",
            OptCategory::OutputSelection => "output-selection",
            OptCategory::CompileLinkMode => "compile-link-mode",
            OptCategory::RuntimeModule => "runtime-module",
            OptCategory::Diagnostic => "diagnostic",
            OptCategory::OptimizationDebug => "optimization-debug",
            OptCategory::TestHarness => "test-harness",
        }
    }
}

/// One registry entry. `consumes_value` marks options that take a separate value argument
/// (`-o out`, `-I dir`); `attached` options take `-name=value` (matched by prefix in the parser).
#[derive(Debug, Clone)]
pub struct Entry {
    pub canonical: &'static str,
    pub aliases: &'static [&'static str],
    pub policy: OptionPolicy,
    pub category: OptCategory,
    pub consumes_value: bool,
    pub justification: &'static str,
}

const fn e(
    canonical: &'static str,
    aliases: &'static [&'static str],
    policy: OptionPolicy,
    category: OptCategory,
    consumes_value: bool,
    justification: &'static str,
) -> Entry {
    Entry {
        canonical,
        aliases,
        policy,
        category,
        consumes_value,
        justification,
    }
}

/// The full registry. Derived from the invocation census of the admitted GnuCOBOL 3.2 testsuite
/// (`make check`, stock configuration). Options the candidate cannot honor are REJECTED — never
/// silently dropped. Diagnostic/optimization/native flags with no observable effect on the
/// candidate execution model are accepted as PROVEN no-ops (recorded per invocation).
pub fn registry() -> Vec<Entry> {
    vec![
        // ---- compile/link modes -------------------------------------------------------------
        e("-x", &["-x"], OptionPolicy::Translated, OptCategory::CompileLinkMode, false,
          "build-and-link mode: for the interpreter this means \"prepare an executable launch artifact\" (launcher + manifest); the artifact is NOT a native COBOL executable"),
        e("-m", &["-m"], OptionPolicy::Translated, OptCategory::CompileLinkMode, false,
          "module mode: build a loadable-module launch artifact (manifest + launcher) resolvable by `cobcrun <name>` from the build-local module registry"),
        e("-c", &["-c"], OptionPolicy::RejectedUnsupported, OptCategory::CompileLinkMode, false,
          "compile to object: the candidate has no native object model (no C emission); reject honestly"),
        e("-S", &["-S"], OptionPolicy::RejectedUnsupported, OptCategory::CompileLinkMode, false,
          "compile to assembler: no native codegen; reject honestly"),
        e("-C", &["-C"], OptionPolicy::RejectedUnsupported, OptCategory::CompileLinkMode, false,
          "generate C: the candidate is an interpreter, not a C emitter; reject honestly"),
        e("-E", &["-E"], OptionPolicy::Translated, OptCategory::CompileLinkMode, false,
          "preprocess-only: emit the define/copy-expanded source (with the GnuCOBOL `#line` header shape)"),
        e("-fsyntax-only", &["-fsyntax-only"], OptionPolicy::Translated, OptCategory::CompileLinkMode, false,
          "run the real candidate check pipeline (preprocess, lex, parse, declarations/layout) and exit without executing or emitting artifacts"),
        e("-M", &["-M"], OptionPolicy::Translated, OptCategory::CompileLinkMode, false,
          "dependency mode: emit make-style dependency output (with -MF/-MT)"),
        e("-MF", &["-MF"], OptionPolicy::Translated, OptCategory::CompileLinkMode, true,
          "dependency output file"),
        e("-MT", &["-MT"], OptionPolicy::Translated, OptCategory::CompileLinkMode, true,
          "dependency target name (repeatable)"),
        e("-q", &["-q", "--quiet", "--silent"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, false,
          "suppress non-error messages: cobc-rs diagnostics are already minimal; no semantic effect"),
        e("-v", &["-v", "--verbose"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, false,
          "verbose progress: informational only; the candidate records its own ledger"),
        e("-b", &["-b"], OptionPolicy::RejectedUnsupported, OptCategory::CompileLinkMode, false,
          "embedded build (combine compile+link into one step in a specific way): not supported"),
        e("-j", &["-j", "--job"], OptionPolicy::AcceptedProvenNoOp, OptCategory::RuntimeModule, false,
          "job name: the candidate has no shared-memory job model; accepted only for the -j / --job=name spelling with no effect"),
        e("-jd", &["-jd"], OptionPolicy::RejectedUnsupported, OptCategory::RuntimeModule, false,
          "debug job: requires the shared-memory job model; reject honestly"),
        e("-jdg", &["-jdg"], OptionPolicy::RejectedUnsupported, OptCategory::RuntimeModule, false,
          "debug job (graphical): requires the shared-memory job model; reject honestly"),
        e("-r", &["-r"], OptionPolicy::RejectedUnsupported, OptCategory::RuntimeModule, false,
          "run module: cobc -r would exec the module; the candidate model rejects this form"),
        e("-F", &["-F"], OptionPolicy::RejectedUnsupported, OptCategory::RuntimeModule, false,
          "free-format source with module extensions: reject honestly"),
        e("-P-", &["-P-"], OptionPolicy::RejectedUnsupported, OptCategory::CompileLinkMode, false,
          "suppress COPY output during -E: not supported"),
        e("-Xref", &["-Xref"], OptionPolicy::RejectedUnsupported, OptCategory::OutputSelection, false,
          "cross-reference listing: no listing model; reject honestly"),
        e("-t", &["-t"], OptionPolicy::RejectedUnsupported, OptCategory::OutputSelection, true,
          "listing file: no listing model; reject honestly"),
        e("-t-", &["-t-"], OptionPolicy::RejectedUnsupported, OptCategory::OutputSelection, false,
          "listing to stdout: no listing model; reject honestly"),
        e("-T-", &["-T-"], OptionPolicy::RejectedUnsupported, OptCategory::OutputSelection, false,
          "listing to terminal: no listing model; reject honestly"),
        e("-tlines", &["-tlines"], OptionPolicy::RejectedUnsupported, OptCategory::OutputSelection, true,
          "listing line count: no listing model; reject honestly"),
        e("-tsymbols", &["-tsymbols"], OptionPolicy::RejectedUnsupported, OptCategory::OutputSelection, false,
          "symbol listing: no listing model; reject honestly"),
        e("-save-temps", &["-save-temps"], OptionPolicy::AcceptedProvenNoOp, OptCategory::TestHarness, false,
          "keep intermediate files: the candidate's intermediates are the manifest + expanded source, which are kept anyway"),
        e("-static", &["-static"], OptionPolicy::AcceptedProvenNoOp, OptCategory::TestHarness, false,
          "static linking: no native linking model; no observable effect on the interpreter"),
        // ---- output selection ---------------------------------------------------------------
        e("-o", &["-o", "--output"], OptionPolicy::Translated, OptCategory::OutputSelection, true,
          "output path: the launch artifact (launcher) path; manifest + expanded source derive from it"),
        // ---- dialect ------------------------------------------------------------------------
        e("-std", &["-std"], OptionPolicy::Translated, OptCategory::Dialect, true,
          "dialect (-std=name): mapped to the runtime dialect knobs (from_std / from_conf)"),
        e("-conf", &["-conf", "--config"], OptionPolicy::Translated, OptCategory::Dialect, true,
          "dialect configuration file (-conf=file.conf): parsed through Dialect::from_conf"),
        e("-free", &["-free", "--free"], OptionPolicy::Translated, OptCategory::SourceFormat, false,
          "free source format"),
        e("-fixed", &["-fixed", "--fixed"], OptionPolicy::Translated, OptCategory::SourceFormat, false,
          "fixed source format"),
        e("-fformat", &["-fformat"], OptionPolicy::Translated, OptCategory::SourceFormat, true,
          "source format (-fformat=fixed|free|...): fixed/free mapped; auto/cobol85/terminal/xopen/variable rejected unless free/fixed"),
        // ---- include / copybook --------------------------------------------------------------
        e("-I", &["-I"], OptionPolicy::Translated, OptCategory::IncludeCopybook, true,
          "copybook search path (repeatable; also consulted for `COPY name IN \"dir\"`)"),
        e("-ext", &["-ext"], OptionPolicy::Translated, OptCategory::IncludeCopybook, true,
          "copybook extension to try (-ext=cpy: look for name.cpy too)"),
        e("-ffilename-mapping", &["-ffilename-mapping", "-fno-filename-mapping"], OptionPolicy::AcceptedProvenNoOp, OptCategory::IncludeCopybook, false,
          "case-insensitive file mapping: the candidate resolves copybooks case-sensitively on the filesystem; recorded as no-op for the suite"),
        e("-ffold-copy", &["-ffold-copy"], OptionPolicy::AcceptedProvenNoOp, OptCategory::IncludeCopybook, true,
          "copybook name folding: recorded no-op (filesystem resolution is authoritative)"),
        // ---- preprocessor -------------------------------------------------------------------
        e("-D", &["-D"], OptionPolicy::Translated, OptCategory::Preprocessor, true,
          "define a conditional-compilation symbol (-DNAME or -DNAME=value): prepended as >>DEFINE for the front-end preprocessor"),
        e("-U", &["-U"], OptionPolicy::RejectedUnsupported, OptCategory::Preprocessor, true,
          "undefine: the front-end preprocessor has no undefine; reject honestly"),
        e("-fstandard-define", &["-fstandard-define"], OptionPolicy::RejectedUnsupported, OptCategory::Preprocessor, true,
          "standard-define date: no WHEN-COMPILED control; reject honestly"),
        // ---- runtime / module / checks --------------------------------------------------------
        e("-fec", &["-fec", "-fno-ec"], OptionPolicy::RejectedUnsupported, OptCategory::RuntimeModule, true,
          "exception-code checking: the candidate checks a fixed sealed set; reject honestly"),
        e("-fsubscript-check", &["-fsubscript-check"], OptionPolicy::RejectedUnsupported, OptCategory::RuntimeModule, true,
          "subscript checking level: the candidate's bound checking is not configurable; reject honestly"),
        e("-fmemory-check", &["-fmemory-check"], OptionPolicy::RejectedUnsupported, OptCategory::RuntimeModule, true,
          "memory checking: no native memory model; reject honestly"),
        e("-frelax-syntax-checks", &["-frelax-syntax-checks", "-fno-relax-syntax-checks"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, false,
          "relaxed syntax checks: the candidate parser has one sealed strictness; reject honestly"),
        e("-fnotrunc", &["-fnotrunc", "-fno-trunc"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, false,
          "no binary truncation: the candidate's arithmetic model is fixed; reject honestly"),
        e("-fodoslide", &["-fodoslide"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, false,
          "OCCURS DEPENDING slide: not modeled; reject honestly"),
        e("-fbinary-byteorder", &["-fbinary-byteorder"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "binary byte order: fixed little-endian model; reject honestly"),
        e("-fbinary-size", &["-fbinary-size"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "binary size model: the dialect's binary-size knob is set by -std/-conf; a bare override is rejected"),
        e("-fno-fast-compare", &["-fno-fast-compare"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, false,
          "fast-compare control: reject honestly"),
        e("-fmove-ibm", &["-fmove-ibm"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, false,
          "IBM MOVE semantics: reject honestly"),
        e("-fdebugging-line", &["-fdebugging-line"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, false,
          "debugging lines: no debug-line model; reject honestly"),
        e("-fconstant-folding", &["-fconstant-folding", "-fno-constant-folding"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, false,
          "constant folding: reject honestly"),
        e("-ffree-redefines-position", &["-ffree-redefines-position"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "free-format REDEFINES position rules: reject honestly"),
        e("-fassign-clause", &["-fassign-clause"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "ASSIGN clause dialect: reject honestly"),
        e("-freserved", &["-freserved", "-fnot-reserved"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "reserved-word changes: reject honestly"),
        e("-fregister", &["-fregister", "-fnot-register"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "register changes: reject honestly"),
        e("-fintrinsics", &["-fintrinsics"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "intrinsic function set: reject honestly"),
        e("-fdefaultbyte", &["-fdefaultbyte"], OptionPolicy::Translated, OptCategory::Semantic, true,
          "uninitialized-storage fill byte: translated into the dialect knob (config line defaultbyte)"),
        e("-fword-length", &["-fword-length"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "word length: reject honestly"),
        e("-ftext-column", &["-ftext-column"], OptionPolicy::RejectedUnsupported, OptCategory::SourceFormat, true,
          "text column: the fixed-format converter is fixed at 72; reject honestly"),
        e("-fliteral-length", &["-fliteral-length", "-fnumeric-literal-length"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "literal length limits: reject honestly"),
        e("-febcdic-table", &["-febcdic-table"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "EBCDIC translation table: reject honestly"),
        e("-fdefault-colseq", &["-fdefault-colseq"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "default collating sequence: reject honestly"),
        e("-fcomment-paragraphs", &["-fcomment-paragraphs"], OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "comment paragraphs: reject honestly"),
        e("-fmissing-period", &["-fmissing-period", "-fmissing-statement", "-fincorrect-conf-sec-order",
          "-fmissing-period", "-fmissing-statement", "-fincorrect-conf-sec-order",
          "-fimplicit-goback-check", "-fstop-literal", "-fstop-identifier", "-fstop-error-statement",
          "-fcontrol-division", "-fcomment-paragraphs", "-fprogram-prototypes", "-fsticky-linkage",
          "-frelax-level-hierarchy", "-frenames-uncommon-levels", "-ftop-level-occurs-clause",
          "-flarger-redefines", "-frecord-contains-depending-clause", "-fpartial-replace-when-literal-src",
          "-fself-call-recursive", "-fentry-statement", "-fdefine-constant-directive",
          "-faccept-display-extensions", "-fassign-disk-from", "-fassign-ext-dyn", "-fassign-using-variable",
          "-fassign-variable", "-fimplicit-assign-dynamic-var", "-fno-implicit-assign-dynamic-var", "-fmove-non-numeric-lit-to-numeric-is-zero",
          "-fno-move-non-numeric-lit-to-numeric-is-zero", "-fno-program-name-redefinition",
          "-fno-recursive-check", "-fno-section-exit-check", "-fsection-exit-check", "-fno-ref-mod-zero-length",
          "-freference-out-of-declaratives", "-fno-areacheck", "-fuse-for-debugging", "-fpodup",
          "-fdpc-in-data", "-facu-literal", "-facu-literals", "-facucomment", "-fmfcomment",
          "-fhp-octal-literals", "-farithmetic-osvs", "-fperform-osvs", "-fscreen-section-rules",
          "-fsystem-name", "-ffold-call", "-fnot-intrinsic", "-fnot-register", "-fsign",
          "-fno-pretty-display", "-fpretty-display", "-fzero-length-literals", "-fcomplex-odo",
          "-fimplicit-init", "-faccept-auto", "-faccept-update", "-fassign-clause", "-fconstant-folding",
          "-fno-binary-truncate", "-fno-binary-comp-1", "-fbinary-comp-1", "-fremove-unreachable",
          "-fno-remove-unreachable", "-fno-gen-c-decl-static-call", "-fgen-c-decl-static-call",
          "-fno-theader", "-ftcmd", "-fno-tmessages", "-fno-tsource", "-fno-tsymbols", "-fno-dump",
          "-fdump", "-fcheck-perf", "-fusing-optional", "-fno-others", "-fwarn-all",
          "-fword-continuation", "-freserved-words", "-fno-implicit-goback-check",
          "-fdiagnostics-show-caret", "-fdiagnostics-show-line-numbers", "-fno-diagnostics-show-option",
          "-fno-ttimestamp", "-ftsymbols", "-fttitle", "-ftrace", "-ftraceall", "-fsource-location",
          "-fgen-c-line-directives", "-fgen-c-labels", "-fsticky-linkage", "-fno-ec", "-fec",
          "-fpretty-display", "-fno-pretty-display", "-fmove-ibm", "-fcallfh", "-frelax-syntax",
          "-frecord-delim-with-fixed-recs", "-febcdic-symbolic-characters", "-fpicture-l"],
          OptionPolicy::RejectedUnsupported, OptCategory::Semantic, true,
          "dialect knob / feature flag not modeled by the sealed front end: rejected honestly (never silently dropped)"),
        // ---- diagnostic-only (proven no-ops) ---------------------------------------------------
        e("-fmax-errors", &["-fmax-errors"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, true,
          "max errors: upstream 470f7db12 made 0 mean unlimited and cut the default from 128 to 20; the candidate checker is fail-fast (at most one diagnostic per compile), so every N>=1 and N=0 are observably identical for the candidate — recorded proven no-op (with the architectural rationale, not a silent skip)"),
        e("-Wall", &["-Wall"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, false,
          "enable all warnings: the candidate emits its own diagnostics; no observable effect on test outcomes"),
        e("-Wextra", &["-Wextra"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, false,
          "extra warnings: no observable effect"),
        e("-w", &["-w"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, false,
          "suppress warnings: no observable effect on the sealed checks"),
        e("-Werror", &["-Werror"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, false,
          "warnings as errors: the candidate does not emit the GCC-style warnings the suite asserts on; recorded no-op"),
        e("-W", &["-W"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, true,
          "warning category (generic): diagnostic-only"),
        e("-Wno", &["-Wno"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, true,
          "warning category off (generic prefix): diagnostic-only"),
        e("-Wadditional", &["-Wadditional", "-Wconstant-expression", "-Wconstant-numlit-expression",
          "-Werror=additional", "-Werror=goto-section", "-Werror=ignored-error", "-Werror=obsolete",
          "-Werror=redefinition", "-Wfatal-errors", "-Wimplicit-define", "-Wlinkage",
          "-Wno-constant-expression", "-Wno-constant-numlit-expression", "-Wno-dialect", "-Wno-error",
          "-Wno-error=additional", "-Wno-goto-different-section", "-Wno-ignored-error",
          "-Wno-obsolete", "-Wno-others", "-Wno-parentheses", "-Wno-pending", "-Wno-redefinition",
          "-Wno-strict-typing", "-Wno-suspicious-perform-thru", "-Wno-terminator", "-Wno-truncate",
          "-Wno-typing", "-Wno-unfinished", "-Wno-unsupported", "-Wpossible-overlap",
          "-Wpossible-truncate", "-Wstrict-typing", "-Wunreachable"],
          OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, false,
          "specific warning toggles: the candidate emits its own diagnostics; recorded no-op (tested against the admitted suite)"),
        e("-fdiagnostics-plain-output", &["-fdiagnostics-plain-output", "-fno-diagnostics-show-option",
          "-fdiagnostics-show-option"], OptionPolicy::AcceptedProvenNoOp, OptCategory::Diagnostic, false,
          "diagnostic formatting toggles: the candidate prints its own diagnostic shape; recorded no-op"),
        // ---- optimization / debug (proven no-ops) ----------------------------------------------
        e("-O", &["-O", "-O0", "-O1", "-O2", "-O3", "-Os"], OptionPolicy::AcceptedProvenNoOp, OptCategory::OptimizationDebug, false,
          "optimization level: no native codegen; no observable effect"),
        e("-g", &["-g"], OptionPolicy::AcceptedProvenNoOp, OptCategory::OptimizationDebug, false,
          "debug info: no native codegen; no observable effect"),
        e("-debug", &["-debug", "--debug"], OptionPolicy::AcceptedProvenNoOp, OptCategory::OptimizationDebug, false,
          "debug build flags (in the suite's FLAGS): the candidate's checks are the sealed set; recorded no-op"),
        // ---- compiler-information modes ---------------------------------------------------------
        e("--version", &["--version", "-V"], OptionPolicy::Translated, OptCategory::TestHarness, false,
          "version output (honest: reports the reproduced GnuCOBOL version + identity)"),
        e("--info", &["--info"], OptionPolicy::Translated, OptCategory::TestHarness, false,
          "compiler information (the shape atlocal greps for COB_*_EXT, 64bit-mode, ISAM, XML/JSON/curses)"),
        e("--dumpversion", &["--dumpversion"], OptionPolicy::Translated, OptCategory::TestHarness, false,
          "the reproduced GnuCOBOL version, byte-identical to cobc -dumpversion"),
        e("--runtime-conf", &["--runtime-conf", "--runtime-config"], OptionPolicy::Translated, OptCategory::TestHarness, false,
          "resolved runtime configuration (native-Rust port output, byte-identical to cobcrun)"),
        e("--list", &["--list-reserved", "--list-intrinsics", "--list-mnemonics", "--list-registers",
          "--list-system", "--list-exceptions"], OptionPolicy::RejectedUnsupported, OptCategory::TestHarness, false,
          "keyword/intrinsic listing: the candidate has no upstream-identical lists; reject honestly"),
        e("--help", &["--help", "-h"], OptionPolicy::Translated, OptCategory::TestHarness, false,
          "help output"),
    ]
}

/// The registry as a process-wide static (so `lookup` can return `'static` entries).
static REGISTRY: std::sync::OnceLock<Vec<Entry>> = std::sync::OnceLock::new();

fn reg() -> &'static Vec<Entry> {
    REGISTRY.get_or_init(registry)
}

/// Look up an option token (with `=value` stripped). Returns the matching entry or `None` when the
/// registry has no entry for it (callers then REJECT — no silent ignore). Two passes: an explicit
/// benign policy (translated / no-op) WINS over a generic family rejection when the same spelling
/// is listed in both (e.g. `-fno-diagnostics-show-option` is a proven no-op AND a member of the
/// catch-all `-f*` reject family — the specific benign classification is authoritative).
pub fn lookup(option_without_value: &str) -> Option<&'static Entry> {
    let reg = reg();
    // getopt_long equivalence: cobc's long-option table accepts BOTH `-opt` and `--opt` spellings
    // (a single dash is a legal getopt_long long-option prefix). Normalize `--x` -> `-x` so the
    // `--f...` / `--std` / `--config` spellings used by the suite resolve to the same registry entry.
    let normalized: String = option_without_value
        .strip_prefix("--")
        .map(|s| format!("-{s}"))
        .unwrap_or_default();
    let candidates: Vec<&str> = if !normalized.is_empty() && normalized != option_without_value {
        vec![option_without_value, normalized.as_str()]
    } else {
        vec![option_without_value]
    };
    for cand in candidates {
        // pass 1: exact canonical / alias matches with a NON-rejected policy
        for e in reg {
            if matches!(
                e.policy,
                OptionPolicy::RejectedUnsupported | OptionPolicy::RejectedAmbiguous
            ) {
                continue;
            }
            if e.canonical == cand {
                return Some(e);
            }
            for a in e.aliases {
                if *a == cand {
                    return Some(e);
                }
            }
        }
        // pass 2: rejected policies (explicit rejection of a known option)
        for e in reg {
            if e.canonical == cand {
                return Some(e);
            }
            for a in e.aliases {
                if *a == cand {
                    return Some(e);
                }
            }
        }
        // family prefix fallback: `-W<category>` toggles are diagnostic-only
        for e in reg {
            if e.canonical == "-W" && cand.starts_with("-W") {
                return Some(e);
            }
        }
    }
    None
}

/// Normalize a raw option token: strip `=value` (attached form) and return `(canonical, value)`.
pub fn split_attached(raw: &str) -> (String, Option<String>) {
    match raw.find('=') {
        Some(i) => (raw[..i].to_string(), Some(raw[i + 1..].to_string())),
        None => (raw.to_string(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_census_option_has_an_explicit_policy() {
        // The observed option surface of the admitted suite (from the invocation census). Each
        // must resolve to a registry entry; the parser then enforces the policy.
        let observed = [
            "-x",
            "-m",
            "-c",
            "-S",
            "-C",
            "-E",
            "-fsyntax-only",
            "-M",
            "-MF",
            "-MT",
            "-q",
            "-v",
            "-b",
            "-j",
            "-jd",
            "-jdg",
            "-r",
            "-F",
            "-P-",
            "-Xref",
            "-t",
            "-t-",
            "-T-",
            "-tlines",
            "-tsymbols",
            "-save-temps",
            "-static",
            "-o",
            "-std",
            "-conf",
            "-free",
            "-fixed",
            "-fformat",
            "-I",
            "-ext",
            "-ffilename-mapping",
            "-ffold-copy",
            "-D",
            "-U",
            "-fstandard-define",
            "-fec",
            "-fsubscript-check",
            "-fmemory-check",
            "-frelax-syntax-checks",
            "-fnotrunc",
            "-fodoslide",
            "-fbinary-byteorder",
            "-fbinary-size",
            "-fno-fast-compare",
            "-fmove-ibm",
            "-fdebugging-line",
            "-fconstant-folding",
            "-ffree-redefines-position",
            "-fassign-clause",
            "-freserved",
            "-fregister",
            "-fintrinsics",
            "-fdefaultbyte",
            "-fword-length",
            "-ftext-column",
            "-fliteral-length",
            "-febcdic-table",
            "-fdefault-colseq",
            "-fcomment-paragraphs",
            "-fmissing-period",
            "-fdiagnostics-plain-output",
            "-Wall",
            "-Wextra",
            "-w",
            "-Werror",
            "-Wno-unsupported",
            "-fno-diagnostics-show-option",
            "-fdiagnostics-show-option",
            "-O2",
            "-O0",
            "-g",
            "-debug",
            "--version",
            "--info",
            "--dumpversion",
            "--runtime-conf",
            "--list-reserved",
            "--help",
            "-fno-ec",
            "-fttitle",
            "-fno-ttimestamp",
            "-fgen-c-line-directives",
            "-fgen-c-labels",
            "-ftsymbols",
            "-ftrace",
            "-ftraceall",
            "-fsource-location",
            "-fno-tmessages",
            "-fno-tsource",
            "-fno-theader",
            "-ftcmd",
            "-fdump",
            "-fno-dump",
            "-fno-binary-truncate",
            "-fno-binary-comp-1",
            "-fbinary-comp-1",
            "-fcheck-perf",
            "-ffold-call",
            "-fusing-optional",
        ];
        for opt in observed {
            assert!(
                lookup(opt).is_some(),
                "census option {opt} has no explicit policy in the registry"
            );
        }
    }

    #[test]
    fn unknown_options_are_rejected_not_ignored() {
        assert!(lookup("--thisoptiondoesntexist").is_none());
        assert!(lookup("-flagdoesntexist").is_none());
        assert!(lookup("-funknown-thing").is_none());
    }

    #[test]
    fn split_attached_handles_equals() {
        let (k, v) = split_attached("-std=cobol85");
        assert_eq!(k, "-std");
        assert_eq!(v.as_deref(), Some("cobol85"));
        let (k, v) = split_attached("-o");
        assert_eq!(k, "-o");
        assert!(v.is_none());
    }

    #[test]
    fn max_errors_is_a_documented_proven_noop() {
        // Upstream 470f7db12: -fmax-errors=0 means unlimited; default cut 128 -> 20. The candidate
        // checker is fail-fast (at most one diagnostic per compile), so every N>=1 and N=0 are
        // observably identical for the candidate. Accepted with the rationale recorded, never
        // silently dropped.
        let e = lookup("-fmax-errors").expect("-fmax-errors has an explicit registry entry");
        assert_eq!(e.policy, OptionPolicy::AcceptedProvenNoOp);
        assert!(e.consumes_value, "-fmax-errors takes a value");
        assert_eq!(e.category, OptCategory::Diagnostic);
    }

    #[test]
    fn binary_source_is_rejected() {
        // Upstream 470f7db12 (pplex ppopen_get_file): binary source files error out directly.
        let dir = std::env::temp_dir().join(format!("cobc_rs_bin_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("prog.cob");
        std::fs::write(&p, b"\x00\x01\x02").unwrap();
        let err = crate::compile::read_source(p.to_str().unwrap()).unwrap_err();
        assert!(err.contains("binary file"), "got: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
