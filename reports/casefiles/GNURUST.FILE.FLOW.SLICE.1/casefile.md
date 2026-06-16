<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILE.FLOW.SLICE.1 (court-casefile)

**Verdict: PASS** · 2/2 pass, 0 fail · crate `gnucobol-rs` 0.7.84

- **Oracle:** cobc file read-loop (libcob/fileio.c + cobc control flow)
- **Byte domain(s):** OPEN INPUT + PERFORM UNTIL EOF READ + accumulate -> resulting WORKING-STORAGE
- **Replay:** `bash lab/oracle/file_flow_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the read-loop where file I/O meets control flow -- the shape of essentially every COBOL batch program -- executed to oracle-identical WORKING-STORAGE (verified 2/2: record count + control total). The slice runs the canonical OPEN INPUT / priming READ / PERFORM UNTIL EOF (ADD 1 TO count, ADD record-field TO sum, READ at bottom) / CLOSE, composing the sealed sequential READ (GNURUST.FILE.SEQUENTIAL.1) with looped numeric accumulation: each record is processed EXACTLY ONCE. Completes the spine from bytes (decimal MOVE) to a running program (read-process-accumulate)

## Negative claims (10) — negative capability is the trust surface
- indexed/relative organizations
- signed/packed accumulators
- numeric SIZE ERROR on accumulators
- per-record IF/MOVE/general statements
- WRITE/REWRITE in the loop
- multi-file loops
- file-status beyond AT END
- READ INTO
- all dialects
- lie prevented: a read-loop is just a for-each -- NO: it is a PRIMING-READ plus READ-AT-BOTTOM controlled by a TEST-BEFORE PERFORM UNTIL the AT-END switch, so each record is processed EXACTLY ONCE and an empty file processes ZERO records; a mis-structured loop double-processes the first/last record or runs the body on AT END

## Damage if overclaimed
treating this narrow read-accumulate slice as a general batch engine would run unsealed per-record logic / writes / multi-file joins whose semantics are not oracle-bound

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
