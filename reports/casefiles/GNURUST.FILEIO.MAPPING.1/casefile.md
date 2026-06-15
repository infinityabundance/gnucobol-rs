<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.MAPPING.1 (court-casefile)

**Verdict: PASS** · 2/2 pass, 0 fail · crate `gnucobol-rs` 0.7.79

- **Oracle:** cobc OPEN of files ASSIGNed names resolved via DD_*/COB_FILE_PATH env (libcob/fileio.c)
- **Byte domain(s):** a COBOL ASSIGN name + environment (DD_*/COB_FILE_PATH) -> the resolved filesystem path
- **Replay:** `bash lab/oracle/map_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- a faithful port of fileio.c's cob_chk_file_env + the simple-case path of cob_chk_file_mapping -- how an ORGANIZATION OPEN resolves a COBOL ASSIGN name to a filesystem path, proven to match the admitted libcob's resolution (map_sweep 2/0: the oracle creates each file at exactly the path the Rust mapping predicts): a bare name (no separator, not absolute) is looked up via the environment variables DD_<name>, dd_<name>, <name> in order (with '.' mangled to '_', or all non-alnum under COB_ENV_MANGLE
- surrounding quotes stripped), an absolute env value is used as-is, otherwise the name is prefixed by COB_FILE_PATH if set
- a name starting with '.', '-', or a digit is not env-mapped
- an ACU-hyphen name (-F/-D + space) is device-translated. Verified end-to-end: a DD_-mapped ASSIGN resolves to its env value, an unmapped name to COB_FILE_PATH/name

## Negative claims (7) — negative capability is the trust surface
- the complex multi-element path mapping (per-slash-element $ / DD_ resolution)
- COB_ENV_MANGLE non-default
- the ACU repeated-resolution recursion
- the dialect flag_filename_mapping=off case
- concatenated (multi-file) input names
- the actual getenv ordering beyond the three prefixes
- lie prevented: a COBOL ASSIGN name is the filename -- NO: a bare name is first resolved through DD_<name>/dd_<name>/<name> environment variables (with '.'->'_' mangling), an absolute env value replaces it, and otherwise COB_FILE_PATH is prefixed; a leading '.'/'-'/digit or a $/quote changes the rule

## Damage if overclaimed
treating the ASSIGN literal as the path ignores the DD_*/COB_FILE_PATH indirection every operational deployment relies on to place its files

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
