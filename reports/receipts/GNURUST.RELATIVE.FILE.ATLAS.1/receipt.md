<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.RELATIVE.FILE.ATLAS.1 — observed relative-file atlas

**Verdict: PASS** · replay `PASS=3 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.RELATIVE.FILE.ATLAS.1` |
| court | observed relative-file atlas |
| crate_version | `0.7.68` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | RELATIVE random access by record number + status (23 empty slot) |
| replay command | `bash lab/oracle/relative_file_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- relative file execution
- on-disk slotted format
- sequential/dynamic access modes
- REWRITE/DELETE/START
- indexed files
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
