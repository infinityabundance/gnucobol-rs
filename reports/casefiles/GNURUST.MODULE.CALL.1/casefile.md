<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.MODULE.CALL.1 (court-casefile)

**Verdict: PASS** · crates/cobc-rs/tests/module_courts.rs (separately_compiled_callee_is_called_through_the_module) · crate `gnucobol-rs` 0.8.57

- **Oracle:** the admitted suite's caller/callee module tests (baseline stdout)
- **Byte domain(s):** caller/callee stdout, EXTERNAL storage flow, exit status
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- caller CALLs a separately compiled callee module (the suite's caller.cob/callee.cob pattern): the callee source is resolved at compile time and EXTERNAL items are shared across the call (run-unit store), producing the oracle's exact stdout (Hello/World)
- exercised by GNURUST.MODULE.MULTI.1 and the suite rerun

## Negative claims (3) — negative capability is the trust surface
- no dynamic loading of arbitrary DSOs
- modules are source-resolved, not dlopen'ed
- lie prevented: 'dynamic CALL loads native modules' is the lie this prevents

## Damage if overclaimed
overclaiming the module model as native dynamic loading

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
