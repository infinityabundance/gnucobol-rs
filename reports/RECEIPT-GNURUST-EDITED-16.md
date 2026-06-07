---
receipt_schema: gnurust-campaign-receipt-v1
campaign: GNURUST.16
subset: 16a + 16b (decode-only)
byte_domain: edited DISPLAY field bytes → recovered value + presentation text
oracle: GnuCOBOL 3.2.0 cobc — MOVE numeric → edited field, DISPLAY the edited bytes
sweeps:
  edited_sweep.sh: PASS=92 FAIL=0 (16a Z 9 , . - + AND 16b $ * CR DB B 0 / edits)
fuzz: edited target 6_000_000 runs, 0 crashes
sealed_version: gnucobol-rs 0.6.1
---

# RECEIPT-GNURUST-EDITED-16 — sealed: edited-picture DECODE (16a + 16b subsets)

**Campaign GNURUST.16, subsets 16a + 16b (decode-only).** Goal: read display/report-shaped edited fields
back into a value + presentation text, without pretending to execute reports or format output.

## Doctrine (the one sentence)

> GNURUST.16 admits edited pictures only as an oracle-proven decode boundary for DISPLAY-shaped edited
> fields: it interprets admitted edited bytes and presentation markers without claiming report
> execution, numeric-to-edited formatting, locale/currency generality, EBCDIC zoned editing, or
> arithmetic semantics.

## Claim (exact)

For the admitted edited subset — **16a** (`Z 9 , . - +`) **and 16b** (the financial decorations `$`
currency, `*` check-protection, `CR`/`DB` trailing sign, `B` blank, `0` zero, `/` slash insertions),
with `(n)` repeats — `edited::decode_edited(pic, bytes)` recovers the field's
`numeric_value` and presentation `raw_text` from the bytes GnuCOBOL stores for an edited field. Proven
by moving values into the edited field with `cobc` (`MOVE numeric → edited`), capturing the displayed
bytes, and checking the decode recovers the moved-in value (92/92) with `edited_size` matching.

## Non-claims (fail closed / deferred)

This is **decode-only**. It does **not** produce edited output (`MOVE numeric → edited`, the formatting
direction — deferred to `16c`). Decode is **slot-based** (picture-position-aware), required so an inserted
literal `0` is recognised as a fixed char, not a value digit. Also not claimed: report-writer/printing semantics, locale/currency
policy, EBCDIC edited numeric, arithmetic over edited fields, edited `VALUE` images, and edited fields
under `COMP`/`COMP-3` or cp500. Corrupt/foreign bytes (`InvalidByte`), wrong width (`SizeMismatch`),
and out-of-subset symbols all **fail closed** — never a silent mis-read. Binary/packed fields are not
routed into the edited decoder.

## Oracle

`edited_sweep.sh` builds one COBOL program (`MOVE <value> TO <edited>` + bracketed `DISPLAY`),
compiles+runs it with the built `cobc`, and the Rust `edited_rows` mirror decodes the displayed bytes
and checks the recovered value (scale-normalized) + size.

## Evidence

| Check | Result |
|-------|--------|
| Edited decode vs `cobc` MOVE→edited→DISPLAY | **PASS=92 FAIL=0** (`edited_sweep.sh`): 16a + 16b (`$ * CR DB B 0 /`, floating currency, literal-`0` insertion) |
| `cargo test` (sizes, decode, fail-closed) | 4 edited tests green |
| Fuzz (edited target) | **6,000,000 runs, 0 crashes** |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Versioning note

`16b` adds no public API (the existing `decode_edited`/`edited_size` accept more symbols) — purely
additive behavior, so a **semver-patch** bump to **0.6.1** (minor reserved for breaking changes).
