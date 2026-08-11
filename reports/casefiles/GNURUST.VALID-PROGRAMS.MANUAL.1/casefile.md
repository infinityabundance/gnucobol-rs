<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.VALID-PROGRAMS.MANUAL.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.56

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/gnucobol-manual/{stable-3.2,current}/*
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh valid-manual`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- every manual code block is classified in both lanes (stable-3.2 + current examples.json + snippets.json)
- complete examples are replay-verified

## Negative claims (3) — negative capability is the trust surface
- partial snippets, pseudocode and command examples are not executable programs
- incomplete/obsolete commands are recorded, never silently repaired
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
counting snippets as executable programs would fabricate runnable evidence

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
