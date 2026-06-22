<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FRONTEND.1 — clean-room COBOL front-end -- parse + EXECUTE a program subset to cobc-identical stdout

**Verdict: FAIL** · replay `PASS=172 FAIL=0 (3.1.2 differential-matched=161)`

| field | value |
|-------|-------|
| campaign | `GNURUST.FRONTEND.1` |
| court | clean-room COBOL front-end -- parse + EXECUTE a program subset to cobc-identical stdout |
| crate_version | `0.8.43` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a COBOL program (sealed subset) -> the exact stdout bytes it writes, byte-identical to cobc |
| replay command | `bash lab/oracle/cobol_frontend_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- an INTERPRETER over the runtime, NOT a native-code compiler (no machine-code/.o/.so emission)
- group items / OCCURS / REDEFINES / non-01 levels; COMPUTE + arithmetic expressions
- control flow (IF/PERFORM/EVALUATE/GO TO -- separate execution-slice courts); ACCEPT; file I/O; CALL; multiple programs
- the full PICTURE + statement grammar; any verb/clause outside the listed subset; all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
