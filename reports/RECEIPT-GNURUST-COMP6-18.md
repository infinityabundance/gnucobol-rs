---
receipt_schema: gnurust-campaign-receipt-v1
campaign: GNURUST.18
release_scope: GnuCOBOL 3.2 admitted oracle only
byte_domain: COMP-6 field-storage bytes + DISPLAY<->COMP-6 move-result bytes
oracle: GnuCOBOL 3.2.0 cobc -C attr + LENGTH OF (field model); libcob cob_move (MOVE)
sweeps:
  pic_sweep.sh: PASS=432 FAIL=0 (incl. 16 COMP-6 field-model cases)
  comp6_sweep.sh: PASS=98 FAIL=0 (DISPLAY<->COMP-6 both directions)
sealed_version: gnucobol-rs 0.7.0
---

# RECEIPT-GNURUST-COMP6-18 — sealed: COMP-6 unsigned packed-decimal storage + MOVE

**Campaign GNURUST.18.** Goal: an archaeology-driven read-fidelity court — COMP-6 is a real GnuCOBOL
3.2-era surface (the atlas found its runtime support first appeared in 3.2). Storage + MOVE only.

## Doctrine (the one sentence)

> GNURUST.18 admits COMP-6 only as GnuCOBOL 3.2 unsigned packed-decimal storage and MOVE byte
> semantics: storage size, raw bytes, and DISPLAY round-trip are oracle-proven, while signed forms,
> arithmetic, malformed bytes, dialect portability, and pre-3.2 behavior remain outside the claim.

## Claim (exact)

For `USAGE COMP-6`, `pic::build_field` produces `{type = PACKED (0x12), digits, scale,
flags = NO_SIGN_NIBBLE (0x0100), size = ceil(digits/2)}` byte-identical to `cobc` (proven, not
assumed: `cobc -C` emits `{0x12, 3, 0, 0x0100}` and `b_N[2]` for `9(3) COMP-6`). COMP-6 is **unsigned**
— two digits per byte, **no sign nibble**, odd digit counts pad a leading `0` nibble. `cob_move`
DISPLAY↔COMP-6 move-result bytes match libcob (COMP-6 is the PACKED + `NO_SIGN_NIBBLE` path);
`Decimal::from_packed` decodes it (the `no_sign_nibble` branch).

## Non-claims (fail closed / out of scope)

**Signed `S9(n)` COMP-6 is not claimed** — GnuCOBOL **warns and silently converts it to COMP-3**
(`'S' COMP-6 with sign - changing to COMP-3`), a different field; this court admits unsigned `9(n)`
only. Also not claimed: COMP-6 **arithmetic**, malformed packed bytes, **dialect portability** (strict
`cobol85`/`cobol2002` *reject* COMP-6 — atlas dialect-behavior finding), EBCDIC interaction, and
pre-3.2 behavior. `release_scope = GnuCOBOL 3.2 admitted oracle only`.

## Evidence

| Check | Result |
|-------|--------|
| COMP-6 field model vs `cobc` | **PASS=432 FAIL=0** (`pic_sweep.sh`; 16 COMP-6 cases over digit widths × scales) |
| DISPLAY↔COMP-6 MOVE vs `cob_move` | **PASS=98 FAIL=0** (`comp6_sweep.sh`, both directions) |
| `cargo test` (`comp6_field_model`) | green |

## Versioning note

`pic::Usage` gains a `Comp6` variant — breaking for exhaustive downstream `match`es — so a
**semver-minor** bump to **0.7.0**. `Usage` is now `#[non_exhaustive]`, so future usage variants are
additive (patch).
