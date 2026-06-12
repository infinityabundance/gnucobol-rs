<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.REMAINDER.1 (court-casefile)

**Verdict: PASS** · 768/768 pass, 0 fail · crate `gnucobol-rs` 0.7.39

- **Oracle:** cobc DIVIDE REMAINDER (program-shape)
- **Byte domain(s):** DIVIDE GIVING quotient + REMAINDER receiver field bytes
- **Replay:** `bash lab/oracle/remainder_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- DIVIDE a BY b GIVING q REMAINDER r + a INTO b GIVING q REMAINDER r quotient AND remainder receiver bytes (DISPLAY/COMP-3, signed/unsigned, scaled/unscaled quotient, positive/negative operands, exact/non-exact) matching cobc byte-for-byte
- the remainder is the COBOL definition dividend-(quotient-as-stored*divisor), so it depends on the quotient receiver's truncation
- sign follows the dividend

## Negative claims (8) — negative capability is the trust surface
- ON SIZE ERROR / NOT ON SIZE ERROR control flow
- divide-by-zero (fail-closed)
- COMPUTE / expression evaluation
- Procedure Division execution
- float
- binary/edited receivers
- business correctness
- lie prevented: 'remainder is just sign(dividend) times (dividend mod divisor)' -- the remainder depends on the QUOTIENT receiver's scale/truncation (dividend - stored-quotient * divisor), and the receiver bytes/sign/scale all matter

## Damage if overclaimed
a wrong remainder (wrong quotient-scale dependence, sign, or receiver bytes) misstates reconciliation residuals, allocation leftovers, or check digits while looking plausible

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
