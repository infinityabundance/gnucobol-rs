<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.5 (court-casefile)

**Verdict: PASS** · 7 programs sweep + 4M fuzz · crate `gnucobol-rs` 0.7.59

- **Oracle:** cobc -P
- **Byte domain(s):** expanded source text-word stream
- **Replay:** `bash lab/oracle/copy_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- COPY <name>. splice matching cobc -P text-word stream (nested, cycle-detected, provenance-mapped)

## Negative claims (4) — negative capability is the trust surface
- inline/multi-line COPY
- OF/IN library
- SUPPRESS
- lie prevented: 'substring replacement is close enough' — COPY REPLACING is whole-text-word

## Damage if overclaimed
a wrong COPY expansion lays out the file against a copybook the data was never written with

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
