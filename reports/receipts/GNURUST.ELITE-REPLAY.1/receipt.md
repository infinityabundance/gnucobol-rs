<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.ELITE-REPLAY.1 — opencbs real-program replay -- run the public third-party opencbs COBOL programs through cobc AND cobrun; observable behaviour (stdout bytes + process exit status + stderr-clean) must agree

**Verdict: PASS** · replay `PASS=31 FAIL=0 SKIP=22 MATCH=31`

| field | value |
|-------|-------|
| campaign | `GNURUST.ELITE-REPLAY.1` |
| court | opencbs real-program replay -- run the public third-party opencbs COBOL programs through cobc AND cobrun; observable behaviour (stdout bytes + process exit status + stderr-clean) must agree |
| crate_version | `0.8.48` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a real third-party COBOL program -> its exact stdout bytes + process exit status, byte-identical to cobc, for every in-scope program; a ratchet floors the MATCH count so it can only rise |
| replay command | `bash lab/oracle/opencbs_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the NOT-YET-built set below is DOABLE, not a permanent boundary (cobc runs them all observably); each is a target driven toward MATCH one verified conversion at a time
- CALL to a module that does not exist -- the faithful behaviour is libcob module-not-found + exit 1 (DF18/31/45CALL)
- ORGANIZATION INDEXED / SORT-from-file / VARYING-length / OPEN I-O / REWRITE real-file access (DF03/05/22/25/46TEST)
- a qualified+subscripted compound condition operand (DF02TEST)
- out of scope: the deliberately-broken defect snippets the suite ships that cobc itself cannot compile (no oracle baseline)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
