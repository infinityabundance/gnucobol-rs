<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.10 (court-casefile)

**Verdict: PASS** · 30 sweep + 4M fuzz · crate `gnucobol-rs` 0.7.27

- **Oracle:** cobc -C storage allocation
- **Byte domain(s):** generated-C storage size b_REC[size] (NOT runtime LENGTH OF)
- **Replay:** `bash lab/oracle/odo_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- single trailing OCCURS DEPENDING ON physical maximum record size matching cobc allocation

## Negative claims (5) — negative capability is the trust surface
- active/logical occurrence count
- sliding
- runtime validation
- multiple/nested ODO
- lie prevented: 'LENGTH OF proves physical allocation' — ODO physical-max != logical length

## Damage if overclaimed
treating ODO physical-max as the live length over/under-reads variable records

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
