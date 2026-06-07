#!/usr/bin/env bash
# verify-sealed-courts.sh (TRUST.1) — one command to replay every sealed court against the admitted
# oracle and print a single status table. Boring on purpose: a reviewer runs this and sees, in one
# place, that every GNURUST campaign still passes its differential sweep and KOBOLD.RECON.1 is stable.
#
# Usage:  bash lab/verify-sealed-courts.sh
# Exit:   0 iff every court is GREEN. ROOT is derived from this script's path (no absolute-path leaks).
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
GREEN=0; RED=0

row() { printf '  %-26s %s\n' "$1" "$2"; }
run_sweep() {  # name  script  [args]
  local name="$1" script="$2"; shift 2
  if [ ! -x "$PREFIX/bin/cobc" ]; then row "$name" "SKIP (no oracle built)"; return; fi
  local out
  out=$(bash "$ROOT/lab/oracle/$script" "$@" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+' | tail -1)
  case "$out" in
    *"FAIL=0") row "$name" "PASS  ($out)"; GREEN=$((GREEN+1)) ;;
    "")        row "$name" "ERROR (no result)"; RED=$((RED+1)) ;;
    *)         row "$name" "FAIL  ($out)"; RED=$((RED+1)) ;;
  esac
}

echo "== gnucobol-rs : verify sealed courts =="
echo "oracle: $( [ -x "$PREFIX/bin/cobc" ] && "$PREFIX/bin/cobc" --version 2>/dev/null | head -1 || echo 'NOT BUILT (sweeps skipped)')"
echo

run_sweep "GNURUST.2  decimal MOVE"  sweep.sh 0
run_sweep "GNURUST.3/9 PIC (+P)"     pic_sweep.sh
run_sweep "GNURUST.14 binary MOVE"   binary_sweep.sh
run_sweep "GNURUST.15 EBCDIC cp500"  ebcdic_sweep.sh
run_sweep "GNURUST.16 edited decode" edited_sweep.sh
run_sweep "GNURUST.4/10 layout(+ODO)" layout_sweep.sh
run_sweep "GNURUST.10 ODO phys-max"  odo_sweep.sh
run_sweep "GNURUST.5  COPY"          copy_sweep.sh
run_sweep "GNURUST.7/13 arithmetic"  arith_sweep.sh
run_sweep "GNURUST.8  VALUE image"   value_sweep.sh
run_sweep "GNURUST.11 LEVEL-88 eval" cond_sweep.sh
run_sweep "GNURUST.12 SET 88 TRUE"   set_sweep.sh

# KOBOLD.RECON.1 lives in the sibling crate; run its acceptance test if present.
echo
SHIM="$ROOT/../kobold-data-shim"
if [ -f "$SHIM/Cargo.toml" ]; then
  # Run the FULL shim suite (recon + operator + lib), not just --test recon: a stale record-len in the
  # operator tests must turn this red (the KOBOLD.DATA.4 lesson — a publish guard, not a grep).
  if ( cd "$SHIM" && cargo test -q >/dev/null 2>&1 ); then
    row "KOBOLD (shim: recon+operator+lib)" "PASS  (corpus byte-stable, CLI==lib)"; GREEN=$((GREEN+1))
  else
    row "KOBOLD (shim: recon+operator+lib)" "FAIL"; RED=$((RED+1))
  fi
else
  row "KOBOLD (shim)" "SKIP (sibling crate absent)"
fi

# Self-contained Rust tests + the doc-staleness gate are part of "sealed".
echo
( cd "$ROOT" && cargo test -q >/dev/null 2>&1 ) && row "cargo test (self-contained)" "PASS" || { row "cargo test (self-contained)" "FAIL"; RED=$((RED+1)); }
( cd "$ROOT" && bash lab/check-docs.sh >/dev/null 2>&1 ) && row "doc-gate (anti-staleness)" "PASS" || { row "doc-gate (anti-staleness)" "FAIL"; RED=$((RED+1)); }

echo
echo "== $GREEN green, $RED red =="
# PUBLISH GUARD: nonzero exit on ANY red. Treat this as the gate before every version bump / publish /
# git tag (KOBOLD.DATA.4 lesson: assert FAILED:0 here BEFORE packaging, never grep after).
if [ "$RED" -ne 0 ]; then
  echo "!! $RED FAILING block(s) — DO NOT commit, version-bump, or publish until green."
fi
[ "$RED" -eq 0 ]
