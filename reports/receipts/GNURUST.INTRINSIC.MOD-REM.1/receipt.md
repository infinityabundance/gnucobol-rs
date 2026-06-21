<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INTRINSIC.MOD-REM.1 — FUNCTION MOD/REM integer modulo/remainder

**Verdict: PASS** · replay `PASS=20 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INTRINSIC.MOD-REM.1` |
| court | FUNCTION MOD/REM integer modulo/remainder |
| crate_version | `0.8.28` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | FUNCTION MOD(a,b) (divisor sign) / REM(a,b) (dividend sign) for integers |
| replay command | `bash lab/oracle/modrem_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- non-integer operands
- MOD/REM by zero
- MOD and REM interchangeable
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
