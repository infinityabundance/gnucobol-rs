<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.VALID-PROGRAMS.OMP.1 — valid-program corpus -- the Open Mainframe Project course repository is fully inventoried with platform dependencies typed (omp programs.json + inventory.json)

**Verdict: PASS** · replay `PASS=1 FAIL=0 valid-omp`

| field | value |
|-------|-------|
| campaign | `GNURUST.VALID-PROGRAMS.OMP.1` |
| court | valid-program corpus -- the Open Mainframe Project course repository is fully inventoried with platform dependencies typed (omp programs.json + inventory.json) |
| crate_version | `0.8.56` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | omp programs.json + inventory.json |
| replay command | `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh valid-omp` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- platform-service dependencies (z/OS, DB2, CICS, VSAM, JCL) are typed boundaries, never parser failures

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.
