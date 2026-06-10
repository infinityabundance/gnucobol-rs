<!-- GENERATED from receipt.json by lab/receipt/run.py — DO NOT EDIT BY HAND.
     Regenerate: python3 lab/receipt/run.py generate -->
# GNURUST.LINEAGE.CORPUS.20M.SMOKE — 200K real-cobc COBOL-witness lineage burn (pilot)

**Verdict: PASS** · replay `PASS=200000 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.LINEAGE.CORPUS.20M.SMOKE` |
| court | 200K real-cobc COBOL-witness lineage burn (pilot) |
| crate_version | `0.7.27` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | 200K real-cobc witnesses: storage differential + directive variant; gate=Merkle+replay+isolation+findings |
| replay command | `bash lab/oracle/lineage_corpus_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the full 20M run (200K pilot; full 20M is .20M.1 detached)
- full parity / conformance
- all dialects / public-corpus parity
- business correctness
- the negative-zero finding being fixed
- families not yet emitting (storage + directive only)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `python3 lab/receipt/run.py generate`.
