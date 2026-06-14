<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.REFMOD.1 (court-casefile)

**Verdict: PASS** · 16/16 pass, 0 fail · crate `gnucobol-rs` 0.7.53

- **Oracle:** cobc DISPLAY field(start:length) / MOVE TO field(start:length)
- **Byte domain(s):** field bytes + (start,length) -> substring bytes / overwritten field
- **Replay:** `bash lab/oracle/refmod_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- field(start:length) substring extraction, field(start:) to-end, and the receiver MOVE src TO field(start:length) (1-based start, alphanumeric overwrite) matching cobc in bounds

## Negative claims (5) — negative capability is the trust surface
- out-of-bounds refmod (fail-closed by design, NOT cobc's flag-dependent read-past)
- subscripted/table refmod
- numeric-edited or national operands
- refmod inside arithmetic
- lie prevented: 'refmod is just a slice cobc and we agree on everywhere' -- out of bounds we FAIL CLOSED rather than read adjacent storage like cobc may

## Damage if overclaimed
an out-of-bounds substring silently reads or corrupts neighbouring fields

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
