<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.LINEAGE.CORPUS.20M.0 — deterministic COBOL lineage corpus ENGINE (self-test)

**Verdict: PASS** · replay `PASS=10 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.LINEAGE.CORPUS.20M.0` |
| court | deterministic COBOL lineage corpus ENGINE (self-test) |
| crate_version | `0.7.43` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | meta-engine: deterministic gen + Merkle + replay + isolation + findings path (no burn) |
| replay command | `bash lab/oracle/lineage_engine_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- full GnuCOBOL parity / standard conformance
- all dialects / public-corpus execution parity
- business correctness / compiler replacement
- automatic build-profile detection
- the full 20M run (engine + 200K pilot sealed; full 20M detached as .20M.1)
- the negative-zero finding being fixed (filed as GNURUST.VALUE.NEGZERO.EDGE.1 candidate)
- families not yet emitting (v0 emits storage + directive of 15 planned)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
