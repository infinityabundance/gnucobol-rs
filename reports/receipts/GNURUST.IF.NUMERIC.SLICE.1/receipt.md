<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.IF.NUMERIC.SLICE.1 — numeric IF/EVALUATE execution slice

**Verdict: PASS** · replay `PASS=7 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.IF.NUMERIC.SLICE.1` |
| court | numeric IF/EVALUATE execution slice |
| crate_version | `0.7.25` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | execute numeric IF/EVALUATE over 9(n) fields -> resulting storage bytes |
| replay command | `bash lab/oracle/if_numeric_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- signed/packed/V-scaled numerics
- MOVE field TO field (literals only)
- numeric SIZE ERROR
- compound/class conditions
- 88-level (GNURUST.11)
- range/THRU WHEN
- non-MOVE branches
- nested flow
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
