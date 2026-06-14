<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.SYS.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.7.49

- **Oracle:** cobc CALL CBL_* system routines, RETURN-CODE after each (libcob/fileio.c)
- **Byte domain(s):** CBL_DELETE_FILE/COPY_FILE/RENAME_FILE/CREATE_DIR/DELETE_DIR/CHANGE_DIR/GET_CURRENT_DIR -> RETURN-CODE (0/35/128/129/-1) + the GET_CURRENT_DIR field bytes
- **Replay:** `bash lab/oracle/cob_sys_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- a faithful port of fileio.c's cob_sys_delete_file/cob_sys_copy_file/cob_sys_rename_file/cob_sys_create_dir/cob_sys_delete_dir/cob_sys_change_dir/cob_sys_get_current_dir -- the CBL_DELETE_FILE/CBL_COPY_FILE/CBL_RENAME_FILE/CBL_CREATE_DIR/CBL_DELETE_DIR/CBL_CHANGE_DIR/CBL_GET_CURRENT_DIR library routines a COBOL program CALLs -- with the documented status codes, proven to match the admitted libcob's RETURN-CODE sequence (cob_sys_sweep 1/0): each does the syscall (via std::fs/std::env) and returns 0 on success, 128 on failure, 35 when a copy source is absent, 129 for CBL_GET_CURRENT_DIR with nonzero flags, and -1 for a missing parameter
- CBL_GET_CURRENT_DIR writes the cwd into the field space-filled and double-quoted when it contains a space. Verified end-to-end against a fixed CALL sequence (create dir, create again -> 128, delete missing -> 128, delete dir, delete again -> 128, change to a bad dir -> 128, get cwd -> 0)

## Negative claims (9) — negative capability is the trust surface
- the localtime-dependent CBL_CHECK_FILE_EXIST date/time formatting
- CBL_OPEN_FILE/READ_FILE/WRITE_FILE/CLOSE_FILE/FLUSH_FILE handle-based I/O
- cob_sys_file_info
- the ACUCOBOL hyphen / filename-mapping path
- non-UTF-8 path bytes
- the exact errno->status mapping beyond 35/128
- concurrent filesystem races
- the underlying syscalls (declared OS boundary)
- lie prevented: the CBL_* routines all just succeed -- NO: each returns a specific status (128 on any failure, 35 when a copy source is missing, 129 for GET_CURRENT_DIR flags, -1 for a missing parameter), and GET_CURRENT_DIR double-quotes a path containing a space and fails 128 if it does not fit the field

## Damage if overclaimed
treating a failed CBL_* call as success (ignoring 128/35) loses the very error a batch job's RETURN-CODE check depends on

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
