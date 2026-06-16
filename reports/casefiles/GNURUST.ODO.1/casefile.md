<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.ODO.1 (court-casefile)

**Verdict: PASS** · 10/10 pass, 0 fail · crate `gnucobol-rs` 0.7.85

- **Oracle:** cobc LENGTH OF / E(i) over an OCCURS DEPENDING ON group
- **Byte domain(s):** controlling value + table -> used length / active element bytes
- **Replay:** `bash lab/oracle/odo_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- the used byte length of an OCCURS 1 TO max DEPENDING ON N group (fixed_prefix
- N*element_size, matching cobc LENGTH OF) and bounded element access E(i) for 1<=i<=N

## Negative claims (6) — negative capability is the trust surface
- element access beyond the active count (fail-closed by design)
- nested/2-D ODO
- ODO in a file record with VARYING
- SET of the controlling field mid-access
- signed/packed elements
- lie prevented: 'an ODO group is always its maximum length' -- NO, its used length is fixed_prefix + N*element_size and only N elements are active

## Damage if overclaimed
treating an ODO record as max-length over-reads or over-writes a variable-length record, corrupting the next record or trailing data

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
