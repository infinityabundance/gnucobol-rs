<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.GNUCOBOL-RUNTIME-MATH.1 (court-casefile)

**Verdict: PASS** · reports/gnucobol-runtime-tests/math-correctness.{json,md} · crate `gnucobol-rs` 0.8.54

- **Oracle:** the TESTSUITE.1 baseline
- **Byte domain(s):** per-math-test classification + the underlying raw outputs
- **Replay:** `the TESTSUITE.1 baseline`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- the math/runtime subset (data_binary, data_display, data_packed, data_pointer, run_fundamental, run_functions, syn_multiply, syn_value, syn_literals) is classified from the SAME differential results as the whole suite (no favorable selection): 323 math tests with per-test oracle/candidate outcome pairs and first-failure attribution
- performance is reported SEPARATELY and only for tests passing on both sides

## Negative claims (3) — negative capability is the trust surface
- no performance claim here
- correctness is the suite's AT_CHECK outcome in this environment
- lie prevented: 'the math tests pass' is the lie this prevents -- the classification shows exactly which math tests match, which fail closed, and which are module-model blocked

## Damage if overclaimed
claiming math parity from a favorable subset would be unscientific

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
