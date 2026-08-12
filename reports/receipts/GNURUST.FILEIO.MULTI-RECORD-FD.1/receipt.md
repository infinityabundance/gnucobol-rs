<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.MULTI-RECORD-FD.1 — multiple 01-level record descriptions beneath one FD (shared record area)

**Verdict: PASS** · replay `PASS=7 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.MULTI-RECORD-FD.1` |
| court | multiple 01-level record descriptions beneath one FD (shared record area) |
| crate_version | `0.8.57` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | WRITE/REWRITE of ANY declared FD record -> the NAMED record's bytes over ONE shared record area (GnuCOBOL union), byte-identical to cobc; different-length + group records; the CCVS85 WRITE ... AFTER ADVANCING shape |
| replay command | `bash lab/oracle/multirecord_fd_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- read-back of printer-style (ADVANCING-written) files -- outside the in-memory logical-record model (the line-control bytes are asserted oracle-side)
- duplicate record names across files -- cobc rejects them as ambiguous (needs qualification); the front-end keeps first-declaration
- deeper plain sub-groups inside an alternative record -- the REDEFINES-group alias maps direct leaves only
- multi-record key selection for INDEXED/RELATIVE beyond the primary record

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
