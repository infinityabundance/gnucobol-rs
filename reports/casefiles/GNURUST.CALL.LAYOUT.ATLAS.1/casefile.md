<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.CALL.LAYOUT.ATLAS.1 (court-casefile)

**Verdict: PASS** · 5/5 pass, 0 fail · crate `gnucobol-rs` 0.7.57

- **Oracle:** cobc CALL USING + LINKAGE parameter layout (libcob/call.c address passing)
- **Byte domain(s):** CALL USING parameter byte layout: BY REFERENCE address overlay (into adjacent storage), BY CONTENT sized copy, numeric length-mismatch leading-byte overlay
- **Replay:** `bash lab/oracle/call_layout_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- observed the BYTE-EXACT CALL USING parameter layout and length-mismatch behavior under the gnucobol-3.2.0-default witness, verified by the sweep (5/0), deepening GNURUST.CALL.EXTENSION.ATLAS.1: BY REFERENCE is a pure ADDRESS OVERLAY so a callee LINKAGE item LARGER than the caller's field reads PAST it into ADJACENT caller storage (X(5) over X(3) 'ABC' beside 'XYZ' -> the callee sees 'ABCXY', no truncation/padding/bounds) and the callee's write lands back in the caller's bytes ('ABC' -> 'ZBC')
- BY CONTENT passes a SIZED COPY leaving the caller UNCHANGED ('DEF')
- and a numeric LINKAGE 9(2) over a caller 9(4)=1234 overlays the LEADING display bytes positionally ('12')

## Negative claims (8) — negative capability is the trust surface
- subprogram execution (gnucobol-rs runs no subprograms -- multi-module runtime is L8)
- the value of a BY CONTENT over-read past the copy (undefined / uninitialized memory)
- BY VALUE byte layout
- OCCURS DEPENDING ON across the linkage boundary
- OPTIONAL / OMITTED parameters
- the RETURNING phrase
- all dialects
- lie prevented: a CALL parameter is length-checked and a mismatch is truncated or padded -- NO: BY REFERENCE is a raw ADDRESS OVERLAY with NO bounds, so a callee item longer than the caller's field silently reads (and a write corrupts) ADJACENT caller storage -- a mismatched-length copybook does not error, it reads the neighboring field; BY CONTENT instead copies (caller safe) and an over-read past that copy is UNDEFINED, not space-padded

## Damage if overclaimed
assuming CALL parameters are length-checked/truncated hides the real hazard -- a LINKAGE length that disagrees with the caller silently overlays adjacent storage (read and write); trusting a BY CONTENT over-read value reads uninitialized memory

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
