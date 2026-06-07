# A18 — COBOL standard atlas (+ historical / government axes)

The **standard axis**: formal ISO/ANSI standards, the pre-ANSI historical specs, amendments, and
portability/government profiles — as [`standards.json`](standards.json). **No standard text is
reproduced** (ISO texts are paywalled/copyright); rows store *feature labels + citations* only, tagged
with `authority_kind` (`historical_standard` / `standard` / `amendment` / `portability_profile` /
`government_profile`). These are **`reference_curated` shadow witnesses** — they never override the
admitted GnuCOBOL 3.2 oracle in a sealed court.

## Lineage (labels only)
```
COBOL-60/61/61-Ext/65 (historical)  →  COBOL-68 (first ANSI)  →  74  →  85
  (+ 1992 intrinsic, +1994 correction amendments)  →  2002  →  2014  →  2023 (current)
X/Open COBOL · The Open Group profile · FIPS PUB 21.x (government)
```

## Files
- [`standards.json`](standards.json) — the standard/historical/amendment/profile rows.
- [`surface-taxonomy.json`](surface-taxonomy.json) — the ~29 feature *surfaces* the join ranges over,
  each carrying `gnucobol_rs_status` (the roadmap hook).
- [`feature-index.json`](feature-index.json) — per-standard surface presence (`present`/`absent`/
  `unknown`), `reference_curated`.

## Discipline
Standard presence is a *citation-backed label*, not a behavior claim. The join (A19) is where these
labels are crossed with what GnuCOBOL 3.2 actually **does** (oracle-grounded) — and the disagreements
(`drift`) are the point.
