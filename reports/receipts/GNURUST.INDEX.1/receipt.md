<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.INDEX.1 — USAGE INDEX storage + SET arithmetic

**Verdict: PASS** · replay `PASS=41 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.INDEX.1` |
| court | USAGE INDEX storage + SET arithmetic |
| crate_version | `0.8.45` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | occurrence value + SET op -> 4 native-endian index bytes |
| replay command | `bash lab/oracle/index_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- 8-byte LP64 index width
- SEARCH / SEARCH ALL execution
- index used as a subscript (stride multiply lives in GNURUST.SUBSCRIPT.1)
- big-endian host
- SET index TO pointer/address

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
