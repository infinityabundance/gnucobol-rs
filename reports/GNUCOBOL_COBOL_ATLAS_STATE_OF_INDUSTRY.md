# GnuCOBOL / COBOL atlas — state of the industry

A reading of the machine-readable atlas in [`archaeology/`](../archaeology/ATLAS.md). Every claim here
traces to an atlas file; the **only authoritative axis is the admitted GnuCOBOL 3.2 oracle** — standards
and vendor rows are cited shadow witnesses, never overrides.

> **Doctrine.** The COBOL Atlas separates formal standards, historical CODASYL specifications, government
> profiles, GnuCOBOL dialect modes, vendor compiler generations, platform runtimes, and preprocessor
> ecosystems. A surface is not "COBOL" in the abstract until the atlas says which axis admits it, whether
> it is syntax-only or runtime-functional, and which oracle or document family supports the claim.

## 1. Release lineage map
OpenCOBOL 1.0/1.1 → GNU Cobol 1.1 → GnuCOBOL 2.x (2.2 anchor) → 3.1/3.1.2 → **3.2 [ADMITTED]** → 4.x
(shadow). Only 3.2 is built here. See `atlases/A17-gnucobol-release-atlas/`.

## 2. Standards lineage map
COBOL-60/61/61-Ext/65 (historical) → COBOL-68 (first ANSI) → 74 → 85 (+1992 intrinsic, +1994 correction
amendments) → 2002 → 2014 → **2023 (current)**; plus X/Open and FIPS 21.x profiles. Labels + citations
only (no standard text). See `atlases/A18-cobol-standard-atlas/`.

## 3. Standard-vs-GnuCOBOL feature drift
The join (`atlases/A19-standard-release-join-atlas/`) tags each surface. Highlights:
`OCCURS DEPENDING` physical-vs-logical byte domain; `COMP-X` storage differs from `COMP`/`COMP-5`;
`SET..TO FALSE` standard-before-GnuCOBOL-court; EBCDIC is platform/table-specific (cp500 shipped, not
cp037).

## 4. GnuCOBOL feature maturity classes
`runtime_functional` (storage/MOVE/edited/EBCDIC) · `backend_dependent` (JSON/XML, indexed I/O, screen)
· `platform_dependent` (SYNC, code pages) · `reserved_word_only`/`parsed_inert` (much of the vendor
`-std` surface). The `-std` reserved-word deltas are oracle-generated (G-axis).

## 5. Backend-dependent features
`JSON/XML GENERATE` (cJSON/JSON-C, libxml2), indexed files (BDB/VBISAM), screen section (curses). Syntax
presence ≠ functional; a build without the lib changes the answer.

## 6. Parsed-inert compatibility ghosts
Vendor `-std` dialects add hundreds of reserved words (e.g. `mf-strict` +111 vs `default`) with no
implied runtime behavior. Recognition is not function.

## 7. Version scars
IBM documents syntax/semantic drift OS/VS → Enterprise v6 (same syntax, different execution). Micro Focus
behavior is a directive *family*, not a version. These are V-axis rows, deferred to shadow-witness work.

## 8. What gnucobol-rs has sealed
16 courts (see `reports/claim-ladder.json`): decimal MOVE, PIC/field-model (+P), layout (+ODO
physical-max), COPY/REPLACING, ADD/SUB/MUL (+packed), VALUE, LEVEL-88 (+SET TRUE), COMP/COMP-5/COMP-X
binary, cp500 EBCDIC decode, edited-picture decode (16a) — each oracle-proven with explicit non-claims.

## 9. What gnucobol-rs should avoid
Procedure-Division execution, files/indexed/SORT, report writer, screen, system routines, JSON/XML,
CALL/CANCEL, OO — all `out_of_model`. Touch only when they directly improve migration evidence.

## 10. Highest-priority hidden surfaces (atlas-ranked)
From `fixture-candidates.json`: EBCDIC numeric zoned sign, edited-picture financial decorations (16b),
`SET..TO FALSE`, `SYNCHRONIZED`, DIVIDE. See `gnucobol-rs-roadmap-from-atlas.md`.
