<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.MODULE.SEARCH.1 (court-casefile)

**Verdict: PASS** · crates/cobc-rs/tests/module_courts.rs (cobcrun_m_searches_the_module_directory, cobcrun_module_search_uses_cwd_and_library_path, cobcrun_error_messages_match_cobcrun) · crate `gnucobol-rs` 0.8.55

- **Oracle:** the admitted suite's used_binaries.at module tests (0010, 0014, 0015, 0018)
- **Byte domain(s):** search-path resolution + diagnostic stdout/stderr/exit
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- cobcrun-rs resolves modules via -M <dir> (with and without the trailing slash), cwd and COB_LIBRARY_PATH, and emits the cobcrun diagnostics for missing program name / invalid module argument / cannot find module with cobcrun's exit codes

## Negative claims (2) — negative capability is the trust surface
- no claim that module-name case folding matches cobcrun beyond the tested forms
- lie prevented: 'cobcrun can't find anything' is the lie this prevents

## Damage if overclaimed
overclaiming search-path parity beyond the tested surface

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
