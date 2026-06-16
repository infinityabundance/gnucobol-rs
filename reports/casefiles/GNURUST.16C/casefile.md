<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.16C (court-casefile)

**Verdict: PASS** · 153/153 pass, 0 fail · crate `gnucobol-rs` 0.7.84

- **Oracle:** cobc MOVE numeric -> edited, DISPLAY edited bytes (GnuCOBOL 3.2; complete-suppression cases differentially confirmed on 3.1.2)
- **Byte domain(s):** numeric value -> edited DISPLAY field bytes
- **Replay:** `bash lab/oracle/edited_encode_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- numeric value -> edited DISPLAY field bytes byte-faithful to cobc for Z 9 , . fixed and floating +/- signs (sign-aware) fixed-and-floating $ * CR DB B 0 / (zero-suppression, check-protection, floating currency/sign, sign placement, insertion), slot-based -- INCLUDING complete suppression of an exactly-zero value: an all-`Z` field (no forced `9`) blanks entirely (incl. the decimal point + comma), and an all-`*` field stars every position EXCEPT the decimal point (incl. comma + trailing sign). Verified against BOTH GnuCOBOL 3.2 and 3.1.2 (the complete-suppression cases byte-identical on both).

## Negative claims (5) — negative capability is the trust surface
- report writer
- locale/currency CURRENCY SIGN
- EBCDIC edited
- edited arithmetic/VALUE
- lie prevented: 'an edited field can be produced from a value without pinning the exact cobc byte layout' -- the float-symbol position, zero-suppression fill, and sign placement are pinned byte-exact to the oracle

## Damage if overclaimed
a wrong edited byte layout (misplaced float symbol, wrong suppression fill, wrong sign) silently mis-prints financial figures on statements and reports

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
