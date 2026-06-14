<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.STRING.UNSTRING.1 (court-casefile)

**Verdict: PASS** · 7/7 pass, 0 fail · crate `gnucobol-rs` 0.7.57

- **Oracle:** cobc STRING/UNSTRING (program-shape, FUNCTION HEX-OF result-record dump)
- **Byte domain(s):** STRING/UNSTRING receiver + pointer/count/delimiter/tally bytes
- **Replay:** `bash lab/oracle/string_unstring_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- the receiver bytes + pointer/count/delimiter/tally/overflow of narrow STRING and UNSTRING statements, matching cobc/libcob byte-for-byte: STRING concatenates sources left-to-right at a 1-based POINTER (DELIMITED BY SIZE = whole operand, DELIMITED BY lit = operand up to the first lit), PRESERVES the unwritten target tail (no space-fill), keeps partial writes on ON OVERFLOW
- UNSTRING splits the source by a delimiter into fields where COUNT IN is the SOURCE-FIELD LENGTH BEFORE TRUNCATION, DELIMITER IN is the ending delimiter (SPACE when ended at source exhaustion), an empty field between adjacent delimiters has count 0, TALLYING IN counts filled fields, the POINTER is 1-based, and leftover source after the last field is overflow

## Negative claims (7) — negative capability is the trust surface
- full Procedure Division execution
- national/UTF-8 multibyte
- multi-delimiter/ALL generalization
- locale/collation
- business parsing correctness
- all dialects
- lie prevented: 'STRING/UNSTRING just concatenate and split' -- STRING PRESERVES the target tail (does not space-fill) and keeps partial writes on overflow, the POINTER is 1-based, UNSTRING COUNT IN is the SOURCE length BEFORE truncation (a 7-char field into X(4) reports count 7), and DELIMITER IN is SPACE when the field ends at source exhaustion

## Damage if overclaimed
a wrong pointer base, count-vs-truncation, tail-preservation, or delimiter assumption in a STRING/UNSTRING transform silently corrupts assembled/split fields

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
