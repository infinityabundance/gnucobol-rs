<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTRINSIC.CASE.1 (court-casefile)

**Verdict: PASS** · 10/10 pass, 0 fail · crate `gnucobol-rs` 0.8.53

- **Oracle:** cobc FUNCTION UPPER-CASE/LOWER-CASE/REVERSE (libcob/intrinsic.c)
- **Byte domain(s):** ASCII case fold + byte reversal
- **Replay:** `bash lab/oracle/case_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- FUNCTION UPPER-CASE (ASCII a..z -> A..Z), LOWER-CASE (A..Z -> a..z), and REVERSE (byte reversal) match cobc/libcob byte-for-byte (verified 10/0): non-alphabetic bytes (digits, spaces, punctuation) are UNCHANGED by case folding, the result is the SAME LENGTH as the input, and REVERSE reverses ALL bytes including spaces. The implemented string-transform intrinsics, split from GNURUST.INTRINSIC.ATLAS.1

## Negative claims (4) — negative capability is the trust surface
- locale/national case folding (non-ASCII)
- multibyte/UTF-8 REVERSE
- all dialects
- lie prevented: UPPER-CASE uppercases everything -- NO: only ASCII a-z fold (digits/spaces/punctuation and any non-ASCII byte are UNCHANGED), and REVERSE reverses raw BYTES including trailing spaces (not a logical trim-then-reverse)

## Damage if overclaimed
assuming locale/accented case folding, or that REVERSE trims spaces, corrupts text normalization in a ported program

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
