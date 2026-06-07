# Negative claims (machine-readable)

Every non-claim `gnucobol-rs` currently makes, in one place. `claimed: false` means: not
proven, not implied, and not to be inferred from any passing test in a lower court.

```yaml
schema: gnucobol-rs-negative-claims-v1
oracle: GnuCOBOL 3.2.0 (see reports/admission/RECEIPT-ADMISSION.md)
as_of_milestone: GNURUST.2
claimed_now:
  - id: storage-parity-decimal
    what: byte layout of COMP-3 / zoned / display numeric fields
    scope: admitted PICs, LC_ALL=C.UTF-8, little-endian ASCII host, recorded config identity
  - id: move-parity-decimal
    what: bytes after MOVE between COMP-3 / zoned / display
    scope: same as above
  - id: pic-field-model
    what: PIC clause + USAGE -> {type, digits, scale, flags, size}, matching cobc's field attrs
    scope: sealed subset 9/X/A/S/V, repeats, SIGN clause, USAGE DISPLAY/COMP-3 (GNURUST.3)
  - id: record-layout
    what: DATA DIVISION item byte offsets / group sizes, matching cobc's record layout
    scope: nested groups, fixed OCCURS, REDEFINES (<= target), FILLER (GNURUST.4)
  - id: copy-expansion
    what: COPY <name>. copybook splice matching the cobc -P preprocessor (text-word stream)
    scope: line-oriented COPY, nested, cycle-detected, provenance-mapped (GNURUST.5)
not_claimed:
  - { id: arithmetic,            note: "ADD/SUBTRACT/MULTIPLY/DIVIDE/ROUNDED/ON SIZE ERROR — deferred (GMP-backed)" }
  - { id: edited-pictures,       note: "Z/$/CR/DB/*/BLANK WHEN ZERO rendering (GNURUST.EDITED.0)" }
  - { id: display-stdout,        note: "what DISPLAY prints — distinct from storage bytes" }
  - { id: comparison,            note: "IF/SEARCH ALL ordering; COLLATING SEQUENCE (GNURUST.COLLATE.0)" }
  - { id: pic-scaling-p,         note: "PIC 'P' scaling symbol — asymmetric leading/trailing rules (GNURUST.PIC-SCALING-P.0)" }
  - { id: pic-edited,            note: "edited PIC (Z/$/CR/DB/*/.) — fails closed (GNURUST.EDITED.0)" }
  - { id: value-init,            note: "VALUE initial memory image (GNURUST.VALUE.0)" }
  - { id: figurative-constants,  note: "ZERO/SPACE/HIGH-/LOW-VALUE/QUOTE/ALL bytes (GNURUST.FIGCONST.0)" }
  - { id: move-corresponding,    note: "group name-matched moves (GNURUST.CORR.0)" }
  - { id: occurs-depending-on,   note: "logical vs physical array length (GNURUST.ODO.0)" }
  - { id: redefines-larger,      note: "REDEFINES larger than its target — fails closed (GNURUST.4 non-claim)" }
  - { id: synchronized-align,    note: "SYNCHRONIZED/alignment in record layout (GNURUST.BINARY.0)" }
  - { id: redefines-variant,     note: "active overlay discriminator (GNURUST.REDEFINES.VARIANT.0)" }
  - { id: level-88,              note: "condition-name predicates (GNURUST.LEVEL88.0)" }
  - { id: binary-comp,           note: "COMP/BINARY size/byteorder/SYNC alignment (GNURUST.BINARY.0)" }
  - { id: source-format,         note: "fixed/free/variant parsing & preprocessing (GNURUST.SOURCEFORM.0)" }
  - { id: copy-replacing,        note: "COPY ... REPLACING whole-text-word algorithm (GNURUST.6/REPLACEALG.0)" }
  - { id: copybook-advanced,     note: "inline/multi-line COPY, OF/IN library, SUPPRESS (GNURUST.COPYMAP.0)" }
  - { id: call-cancel,           note: "CALL/linkage/CANCEL lifecycle (GNURUST.CALLABI.0)" }
  - { id: storage-lifetime,      note: "WORKING/LOCAL/LINKAGE/FILE/SCREEN/REPORT lifetimes" }
  - { id: accept-display,        note: "console/DATE/TIME/ENVIRONMENT/CRT (GNURUST.ACCEPTDISPLAY.0)" }
  - { id: screen-section,        note: "Screen Section (GNURUST.SCREEN.0)" }
  - { id: report-writer,         note: "Report Writer / LINAGE (GNURUST.REPORT.0)" }
  - { id: files-sequential,      note: "sequential vs line-sequential records (GNURUST.FILESEQ.0)" }
  - { id: files-indexed-sort,    note: "indexed files & SORT/MERGE backends (GNURUST.BACKEND.0)" }
  - { id: assign-resolution,     note: "ASSIGN file-name resolution (GNURUST.ASSIGN.0)" }
  - { id: ebcdic,                note: "EBCDIC code pages — a declared parameter, never inferred (GNURUST.EBCDIC.0)" }
  - { id: ebcdic-host-sign,      note: "EBCDIC-machine zoned sign processing (COB_EBCDIC_MACHINE)" }
  - { id: decimal-point-comma,   note: "DECIMAL-POINT IS COMMA" }
  - { id: diagnostics,           note: "compiler message wording/class/phase (GNURUST.DIAGPHASE.0)" }
  - { id: compiler-replacement,  note: "emitting native code / being a COBOL compiler" }
  - { id: non-linux-platforms,   note: "Windows/macOS/BSD (GNURUST.PLATFORM.0)" }
  - { id: dialect-witnesses,     note: "IBM/Micro Focus/ACU/RM parity — GnuCOBOL is the only authority" }
```
