<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.INDEXED.1 — INDEXED organization (keyed store + record locking)

**Verdict: PASS** · replay `PASS=18 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.INDEXED.1` |
| court | INDEXED organization (keyed store + record locking) |
| crate_version | `0.8.55` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | INDEXED operations (WRITE/READ/READ NEXT/START/REWRITE/DELETE + record/file locks) over a primary key -> FILE STATUS + record bytes + key order |
| replay command | `bash lab/oracle/indexed_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the on-disk index file format (BDB/VBISAM/EXTFH -- the declared OS boundary)
- ALTERNATE RECORD KEYs and WITH DUPLICATES (only the primary key is the court)
- READ PREVIOUS / descending traversal
- the actual cross-process BDB lock environment (52 deadlock / 30 permanent-error)
- variable-length indexed records
- the bdb_*/EXTFH substrate functions

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
