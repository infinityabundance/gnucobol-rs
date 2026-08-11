<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.ELITE-REPLAY.1 — opencbs real-program replay -- run the public third-party opencbs COBOL programs through cobc AND cobrun; observable behaviour (stdout bytes + process exit status + stderr-clean) must agree

**Verdict: PASS** · replay `PASS=39 FAIL=0 SKIP=14 MATCH=39`

| field | value |
|-------|-------|
| campaign | `GNURUST.ELITE-REPLAY.1` |
| court | opencbs real-program replay -- run the public third-party opencbs COBOL programs through cobc AND cobrun; observable behaviour (stdout bytes + process exit status + stderr-clean) must agree |
| crate_version | `0.8.56` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a real third-party COBOL program -> its exact stdout bytes + process exit status, byte-identical to cobc, for every in-scope program; a ratchet floors the MATCH count so it can only rise |
| replay command | `bash lab/oracle/opencbs_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- MATCH 39/39 cobc-runnable programs (byte-identical stdout + exit + stderr-clean): every opencbs program cobc can run, cobrun reproduces. NOTYET and BOUNDARY are EMPTY.
- the 3 external-CALL programs (DF18/31/45CALL) match via REAL separate-file CALL: cobrun resolves the called sibling .CBL as another program unit and runs it (USING args by reference), mirroring cobc compiling the callees as modules; the callees (DF18/31/45TEST) are subprograms compiled as cobc -m
- writing programs (REWRITE / OPEN OUTPUT) run with per-program data-file snapshot/restore so the differential is sound (cobc persists disk writes; cobrun is read-only) and the committed corpus is never mutated
- out of scope: 11 deliberately-broken DEFECT-DEMONSTRATION snippets (missing PROGRAM-ID / syntax errors / undefined names) that cobc ITSELF rejects -- making them compile would mean editing the third-party fixtures to fix the bugs they exist to demonstrate

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
