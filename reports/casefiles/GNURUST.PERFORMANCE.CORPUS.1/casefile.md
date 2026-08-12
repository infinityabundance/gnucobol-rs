<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.PERFORMANCE.CORPUS.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.57

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/performance/views.json (View E) + raw/view_e.json
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh performance`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- one full pass over 10 workloads x 4 scales per lane (oracle compile+run vs candidate prepare+run) with peak memory and raw samples retained
- unfavorable results are never discarded

## Negative claims (3) — negative capability is the trust surface
- no equivalence between compiled-native and interpreted runtime work
- no equivalence claim for the unlike View-A lanes
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
discarding slow candidate results would hide genuine performance characteristics

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
