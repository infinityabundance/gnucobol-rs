# A19 — standard × release × surface JOIN atlas

The payoff axis. [`join.json`](join.json) crosses the **surface taxonomy** (A18) with the **standard
axis** (A18), the **GnuCOBOL release/dialect axis** (A17 + G), and — the move that makes this a roadmap
generator — the **`gnucobol_rs` axis** (this project's sealed courts). [`drift-matrix.tsv`](drift-matrix.tsv)
is the flat view; [`fixture-candidates.json`](fixture-candidates.json) is the ranked next-campaign queue;
[`hidden-surfaces.md`](hidden-surfaces.md) is the human reading.

## How to read a join row

```
surface  ×  {what the STANDARD says}  ×  {what GnuCOBOL 3.2 DOES}  ×  {what gnucobol-rs SEALED}  →  drift
```

The interesting cells are the **disagreements** — the `drift` column:

- `physical_vs_logical_byte_domain` — `OCCURS DEPENDING`: GnuCOBOL's generated-C allocates the
  physical maximum; the *logical* length is a different byte domain. `gnucobol-rs` sealed physical-max
  only (`GNURUST.10`) and says so.
- `standard_behavior_differs_from_gnucobol` — `COMP-X` sizing: GnuCOBOL's tight `256^k>=10^digits`
  table differs from the `1-2-4-8` table of `COMP`/`COMP-5` (a real catch, `GNURUST.14`).
- `backend_dependent_feature` — `JSON GENERATE`, indexed I/O, screen: parsed, but runtime needs
  cJSON/JSON-C, BDB/VBISAM, curses. Syntax presence ≠ functional.
- `standard_before_gnucobol` — `SET..TO FALSE`: in the 2002 standard; `gnucobol-rs` seals TRUE and
  **fails closed** on FALSE (`GNURUST.12`).
- `platform_dependent_feature` — `SYNCHRONIZED`, EBCDIC code page: follow host/ABI/table, not the
  abstract standard.

## The discipline (unchanged)

`gnucobol_rs_axis` cells are **oracle-grounded** (every sealed campaign is proven against the admitted
3.2 oracle). `standard_axis` / vendor cells are **`reference_curated`** — citations, never copied text,
and they **never override** the oracle in a sealed court.
