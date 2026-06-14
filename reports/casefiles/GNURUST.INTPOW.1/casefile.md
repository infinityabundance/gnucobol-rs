<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INTPOW.1 (court-casefile)

**Verdict: PASS** · pow_sweep 588/0 (bases incl 0/1/-1/overflowing x powers incl 0/negative/overflowing, both widths) · crate `gnucobol-rs` 0.7.46

- **Oracle:** libcob cob_s32_pow / cob_s64_pow
- **Byte domain(s):** (base, power, width 32/64) -> integer result (two's-complement wrapping)
- **Replay:** `bash lab/oracle/pow_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- the integer `**` power that cobc lowers to cob_s32_pow/cob_s64_pow (numeric.c POW_IMPL), byte-faithful incl. the exact edge cases: power 0 or |base|==1 -> 1 (base==-1 returns 1 for ANY power, a libcob quirk, so (-1)**3 = 1 not -1)
- negative power -> 0 (0 ** negative raises SIGFPE in libcob -> fail closed)
- otherwise repeated multiply WRAPPING in the target integer width

## Negative claims (4) — negative capability is the trust surface
- the decimal `**` operator (cob_decimal_pow lives in intrinsic.c, a separate court)
- fractional exponents
- 0 ** negative (fails closed rather than SIGFPE-crashing)
- lie prevented: '(-1) ** odd = -1' -- NO, libcob's POW_IMPL returns 1 for base==-1 regardless of the power; a 'correct' math result is a divergence from the oracle

## Damage if overclaimed
a 'nicer' power (correct sign, saturating instead of wrapping) diverges from generated GnuCOBOL code on every integer ** with a -1 base or an overflowing result

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
