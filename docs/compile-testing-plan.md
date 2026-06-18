# gnucobol-rs Compile-Testing Plan (GNURUST.FRONTEND.1)

**Scope honesty up front.** This is a plan for a *front-end interpreter* over the ported `libcob` runtime — a clean-room COBOL reader + executor with **no `cobc` and no `libcob` linked**. It is **not** a native-code compiler: there is no `.o` / `.s` / `.so` / executable emission, no codegen, no optimizer. "Compile-testing" here means: drive a COBOL source through all front-end phases (host driver → copybook → conditional-compilation preprocessor → case-fold/lexer → SPECIAL-NAMES scan → program-split/structural parse → field/data-layout build → execution) and judge correctness by **stdout bytes being byte-identical to the admitted `cobc` 3.2 oracle**. Conformance to NIST COBOL-85 or the full ISO grammar is **explicitly not claimed** (`NEG.COBOL.NIST_CONFORMANCE`). The design contract is **fail-closed**: anything outside the sealed subset returns a typed `RunError::{Unsupported, UndefinedName, Runtime, SizeError}` (`frontend.rs:69-93`) rather than guessing.

---

## Part A — What the front-end sweep already proves

- **Court:** `GNURUST.FRONTEND.1` — "clean-room COBOL front-end — parse + EXECUTE a program subset to cobc-identical stdout."
- **Harness:** `/home/one/gnucobol-rs/lab/oracle/cobol_frontend_sweep.sh` (88 lines).
- **Corpus:** `/home/one/gnucobol-rs/lab/corpus/frontend/p01_add.cob … p93_goto_depending.cob` (**93 programs**), numbered chronologically by seal order (not phase-organized).
- **Interpreter:** `/home/one/gnucobol-rs/crates/gnucobol-rs/src/frontend.rs` (6595 lines) via entry `/home/one/gnucobol-rs/crates/gnucobol-rs/examples/cobrun.rs`.
- **Receipt of record:** `/home/one/gnucobol-rs/reports/receipts/GNURUST.FRONTEND.1/receipt.md` — replay `PASS=93 FAIL=0 (3.1.2 differential-matched=84)`, crate 0.8.7.

### What each sweep iteration asserts

For every `*.cob` in the corpus, under a pinned environment (`LC_ALL=C.UTF-8 TZ=UTC0`, the admitted 3.2 oracle on `PATH`):

1. **Oracle compile+run.** `cobc -x [-fixed|-free] [-std=…] -o p <file>` then run `p` → `oracle.out`. A `cobc` compile failure is a hard sweep failure.
2. **Clean-room parse+execute.** `target/release/examples/cobrun [-fixed] [-std=…] <file>` → `rust.out`, `rust.err`. **No `cobc` / `libcob` in this path** — `cobrun` runs the whole pipeline and the ported runtime.
3. **Byte-identity.** `cmp -s oracle.out rust.out`. Any non-empty `rust.err` (i.e. a `RunError`) is an **unconditional hard fail** — the interpreter is forbidden from "succeeding" by emitting a diagnostic. `cobrun`'s exit status carries the program's `RETURN-CODE`, so a non-zero exit is *not* itself a failure; only stderr bytes are.
4. **3.1.2 differential.** The same source through a second pinned oracle (`lab/oracle/prefix-312`) must *also* match `rust.out` — a version-stability cross-check. Programs may opt out via `*> @std: NAME` (dialect config legitimately evolves across versions) or `*> @no312: <reason>` (a behaviour that legitimately changed between 3.1.2 and 3.2).

**Gate:** `echo "PASS=$PASS FAIL=$FAIL (3.1.2 differential-matched=$DIFF)"; [ "$FAIL" -eq 0 ] || exit 1`. Wired into the guard at README line 142.

### Per-program header vocabulary (harness-driven)

`*> @std: NAME` (selects `-std=` for both oracle and cobrun); `*> @env: NAME=VALUE` (exported for compile *and* run — e.g. `SOURCE_DATE_EPOCH`, `COB_SWITCH_n`, `COB_CURRENT_DATE`); `*> @format: fixed` (fixed-format both sides); `*> @clock` (live-clock program — up to 8 retries to absorb a second-boundary straddle); `*> @no312: <reason>` (exempt the 3.1.2 cross-check).

### Coverage the 93-program corpus already exercises (by phase reached)

- **Phase 0 host driver** — fixed↔free (`p34`), `-std` selection (`p26`), DISPLAY UPON PRINTER redirect (`p35`), runtime-config locale/username resolution.
- **Phase 1 copybook** — `copybook.rs::expand` is a standalone pass with its own `#[cfg(kani)]` totality proof; exercised separately, **not yet wired into the sweep** (see Part C item 6).
- **Phase 2 preprocessor** — `>>DEFINE`/`>>IF`/`>>ELSE`/`>>END-IF` (`p33`), `>>TURN EC-BOUND-SUBSCRIPT CHECKING` (`p42`).
- **Phase 3 lexer / case-fold** — every program; currency `$` (`p24`), DECIMAL-POINT IS COMMA (`p25`, `p36`).
- **Phase 4 SPECIAL-NAMES** — UPSI switches (`p31`, `p32`), EBCDIC collation (`p39`).
- **Phase 5 parse** — OCCURS (`p40`, `p85` ODO), REDEFINES (`p41`), RENAMES (`p86`), level-66/88/77, groups (`p56`, `p74`), SYNC (`p87`), EXTERNAL (`p88`), file-control/FD/SD, report section (`p63`).
- **Phase 6 data-layout** — USAGE matrix (`p73`), float COMP-1/2 (`p84`), SIGN (`p76`), JUSTIFIED/BLANK WHEN ZERO (`p81`), INDEX/GLOBAL (`p82`).
- **Phase 7 exec** — full arithmetic (`p01`–`p16`), COMPUTE incl. paren/pow/pct (`p11`–`p15`), IF/EVALUATE/PERFORM/GO TO/GO TO DEPENDING (`p17`–`p23`, `p43`, `p50`, `p58`, `p92`, `p93`), SEARCH (`p51`, serial), MOVE/INITIALIZE/INSPECT/STRING/UNSTRING/EXAMINE (`p46`–`p48`, `p64`), file I/O seq/relative/rewrite (`p52`–`p54`), SORT/MERGE incl. procedures (`p55`, `p57`, `p59`), CALL/CANCEL (`p27`, `p28`, `p37`, `p38`), JSON (`p60`), ~94 intrinsics (`p67`–`p72`, `p75`, `p89`, `p90`), SIZE ERROR (`p29`), ROUNDED (`p91`), exception status/file/location (`p79`, `p80`, `p83`).

### Strengths

Real **differential against two locally-built oracles** (3.2 + 3.1.2), **byte-exact**, **fail-closed** (a diagnostic is a fail, never a pass), pinned-deterministic environment.

### Honest weaknesses (these motivate Part B / C)

1. **Success-paths only.** Every corpus program is *expected to match*. There is **no negative corpus** proving the fail-closed boundary actually fires — the 183 `RunError::Unsupported` emit sites and the casefile's prose non-claims are asserted, not executed.
2. **Verb-flat, not phase-targeted.** Filenames are chronological by seal order; a lexer or preprocessor regression surfaces only incidentally, with no per-phase attribution on a diff.
3. **3.1.2 differential is opportunistic** — silently skipped if `prefix-312` is absent; present-but-skipped is not separately counted in CI.
4. **Receipt/casefile drift.** The casefile (`/home/one/gnucobol-rs/reports/casefiles/GNURUST.FRONTEND.1/casefile.md`) is **STALE** — reads `23/23 … 0.7.85` and lists negative claims (EVALUATE/GO TO/file I/O/CALL) the corpus has since sealed (p43 evaluate, p50 goto, p52–54 fileio, p27 call). The replay number (93/0) is current; the prose around it is not. **Regenerating it is the single most embarrassing current gap.**

---

## Part B — Concrete buildable expansion

Two new on-disk axes plus attribution/gating work. Filenames stay chronological (`p94+`); a new **harness-inert** `*> @phase:` tag carries the phase for reporting only.

### B1. Phase-targeted positive corpus (extend `lab/corpus/frontend/`, `p94+`)

Add `*> @phase: lexer|preproc|parser|typecheck|exec` to every program (existing and new) — purely a reporting label; harness behaviour is unchanged. New programs, each still byte-matched vs cobc 3.2 (and 3.1.2 where not exempt):

- **`@phase: lexer`** — apostrophe-inside-`*>`-comment (regression-lock for the documented `frontend.rs:174` hazard where inline-comment stripping runs *before* quote tokenization); `'`-vs-`"` doubled-quote escape; long-line / continuation forms; mixed tabs vs spaces; numeric-literal edge forms; the PIC-glued-dot case (`ZZ9.99` must not split the `.` into a sentence `Dot`).
- **`@phase: preproc`** — nested `>>IF/>>ELSE/>>END-IF` (deepen beyond `p33`); `>>DEFINE x AS v` + `name = value` equality; `NOT name DEFINED`; an unrecognized `>>` (e.g. `>>SOURCE FORMAT`) that must pass through to a later phase unharmed; directive lines must never appear in output.
- **`@phase: parser`** — deeply-nested groups; OCCURS inside REDEFINES; level-66 RENAMES spanning subgroups; period-terminated vs explicit `END-…` scope forms that must produce *identical* output.
- **`@phase: typecheck`** (PIC/data-layout) — MOVE truncation matrix (alnum→numeric→edited round-trips); SIGN LEADING/TRAILING SEPARATE; JUSTIFIED; BLANK WHEN ZERO; P-scaling display.
- **`@phase: exec`** — PERFORM VARYING (the *supported* single-level form, since nested AFTER is fail-closed — that goes in B2); EVALUATE multi-WHEN / WHEN ALSO; GO TO DEPENDING (extend `p93`); recursive PERFORM THRU.

### B2. Negative / fail-closed corpus — the missing half of the trust surface

New directory `/home/one/gnucobol-rs/lab/corpus/frontend-reject/` (does not exist yet) + a sibling gate `/home/one/gnucobol-rs/lab/oracle/cobol_frontend_reject_sweep.sh`.

**Inverted assertion.** Each program uses an out-of-subset or malformed construct. The sweep asserts:

- **`cobrun` MUST emit a `RunError` on stderr** (non-empty `rust.err`), and
- **MUST NOT accidentally produce a matching stdout** (no silent run-through).

Classify each reject by a `*> @reject:` tag into the brief's two true kinds:

- **`@reject: true-negative`** — `cobc` *also* rejects (e.g. USAGE `NATIONAL`; the COMMUNICATION verbs SEND/RECEIVE/PURGE/ENABLE/DISABLE; ACUCOBOL `MODIFY`/`INQUIRE`; `ENTRY` in a nested program; the 16 boundary intrinsics). Optionally assert the `cobc` compile also fails.
- **`@reject: declared-boundary`** — `cobc` *accepts* and runs, but the port deliberately declines (e.g. nested `PERFORM VARYING … AFTER`; `SEARCH ALL` binary; `BINARY-CHAR/SHORT/LONG/DOUBLE` *until the BINARY-\* court lands*; external-`.so` uncontained `CALL`). These are the honest "we could but haven't sealed it" lines.

**Source these programs directly from the brief's enumerated emit sites** so the corpus is a 1:1 executable image of the fail-closed contract — e.g. PERFORM VARYING AFTER (`frontend.rs:2684`), SEARCH ALL (2554), USAGE NATIONAL (640/641), unrecognized USAGE incl. today's `BINARY-*` (1421 — note: the *generic* unrecognized-USAGE path, **not** the NATIONAL message), COMPUTE with no `=` (3622), INSPECT CONVERTING-without-TO (4156), STRING ON OVERFLOW missing INTO (4327), OPEN mode outside INPUT/OUTPUT/EXTEND/I-O (4540), SORT multi-KEY (4716), START INVALID KEY (5047), FUNCTION arity errors (5274 …). This converts the casefile's ~15 prose non-claims into **executed evidence**.

**Gate:** every reject program must fail-close; the gate's `FAIL` counts any program that *ran* (empty stderr) or whose stdout accidentally matched the oracle. `FAIL=0` ⇒ none leaked.

### B3. Resolve two documented undefined spots into sealed courts or declared boundaries

- **Fixed-format tabs.** `fixed_to_free` (`frontend.rs:266`) carries the note "tabs not expanded — sealed corpus uses spaces." Add a fixed-format program containing tabs: either seal cobc-identical tab handling, or admit it as a declared boundary with a receipt.
- **`@clock` flake vector.** The `@clock` 8-retry second-straddle is a CI flake source. Quarantine `@clock` programs into a separately-reported subset so a wall-clock straddle never reds the main gate.

---

## Part C — Gating, attribution, and tie-back

5. **Per-`@phase` rollup into the receipt JSON.** Today the sweep only prints `PASS/FAIL` to stdout. Aggregate counts per `@phase` (lexer/preproc/parser/typecheck/exec) and per reject-kind (true-negative / declared-boundary) into `receipt.json`, so a regression is attributable to a phase, not just "the front end broke."
6. **Wire the copybook pass into the evidenced surface.** `copybook.rs::expand` (`copybook.rs:308`) is a real Phase-1 pass with a `#[cfg(kani)]` totality proof but is *not* in the sweep. Add COPY/REPLACING programs (resolver-backed) so copybook expansion is byte-attested alongside the rest, and its `CopyError::ReplacingDeferred` fail-closed cases land in the B2 reject corpus.
7. **Make the 3.1.2 differential non-silent.** Count and report present-but-skipped 3.1.2 checks explicitly (skipped-because-`@std` vs skipped-because-`@no312` vs skipped-because-absent-prefix). In CI, an *absent* `prefix-312` should be a loud "differential not run," not a silent zero.
8. **One guard, two gates.** Run `cobol_frontend_reject_sweep.sh` from the same guard entry that runs `cobol_frontend_sweep.sh` (README line 142). Both feed **one regenerated receipt**; split the receipt's replay into two commands — accept sweep + reject sweep.
9. **Tie-back / regenerate the court artifacts.** Regenerate **both** the receipt and the casefile (`cargo run -p xtask -- receipt generate`) so that:
   - the casefile stops reading `23/23 … 0.7.85` and its Non-claims prose stops listing EVALUATE/GO TO/file I/O/CALL/OCCURS/REDEFINES as un-claimed (they are sealed in the 93-program corpus);
   - the positive-claim count, corpus size (93→N), and the **negative-capability list all derive from the EXECUTED reject corpus**, not stale prose;
   - the release snapshot `reports/releases/.../negative-capabilities-snapshot.json` becomes evidence-backed (executed), not asserted.

---

## Honest scope statement (carry verbatim into the doc)

- This plan tests an **interpreter over the ported `libcob` runtime**, judged by **stdout byte-identity to `cobc` 3.2**. It does **not** test native code generation, `.o`/`.so`/executable emission, linking, or optimization — none exist.
- It is **not** a NIST COBOL-85 conformance suite and makes no language-conformance claim (`NEG.COBOL.NIST_CONFORMANCE`).
- The **executable subset is narrower than the byte-court list**; `cobrun`'s job at a boundary is to **fail closed with a typed `RunError`**, which the new reject corpus (B2) is the instrument to *prove*, not assert.
- `cobrun` is **not a `cobc` drop-in** and does not auto-detect build profiles from bytes (`GNURUST.BUILD.PROFILE.1`).
- Subprogram linkage, `RETURNING`, dynamic `.so` CALL, the CANCEL state machine, and recursion are **observed-only**, not sealed by this court.

---

## Key paths

- Sweep: `/home/one/gnucobol-rs/lab/oracle/cobol_frontend_sweep.sh`
- Corpus: `/home/one/gnucobol-rs/lab/corpus/frontend/p01_add.cob … p93_goto_depending.cob`
- New reject dir (to create): `/home/one/gnucobol-rs/lab/corpus/frontend-reject/`
- New reject gate (to create): `/home/one/gnucobol-rs/lab/oracle/cobol_frontend_reject_sweep.sh`
- Interpreter: `/home/one/gnucobol-rs/crates/gnucobol-rs/src/frontend.rs` (preprocess l.292, fixed_to_free l.266, lex l.167, parse_* l.842+, run_*/exec_* l.2020+; 183 `RunError::Unsupported` emit sites)
- Copybook pass: `/home/one/gnucobol-rs/crates/gnucobol-rs/src/copybook.rs` (`expand` l.308)
- Entry: `/home/one/gnucobol-rs/crates/gnucobol-rs/examples/cobrun.rs`
- Receipt (current 93/0): `/home/one/gnucobol-rs/reports/receipts/GNURUST.FRONTEND.1/receipt.md`
- Casefile (STALE 23/23 0.7.85 — regenerate): `/home/one/gnucobol-rs/reports/casefiles/GNURUST.FRONTEND.1/casefile.md`
- Guard wiring: `/home/one/gnucobol-rs/README.md` line 142
- 3.1.2 oracle: `/home/one/gnucobol-rs/lab/oracle/prefix-312` (present)