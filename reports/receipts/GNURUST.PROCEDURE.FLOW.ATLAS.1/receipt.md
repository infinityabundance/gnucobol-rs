<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.PROCEDURE.FLOW.ATLAS.1 — observed control-flow atlas

**Verdict: PASS** · replay `PASS=8 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.PROCEDURE.FLOW.ATLAS.1` |
| court | observed control-flow atlas |
| crate_version | `0.7.23` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | control-flow statement class -> observed behavior (IF/EVALUATE/PERFORM/GO TO) |
| replay command | `bash lab/oracle/procedure_flow_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- Procedure Division execution
- control-flow execution
- branch coverage
- termination analysis
- general condition evaluation
- status is not implementation
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
