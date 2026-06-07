---
receipt_schema: gnurust-campaign-receipt-v1
campaign: GNURUST.15
byte_domain: raw EBCDIC field-storage bytes → decoded text (Unicode/Latin-1 string)
oracle: GnuCOBOL 3.2.0 libcob cob_load_collation (the function cobc itself uses to embed tables)
oracle_config: ebcdic500_ascii8bit.ttbl (shipped in the admitted oracle's share/gnucobol/config)
sweeps:
  ebcdic_sweep.sh: PASS=256 FAIL=0 (full byte-range table match)
fuzz: __fuzz_ebcdic (decode is total over arbitrary bytes; length-preserving)
sealed_version: gnucobol-rs 0.5.0
---

# RECEIPT-GNURUST-EBCDIC-15 — sealed: cp500 EBCDIC decode boundary for DISPLAY-class fields

**Campaign GNURUST.15.** Goal: read current legacy data from EBCDIC hosts without pretending all
bytes are text — an explicit, named code-page boundary for **alphanumeric DISPLAY** fields.

## Doctrine (the one sentence)

> GNURUST.15 admits EBCDIC only as an explicit code-page boundary for DISPLAY-class field decoding:
> text bytes are decoded under a named table, binary and packed fields remain raw storage domains,
> and no mixed-encoding, auto-detection, collation, national/DBCS, or business-meaning claim is made.

## Claim (exact)

`ebcdic::decode_display(CodePage::Cp500, bytes)` maps each raw EBCDIC byte to its ASCII/Latin-1
character via the **cp500** table, producing the text the admitted oracle would. The 256-entry table
(`CP500_TO_ASCII`) is byte-for-byte the one the admitted oracle's `cob_load_collation` produces for
its shipped `ebcdic500_ascii8bit.ttbl` — and because decode is a pure per-byte table application, the
full-range table match is a **complete** proof of the alphanumeric decode (every possible byte).

## Oracle-faithful code-page choice (important)

The admitted GnuCOBOL 3.2 oracle **ships cp500** (`ebcdic500_*` tables), loadable via the exported
`cob_load_collation` — the same function `cobc` uses to embed translation tables in generated code.
It does **not** ship cp037. Per the project doctrine (the oracle, never a spec, is the authority), a
code page is admitted only when its table comes from the oracle — so **cp500 is admitted and cp037 is
deferred** until a cp037 table is admitted into the oracle. (This is a deliberate, surfaced deviation
from the suggested starting page.)

## Non-claims (fail closed / out of scope)

Any other code page (`UnknownCodePage`; the enum is `#[non_exhaustive]`), **numeric EBCDIC zoned sign
processing** (the EBCDIC-machine sign mode — `0xC`/`0xD`/`0xF` zone nibbles — a separate court),
national/DBCS, collation/ordering, mixed or auto-detected encodings, and **binary/packed conversion**
(those bytes are raw storage and must pass through untouched — EBCDIC is **not** a record-wide
"convert everything"). Encoding (ASCII→EBCDIC) is not yet admitted (decode-first).

## Oracle

`ebcdic_harness.c` calls `cob_load_collation("ebcdic500_ascii8bit", ebc2asc, NULL)` and dumps the
256-byte table; `ebcdic_sweep.sh` compares it to the Rust `CP500_TO_ASCII` (via `examples/ebcdic_rows`).

## Evidence

| Check | Result |
|-------|--------|
| cp500 table vs `cob_load_collation` | **PASS=256 FAIL=0** (`ebcdic_sweep.sh`) — every byte |
| `cargo test` (anchors, text, totality/bijection) | 3 ebcdic tests green |
| Decode totality | `decode_display` is total over all 256 bytes, length-preserving, table is a bijection |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Versioning note

New public capability module `ebcdic` (+ `CodePage`/`EbcdicError`/`decode_display`/`translate_byte`) —
purely additive, so a **semver-minor** bump to **0.5.0**.
