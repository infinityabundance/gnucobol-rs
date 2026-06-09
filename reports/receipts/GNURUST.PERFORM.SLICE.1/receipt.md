<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.PERFORM.SLICE.1 — PERFORM execution slice (TIMES/UNTIL/VARYING)

**Verdict: PASS** · replay `PASS=10 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.PERFORM.SLICE.1` |
| court | PERFORM execution slice (TIMES/UNTIL/VARYING) |
| crate_version | `0.7.25` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | execute PERFORM loop over numeric counters -> resulting storage bytes |
| replay command | `bash lab/oracle/perform_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- signed/packed/binary counters
- numeric SIZE ERROR on the body
- non-ADD body statements
- compound/class conditions
- PERFORM THRU / out-of-line paragraph
- WITH TEST AFTER
- nested PERFORM / GO TO
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
