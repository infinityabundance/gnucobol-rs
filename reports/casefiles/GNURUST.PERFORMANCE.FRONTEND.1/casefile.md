<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.PERFORMANCE.FRONTEND.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.57

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/performance/phase-metrics.json + views.json (View B)
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh performance`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- candidate per-phase timings (preprocess/lex/parse/resolution/layout/check/prepare) are measured separately from oracle compile (phase-metrics.json + views.json View B)

## Negative claims (3) — negative capability is the trust surface
- no native-code performance claim without a native candidate path
- View A is labelled 'unlike workflows' (compiled vs interpreted) and is never described as equivalent runtime work
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
conflating compile+run with parse/check+interpret would misrepresent what is being timed

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
