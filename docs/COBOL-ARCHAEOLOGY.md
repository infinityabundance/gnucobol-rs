<!-- GENERATED from reports/archaeology/findings.json by `xtask archaeology generate` — DO NOT EDIT BY HAND. -->
# COBOL Archaeology Atlas

COBOL operational facts distilled **in our own words** from the recurring themes in the archive
research and verified against standard COBOL — each a specific rule, where it bites in `gnucobol-rs` /
the KOBOLD data layer, and a candidate court. No verbatim source text, no sources/links, no generic
tags; only verifiable substance.

- **40** distilled findings

## Findings by topic

| topic | findings | high-confidence |
|---|---:|---:|
| batch_control | 6 | 3 |
| dialect | 4 | 2 |
| dirty_numeric | 11 | 10 |
| file_status | 7 | 4 |
| report_writer | 7 | 5 |
| tape_batch | 5 | 4 |

## batch_control

| id | conf | finding (rule) | gnucobol-rs / KOBOLD touchpoint | candidate court |
|---|---|---|---|---|
| af-bat-01 | high | A master/transaction (balanced-line) update assumes both files are pre-sorted on the matching key and walks them in lockstep, applying transactions to the master record with the equal key. Correctness depends on that ordering -- a single mis-ordered key silently corrupts the merge. | fileio.c sequential merge discipline | KOBOLD.BATCH.MASTER-TRANSACTION.1 |
| af-bat-02 | high | A sequence check verifies each input key is not less than the previous one and aborts (or reroutes to an exception report) on a drop in order. It is the historical guard that catches an unsorted or partially-merged input before it produces a wrong master file. | frontend.rs control flow; fileio.c | KOBOLD.BATCH.SEQUENCE-CHECK.1 |
| af-bat-03 | high | Batch reconciliation uses three independent totals: a record COUNT (number of records), a control TOTAL (sum of a money/quantity field), and a HASH total (sum of a non-additive field such as an account number, meaningful only for equality). Out == in on all three is the proof a run dropped or duplicated nothing. | reconciliation evidence (KOBOLD) | KOBOLD.BATCH.CONTROL-TOTALS.1 |
| af-bat-04 | medium | Subtotals and grand totals are control-total roll-ups: a subtotal accumulates within a control group and resets on its break, a grand total accumulates across the whole run and is emitted at end-of-job. They must reconcile -- the grand total equals the sum of the subtotals. | reportio.c / reconciliation | KOBOLD.BATCH.TOTAL-ROLLUP.1 |
| af-bat-05 | medium | Balancing pairs every input record with exactly one disposition: applied, rejected, or carried forward; input count must equal applied + rejected + carried. Rejected records go to an exception report / audit trail so no record is silently lost -- the batch-era analogue of fail-closed. | reconciliation + exception handling | KOBOLD.BATCH.BALANCING.1 |
| af-bat-06 | medium | A control/header record at the front of a batch file carries the run date, an identifying batch number, and the expected record count; the processing program reads it first and validates the trailing count against it, refusing the file if they disagree. | fileio.c header/trailer validation | KOBOLD.BATCH.HEADER-CONTROL.1 |

## dialect

| id | conf | finding (rule) | gnucobol-rs / KOBOLD touchpoint | candidate court |
|---|---|---|---|---|
| af-dia-01 | high | The IBM S/360 USAGE layout -- DISPLAY (zoned), COMPUTATIONAL (big-endian binary), COMPUTATIONAL-3 (packed) -- is the de-facto on-disk byte layout that real-world COBOL data carries and that gnucobol-rs reproduces; it predates and underlies the ANSI standard's USAGE wording. | numeric.c / pic.rs usage layout | KOBOLD.DIALECT.S360-LAYOUT.1 |
| af-dia-02 | medium | Reserved-word sets differ by dialect/era: a data-name that is legal in one compiler may be a reserved word in another (e.g. vendor verbs, COUNT, EXAMINE). A faithful port must reproduce the admitted compiler's reserved list, not a superset, or it will reject programs the oracle accepts (and vice versa). | frontend.rs reserved-word handling | KOBOLD.DIALECT.RESERVED-WORDS.1 |
| af-dia-03 | medium | Many 'modern' COBOL features (figurative constants, PICTURE editing, report writer, packed/zoned usage) were specified in the CODASYL Journal of Development before ANSI standardization; their exact early semantics survive in production data and must be matched at the byte level, not approximated. | whole runtime (lineage) | KOBOLD.DIALECT.CODASYL-LINEAGE.1 |
| af-dia-04 | high | Manufacturer/non-standard extensions (vendor-specific verbs, GUI clauses, hardware-tied options) must fail CLOSED in a faithful interpreter -- raise a typed unsupported error rather than guess -- so a program using a vendor extension is never silently mis-run as if the extension did nothing. | frontend.rs fail-closed on unknown constructs | KOBOLD.DIALECT.NONSTANDARD-FAILCLOSED.1 |

## dirty_numeric

| id | conf | finding (rule) | gnucobol-rs / KOBOLD touchpoint | candidate court |
|---|---|---|---|---|
| af-num-01 | high | An assumed decimal point (PICTURE V) occupies no byte -- it is positional scale metadata only. A stored numeric field holds digits with no physical '.', so a decoder applies the implied scale from the PICTURE and never expects or emits a literal point for V. | pic.rs scale; numeric.c decode | KOBOLD.DIRTY-NUMERIC.ASSUMED-POINT.1 |
| af-num-02 | high | PIC P digits are positional scale only and occupy no stored byte; they shift the assumed decimal point left (leading P) or right (trailing P) beyond the stored digits. Reading/writing a P-scaled item must add or strip those implied zero positions, never store them. | pic.rs P-scaling; move.c | KOBOLD.DIRTY-NUMERIC.P-SCALING.1 |
| af-num-03 | high | Zoned (DISPLAY) decimal stores one digit per byte. The sign of a signed DISPLAY item is, by default, an overpunch on the zone half of the trailing digit byte (SIGN TRAILING combined); SIGN LEADING/TRAILING SEPARATE instead stores a distinct '+'/'-' byte adjacent to the digits. | numeric.c zoned decode/encode; SIGN clause | KOBOLD.DIRTY-NUMERIC.ZONED-SIGN.1 |
| af-num-04 | high | The trailing-overpunch sign encodes the units digit and sign in one byte: EBCDIC C0-C9 / { A-I are +0..+9, D0-D9 / } J-R are -0..-9; ASCII variants set the zone bits of '0'..'9'. A reader must distinguish an overpunched signed digit from a plain digit, or the magnitude/sign is wrong. | numeric.c overpunch table | KOBOLD.DIRTY-NUMERIC.OVERPUNCH.1 |
| af-num-05 | high | COMP-3 (packed decimal) stores two digits per byte with the sign in the LOW nibble of the LAST byte: 0xC = positive, 0xD = negative, 0xF = unsigned. An odd digit count leaves the high nibble of the first byte zero. Byte length is (digits/2)+1 rounded up. | numeric.c packed decode/encode | KOBOLD.DIRTY-NUMERIC.PACKED.1 |
| af-num-06 | medium | COMP / COMP-4 is native binary (2/4/8 bytes for up to 4/9/18 digits) stored big-endian under the admitted build profile; COMP-5 is the same width but native machine endianness. Decode must honour the field's declared digit count for truncation, not just the byte width. | numeric.c binary decode; build-profile endianness | KOBOLD.DIRTY-NUMERIC.BINARY.1 |
| af-num-07 | high | A data item satisfies the ALPHABETIC class condition only when every character is a letter A-Z or space. A numeric DISPLAY field whose trailing byte carries an overpunch sign therefore tests NOT ALPHABETIC and NOT cleanly NUMERIC -- a dirty sign nibble must be classified explicitly, never silently accepted. | numeric.c class conditions; INSPECT | KOBOLD.DIRTY-NUMERIC.CLASS-TEST.1 |
| af-num-08 | high | BLANK WHEN ZERO sets the entire receiving edited item to spaces when the source value is zero, independently of zero-suppression and floating-insertion editing. Editing (suppression / floating currency / check protection) resolves first; then the whole field is blanked iff the value is zero. | edited.rs edited PICTURE encode | KOBOLD.EDITED.BLANK-WHEN-ZERO.1 |
| af-num-09 | high | Zero suppression replaces leading non-significant zeros (and the commas among them) with spaces (Z) or asterisks (* / check protection), stopping at the first significant digit or the decimal point. Asterisk protection is used so a blank cheque amount cannot be altered. | edited.rs Z/* suppression | KOBOLD.EDITED.ZERO-SUPPRESS.1 |
| af-num-10 | high | A numeric-edited item is alphanumeric in storage and carries editing characters; it cannot participate in arithmetic. It must be de-edited (moved into an elementary numeric item, which strips insertion/editing symbols) first; using an edited field as an operand is an error, not a silent coercion. | arith.rs operand typing; move.c de-edit | KOBOLD.DIRTY-NUMERIC.NO-EDIT-IN-ARITH.1 |
| af-num-11 | high | Figurative-constant bounds differ by class: LOW-VALUE (0x00) / HIGH-VALUE (0xFF) are the sentinels for non-numeric items, ZERO for numeric. Moving HIGH-VALUE (or any non-digit fill) into a numeric DISPLAY item yields a dirty value a later NUMERIC class test must catch rather than treat as a magnitude. | move.c figurative moves; numeric class test | KOBOLD.DIRTY-NUMERIC.FIGURATIVE-FILL.1 |

## file_status

| id | conf | finding (rule) | gnucobol-rs / KOBOLD touchpoint | candidate court |
|---|---|---|---|---|
| af-file-06 | high | OPEN automatically checks (INPUT) or writes (OUTPUT) the standard beginning-of-file label and sets FILE STATUS; OPEN does NOT make a record available -- a READ is required before the first record can be processed. A missing/!valid file is reported via FILE STATUS, not a crash. | fileio.c OPEN label handling + first READ | KOBOLD.FILEIO.OPEN-LABEL.1 |
| af-file-07 | medium | LABEL RECORDS STANDARD validates header/trailer on OPEN/CLOSE and runs any USE ... AFTER STANDARD LABEL declarative for BOTH reel and file labels; LABEL RECORDS OMITTED skips that validation entirely. The choice changes which integrity checks the runtime performs. | fileio.c LABEL RECORDS; USE AFTER LABEL | KOBOLD.FILEIO.LABEL-MODE.1 |
| af-file-08 | high | FILE STATUS is a 2-character code read after every I/O: '00' success, '02' duplicate alternate key (op still succeeds), '10' end-of-file, '22' duplicate key on WRITE, '23' record not found, '24' boundary violation/key out of range, '35' file not found on OPEN. Programs branch on it; a runtime must set it faithfully. | fileio.c status codes | KOBOLD.FILEIO.STATUS-CODES.1 |
| af-file-09 | high | INVALID KEY (and its status 22/23/24) fires on a keyed (random) RELATIVE/INDEXED operation that fails -- duplicate on WRITE, not-found on READ/REWRITE/DELETE/START, or key outside the file's bounds. Under ACCESS SEQUENTIAL a plain READ is sequential, so its end is EOF ('10'), not 'not found'. | fileio.c INVALID KEY vs AT END | KOBOLD.FILEIO.INVALID-KEY.1 |
| af-file-10 | high | REWRITE replaces the record most recently READ on an I-O file (its key must match the read record for keyed files); DELETE removes a record by key on an I-O file. Both require the file open I-O and a successful prior positioning, or they fail with an invalid-key/logic status. | fileio.c REWRITE/DELETE | KOBOLD.FILEIO.REWRITE-DELETE.1 |
| af-file-11 | medium | ALTERNATE RECORD KEY WITH DUPLICATES permits several records to share an alternate-key value; a WRITE/REWRITE creating such a duplicate returns status '02' (success-with-duplicate), and READ NEXT on that alternate key returns the duplicates in insertion order. Without WITH DUPLICATES the same act is a '22' error. | fileio.c alternate keys / duplicates | KOBOLD.FILEIO.ALT-KEY-DUP.1 |
| af-file-12 | medium | SELECT OPTIONAL marks a file that may be absent at run time: OPEN INPUT of a missing OPTIONAL file succeeds with status '05' and the first READ returns AT END immediately, rather than the '35' (file-not-found) a non-optional file would give. Programs rely on this to treat 'no file' as 'empty file'. | fileio.c OPTIONAL open semantics | KOBOLD.FILEIO.OPTIONAL.1 |

## report_writer

| id | conf | finding (rule) | gnucobol-rs / KOBOLD touchpoint | candidate court |
|---|---|---|---|---|
| af-rpt-01 | high | An RD (report description) defines a report's page geometry -- PAGE LIMIT, HEADING line, FIRST DETAIL, LAST DETAIL, FOOTING -- and its CONTROLS list (the control-break hierarchy, declared major to minor). These bound where each report group may print and when a page breaks. | reportio.c RD parsing + page geometry | KOBOLD.REPORT.RD-GEOMETRY.1 |
| af-rpt-02 | high | A control break at a higher level forces a break at every lower level. On a break, CONTROL FOOTING groups print most-minor first; CONTROL HEADING groups for the new value print most-major first. CONTROL HEADING/FOOTING FINAL each occur once -- FINAL heading before the first detail, FINAL footing at TERMINATE. | reportio.c control-break ordering | KOBOLD.REPORT.CONTROL-BREAK.1 |
| af-rpt-03 | high | A SUM counter accumulates its named source operand(s) at each detail GENERATE and resets to zero when its associated control breaks; a higher-level SUM rolls up (adds) the totals of the controls below it rather than re-scanning the detail lines. | reportio.c SUM counters + reset on break | KOBOLD.REPORT.SUM-COUNTER.1 |
| af-rpt-04 | high | INITIATE clears the report's LINE/PAGE counters and SUM accumulators and must precede the first GENERATE. TERMINATE produces all pending CONTROL FOOTING groups (including FINAL) plus the report footing; it does not itself close the report file. | reportio.c INITIATE/TERMINATE lifecycle | KOBOLD.REPORT.INITIATE-TERMINATE.1 |
| af-rpt-05 | high | GENERATE detail-group produces that detail line and drives control-break processing for the cycle; GENERATE report-name (summary reporting) produces only the control totals with no detail. The two forms must not be mixed for the same report. | reportio.c GENERATE detail vs summary | KOBOLD.REPORT.GENERATE-MODE.1 |
| af-rpt-06 | medium | SUPPRESS PRINTING, issued inside a USE BEFORE REPORTING declarative, omits printing of the current report group for this cycle only (its SUM counters still update). It is the sanctioned way to conditionally hide a group without disturbing totals. | reportio.c USE BEFORE REPORTING + SUPPRESS | KOBOLD.REPORT.SUPPRESS.1 |
| af-rpt-07 | medium | NEXT GROUP (NEXT PAGE / PLUS n / line n) sets the vertical advance taken AFTER a report group prints, distinct from the group's own LINE clauses; it is how spacing between groups and forced page ejects are controlled declaratively. | reportio.c NEXT GROUP spacing | KOBOLD.REPORT.NEXT-GROUP.1 |

## tape_batch

| id | conf | finding (rule) | gnucobol-rs / KOBOLD touchpoint | candidate court |
|---|---|---|---|---|
| af-file-01 | high | BLOCK CONTAINS groups N logical records into one physical block; the runtime blocks on WRITE and deblocks on READ. Blocking changes only I/O packaging and efficiency, never the logical record's bytes -- a reader must not confuse block boundaries with record boundaries. | fileio.c blocking/deblocking | KOBOLD.FILEIO.BLOCKING.1 |
| af-file-02 | high | CLOSE WITH LOCK on a tape/reel file performs the standard close, rewinds, and locks the file so it cannot be reopened in the same run unit. CLOSE REEL/UNIT on a multi-reel file advances to the next volume instead of closing; on mass storage CLOSE UNIT behaves as CLOSE. | fileio.c CLOSE WITH LOCK / REEL / UNIT | KOBOLD.FILEIO.CLOSE-LOCK.1 |
| af-file-03 | high | A variable-length record-sequential file prefixes each record with its length so the reader can recover the record boundary; the FD's RECORD IS VARYING ... DEPENDING ON counter is set to that length on READ. Fixed-format readers must not assume a constant record stride for such files. | fileio.c / file_seq.rs varying records | KOBOLD.FILEIO.VARYING-RECORD.1 |
| af-file-04 | medium | Multi-reel/volume tape files carry volume serial numbers and reel sequence in their labels; the runtime requests the next volume at end-of-reel and validates the sequence. Volume switching is transparent to the program -- a READ simply continues across the reel boundary. | fileio.c multi-volume sequencing | KOBOLD.FILEIO.MULTI-VOLUME.1 |
| af-file-05 | high | MERGE combines two or more already-sorted input files into one ordered output on a named key; unlike SORT it assumes its inputs are pre-ordered and only interleaves them. Feeding an unsorted file to MERGE produces a wrong (still-interleaved) result, not an error. | fileio.c MERGE | KOBOLD.FILEIO.MERGE.1 |

