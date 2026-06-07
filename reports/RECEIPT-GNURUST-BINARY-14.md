---
receipt_schema: gnurust-campaign-receipt-v1
campaign: GNURUST.14
byte_domain: field-storage bytes (binary) + move-result bytes
oracle: GnuCOBOL 3.2.0 cobc -C attr witness + LENGTH OF (field model); libcob cob_move (binary MOVE)
oracle_config: binary-size 1-2-4-8, binary-byteorder big-endian, binary-truncate yes
sweeps:
  pic_sweep.sh: PASS=416 FAIL=0 (incl. 128 binary cases)
  binary_sweep.sh: PASS=546 FAIL=0 (DISPLAY<->COMP/COMP-5/COMP-X)
fuzz: cob_move target 10_000_000 runs, 0 crashes (binary on the hostile surface)
sealed_version: gnucobol-rs 0.4.0
---

# RECEIPT-GNURUST-BINARY-14 — sealed: COMP / BINARY / COMP-5 / COMP-X storage + MOVE

**Campaign GNURUST.14.** Goal: read fidelity for binary numeric usages — the most common remaining
hole for real fixed-record copybooks. **Storage and MOVE byte semantics only.**

## Doctrine (the one sentence)

> GNURUST.14 admits binary numeric usages only as storage and MOVE byte semantics for the admitted
> GnuCOBOL oracle row; it does not claim binary arithmetic, host-portable endian behavior,
> SYNCHRONIZED alignment, or expression semantics.

## Claim (exact)

For `USAGE COMP`/`BINARY`/`COMPUTATIONAL`, `COMP-5`, `COMP-X`, under the admitted oracle config
(`binary-size: 1-2-4-8`, `binary-byteorder: big-endian`, `binary-truncate: yes`):

- **Field model** (`pic::build_field`): `{type = COB_TYPE_NUMERIC_BINARY (0x11), digits, scale,
  flags, size}` byte-identical to `cobc`. Flags distinguish the family — **COMP** = `BINARY_SWAP |
  BINARY_TRUNC` (big-endian, truncated to the PIC digit range), **COMP-X** = `BINARY_SWAP`
  (big-endian, full range), **COMP-5** = `REAL_BINARY` (native byte order, full range), `+HAVE_SIGN`
  when signed. **Size:** COMP/COMP-5 use the `1-2-4-8` table (`1-2`→1, `3-4`→2, `5-9`→4, `10-18`→8,
  `19-38`→16); **COMP-X uses a tighter table** — the smallest `k` with `256^k ≥ 10^digits`
  (`9(6)`→3, `9(10)`→5) — diagnosed against `cobc`, distinct from `1-2-4-8`.
- **MOVE bytes** (`cob_move`): DISPLAY ↔ binary (and PACKED ↔ binary) move-result **bytes** match
  libcob — big-endian / native two's complement, COMP digit-truncation (`999`→`99`),
  COMP-X/COMP-5 byte-masking (`999`→`0xE7`), signed two's complement (`-2`→`0xFFFE`). Binary moves
  route through a DISPLAY temp and the sealed `GNURUST.2` display/packed moves.
- **Decode** (`Decimal::from_binary`): binary bytes → value, for the reconciliation read path.

## Non-claims (fail closed / out of scope)

Binary **arithmetic** (no `cob_add`/`cob_mul` over binary admitted), expression semantics,
**SYNCHRONIZED**/alignment, `COMP-6`, float (`COMP-1`/`COMP-2`), `COMP-5` on a big-endian host or any
**host-portable** endian claim (the claim is pinned to this little-endian oracle host), and malformed
binary beyond the sealed decode are **not** claimed. The non-claims are machine-readable in
`reports/negative-capabilities.md` / `claim-ladder.json`.

## Oracle

`pic_sweep.sh` (generated-C `cob_field_attr` + `LENGTH OF`) for the field model; `binary_sweep.sh`
(libcob `cob_move` via `decimal_harness`) for the MOVE bytes, both directions.

## Evidence

| Check | Result |
|-------|--------|
| PIC field-model sweep vs `cobc` | **PASS=416 FAIL=0** (`pic_sweep.sh`); 128 binary cases over the size boundaries × COMP/COMP-5/COMP-X × signed/unsigned × integer/V-scaled |
| Binary MOVE sweep vs `cob_move` | **PASS=546 FAIL=0** (`binary_sweep.sh`): DISPLAY↔binary both directions, endian, truncation/masking, sign |
| Self-contained `cargo test` | 32 tests incl. `pic::binary_field_model_matches_oracle` |
| Fuzz (`cob_move`, binary on the surface) | **10,000,000 runs, 0 crashes** |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Versioning note

`pic::Usage` gains `Comp`/`Comp5`/`CompX` variants — adding variants to a non-`#[non_exhaustive]`
public enum can break exhaustive `match`es, so this is a **semver-minor (breaking, pre-1.0)** bump to
**0.4.0**. Published companion crates pin `^0.3`, so they are unaffected.
