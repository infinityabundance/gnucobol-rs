# Diagnostic-unblocked lane — claim boundaries

_schema: `gnurust-diag-unblocked-claim-boundaries-v1` · lane: `GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1`_

This lane exists for ONE narrow purpose: to let later semantic/runtime checks in upstream
Autotest groups run even when `gnucobol-rs` does not reproduce exact compiler diagnostic text.
It is an **additive experimental derivative** of the admitted GnuCOBOL Autotest suite — never a
replacement for upstream truth, and never a weakening of any other part of the testsuite.

## The three views stay separate

| view | what it answers | authority |
|---|---|---|
| A. PRISTINE upstream testsuite | how closely does the candidate satisfy the *original* upstream testsuite? | **the compatibility authority; unchanged and authoritative** |
| B. DIAGNOSTIC-UNBLOCKED testsuite | if compiler diagnostic wording is ignored, how much further through the real semantic tests does the candidate get? | this lane; never conflated with A |
| C. STEP/CORPUS phase probes | where exactly does a valid program first fail (preprocess / parse / resolve / layout / check / prepare / run)? | existing extraction machinery; preserved |

## What the lane MAY change

- ONLY expected **compiler-diagnostic stdout/stderr** fields of an `AT_CHECK` may become Autotest
  `ignore`, and ONLY when the transformer can prove the stream is purely compiler diagnostic
  output (621 expectations across 404 groups admitted; 620 stderr, 1 stdout).

## What the lane NEVER changes

- expected **exit statuses** (rule 5 — `[1]` stays `[1]`, `[0]` stays `[0]`);
- **commands** (rule 4 — byte-identical);
- **COBOL source** (rule 1/12 — no `.cob`, `.cpy`, C, shell or config input may change);
- **runtime stdout/stderr** (rules 6-9 — semantic output is never ignored);
- **generated-file expectations** (rule 11 — `AT_CAPTURE_FILE` and the 5th `AT_CHECK` argument
  are never weakened);
- **skip/xfail/ordering/grouping** (rules 12-14 — `AT_SKIP_IF`, `AT_XFAIL_IF`, `AT_SETUP`,
  `AT_DATA`, `AT_KEYWORDS`, test counts);
- the **pristine lane and all historical evidence**.

Every one of these is enforced by an INDEPENDENT patch-policy gate that parses the actual diff
(`diag-unblocked gate`), not by trusting the transformer. 24 unit tests cover the gate,
including adversarial fixtures that attempt each weakening and require failure.

## Claims the lane makes

- **Semantic reachability**: with diagnostic text no longer gating groups, 377 of the 404
  affected groups progress further; 140 later semantic checks become reachable (27 runtime
  executions, of which 12 match; 17 compiler checks fail). All figures are machine-derived from
  committed raw evidence (`semantic-reachability.json`).
- **The lane is mechanically restricted**: the patch regenerates byte-identically from the
  pristine sources (`patch_reproducible`), the group index is identical across lanes
  (`group_index_identical`), and AT_SETUP (1344==1344) / AT_CHECK (3422==3422) counts reconcile
  (`pristine-vs-diagnostic-unblocked.json`).
- **The oracle itself is diagnostic-text-gated in 4 places**: groups 116, 323, 336 and 350 are
  `AT_XFAIL_IF([true])` groups whose only blocking checks for the oracle are diagnostic-text
  checks; with the text ignored the oracle passes them (4 UNEXPECTED PASS in the unblocked
  oracle run, 0 in the pristine run). This is evidence that the suite's exact diagnostic text
  does not even match GnuCOBOL 3.2 in those groups — a suite-vs-oracle drift, not a candidate
  property.

## Claims the lane explicitly does NOT make

- **NO** “GnuCOBOL testsuite parity”. The pristine suite remains the compatibility authority;
  diagnostic-unblocked results are NOT pristine passes.
- **NO** “tests passed after fixing the suite”. The lane changes no test semantics.
- **NO** diagnostic compatibility. Ignored compiler diagnostic text is NOT diagnostic
  compatibility; the candidate does not fake `cobc` diagnostic strings and its own error model
  is unchanged.
- **NO** weakened exit-status semantics. Every expected status is still enforced exactly.
- **NO** weakened program output or generated artifacts. Semantic runtime output and
  generated-file expectations are still compared exactly.
- **NO** new language/runtime compatibility claims from diagnostic-only steps. A step whose
  PURPOSE is compiler diagnostic behaviour may be unblocked, but it is not counted as newly
  demonstrated language or runtime compatibility.
- **NO** self-validating evidence. The transformer decides solely from the upstream test
  structure and the nature of the expected compiler diagnostic contract — never from candidate
  behaviour — and the gate independently re-verifies the diff.

## The correct vocabulary

- **“Diagnostic-unblocked semantic reachability”** — the primary result of this lane.
- **NOT** “GnuCOBOL testsuite parity”.
- **NOT** “tests passed after fixing the suite”.

## Replay

```sh
bash lab/gnucobol-testsuite/run-diagnostic-unblocked-docker.sh          # two fresh passes
cargo run -p gnucobol-rs-corpus -- diag-unblocked reachability          # semantic reachability
cargo run -p gnucobol-rs-corpus -- diag-unblocked reconcile             # pristine vs unblocked
cargo run -p gnucobol-rs-corpus -- diag-unblocked cross-check           # corpus cross-check
```
