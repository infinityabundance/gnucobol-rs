<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.IF.NUMERIC.SLICE.1 (court-casefile)

**Verdict: PASS** · 7/7 pass, 0 fail · crate `gnucobol-rs` 0.7.49

- **Oracle:** cobc numeric IF/EVALUATE + MOVE (cobc/typeck.c + codegen.c, libcob)
- **Byte domain(s):** execute numeric IF/EVALUATE over 9(n) fields -> resulting storage bytes
- **Replay:** `bash lab/oracle/if_numeric_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- the numeric counterpart of GNURUST.IF.EVALUATE.SLICE.1 (which is alphanumeric): a narrow interpreter EXECUTES an IF/EVALUATE over unsigned 9(n) numeric fields and produces the same resulting STORAGE BYTES as cobc/libcob (verified 7/0). The condition compares the field's decoded VALUE to a literal (= NOT= > < >= <=) -- so the comparison is numeric (N=50, N>100 false) not byte-wise
- EVALUATE matches a numeric subject against WHEN literals else WHEN OTHER
- branches MOVE a literal into the numeric field width. Completes the conditional-logic slice for BOTH data classes

## Negative claims (10) — negative capability is the trust surface
- signed/packed/V-scaled numerics
- MOVE field TO field
- numeric SIZE ERROR
- compound/class conditions
- 88-level
- range/THRU WHEN
- non-MOVE branches
- nested flow
- all dialects
- lie prevented: a numeric IF is the same as an alphanumeric IF -- NO: a numeric comparison compares VALUES (50 > 100 is false, 5 > 9 is false) while the alphanumeric slice compares BYTES (where the padded strings can order differently); using the wrong slice on a field takes the wrong branch

## Damage if overclaimed
comparing numeric amounts as bytes (or vice versa) takes the wrong IF/EVALUATE branch, corrupting derived flags and classifications

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.
