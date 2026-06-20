<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILE.STATUS.1 — observed FILE STATUS bytes for declared file-operation fixtures

**Verdict: PASS** · replay `PASS=7 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILE.STATUS.1` |
| court | observed FILE STATUS bytes for declared file-operation fixtures |
| crate_version | `0.8.11` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | declared OPEN/READ/CLOSE condition -> observed FILE STATUS byte (00/06/10/35/42/46) |
| replay command | `bash lab/oracle/file_status_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- full file I/O parity
- indexed/relative/VSAM
- locking/sharing
- host I/O error (30) generalization
- attribute conflict (39)
- Procedure Division control flow
- business completeness

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
