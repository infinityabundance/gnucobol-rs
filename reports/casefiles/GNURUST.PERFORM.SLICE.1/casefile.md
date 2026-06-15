<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.PERFORM.SLICE.1 (court-casefile)

**Verdict: PASS** · 10/10 pass, 0 fail · crate `gnucobol-rs` 0.7.81

- **Oracle:** cobc PERFORM (cobc/typeck.c + codegen.c, libcob)
- **Byte domain(s):** execute PERFORM loop over numeric counters -> resulting storage bytes
- **Replay:** `bash lab/oracle/perform_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- the second execution slice: a narrow interpreter EXECUTES a PERFORM loop over unsigned 9(n) counter fields and produces the same resulting STORAGE BYTES as cobc/libcob (verified 10/0). PERFORM n TIMES runs the body n times (n<=0 -> none)
- PERFORM UNTIL tests BEFORE each iteration (a satisfied condition runs the body zero times)
- PERFORM VARYING v FROM a BY b UNTIL sets v=a then test-before, running the body and adding b to v, so the control variable ends ONE STEP PAST the limit when the loop ran (FROM 2 BY 3 UNTIL I>10 ends I=11) or stays at a if never entered. Composes ADD + numeric comparison under the witnessed PROCEDURE.FLOW.ATLAS.1 loop semantics

## Negative claims (9) — negative capability is the trust surface
- signed/packed/binary counters
- numeric SIZE ERROR on the body
- non-ADD body statements
- compound/class conditions
- PERFORM THRU / out-of-line paragraph
- WITH TEST AFTER
- nested PERFORM/GO TO
- all dialects
- lie prevented: PERFORM VARYING stops AT the limit -- NO: it tests BEFORE and the control variable ends ONE STEP PAST the limit (FROM 1 BY 1 UNTIL I>4 leaves I=5, not 4), and a TEST-BEFORE loop whose condition is already true runs the body ZERO times; off-by-one here corrupts every loop-bounded computation

## Damage if overclaimed
assuming the control variable stops at the limit (or that the body always runs at least once) is an off-by-one in iteration counts and post-loop variable values

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
