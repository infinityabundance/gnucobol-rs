# Diagnostic-unblocked × corpus cross-check

_schema: gnurust-diag-unblocked-corpus-cross-check-v1

Three independent perspectives cross-check one another:
1. pristine upstream harness (authority);
2. diagnostic-unblocked upstream harness (this lane);
3. extracted AT_CHECK-level corpus with phase attribution.

## Totals

| steps_in_unblocked_logs | 1670 |
| matched_in_corpus | 33 |
| matched_passed | 24 |
| matched_failed | 9 |
| agreed | 33 |
| candidate_failures_on_valid_steps | 9 |
| contract_contradictions | 0 |
| not_in_corpus | 1637 |

## Findings (disagreements — never silently reconciled)

- newly-exposed candidate failure on corpus-valid step: group 1 step 7 ( $COBC --list-reserved)
- newly-exposed candidate failure on corpus-valid step: group 4 step 2 ( $COBC -I sub/copy prog.c -o prog.$COB_OBJECT_EXT)
- newly-exposed candidate failure on corpus-valid step: group 14 step 3 ( $COBCRUN -M ./caller inside again)
- newly-exposed candidate failure on corpus-valid step: group 16 step 2 ( $COBCRUN -M ./caller inside)
- newly-exposed candidate failure on corpus-valid step: group 17 step 0 ( $COBC -b ${FLAGS} mainer.cob called.cob)
- newly-exposed candidate failure on corpus-valid step: group 20 step 0 ( $COMPILE -jd prog.cob)
- newly-exposed candidate failure on corpus-valid step: group 22 step 0 ( $COMPILE -j="job 123" prog.cob)
- newly-exposed candidate failure on corpus-valid step: group 24 step 0 ( cat prog.cob | $COMPILE -j -)
- newly-exposed candidate failure on corpus-valid step: group 24 step 1 ( cat prog.cob | $COMPILE -vv -j -)

_Cross-checked from committed raw evidence._
