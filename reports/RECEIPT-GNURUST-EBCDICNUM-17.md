---
receipt_schema: gnurust-campaign-receipt-v1
campaign: GNURUST.17
byte_domain: raw cp500 EBCDIC zoned-decimal field bytes → recovered numeric value
oracle: GnuCOBOL 3.2.0 cobc -fsign=EBCDIC (cob_get_sign_ebcdic), via an edited intermediary decoded by GNURUST.16
sweeps:
  ebcdic_num_sweep.sh: PASS=120 FAIL=0 (signed/unsigned, +/-/0, scaled)
sealed_version: gnucobol-rs 0.6.3
---

# RECEIPT-GNURUST-EBCDICNUM-17 — sealed: cp500 EBCDIC zoned-decimal numeric DISPLAY decode

**Campaign GNURUST.17.** Goal: close the largest remaining EBCDIC read-fidelity hole — raw mainframe
**zoned-decimal numeric DISPLAY** fields (the sign-bearing final digit), which `GNURUST.15` (text) and
`KOBOLD.DATA.3` correctly fail closed on.

## Doctrine (the one sentence)

> GNURUST.17 admits cp500 numeric DISPLAY only as an explicit zoned-decimal byte decoding court: EBCDIC
> digit and sign bytes are interpreted under the admitted oracle table, while binary/packed storage,
> edited pictures, mixed encodings, auto-detection, national/DBCS, and business truth remain outside
> the claim.

## Claim (exact)

`Decimal::from_ebcdic_zoned(data, attr)` decodes a raw cp500 EBCDIC zoned-decimal field to its value:
each byte is translated through the sealed **cp500** table (`GNURUST.15`), then the resulting
ASCII-overpunch form is decoded per GnuCOBOL's `cob_get_sign_ebcdic` — in the **final** byte
`'A'..'I'` → positive `1..9`, `'{'` → `+0`, `'J'..'R'` → negative `1..9`, `'}'` → `-0`, `'0'..'9'` →
unsigned; the scale is `attr.scale`. Equivalently, on the raw bytes: zone `0xC` = positive, `0xD` =
negative, `0xF` = unsigned; digit = low nibble.

This is a **composition of two sealed courts** — cp500 translate (`GNURUST.15`) and the ASCII-overpunch
sign decode — both grounded in the admitted oracle. (GnuCOBOL's source *comments* mislabel the C0/D0
zones; the **return values** and the compiled `#else` branch — `'{'` → positive — are authoritative.)

## Non-claims (fail closed / out of scope)

cp037 and other code pages, **edited numeric** under cp500 (that stays the `edited` court, ASCII-only),
binary/packed routed through the EBCDIC numeric decoder (raw-storage passthrough invariant untouched),
mixed/auto-detected encodings, national/DBCS, collation, and any change to the **ASCII** zoned-decimal
path (`from_display`) — all unchanged. Business truth is outside the claim.

## Oracle

`ebcdic_num_sweep.sh` builds one program compiled **`-fsign=EBCDIC`**: each case `MOVE`s the
cp500-translated overpunch bytes into a signed zoned field, then into an **edited** field, and
`DISPLAY`s the edited bytes. The Rust mirror decodes the **raw** EBCDIC via `from_ebcdic_zoned` and
checks it equals both the expected value **and** GnuCOBOL's own decode (the edited output, via the
sealed `decode_edited` — no DISPLAY-format ambiguity).

## Evidence

| Check | Result |
|-------|--------|
| cp500 zoned decode vs `cobc -fsign=EBCDIC` | **PASS=120 FAIL=0** (`ebcdic_num_sweep.sh`): S9/9, V-scale, +/−/0, unsigned |
| `cargo test` (`cp500_zoned_sign`: zones, −0, scale) | green |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Versioning note

New method `Decimal::from_ebcdic_zoned` (additive) — **semver-patch** to **0.6.3** (minor reserved for
breaking changes).
