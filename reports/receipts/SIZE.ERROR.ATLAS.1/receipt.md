<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# SIZE.ERROR.ATLAS.1 — arithmetic size-error behavior atlas

**Verdict: PASS** · replay `PASS=12 FAIL=0`

| field | value |
|-------|-------|
| campaign | `SIZE.ERROR.ATLAS.1` |
| court | arithmetic size-error behavior atlas |
| crate_version | `0.7.17` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | observed receiver-written vs preserved + size_error_signaled for ADD/SUB/MUL/DIVIDE overflow + divide-by-zero (DISPLAY/COMP-3) |
| replay command | `bash lab/oracle/size_error_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- ON SIZE ERROR / NOT ON SIZE ERROR control flow (not implemented)
- Procedure Division execution
- receiver-write inference (observed, not inferred)
- branch execution
- business arithmetic correctness

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
