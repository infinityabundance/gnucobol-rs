<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.SUBSCRIPT.1 — table subscript TABLE(i[,j])

**Verdict: PASS** · replay `PASS=17 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.SUBSCRIPT.1` |
| court | table subscript TABLE(i[,j]) |
| crate_version | `0.8.37` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | OCCURS table bytes + subscripts -> element bytes |
| replay command | `bash lab/oracle/subscript_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- out-of-bounds subscript (fail-closed)
- OCCURS DEPENDING ON
- INDEXED BY index-names
- subscript arithmetic expressions

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
