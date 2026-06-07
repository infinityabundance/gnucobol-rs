# RECEIPT-GNURUST-ODO-10 — sealed: OCCURS DEPENDING ON physical-max layout

**Campaign GNURUST.10.** Goal: admit `OCCURS DEPENDING ON` into the layout court (`GNURUST.4`) as a
**physical maximum-layout fact only**.

## Doctrine (the one sentence)

> GNURUST.10 admits ODO only as a physical maximum-layout fact: bytes and offsets are proven, the
> active occurrence count and runtime record meaning remain non-claimed.

## Claim (exact)

For an `01` record whose **single, trailing** item is
`OCCURS min TO max TIMES DEPENDING ON <ctrl>` (elementary or a group), `gnucobol_rs::lay_out`
allocates the item its **physical maximum** span (`max` occurrences) and reports a record total
**byte-identical to GnuCOBOL's physical record allocation** — the generated-C storage
`b_REC[size]` — for DISPLAY and COMP-3 elements, varied bounds, and any fixed fields before the ODO.

The layout result carries, per ODO item:
```text
physical_size_bytes : max * element_size            (proven against cobc b_REC[size])
logical_size_policy : not_admitted                  (active count is NOT modelled)
depending_on        : { item, min, max }            (metadata, carried on layout::Odo)
```
so a consumer can never mistake the physical maximum for the active logical length.

## Non-claims (fail closed)

Explicitly **not** claimed, and rejected with a typed `LayoutError::OdoUnsupported` (or simply not
expressible): the **active/logical occurrence count**, runtime `DEPENDING ON` value validation,
serialization trimming, ODO **sliding**/non-sliding, `VALUE` under ODO, REDEFINES over an ODO item,
ODO combined with REDEFINES, **more than one ODO** in a record (incl. nested ODO), an ODO item that
is **not the last** in its group (GnuCOBOL itself rejects a field after it), and `max <= min` /
`max == 0` (GnuCOBOL: "OCCURS TO must be greater than OCCURS FROM"), and a `DEPENDING ON` item that
is not a field of the record.

## Oracle

`lab/oracle/odo_sweep.sh` builds one program per case and reads the record's **physical** storage
allocation `b_REC[size]` from `cobc -free -C` — deliberately *not* runtime `LENGTH OF`, which returns
the unclaimed **logical** length (it reflects the `DEPENDING ON` value, e.g. `0` at init). The Rust
`lay_out` total is compared field-for-field.

## Evidence

| Check | Result |
|-------|--------|
| ODO physical-max differential sweep vs `cobc b_REC[size]` | **records=30 PASS=30 FAIL=0** (`lab/oracle/odo_sweep.sh`): elementary & group ODO, DISPLAY/COMP-3 elements, bounds `0..5/1..4/0..10/2..3/3..7`, with/without pre-ODO fixed fields |
| GNURUST.4 fixed-layout regression sweep | **records=6 PASS=32 FAIL=0** (unchanged) |
| Self-contained `cargo test` | layout: 4/4 (ODO physical-max for elementary + group; not-last / bad-depending / multiple-ODO fail closed) |
| Fuzz (`layout`, now with ODO rules) | 4M+ runs, 0 crashes |
| `fmt` / `clippy -D warnings` / doc-gate (now runs the ODO sweep) | clean |

## Determinism

Pure function of `items`; same pinned oracle/env. The ODO element's offset/size compose from the
sealed fixed-`OCCURS` machinery (`GNURUST.4`); only the **count = max** and the new fail-closed rules
are added.
