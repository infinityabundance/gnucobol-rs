<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.COMMON.BOUNDCHECK.1 — Runtime bounds-check diagnostics (subscript / reference-mod / OCCURS DEPENDING ON)

**Verdict: PASS** · replay `PASS=6 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.COMMON.BOUNDCHECK.1` |
| court | Runtime bounds-check diagnostics (subscript / reference-mod / OCCURS DEPENDING ON) |
| crate_version | `0.8.46` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a bounds-check input -> the exact runtime EC-BOUND diagnostic message + hint bytes (verified vs both oracles 3.1.2 + 3.2) |
| replay command | `bash lab/oracle/bounds_check_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the libcob: <file>:<line>: error: / note: prefix framing (the runtime wrapper, not the check)
- the not-numeric message is sealed separately (GNURUST.COMMON.NUMCHECK.1)
- the cob_hard_failure abort + exit code + EC numeric ids
- the 2.0-ABI cannot_check_subscript state path beyond the zero-subscript case

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
