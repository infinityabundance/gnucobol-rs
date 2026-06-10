<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.CALL.LAYOUT.ATLAS.1 — observed CALL parameter byte-layout atlas

**Verdict: PASS** · replay `PASS=5 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.CALL.LAYOUT.ATLAS.1` |
| court | observed CALL parameter byte-layout atlas |
| crate_version | `0.7.27` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | CALL USING parameter byte layout: BY REFERENCE address overlay (into adjacent storage), BY CONTENT sized copy, numeric length-mismatch leading-byte overlay |
| replay command | `bash lab/oracle/call_layout_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- subprogram execution
- BY CONTENT over-read value (undefined)
- BY VALUE byte layout
- OCCURS DEPENDING ON across linkage
- OPTIONAL / OMITTED parameters
- the RETURNING phrase
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
