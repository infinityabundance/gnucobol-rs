# The GnuCOBOL / COBOL evidence atlas

> **Doctrine.** The COBOL Atlas separates formal standards, historical CODASYL specifications,
> government profiles, GnuCOBOL dialect modes, vendor compiler generations, platform runtimes, and
> preprocessor ecosystems. A surface is not "COBOL" in the abstract until the atlas says which axis
> admits it, whether it is syntax-only or runtime-functional, and which oracle or document family
> supports the claim.

This atlas exists so the project never confuses *"GnuCOBOL says,"* *"the ISO standard says,"* *"a
vendor dialect does,"* and *"an old forum thread suggests."* It is **machine-readable evidence first,
narrative second** — and every row is tagged with its `evidence_kind`.

## Evidence-kind discipline

| `evidence_kind` | meaning | trust |
|-----------------|---------|-------|
| `oracle_generated` | produced by the admitted GnuCOBOL 3.2 `cobc`/`libcob` (reproducible here) | **authoritative for this project** |
| `oracle_source` | read from the admitted GnuCOBOL 3.2 source tree | authoritative |
| `reference_curated` | curated from public docs with citations; **no standard text copied** | shadow witness only |
| `shadow_witness` | another compiler/version (not the admitted oracle) | comparison only, never overrides the oracle |
| `not_attempted` | a historical release we have not built | placeholder |

The hard rule, repeated from the project doctrine: **standard COBOL never overrides observed GnuCOBOL
behavior in `gnucobol-rs`.** The admitted oracle is the only authority for a sealed court.

## Axes

| Axis | What | Status in this atlas |
|------|------|-----------------------|
| **S** — Standards / amendments | COBOL-68/74/85, intrinsic-function amendment (1989), correction amendment (1993), 2002/2014/2023, X/Open, FIPS profiles | `reference_curated` stub (`S-standards-axis/`) |
| **H** — Historical CODASYL / government | COBOL-60/61/61-Ext/65, CODASYL JoD, FIPS PUB 21.x | stub (folded into S for now) |
| **G** — GnuCOBOL releases + `-std` dialect modes | the lineage, and `-std=` dialect profiles | **`oracle_generated`** (`G-gnucobol-dialect-axis/`) |
| **V** — Vendor compiler generations | IBM OS/VS→VS COBOL II→Enterprise COBOL v3–v6, Micro Focus, ACU, RM, BS2000, GCOS, … | `reference_curated` stub (`V-vendor-axis/`) |
| **P** — Platform/runtime families | z/OS, z/VSE, IBM i/ILE, AIX, Linux, Windows/MinGW, OpenVMS, NonStop, … | future |
| **X** — Preprocessor / application | CICS, IMS, DB2 embedded SQL, Pro*COBOL, ESQL | future (default non-claim) |
| **D** — Dataset / corpus profiles | the KOBOLD reconciliation fixture families | composed in `kobold-data-shim/recon/` |

## What the oracle already reveals (G-axis, real)

`lab/atlas/build-dialect-axis.sh` generates [`G-gnucobol-dialect-axis/dialect-axis.json`](atlases/G-gnucobol-dialect-axis/dialect-axis.json)
+ `reserved-deltas.tsv` directly from the admitted `cobc -std=<dialect>`. Reproducible findings (GnuCOBOL 3.2):

- `default` exposes ~995 reserved words; **`cobol85` is 650 leaner**, **`mf` adds ~112** beyond default,
  **`mf-strict` carries 111 reserved words that `default` does not** (Micro Focus extension surface).
- The IBM/MVS/ACU/RM/BS2000 *non-strict* dialects are near-supersets of `default` (they *add* vendor
  words); their `-strict` variants *remove* hundreds (enforcing the vendor's own subset).
- `rm-strict` exposes a **single** intrinsic function; `default`/`ibm`/`mf` expose 116–117.

These are dialect *recognition* surfaces (reserved words / intrinsics / system routines), **not**
runtime-behavior claims — a reserved word being present does not mean the feature is functional. That
distinction is the whole point of the status vocabulary below.

## Surface status vocabulary

A surface, per axis cell, is one of:

```
not_present · reserved_word_only · parsed_inert · parsed_warned · parsed_functional ·
runtime_functional · backend_dependent · platform_dependent · documented_not_implemented · unknown
```

## The join (A19) — where `gnucobol-rs` sits

[`A19-court-join/court-join.json`](atlases/A19-court-join/court-join.json) maps each data surface to
its `gnucobol_rs_status` (drawn from `reports/claim-ladder.json`). This turns the atlas into a
**roadmap generator**: an unsealed, real-copybook-likely, oracle-testable surface is the next
campaign candidate.

## Build phases (we are at Phase 1)

1. **Index, do not build** — release metadata, the G-axis from the admitted oracle. ← *here*
2. Extract feature clues (NEWS/reserved/testsuite) from the admitted source tree.
3. Build selected historical anchors only (OpenCOBOL 1.1, GnuCOBOL 2.2/3.1.2/3.2). *Not attempted.*
4. Curate the S/V axes (citations, no standard text).
5. Join + rank fixture candidates → `gnucobol-rs` roadmap.

## What not to do

Do not quote ISO/standard text. Do not build every historical release. Do not flatten OpenCOBOL /
GnuCOBOL / GnuCOBOL 4 / distro builds into one axis. Do not let a curated reference row override the
admitted oracle in a sealed court.
