<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTRINSIC.ORD-CHAR.1 (court-casefile)

**Verdict: PASS** · 15/15 pass, 0 fail · crate `gnucobol-rs` 0.7.44

- **Oracle:** cobc FUNCTION ORD/CHAR (libcob/intrinsic.c)
- **Byte domain(s):** ORD(c)=byte+1 (1-based) / CHAR(n)=byte(n-1)
- **Replay:** `bash lab/oracle/ordchar_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- FUNCTION ORD(c) returns the 1-BASED position of byte c in the native ASCII collating sequence (ORD(c)=c+1, so ORD('A')=66 not 65) and FUNCTION CHAR(n) returns the byte at 1-based position n (CHAR(n)=n-1, CHAR(66)='A'), matching cobc/libcob (verified 15/0 incl the full ORD(CHAR(n))=n round-trip over 1..256). The 1-based inverses
- the last deterministic intrinsic split from GNURUST.INTRINSIC.ATLAS.1

## Negative claims (5) — negative capability is the trust surface
- non-default collating sequences
- national/UTF-8
- CHAR(n) outside 1..256
- all dialects
- lie prevented: ORD('A') is 65 -- NO: ORD is 1-BASED so ORD('A')=66 (the byte value PLUS ONE), and CHAR(66)='A' (byte 65); off-by-one between ORD/CHAR and a raw byte value is the classic trap

## Damage if overclaimed
treating ORD as the raw byte value (or CHAR as 0-based) is a silent off-by-one in every character-code computation

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
