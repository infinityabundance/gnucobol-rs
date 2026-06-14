<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.STRING.UNSTRING.1 — STRING/UNSTRING byte effects

**Verdict: PASS** · replay `PASS=7 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.STRING.UNSTRING.1` |
| court | STRING/UNSTRING byte effects |
| crate_version | `0.7.52` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | STRING target+pointer+overflow; UNSTRING field+count+delimiter+tally+pointer+overflow bytes |
| replay command | `bash lab/oracle/string_unstring_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- full Procedure Division execution
- national/UTF-8 multibyte
- multi-delimiter/ALL generalization
- locale/collation
- business parsing correctness
- all dialects

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
