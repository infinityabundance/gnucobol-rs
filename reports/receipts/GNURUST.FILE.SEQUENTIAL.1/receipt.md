<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILE.SEQUENTIAL.1 — sequential file READ record bytes + file status

**Verdict: PASS** · replay `PASS=10 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILE.SEQUENTIAL.1` |
| court | sequential file READ record bytes + file status |
| crate_version | `0.8.52` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OPEN INPUT / READ NEXT / AT END record bytes + status for RECORD/LINE SEQUENTIAL |
| replay command | `bash lab/oracle/seqfile_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- indexed / relative / VSAM
- WRITE / REWRITE / START / DELETE
- OPEN I-O / EXTEND
- file-status codes beyond 00/06/10
- record locking
- Procedure Division control flow

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
