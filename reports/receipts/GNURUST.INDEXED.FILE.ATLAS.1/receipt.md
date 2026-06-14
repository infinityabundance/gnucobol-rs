<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INDEXED.FILE.ATLAS.1 — observed indexed-file atlas

**Verdict: PASS** · replay `PASS=9 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INDEXED.FILE.ATLAS.1` |
| court | observed indexed-file atlas |
| crate_version | `0.7.56` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | INDEXED keyed access: random READ by key, key-order retrieval, dup/not-found status, START, DELETE |
| replay command | `bash lab/oracle/indexed_file_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- ISAM/BDB/VBISAM on-disk format
- page checksum / atomicity
- alternate keys
- concurrent access/locking
- indexed file execution
- relative files
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
