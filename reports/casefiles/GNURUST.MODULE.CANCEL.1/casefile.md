<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.MODULE.CANCEL.1 (court-casefile)

**Verdict: PASS** · crates/cobc-rs/tests/module_courts.rs (cancel_resets_persisted_working_storage, cancel_of_active_program_is_fatal) · crate `gnucobol-rs` 0.8.55

- **Oracle:** the admitted suite's CANCEL tests (run_fundamental.at:2277-2341)
- **Byte domain(s):** module state across CALL/CANCEL, fatal-error stderr + exit code
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- a called module's WORKING-STORAGE persists across calls and is reset by CANCEL (oracle-shaped C=1/C=2/C=1), and CANCELing the active non-INITIAL program raises the libcob-shaped fatal 'attempt to CANCEL active program' (exit 1, source line)
- exercised by GNURUST.MODULE.CANCEL.1 tests

## Negative claims (3) — negative capability is the trust surface
- no claim about INITIAL-program edge cases beyond the tested forms
- physical CANCEL semantics are not native-unload
- lie prevented: 'CANCEL is a no-op' is the lie this prevents

## Damage if overclaimed
overclaiming physical module unload

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
