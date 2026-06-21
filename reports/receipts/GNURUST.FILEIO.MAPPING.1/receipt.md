<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.MAPPING.1 — COBOL filename mapping (env resolution)

**Verdict: PASS** · replay `PASS=2 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.MAPPING.1` |
| court | COBOL filename mapping (env resolution) |
| crate_version | `0.8.35` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a COBOL ASSIGN name + environment (DD_*/COB_FILE_PATH) -> the resolved filesystem path |
| replay command | `bash lab/oracle/map_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the complex multi-element path mapping (per-slash $/DD_ resolution)
- COB_ENV_MANGLE non-default
- the ACU repeated-resolution recursion
- the flag_filename_mapping=off dialect case
- concatenated (multi-file) input names
- getenv ordering beyond the three prefixes

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
