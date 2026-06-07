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

## The first sealed slice — `gnucobol-rs`

**Claims:** for admitted PICs, the Rust model reproduces GnuCOBOL's exact bytes for
packed-decimal (COMP-3 / PACKED-DECIMAL), zoned-with-sign, and display numeric fields, and
the `MOVE` conversions between them, under `LC_ALL=C.UTF-8` on a little-endian ASCII host.

**Non-claims:** no arithmetic; no edited pictures (`PIC $,9.99`) beyond what is sealed; no
binary (`COMP`/`COMP-5`), float, or `COMP-6` parity beyond what a receipt explicitly records;
no EBCDIC-host sign mode (the ASCII overpunch path is the sealed one); no file I/O.

## Determinism: pinned or classified

Every confounder is either pinned to a reproducible value or classified out of the claim:

| Confounder | Disposition |
|------------|-------------|
| Locale | pinned `LC_ALL=C.UTF-8` |
| Host endianness / charset | classified: little-endian ASCII host (`COB_EBCDIC_MACHINE` off) |
| GnuCOBOL build flags | recorded in the admission receipt as "which upstream" |
| `cobc`/`libcob` version | pinned 3.2.0; recorded with sha256 |

No silent third state: a surface is sealed, pinned, or classified — never blurred.
