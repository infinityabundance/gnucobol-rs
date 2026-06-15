<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.LINEAGE.CORPUS.20M.1 — full 20M real-cobc COBOL-witness lineage run (complete)

**Verdict: PASS** · replay `PASS=4000000 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.LINEAGE.CORPUS.20M.1` |
| court | full 20M real-cobc COBOL-witness lineage run (complete) |
| crate_version | `0.7.66` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | completed 4M real-cobc witness run; evidence-of-record reports/lineage20m/full-run-seal.json (root-of-roots binds the regenerable shards) |
| replay command | `bash lab/oracle/lineage_fullrun_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- full GnuCOBOL parity / standard conformance
- all dialects / public-corpus parity
- business correctness / compiler replacement
- the negative-zero finding being fixed
- the not-yet-emitting families (4M of 20M = storage+directive only; 13 families dropped, not covered)
- the 16M dropped indices being witnesses

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
