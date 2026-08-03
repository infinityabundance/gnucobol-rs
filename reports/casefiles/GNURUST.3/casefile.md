<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.3 (court-casefile)

**Verdict: PASS** · 432/432 pass, 0 fail · crate `gnucobol-rs` 0.8.52

- **Oracle:** cobc -C attr witness
- **Byte domain(s):** generated-C cob_field_attr + LENGTH OF
- **Replay:** `bash lab/oracle/pic_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- PIC+USAGE -> {type,digits,scale,flags,size} matching cobc

## Negative claims (3) — negative capability is the trust surface
- edited pictures
- usages beyond DISPLAY/COMP-3
- lie prevented: 'a PIC width can be eyeballed' — the field model matches cobc's own attrs

## Damage if overclaimed
a mis-modeled field width mis-reads every downstream field in the record

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
