<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.CORPUS.CUSTODY.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** committed corpus evidence files (presence + freeze)
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh custody`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the pre-change repository state is frozen (preflight-repository-state.json + before-state.json + integration-design.md) and every family report directory exists under reports/valid-corpus/

## Negative claims (2) — negative capability is the trust surface
- no validity claim: custody proves the evidence tree exists and was frozen, not that any program is valid
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
presenting custody as validity would certify programs this court never checked

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
