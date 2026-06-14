<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.DECLARATIVES.ATLAS.1 (court-casefile)

**Verdict: PASS** · 5/5 pass, 0 fail · crate `gnucobol-rs` 0.7.60

- **Oracle:** cobc DECLARATIVES / USE AFTER STANDARD ERROR PROCEDURE (cobc/typeck.c + libcob/fileio.c error path)
- **Byte domain(s):** DECLARATIVES/USE runtime control: which op fires the handler, per-file binding, FILE STATUS visibility inside, resume-after-handler
- **Replay:** `bash lab/oracle/declaratives_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- observed the runtime behavior of a USE AFTER STANDARD ERROR PROCEDURE declarative on file I/O under the gnucobol-3.2.0-default witness, verified by the sweep (5/0): a FAILING file op invokes that file's USE declarative (OPEN of a missing file fires it with status 35, CLOSE of a not-open file re-fires it with status 42) while a SUCCESSFUL op invokes nothing
- the binding is PER-FILE (file F's errors run F's section, never another file's)
- FILE STATUS is VISIBLE inside the declarative (the per-op code)
- and after the declarative returns, execution RESUMES at the statement following the failed I/O (the program reaches its end, rc=0)

## Negative claims (8) — negative capability is the trust surface
- executing a declarative (gnucobol-rs runs NO Procedure Division / declaratives -- the L8 multi-statement summit)
- USE FOR DEBUGGING
- non-file exceptions (arithmetic/SIZE ERROR declaratives)
- GLOBAL declaratives across nested programs
- the precedence ordering when multiple declaratives could match
- whether non-file exceptions resume or terminate
- all dialects
- lie prevented: a file error aborts the COBOL program (or is silently ignored) -- NO: a failing I/O INVOKES the file's USE AFTER STANDARD ERROR declarative with the FILE STATUS visible inside it, and then execution RESUMES at the next statement (the program does NOT abort, rc=0); the handler is bound PER FILE and a successful op fires nothing -- so the error path is a structured, resumable, per-file branch, not a crash and not a no-op

## Damage if overclaimed
assuming a file error crashes (or is ignored) misreads the control flow of every batch program with declaratives; treating the atlas as an execution engine would run unsealed Procedure-Division control flow inside the handler

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
