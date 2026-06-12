# PORT-INDEX.1 — typed C↔Rust symbol parity (+ PORT-INDEX.1a false-closure repair)

**Tool:** [`crates/gnucobol-rs-port-index`](../../crates/gnucobol-rs-port-index) · **Outputs:**
[`LIBCOB-PARITY.md`](../../LIBCOB-PARITY.md), [`reports/libcob-parity.json`](../libcob-parity.json),
[`reports/port-index/parity-detailed.json`](../port-index/parity-detailed.json)

## What this seals

`PORT-INDEX.1` replaced grep / name-presence parity with **typed C↔Rust symbol parity**. It indexes the
1170 admitted `libcob` source functions, classifies each by **preprocessor status** (1169 compiled into
the admitted oracle; the rest `#if 0` / config-gated), and joins them against a **real-`fn`** index of the
Rust port that is comment- and string-aware. A libcob function counts as ported **only when a real Rust
`fn` of that name exists** — a name appearing solely in a doc comment is reported as a `doc_only` false
hit, never a port. Each row separates: source · compiled · disabled · active · inactive mirror · test-only
· doc-only · missing · active-parity.

This demoted earlier soft (name-level) matches into explicit `doc_only` buckets and **exposed false
closure inside files that looked sealed**: `numeric.c` had 13, `move.c` 10, and `cobgetopt.c` 1 functions
that were doc-only citations with no real Rust `fn`.

## PORT-INDEX.1a — false-closure repair

Every exposed false hit in the completed files was closed by giving the faithful Rust counterpart its
**exact C name** (or, where genuinely absent, adding the named wrapper):

- **`move.c`** (10): the leaf converters renamed to their exact C names (`cob_move_alphanum_to_alphanum`,
  `cob_move_display_to_packed`, …).
- **`numeric.c`** (13): `logical_*` → `cob_logical_*`, the `Mpz` host-int helpers → `mpz_get_sll` /
  `mpz_get_ull` / `mpz_set_sll`, plus new named wrappers `cob_decimal_set`, `cob_pow_10`,
  `cob_div_by_pow_10`, and `cob_decimal_adjust` over the existing arithmetic primitives.
- **`cobgetopt.c`** (1): `getopt_initialize` → `_getopt_initialize`.

Result — the strict typed scoreboard for the completed files:

| libcob file | source | compiled | active | inactive | doc-only | missing | active parity |
|---|---:|---:|---:|---:|---:|---:|---:|
| `numeric.c` | 105 | 104 | 99 | 5 | 1¹ | 0 | 100.0% |
| `move.c` | 57 | 57 | 56 | 1 | 0 | 0 | 100.0% |
| `strings.c` | 34 | 34 | 34 | 0 | 0 | 0 | 100.0% |
| `cobgetopt.c` | 4 | 4 | 4 | 0 | 0 | 0 | 100.0% |

¹ the one residual `numeric.c` doc-only is a `#if 0`-**disabled** source function (not compiled), so it is
not part of compiled active-parity. All renames are behaviour-preserving — the per-court oracle sweeps
stay `FAIL=0` (logical 2400/0, packed-arith 1800/0, double-move 392/0, numcmp 1024/0, alnum-move 368/0,
getopt 35/0).

## Gates

`gnucobol-rs-port-index check` (anti-staleness: committed map == fresh re-derivation) runs in both
`lab/verify-sealed-courts.sh` and the docs staleness gate `lab/check-docs.sh`. It is complemented by the
doxygen C-vs-Rust coverage gate (`DOXYGEN-PARITY.md`), which is authoritative for the *compiled* C set.
The grep-based `xtask parity` tool was removed.

## Not claimed (next milestones)

Active parity is **symbol** parity — a real Rust `fn` exists — not behaviour parity (the oracle sweeps) and
not yet evidence-mapped. A future `function-evidence` milestone will add an *evidenced* column linking each
active function to its court / sweep / test receipt. The 952 remaining compiled-function gaps across the
un-started files are the honest work-list, not a regression.
