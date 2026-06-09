<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.IF.EVALUATE.SLICE.1 — IF/EVALUATE execution slice (alphanumeric)

**Verdict: PASS** · replay `PASS=9 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.IF.EVALUATE.SLICE.1` |
| court | IF/EVALUATE execution slice (alphanumeric) |
| crate_version | `0.7.23` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | execute IF/EVALUATE fragment over alphanumeric fields -> resulting storage bytes |
| replay command | `bash lab/oracle/if_eval_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- numeric/packed comparison + numeric MOVE
- compound conditions (AND/OR/NOT)
- class conditions
- 88-level (GNURUST.11)
- non-MOVE branches
- nested IF/PERFORM/GO TO
- THRU/range WHEN
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
