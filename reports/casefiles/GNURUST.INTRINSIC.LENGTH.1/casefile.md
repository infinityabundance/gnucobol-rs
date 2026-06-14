<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTRINSIC.LENGTH.1 (court-casefile)

**Verdict: PASS** · 12/12 pass, 0 fail · crate `gnucobol-rs` 0.7.63

- **Oracle:** cobc FUNCTION LENGTH (libcob/intrinsic.c)
- **Byte domain(s):** FUNCTION LENGTH(field) -> storage byte length
- **Replay:** `bash lab/oracle/length_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- FUNCTION LENGTH(elementary field) returns the storage byte length, matching cobc/libcob across DISPLAY (X(n)=n, numeric 9/S9 = digit count incl V implied), COMP-3 (packed byte count), and binary COMP/COMP-5 (storage width) -- the same byte count the sealed field model build_field (GNURUST.3/9/14) computes
- the first IMPLEMENTED intrinsic, split out of GNURUST.INTRINSIC.ATLAS.1

## Negative claims (5) — negative capability is the trust surface
- LENGTH of group/table/reference-modified operand
- LENGTH OF variant
- national/UTF-8 character length
- all dialects
- lie prevented: 'LENGTH is the digit count' -- for COMP-3 it is the PACKED byte count (S9(7)=4 bytes not 7), for binary the storage width (9(4) COMP=2), and the field model and the runtime intrinsic agree byte-for-byte

## Damage if overclaimed
using a digit count where the storage byte length is meant (COMP-3/binary fields) mis-sizes buffers and offsets

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
