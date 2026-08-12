<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.IF.EVALUATE.SLICE.1 (court-casefile)

**Verdict: PASS** · 9/9 pass, 0 fail · crate `gnucobol-rs` 0.8.57

- **Oracle:** cobc IF/EVALUATE + MOVE (cobc/typeck.c + codegen.c, libcob/move.c)
- **Byte domain(s):** execute IF/EVALUATE fragment over alphanumeric fields -> resulting storage bytes
- **Replay:** `bash lab/oracle/if_eval_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- the FIRST execution slice: a narrow interpreter EXECUTES an IF/EVALUATE fragment over alphanumeric PIC X(n) fields and produces the same resulting STORAGE BYTES as cobc/libcob (verified 9/0). IF evaluates a single relation (= NOT= > < >= <=) using the COBOL alphanumeric comparison (pad the shorter operand with spaces, compare byte-by-byte in the ASCII collating sequence), selects the THEN/ELSE branch, and applies its MOVE statements (alphanumeric MOVE: left-justify, space-pad/truncate). EVALUATE selects the first WHEN whose literal equals the subject, else WHEN OTHER. Composes the sealed compare
- MOVE semantics under control flow -- it EXECUTES the fragment, where PROCEDURE.FLOW.ATLAS.1 only OBSERVED control flow

## Negative claims (9) — negative capability is the trust surface
- numeric/packed comparison and numeric MOVE
- compound conditions (AND/OR/NOT)
- class conditions (NUMERIC/ALPHABETIC)
- 88-level (GNURUST.11)
- non-MOVE branch statements
- nested IF/PERFORM/GO TO
- THRU/range WHEN
- all dialects
- lie prevented: a COBOL interpreter is all-or-nothing -- NO: this is a TIGHTLY BOUNDED execution slice (alphanumeric IF/EVALUATE with MOVE branches) that composes already-sealed compare+MOVE courts and produces oracle-identical storage; everything outside the slice (numeric conditions, AND/OR, nesting, non-MOVE branches) fails closed rather than being guessed

## Damage if overclaimed
treating this narrow slice as a general COBOL interpreter would run unsealed control flow whose semantics are not oracle-bound

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
