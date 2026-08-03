<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.MULTI-RECORD-FD.1 (court-casefile)

**Verdict: PASS** · 7/7 pass, 0 fail · crate `gnucobol-rs` 0.8.52

- **Oracle:** cobc multi-record FD WRITE/READ (libcob record area = union of the FD record descriptions; fileio.c cob_seq_write/cob_seq_read + the line-advancing write_opt family)
- **Byte domain(s):** WRITE/REWRITE of any FD record -> the NAMED record's bytes over one shared record area, byte-identical to cobc (stdout via read-back DISPLAY); line-control file bytes asserted oracle-side
- **Replay:** `bash lab/oracle/multirecord_fd_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (8)
- a front-end differential over SEVERAL alternative 01-level records beneath one FD (multirecord_fd_sweep 7/0): WRITE/REWRITE of ANY declared record resolves to its owning file and emits the NAMED record's bytes at that record's own length (the CCVS85 `WRITE DUMMY-RECORD AFTER ADVANCING` shape included)
- all FD records share ONE record area (GnuCOBOL union: MOVE into one record is visible through every other and WRITE emits the shared bytes -- verified against the oracle: `MOVE "11111" TO A
- WRITE B` writes "11111"), so READ fills the shared area and every record view sees it
- equal-length, different-length (variable) and GROUP records lay out independently and emit their own lengths
- records under different FDs never cross-associate
- the record-sequential READ leaves the record-area tail as-is (libcob cob_seq_read: 'we leave the data not read as-is'), matching the oracle's stale-tail behavior
- a WORKING-STORAGE 01 or unknown name still fails closed as 'not an FD record'
- the oracle's line-control file bytes for AFTER ADVANCING n (n x LF before the record + a final LF at close) are asserted oracle-side

## Negative claims (8) — negative capability is the trust surface
- read-back of printer-style (ADVANCING-written) files -- outside the in-memory logical-record model (the line-control bytes are asserted on the oracle's file, not the front-end's stdout)
- duplicate record names across files -- cobc rejects them as ambiguous ('needs qualification'), the front-end keeps first-declaration
- deeper plain sub-groups inside an alternative record -- the REDEFINES-group alias maps direct leaves only (documented limitation)
- multi-record key selection for INDEXED/RELATIVE beyond the primary record WRITE ... ADVANCING PAGE (form-feed page control) fails closed (a declared boundary, not implemented)
- WRITE ... ADVANCING on a RELATIVE/INDEXED file fails closed (advancing is valid only on SEQUENTIAL / LINE SEQUENTIAL)
- the in-memory READ-back of an ADVANCING-written (printer-style) file is outside the sealed subset -- the advancing LFs are file data (mirroring the oracle's disk bytes, which the --dump-files materialization reproduces byte-for-byte), so re-reading a print file is not modelled
- no suite-pass or conformance claim from the CCVS85 differential court -- observation only.
- lie prevented: that the FD records are independent buffers -- NO: they are alternative views of ONE record area (a MOVE into one is visible through every other, exactly as the oracle), and WRITE of a record emits the SHARED bytes at the NAMED record's length

## Damage if overclaimed
reporting multi-record FD support while WRITE still resolves only the first record would silently write the wrong record's bytes in the CCVS85 report-writer pattern

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
