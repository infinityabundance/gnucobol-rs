# Notable vendor-dialect reserved-word extensions (oracle-generated archaeology)

From `deltas/` (reproducible via `lab/atlas/build-dialect-axis.sh`): the *specific* reserved words each
`-std` vendor dialect adds beyond `default`. Presence is **recognition, not function** — but it maps the
vendor surface a migration is likely to hit.

## Cross-vendor legacy verbs (in mf ∧ ibm ∧ acu, NOT in default)

```
ENTER · EXAMINE · TERMINAL
```

These are exactly the *old* COBOL surfaces every vendor preserved for back-compat but GnuCOBOL's
`default` dialect drops: `EXAMINE` (the pre-COBOL-85 ancestor of `INSPECT`), `ENTER` (inter-language
linkage), `TERMINAL` (communications). If a legacy program uses them, it parses under `-std=ibm/mvs/mf`
but not `default` — a real, easily-missed migration scar. (`deltas/cross-vendor-not-default.txt`.)

## Per-dialect flavor (samples — full lists in `deltas/*.txt`)

- **mf** (+112): `BLOB`, `BLOB-FILE`, `B-EXOR`, `AUTO-HYPHEN-SKIP`, `ABSTRACT`, `ACQUIRE` — OO + DB +
  screen extensions.
- **ibm** (+46): `BASIS`, `CBL`, `COM-REG`, `DBCS`, `DEBUG-CONTENTS`, `CLOCK-UNITS`, `BEGINNING` —
  mainframe preprocessor + DBCS + debug surfaces.
- **acu** (+96): `ASSEMBLY-NAME`, `BITMAP-*`, `AX-EVENT-LIST` — Windows/GUI/ActiveX surfaces.

Each is a candidate `gnucobol-rs` **non-claim** by default, and a fixture-priority signal only if it
appears in real fixed-record copybooks (most of these are Procedure-Division/screen, out of model).
