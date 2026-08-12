# GnuCOBOL Autotest suite — corpus extraction (Phase 2)

Classification happens at `AT_CHECK`-step level. Validity is profile-relative:
every step carries its oracle identity, dialect, format and expected contract.

discovered steps (stable 3.2 + current): 7321

## stable-3.2
- steps: 3486
- contract-valid: 2348
- expected rejects: 604
- oracle-contract drift: 301
- skipped under this oracle profile: 233

### candidate first-failure buckets
- check: 273
- layout: 119
- parse: 248
- preprocess: 101
- run: 201

## current
- steps: 3835
- contract-valid: 2384
- expected rejects: 646
- oracle-contract drift: 519
- skipped under this oracle profile: 286

### candidate first-failure buckets
- check: 259
- layout: 128
- parse: 239
- preprocess: 92
- run: 205

## Notes

- A step is valid only under the declared profile (oracle, dialect, format, options).
- `ORACLE_CONTRACT_DRIFT` = the suite declares the step valid but the admitted host
  oracle disagreed on replay; kept as a first-class finding.
- Screen tests (`$RUN_PROG_MANUAL`) and curses tests are skipped under this oracle
  profile (no terminal); their sources are still extracted and compile-probed.
- Raw per-step evidence lives under `GNURUST_COBOL_CORPUS_ROOT/packages/`.
