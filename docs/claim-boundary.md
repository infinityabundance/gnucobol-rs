# Claim boundary

The first law of `gnucobol-rs`: **every claim names its boundary, and every positive claim
is matched by an equally loud non-claim.** A compatibility surface that does not say what it
excludes is a lie of omission.

## What `gnucobol-rs` is

- A **compatibility court**: a set of memory-safe Rust ports of narrow GnuCOBOL semantics,
  each proven against a locally built GnuCOBOL 3.2 oracle.
- **Oracle-first**: "correct" = "byte/verdict-identical to the built `cobc`/`libcob`", never
  "matches our reading of the COBOL standard".
- **Receipt-bearing**: each sealed slice ships an admission receipt (pinned hashes, oracle
  identity, build command, env) and a parity receipt (`PASS=n FAIL=0` + classified rows).

## What `gnucobol-rs` is NOT (yet, or by design)

- **Not a GnuCOBOL replacement.** It reproduces isolated semantics, not the product.
- **Not a COBOL compiler.** No parsing of full programs into native code is claimed.
- **Not a `libcob` replacement.** A handful of runtime primitives are ported, not the runtime.
- **Not decimal arithmetic.** `ADD`/`SUBTRACT`/`MULTIPLY`/`DIVIDE`/`ROUNDED` are GMP-backed
  in upstream and are explicitly deferred to a future, separately sealed campaign.
- **Not a "better" or ergonomic COBOL library.** A divergence from the oracle that happens to
  be "nicer" is a **bug**, not an improvement.
- **Not a definition of COBOL truth.** The ISO/ANSI standard and IBM/Micro Focus behavior are
  not authorities here; the built GnuCOBOL oracle is.
- **Not a diagnostics match.** Compiler message wording is not reproduced.

## Sealed slices

**`GNURUST.2` — decimal byte semantics.** For admitted PICs, the Rust model reproduces GnuCOBOL's
exact bytes for packed-decimal (COMP-3 / PACKED-DECIMAL), zoned-with-sign, and display numeric
fields, and the `MOVE` conversions between them, under `LC_ALL=C.UTF-8` on a little-endian ASCII
host. (`reports/RECEIPT-GNURUST-DECIMAL-1.md`.)

**`GNURUST.3` — PIC → field model.** `pic::build_field` parses the sealed PIC subset
(`9 X A S V`, repeats, `SIGN` clause, `USAGE DISPLAY`/`COMP-3`) into the same
`{type, digits, scale, flags, size}` the compiler emits. (`reports/RECEIPT-GNURUST-PIC-3.md`.)

**`GNURUST.4` — record layout.** `layout::lay_out` assigns each DATA DIVISION item its byte offset
and size within an `01` record (nested groups, fixed `OCCURS`, `REDEFINES` overlay, `FILLER`),
matching the compiler's record layout. (`reports/RECEIPT-GNURUST-LAYOUT-4.md`.)

**`GNURUST.5` — COPY expansion.** `copybook::expand` splices `COPY <name>.` copybooks (recursively,
cycle-detected, provenance-mapped) matching the GnuCOBOL preprocessor (`cobc -P`) at text-word
granularity. (`reports/RECEIPT-GNURUST-COPY-5.md`.)

**`GNURUST.6` — COPY ... REPLACING.** `copybook::expand` applies `COPY name REPLACING ==p== BY ==q==`
at whole-text-word granularity, composing across nesting, matching `cobc -P`. Non-pseudo-text forms
(`LEADING`/`TRAILING`, identifier operands) fail closed. (`reports/RECEIPT-GNURUST-REPLACING-6.md`.)

**`GNURUST.7` — decimal arithmetic.** `cob_arith` computes ADD/SUBTRACT (DISPLAY) and MULTIPLY (DISPLAY/COMP-3) with truncation/ROUNDED in pure-Rust integer decimal, matching libcob `cob_add`/`cob_mul`. ADD/SUBTRACT into PACKED (cob_add_bcd), DIVIDE, other rounding modes, ON SIZE ERROR, and >38-digit inputs fail closed. (`reports/RECEIPT-GNURUST-ARITH-7.md`.)

**`GNURUST.8` — VALUE initial image.** `value_image` computes an `01` record's initial WORKING-STORAGE bytes from `VALUE` clauses (alphanumeric left-justified/space-padded, numeric DISPLAY zoned+overpunch, COMP-3 packed; unvalued DISPLAY→`'0'`, alnum→spaces, COMP-3→packed zero), matching `cobc`. OCCURS/REDEFINES+VALUE, edited/`P` PICs, non-fitting literals fail closed. (`reports/RECEIPT-GNURUST-VALUE-8.md`.)

**`GNURUST.9` — PIC `P` scaling.** `build_field` admits `P`: trailing `digits=9s+P, scale=-P`; leading `digits=9s, scale=9s+P`; `size` = stored `9`s — matching `cobc`. `V`+`P`/both-ends/P-only and VALUE/MOVE on a P field fail closed. (`reports/RECEIPT-GNURUST-PIC-SCALING-9.md`.)

**`GNURUST.10` — ODO physical-max layout.** `lay_out` admits a single trailing `OCCURS min TO max DEPENDING ON` as its **physical maximum** (max occurrences), matching `cobc`'s `b_REC[size]` allocation. The active/logical count, sliding, and runtime meaning are **non-claims**; multiple/nested ODO, ODO-not-last, REDEFINES+ODO, and `max<=min` fail closed. (`reports/RECEIPT-GNURUST-ODO-10.md`.)

**`GNURUST.11` — LEVEL-88 predicate.** `eval_88` proves whether a condition name is true for a parent field's current bytes — alphanumeric (space-padded compare, incl. ranges) and numeric DISPLAY/COMP-3 (value compare, inclusive ranges) — matching `cobc`. `SET`/`FALSE`/expressions/Procedure-Division execution and collating-sensitive ranges are non-claims. (`reports/RECEIPT-GNURUST-COND-11.md`.)

**`GNURUST.12` — SET LEVEL-88 TO TRUE.** `set_88_true` constructs the canonical parent bytes for `SET condition-name TO TRUE` (first VALUE / range lower bound, encoded), matching `cobc`; its output satisfies `eval_88`. `SET TO FALSE`/the FALSE clause/expressions/execution are non-claims. (`reports/RECEIPT-GNURUST-SET88-12.md`.)

**`GNURUST.13` — packed ADD/SUBTRACT.** `cob_arith` seals ADD/SUBTRACT into a PACKED receiver (libcob's `cob_add_bcd` path), matching the receiving-field **bytes** for DISPLAY/COMP-3 operands, scales, truncate/ROUNDED, carry, overflow, and negative-zero-on-truncation. DIVIDE / SIZE ERROR / other rounding modes / bignum are non-claims. (`reports/RECEIPT-GNURUST-ADDBCD-13.md`.)

**`GNURUST.14` — binary storage + MOVE.** `build_field` admits COMP/BINARY/COMP-5/COMP-X (type/digits/scale/flags/size vs `cobc`), and `cob_move` handles DISPLAY↔binary (endian, truncate/mask, two's-complement), with `Decimal::from_binary` for decode. Binary arithmetic, SYNC, host-portable endian are non-claims. (`reports/RECEIPT-GNURUST-BINARY-14.md`.)

**`GNURUST.15` — EBCDIC code-page boundary.** `ebcdic::decode_display` decodes raw EBCDIC alphanumeric DISPLAY bytes to text under the admitted **cp500** table (byte-for-byte the oracle's `cob_load_collation` output, 256/256). cp037, numeric EBCDIC zoned sign, national/DBCS, and binary/packed conversion are non-claims. (`reports/RECEIPT-GNURUST-EBCDIC-15.md`.)

**`GNURUST.16` — edited-picture decode (16a).** `edited::decode_edited` recovers an edited DISPLAY field's value + presentation text for the `Z 9 , . - +` subset (proven vs `cobc` MOVE→edited→DISPLAY, 50/50). Numeric→edited formatting, `$ * CR DB B 0 /` (16b), reports, locale, EBCDIC edited, and edited arithmetic are non-claims. (`reports/RECEIPT-GNURUST-EDITED-16.md`.)

**`GNURUST.17` — cp500 EBCDIC zoned numeric.** `Decimal::from_ebcdic_zoned` decodes raw cp500 zoned-decimal bytes (cp500 translate + cob_get_sign_ebcdic sign), proven vs `cobc -fsign=EBCDIC` (120/0). cp037, edited-numeric under cp500, and binary/packed via this path are non-claims. (`reports/RECEIPT-GNURUST-EBCDICNUM-17.md`.)

**`GNURUST.18` — COMP-6.** `Usage::Comp6` admits unsigned packed-decimal (PACKED+NO_SIGN_NIBBLE, size ceil(n/2)) storage + DISPLAY↔COMP-6 MOVE, proven vs cobc/cob_move (432/0, 98/0). Signed COMP-6 (→COMP-3), arithmetic, malformed bytes, dialect portability are non-claims. (`reports/RECEIPT-GNURUST-COMP6-18.md`.)

**Non-claims:** no arithmetic; no edited pictures (`PIC $,9.99`); no `P` scaling; no binary
(`COMP`/`COMP-5`)/float/`COMP-6` parity beyond what a receipt records; no EBCDIC-host sign mode
(the ASCII overpunch path is the sealed one); no `OCCURS DEPENDING ON`/`SYNCHRONIZED`; no
`REDEFINES` larger than its target; no file I/O. See `reports/negative-claims.md`.

## Determinism: pinned or classified

Every confounder is either pinned to a reproducible value or classified out of the claim:

| Confounder | Disposition |
|------------|-------------|
| Locale | pinned `LC_ALL=C.UTF-8` |
| Host endianness / charset | classified: little-endian ASCII host (`COB_EBCDIC_MACHINE` off) |
| GnuCOBOL build flags | recorded in the admission receipt as "which upstream" |
| `cobc`/`libcob` version | pinned 3.2.0; recorded with sha256 |

No silent third state: a surface is sealed, pinned, or classified — never blurred.
