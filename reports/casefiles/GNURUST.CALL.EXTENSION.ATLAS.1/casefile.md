<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.CALL.EXTENSION.ATLAS.1 (court-casefile)

**Verdict: PASS** · 5/5 pass, 0 fail · crate `gnucobol-rs` 0.7.41

- **Oracle:** cobc CALL/CANCEL + linkage (libcob/call.c) + C$ system routines
- **Byte domain(s):** CALL parameter passing (BY REFERENCE shares / BY CONTENT copies) + C$ extensions + CANCEL + ON EXCEPTION
- **Replay:** `bash lab/oracle/call_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- observed GnuCOBOL CALL / linkage / C$ extension behavior under the gnucobol-3.2.0-default witness, verified by the sweep (5/0) -- the #1 gap by frequency (CALL 959x in the admitted testsuite per GNURUST.PUBLIC.GAP.1): USING BY REFERENCE SHARES the caller's storage so callee mutation is visible in the caller (A 100 -> 101), USING BY CONTENT passes a COPY so the caller is UNCHANGED (B stays 100), a C$ extension (C$TOUPPER) mutates in place, an unresolved CALL fires ON EXCEPTION gracefully, and CANCEL unloads. OBSERVED court: gnucobol-rs does NOT execute subprogram CALLs (multi-module runtime is behavioral-ladder L8, NOT CLAIMED) -- this MAPS the surface, it runs no subprograms

## Negative claims (8) — negative capability is the trust surface
- subprogram execution
- dynamic linking / .so loading
- C$ extension implementation
- BY VALUE to a reference param (undefined)
- recursion/reentrancy
- CANCEL state machine
- all dialects
- lie prevented: CALL parameters all pass the same way -- NO: BY REFERENCE SHARES the caller's bytes (callee writes are visible) while BY CONTENT passes a COPY (caller unchanged), so the passing mode decides whether a subprogram can mutate the caller's storage; and gnucobol-rs EXECUTES no subprogram -- multi-module runtime is the summit (L8), not claimed

## Damage if overclaimed
assuming BY CONTENT shares storage (or BY REFERENCE copies) inverts who can mutate what across a CALL; treating the atlas as an execution engine would run unsealed inter-program control flow

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
