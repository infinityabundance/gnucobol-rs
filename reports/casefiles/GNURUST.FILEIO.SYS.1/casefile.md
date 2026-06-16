<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.SYS.1 (court-casefile)

**Verdict: PASS** · 2/2 pass, 0 fail · crate `gnucobol-rs` 0.7.83

- **Oracle:** cobc CALL CBL_* system routines, RETURN-CODE after each (libcob/fileio.c)
- **Byte domain(s):** CBL_DELETE_FILE/COPY_FILE/RENAME_FILE/CREATE_DIR/DELETE_DIR/CHANGE_DIR/GET_CURRENT_DIR -> RETURN-CODE (0/35/128/129/-1) + the GET_CURRENT_DIR field bytes
- **Replay:** `bash lab/oracle/cob_sys_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- a faithful port of fileio.c's cob_sys_* library routines a COBOL program CALLs, with the documented status codes, proven to match the admitted libcob's RETURN-CODE (cob_sys_sweep 2/0). (1) Directory/file ops -- cob_sys_delete_file/cob_sys_copy_file/cob_sys_rename_file/cob_sys_create_dir/cob_sys_delete_dir/cob_sys_change_dir/cob_sys_get_current_dir (+ the C$DELETE/C$COPY wrappers cob_sys_file_delete/cob_sys_copyfile): each does the syscall (via std::fs/std::env) and returns 0 success, 128 failure, 35 when a copy source is absent, 129 for CBL_GET_CURRENT_DIR with nonzero flags, -1 for a missing parameter
- CBL_GET_CURRENT_DIR writes the cwd space-filled and double-quoted when it contains a space
- verified against a fixed CALL sequence (create dir, create again 128, delete missing 128, delete dir, delete again 128, change to a bad dir 128, get cwd 0). (2) Handle-based byte-stream ops -- open_cbl_file/cob_sys_open_file/cob_sys_create_file/cob_sys_read_file/cob_sys_write_file/cob_sys_close_file/cob_sys_flush_file over an opaque 4-byte handle (a safe File-registry index, since forbid(unsafe_code) precludes a raw fd): access 1 read / 2 write+create+truncate / 3 r-w, positioned read/write (status 0/10/30/-1), a flags&0x80 size query
- verified by a CBL_CREATE_FILE/WRITE_FILE/READ_FILE round-trip that reads back the written bytes

## Negative claims (9) — negative capability is the trust surface
- the localtime-dependent CBL_CHECK_FILE_EXIST + cob_sys_file_info date/time formatting
- the file-lock / file_dev open parameters
- the ACUCOBOL hyphen / filename-mapping path
- non-UTF-8 path bytes
- the exact errno->status mapping beyond 35/128
- concurrent filesystem races
- the handle value matching libcob's raw fd (a safe registry index is used instead)
- the underlying syscalls (declared OS boundary)
- lie prevented: the CBL_* routines all just succeed -- NO: each returns a specific status (128 on any failure, 35 when a copy source is missing, 129 for GET_CURRENT_DIR flags, -1 for a missing parameter), and GET_CURRENT_DIR double-quotes a path containing a space and fails 128 if it does not fit the field

## Damage if overclaimed
treating a failed CBL_* call as success (ignoring 128/35) loses the very error a batch job's RETURN-CODE check depends on

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
