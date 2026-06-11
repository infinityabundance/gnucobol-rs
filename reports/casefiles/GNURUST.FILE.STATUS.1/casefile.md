<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILE.STATUS.1 (court-casefile)

**Verdict: PASS** · 7/7 pass, 0 fail · crate `gnucobol-rs` 0.7.28

- **Oracle:** cobc OPEN INPUT/READ NEXT/CLOSE (libcob/fileio.c)
- **Byte domain(s):** declared OPEN/READ/CLOSE condition -> observed FILE STATUS byte
- **Replay:** `bash lab/oracle/file_status_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- observed GnuCOBOL FILE STATUS bytes for a narrow set of declared file-operation fixtures under the gnucobol-3.2.0-default witness: 00 (OPEN INPUT / successful READ), 06 (LINE SEQUENTIAL line longer than the record, via GNURUST.FILE.SEQUENTIAL.1), 10 (READ at EOF), 35 (OPEN INPUT of a missing file), 42 (CLOSE of a file not open after a failed OPEN), 46 (READ NEXT past EOF / no valid next record), each bound to its triggering condition and verified by the sweep (7/0)
- 30 (host I/O error) and 39 (attribute conflict) are explicitly not_admitted (environment-weather / not reproducible on flat sequential files). OBSERVED court: the pure kernel does no I/O, so it does not produce open/close statuses

## Negative claims (8) — negative capability is the trust surface
- full file I/O parity
- indexed/relative/VSAM
- locking/sharing
- host I/O error (30) generalization
- attribute conflict (39)
- Procedure Division control flow
- business completeness of the status set
- lie prevented: 'byte-correct reads are enough' -- COBOL programs BRANCH on FILE STATUS; this records which status (35 missing, 46 past-EOF, 42 close-not-open, 10 EOF) arises from which OPEN/READ/CLOSE condition, and refuses host-weather (30) + non-reproducible (39) statuses

## Damage if overclaimed
assuming a status value that the host did not actually produce (e.g. generalizing 30, or guessing 39) sends a migration's error-handling branches down the wrong path

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
