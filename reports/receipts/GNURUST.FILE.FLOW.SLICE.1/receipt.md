<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILE.FLOW.SLICE.1 — read-loop execution slice (file x control flow)

**Verdict: PASS** · replay `PASS=2 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILE.FLOW.SLICE.1` |
| court | read-loop execution slice (file x control flow) |
| crate_version | `0.8.43` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OPEN INPUT + PERFORM UNTIL EOF READ + accumulate -> resulting WORKING-STORAGE |
| replay command | `bash lab/oracle/file_flow_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- indexed/relative organizations
- signed/packed accumulators
- numeric SIZE ERROR on accumulators
- per-record IF/MOVE/general statements
- WRITE/REWRITE in the loop
- multi-file loops
- file-status beyond AT END
- READ INTO
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
