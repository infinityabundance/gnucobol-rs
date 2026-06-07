# A17 — GnuCOBOL / OpenCOBOL release atlas

The **release axis**: the OpenCOBOL → GNU Cobol → GnuCOBOL lineage, as machine-readable
[`releases.json`](releases.json). Phase 1 is *index, do not build*: dates and anchors are curated from
public project pages (citations in the JSON), and **only `gnucobol-3.2` is admitted and built** here —
it is the oracle every `gnucobol-rs` court is proven against. Every other release is
`build_status.not_attempted`: a placeholder, never a behavior claim.

## Lineage anchors

```
opencobol-1.0/1.1  →  gnucobol-1.1  →  gnucobol-2.x (2.2 anchor)  →  gnucobol-3.1/3.1.2  →  gnucobol-3.2 [ADMITTED]  →  gnucobol-4.x (shadow only)
```

## Discipline

- `gnucobol-3.2` = `oracle_source` (admitted, built; sha256 + recipe in `reports/admission/`). It is the
  **only** authority for a sealed court.
- `gnucobol-4.x` is a **shadow line**: a future comparison witness, never an override of 3.2.
- The status vocabulary (`not_present … runtime_functional … backend_dependent … unknown`) is the heart
  of the per-release feature map; populating it for historical releases is **Phase 3 (build selected
  anchors)** and is *not attempted* yet — loudly, not silently.

## What is real today vs. placeholder

| Release | evidence_kind | built? |
|---------|---------------|:------:|
| gnucobol-3.2 | `oracle_source` | **yes** (this lab) |
| all others | `reference_curated` / `not_attempted` | no |

The `-std` **dialect** profiles of the admitted 3.2 oracle (`ibm`, `mvs`, `mf`, `acu`, `rm`, `bs2000`,
…) *are* oracle-generated — see [`../G-gnucobol-dialect-axis/`](../G-gnucobol-dialect-axis/).
