<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.PERFORMANCE.PREPARED.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/performance/views.json (View C) + raw samples
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh performance`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- already-compiled native binaries are compared with already-prepared programs run repeatedly WITHOUT reparsing (views.json View C + raw/view_c.json)
- prepared execution never touches the source again

## Negative claims (3) — negative capability is the trust surface
- no runtime-performance claim before correctness is established
- no candidate lane is benchmarked before its output is byte-exact
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
reporting timing for a wrong-output lane would benchmark an incorrect program

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
