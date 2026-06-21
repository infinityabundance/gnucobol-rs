<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.SEQ.1 — RECORD SEQUENTIAL read/write incl. variable-length

**Verdict: PASS** · replay `PASS=9 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.SEQ.1` |
| court | RECORD SEQUENTIAL read/write incl. variable-length |
| crate_version | `0.8.29` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OPEN + WRITE/READ (RECORD SEQUENTIAL) -> fixed full record / variable cob_varseq_type prefix + data + FILE STATUS |
| replay command | `bash lab/oracle/seqrec_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- WRITE ADVANCING (cob_seq_write_opt)
- CODE-SET conversion
- over-long record bytes_to_skip seek-past
- multi-file concatenation
- relative/indexed organizations
- the fd read/write/lseek syscalls (declared OS boundary)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
