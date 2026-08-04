<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.COMMON.CBL.1 — CBL_ logic / bit / case builtins (CBL_AND/OR/XOR/NOR/IMP/NIMP/EQ/NOT/TOUPPER/TOLOWER)

**Verdict: PASS** · replay `PASS=2 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.COMMON.CBL.1` |
| court | CBL_ logic / bit / case builtins (CBL_AND/OR/XOR/NOR/IMP/NIMP/EQ/NOT/TOUPPER/TOLOWER) |
| crate_version | `0.8.54` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | CALL USING buffer(s) + length -> the in-place transformed destination bytes (verified vs both oracles 3.1.2 + 3.2) |
| replay command | `bash lab/oracle/cbl_logic_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the COB_CHK_PARMS parameter-count diagnostic
- CBL_GC_PRINTABLE locale + custom dot-char; cob_sys_x91 multiplexer
- the OS/process CBL routines (getpid/system/fork/nanosleep)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
