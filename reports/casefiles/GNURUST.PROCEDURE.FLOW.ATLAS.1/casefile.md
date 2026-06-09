<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.PROCEDURE.FLOW.ATLAS.1 (court-casefile)

**Verdict: PASS** · 8/8 pass, 0 fail · crate `gnucobol-rs` 0.7.11

- **Oracle:** cobc Procedure Division control flow (cobc/typeck.c + codegen.c)
- **Byte domain(s):** control-flow statement class -> observed behavior
- **Replay:** `bash lab/oracle/procedure_flow_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- observed GnuCOBOL behavior of the control-flow statement classes under the gnucobol-3.2.0-default witness, each verified by the sweep (8/0): IF/ELSE (selects the branch), EVALUATE (first-match WHEN), PERFORM n TIMES (n iterations), PERFORM VARYING (body runs while the condition is false, the control variable ends ONE step past the limit -- I ends at 5 for UNTIL I>4), PERFORM UNTIL (test-before), PERFORM paragraph (out-of-line call + return), GO TO (unconditional jump skipping intervening statements). OBSERVED court: gnucobol-rs does NOT execute Procedure Division -- this MAPS the control-flow surface, it does not run programs

## Negative claims (8) — negative capability is the trust surface
- Procedure Division execution
- control-flow execution
- branch/path coverage
- loop termination analysis
- general condition evaluation
- that an observed statement is implemented
- all dialects
- lie prevented: gnucobol-rs ports COBOL -- NO: it does NOT execute Procedure Division; this atlas OBSERVES control-flow semantics (PERFORM VARYING leaves the control variable one step PAST the limit; GO TO skips intervening statements) but runs no program. Execution is the loudest non-claim of the whole project

## Damage if overclaimed
treating an observed control-flow fact as an execution engine would invite running real programs through a kernel that does no Procedure Division execution

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
