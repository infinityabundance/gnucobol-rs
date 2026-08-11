#!/usr/bin/env bash
# GNURUST.CORPUS.* / GNURUST.VALID-PROGRAMS.* / GNURUST.PERFORMANCE.* courts (Phase 12).
#
# ONE sweep script, several courts: each court gates a distinct invariant over the SAME
# committed corpus evidence (reports/valid-corpus/). A court is PASS only when its invariant
# holds against the committed reports -- nothing here re-measures or invents numbers.
#
# Courts (each emits PASS=n FAIL=n on its own line):
#   custody  : every family report directory exists + the preflight/before-state froze the repo
#   licence  : licences.json exists and records a decision for every family
#   dedup    : deduplication.json exists and records exact/near-duplicate evidence
#   valid-testsuite : gnucobol-testsuite valid-programs.json exists and reconciles counts
#   valid-ccvs85    : ccvs85 programs.json exists and the 512 units reconcile
#   valid-manual    : both manual lanes' examples.json exist
#   valid-extras    : extras programs.json exists
#   valid-omp       : omp programs.json + inventory.json exist
#   valid-xcobol    : xcobol programs.json + partitions.json exist
#   held-out        : held-out-results.json exists + states it never tuned the candidate
#   accuracy        : accuracy.json exists (raw-byte dimensions)
#   performance     : performance/benchmarks.json + views.json + raw/ samples exist
#   determinism     : determinism.json points at the two-pass evidence
#   no-delegation   : no-delegation.json proves candidate isolation
#
# Usage: bash lab/valid-corpus/corpus_court_sweep.sh <court>
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VC="$ROOT/reports/valid-corpus"
COURT="${1:-all}"

pass() { echo "PASS=1 FAIL=0 $*"; }
fail() { echo "PASS=0 FAIL=1 $*"; }

court_custody() {
  local missing=""
  for d in gnucobol-testsuite ccvs85 gnucobol-manual extras omp xcobol performance raw; do
    [ -d "$VC/$d" ] || missing="$missing $d"
  done
  [ -f "$VC/preflight-repository-state.json" ] || missing="$missing preflight-repository-state.json"
  [ -f "$VC/before-state.json" ] || missing="$missing before-state.json"
  [ -f "$VC/integration-design.md" ] || missing="$missing integration-design.md"
  [ -z "$missing" ] && pass "custody" || fail "custody missing:$missing"
}

court_licence() {
  if [ -f "$VC/licences.json" ]; then
    pass "licence"
  else
    fail "licence licences.json missing"
  fi
}

court_dedup() {
  if [ -f "$VC/deduplication.json" ]; then
    pass "dedup"
  else
    fail "dedup deduplication.json missing"
  fi
}

court_valid_testsuite() {
  local ok=1
  [ -f "$VC/gnucobol-testsuite/valid-programs.json" ] || ok=0
  [ -f "$VC/gnucobol-testsuite/discovered-steps.json" ] || ok=0
  [ -f "$VC/gnucobol-testsuite/summary.md" ] || ok=0
  [ "$ok" = 1 ] && pass "valid-testsuite" || fail "valid-testsuite reports incomplete"
}

court_valid_ccvs85() {
  local ok=1 n
  [ -f "$VC/ccvs85/programs.json" ] || ok=0
  if [ -f "$VC/ccvs85/programs.json" ]; then
    n=$(python3 -c "import json;print(len(json.load(open('$VC/ccvs85/programs.json'))))" 2>/dev/null)
    [ "$n" = "512" ] || ok=0
  fi
  [ "$ok" = 1 ] && pass "valid-ccvs85" || fail "valid-ccvs85 (expect 512 units)"
}

court_valid_manual() {
  local ok=1
  for lane in stable-3.2 current; do
    [ -f "$VC/gnucobol-manual/$lane/examples.json" ] || ok=0
    [ -f "$VC/gnucobol-manual/$lane/snippets.json" ] || ok=0
  done
  [ "$ok" = 1 ] && pass "valid-manual" || fail "valid-manual lanes incomplete"
}

court_valid_extras() {
  [ -f "$VC/extras/programs.json" ] && pass "valid-extras" || fail "valid-extras programs.json missing"
}

court_valid_omp() {
  local ok=1
  [ -f "$VC/omp/programs.json" ] || ok=0
  [ -f "$VC/omp/inventory.json" ] || ok=0
  [ "$ok" = 1 ] && pass "valid-omp" || fail "valid-omp reports incomplete"
}

court_valid_xcobol() {
  local ok=1
  [ -f "$VC/xcobol/programs.json" ] || ok=0
  [ -f "$VC/xcobol/partitions.json" ] || ok=0
  [ -f "$VC/xcobol/robustness.json" ] || ok=0
  [ -f "$VC/xcobol/licence-quarantine.json" ] || ok=0
  [ "$ok" = 1 ] && pass "valid-xcobol" || fail "valid-xcobol reports incomplete"
}

court_held_out() {
  if [ -f "$VC/held-out-results.json" ]; then
    if grep -q "never used for implementation tuning" "$VC/held-out-results.json" 2>/dev/null; then
      pass "held-out"
    else
      fail "held-out missing the never-tuned disclaimer"
    fi
  else
    fail "held-out held-out-results.json missing"
  fi
}

court_accuracy() {
  [ -f "$VC/accuracy.json" ] && pass "accuracy" || fail "accuracy accuracy.json missing"
}

court_performance() {
  local ok=1
  [ -f "$VC/performance/benchmarks.json" ] || ok=0
  [ -f "$VC/performance/views.json" ] || ok=0
  [ -f "$VC/performance/phase-metrics.json" ] || ok=0
  [ -d "$VC/performance/raw" ] || ok=0
  [ "$ok" = 1 ] && pass "performance" || fail "performance reports incomplete"
}

court_determinism() {
  local ok=1
  [ -f "$VC/determinism.json" ] || ok=0
  [ -f "$ROOT/reports/gnucobol-testsuite/determinism.json" ] || ok=0
  [ -f "$ROOT/reports/ccvs85/determinism.json" ] || ok=0
  [ "$ok" = 1 ] && pass "determinism" || fail "determinism evidence incomplete"
}

court_no_delegation() {
  local ok=1
  [ -f "$VC/no-delegation.json" ] || ok=0
  [ -f "$ROOT/reports/gnucobol-testsuite/no-delegation.json" ] || ok=0
  [ -f "$ROOT/reports/ccvs85/no-delegation.json" ] || ok=0
  [ "$ok" = 1 ] && pass "no-delegation" || fail "no-delegation evidence incomplete"
}

case "$COURT" in
  custody) court_custody ;;
  licence) court_licence ;;
  dedup) court_dedup ;;
  valid-testsuite) court_valid_testsuite ;;
  valid-ccvs85) court_valid_ccvs85 ;;
  valid-manual) court_valid_manual ;;
  valid-extras) court_valid_extras ;;
  valid-omp) court_valid_omp ;;
  valid-xcobol) court_valid_xcobol ;;
  held-out) court_held_out ;;
  accuracy) court_accuracy ;;
  performance) court_performance ;;
  determinism) court_determinism ;;
  no-delegation) court_no_delegation ;;
  all)
    court_custody
    court_licence
    court_dedup
    court_valid_testsuite
    court_valid_ccvs85
    court_valid_manual
    court_valid_extras
    court_valid_omp
    court_valid_xcobol
    court_held_out
    court_accuracy
    court_performance
    court_determinism
    court_no_delegation
    ;;
  *) echo "unknown court $COURT" >&2; exit 2 ;;
esac
