<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.REFMOD.1 — reference modification field(start:length)

**Verdict: PASS** · replay `PASS=16 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.REFMOD.1` |
| court | reference modification field(start:length) |
| crate_version | `0.8.50` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | field bytes + (start,length) -> substring / overwritten field |
| replay command | `bash lab/oracle/refmod_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- out-of-bounds refmod (fail-closed)
- subscripted/table refmod
- numeric-edited/national operands
- refmod inside arithmetic

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
