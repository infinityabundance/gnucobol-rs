<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FRONTEND.1 (court-casefile)

**Verdict: PASS** · 10/10 pass, 0 fail · crate `gnucobol-rs` 0.7.83

- **Oracle:** cobc -x compiling+running the same source (cobc front-end + libcob), stdout captured and diffed
- **Byte domain(s):** a COBOL program (sealed subset) -> the exact stdout bytes it writes, byte-identical to cobc
- **Replay:** `bash lab/oracle/cobol_frontend_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- a from-scratch (NOT cobc-derived) COBOL front-end -- lexer + parser + executor in src/frontend.rs -- PARSES a whole COBOL program and EXECUTES it on the ported libcob runtime (no cobc, no libcob linked), with stdout proven BYTE-IDENTICAL to the admitted cobc over a 10-program corpus (cobol_frontend_sweep 10/0) plus a 3-program hand-wired corpus (cobol_run_sweep 3/0). This is the first parse+execute proof: it composes the sealed runtime (pic::build_field for PIC->attr, move_ops::cob_move, arith::cob_arith + cob_divide, edited::encode_edited, termio::cob_display) under a real reader. The sealed subset: WORKING-STORAGE 01 elementary items with PIC
- VALUE (numeric 9/V/S/P, alphanumeric X/A, numeric-edited Z/*/$/+/-/,/./CR/DB)
- statements MOVE, ADD, SUBTRACT, MULTIPLY, DIVIDE (TO/FROM/BY/INTO/GIVING), DISPLAY, STOP RUN. Anything outside the subset returns a RunError (fail closed -- never a silent mis-run). Corpus covers ADD/TO, ADD/GIVING, SUBTRACT, MULTIPLY GIVING, DIVIDE GIVING with V99 scale, signed arithmetic + signed edited move, alphanumeric VALUE, decimal V99 with floating-$ editing, numeric->edited MOVE, and multi-statement/multi-operand DISPLAY.

## Negative claims (11) — negative capability is the trust surface
- this is an INTERPRETER over the runtime, NOT a native-code compiler (no codegen to machine code / no .o/.so emission)
- group items / OCCURS / REDEFINES / level numbers other than 01
- COMPUTE and arithmetic expressions
- control flow (IF / PERFORM / EVALUATE / GO TO -- those are the separate execution-slice courts)
- ACCEPT and file I/O
- ON SIZE ERROR / ROUNDED modes beyond truncate
- multiple programs / CALL
- the full PICTURE and statement grammar
- any verb or clause outside the listed subset
- all dialects and non-default runtime config
- lie prevented: 'gnucobol-rs is a COBOL compiler' is the lie this prevents being overstated -- what is PROVEN is parse+EXECUTE of a small, explicit subset to cobc-identical stdout (an interpreter over the sealed runtime), with everything outside the subset failing closed; native-code compilation of the full language is NOT claimed

## Damage if overclaimed
treating this as a full COBOL compiler would imply the whole language parses + runs + compiles to native code, when only the listed subset is interpreted to oracle-identical output

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
