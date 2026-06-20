<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.SYS.1 — CBL_* system file/directory routines

**Verdict: PASS** · replay `PASS=2 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.SYS.1` |
| court | CBL_* system file/directory routines |
| crate_version | `0.8.12` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | CBL_DELETE_FILE/COPY_FILE/RENAME_FILE/CREATE_DIR/DELETE_DIR/CHANGE_DIR/GET_CURRENT_DIR -> RETURN-CODE (0/35/128/129/-1) |
| replay command | `bash lab/oracle/cob_sys_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- CBL_CHECK_FILE_EXIST / cob_sys_file_info localtime date formatting
- the file-lock / file_dev open parameters
- the ACUCOBOL hyphen / filename-mapping path
- non-UTF-8 path bytes
- the exact errno->status mapping beyond 35/128
- concurrent filesystem races
- the handle value matching libcob's raw fd (a safe registry index is used)
- the underlying syscalls (declared OS boundary)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
