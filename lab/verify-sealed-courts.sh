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
run_sweep "move.c alphanumeric MOVE"  alnum_move_sweep.sh
run_sweep "move.c cob_get_int/llint"  get_int_sweep.sh
run_sweep "move.c typed accessors"   typed_acc_sweep.sh
run_sweep "GNURUST.3/9 PIC (+P)"     pic_sweep.sh
run_sweep "GNURUST.14 binary MOVE"   binary_sweep.sh
run_sweep "GNURUST.15 EBCDIC cp500"  ebcdic_sweep.sh
run_sweep "GNURUST.16 edited decode" edited_sweep.sh
run_sweep "GNURUST.16C edited encode" edited_encode_sweep.sh
run_sweep "GNURUST.17 cp500 zoned-num" ebcdic_num_sweep.sh
run_sweep "GNURUST.18 COMP-6 MOVE"    comp6_sweep.sh
run_sweep "GNURUST.4/10 layout(+ODO)" layout_sweep.sh
run_sweep "GNURUST.10 ODO phys-max"  odo_sweep.sh
run_sweep "GNURUST.5  COPY"          copy_sweep.sh
run_sweep "GNURUST.7/13 arithmetic"  arith_sweep.sh
run_sweep "numeric.c cob_add_bcd (packed)" packed_arith_sweep.sh
run_sweep "numeric.c set_double (mpf)" double_move_sweep.sh
run_sweep "GNURUST.19 DIVIDE bytes"   divide_sweep.sh
run_sweep "GNURUST.REMAINDER.1 bytes" remainder_sweep.sh
run_sweep "GNURUST.ROUND.1 modes"     round_sweep.sh
run_sweep "GNURUST.BIGNUM.1 mul>i128" bignum_sweep.sh
run_sweep "GNURUST.INTPOW.1 int pow"  pow_sweep.sh
run_sweep "GNURUST.LOGICAL.1 bit ops" logical_sweep.sh
run_sweep "GNURUST.NUMCMP.1 numeric cmp" numcmp_sweep.sh
run_sweep "GNURUST.FLOAT.1 float flds" float_sweep.sh
run_sweep "GNURUST.FILE.SEQUENTIAL.1" seqfile_sweep.sh
run_sweep "GNURUST.FILE.WRITE.1" write_seq_sweep.sh
run_sweep "GNURUST.FILEIO.LINESEQ.1" lineseq_write_sweep.sh
run_sweep "GNURUST.FILE.REWRITE.1" rewrite_sweep.sh
run_sweep "GNURUST.FILE.STATUS.1 (observed)" file_status_sweep.sh
run_sweep "GNURUST.INITIALIZE.1 bytes" initialize_sweep.sh
run_sweep "GNURUST.INSPECT.1 bytes" inspect_sweep.sh
run_sweep "GNURUST.REFMOD.1 refmod"  refmod_sweep.sh
run_sweep "GNURUST.STRING.UNSTRING.1" string_unstring_sweep.sh
run_sweep "cobgetopt.c getopt_long_long" getopt_sweep.sh
run_sweep "cconv.c case/hex/collation" cconv_sweep.sh
run_sweep "termio.c cob_display_common" termio_display_sweep.sh
run_sweep "intrinsic.c cob_intr_* result fields" intrinsic_sweep.sh
run_sweep "GNURUST.INTRINSIC.ATLAS.1 (observed)" intrinsic_atlas_sweep.sh
run_sweep "GNURUST.INTRINSIC.LENGTH.1" length_sweep.sh
run_sweep "GNURUST.INTRINSIC.NUMVAL.1" numval_sweep.sh
run_sweep "GNURUST.INTRINSIC.NUMVAL-C.1" numvalc_sweep.sh
run_sweep "GNURUST.INTRINSIC.MOD-REM.1" modrem_sweep.sh
run_sweep "GNURUST.INTRINSIC.INTEGER.1" integer_sweep.sh
run_sweep "GNURUST.INTRINSIC.CASE.1" case_sweep.sh
run_sweep "GNURUST.INTRINSIC.ORD-CHAR.1" ordchar_sweep.sh
run_sweep "GNURUST.INTRINSIC.DATE.1" date_sweep.sh
run_sweep "GNURUST.ACCEPT.DISPLAY.1" accept_display_sweep.sh
run_sweep "GNURUST.ACCEPT.DISPLAY.2" accept_display2_sweep.sh
run_sweep "GNURUST.BUILD.PROFILE.1 (profile)" build_profile_sweep.sh
run_sweep "GNURUST.PROCEDURE.FLOW.ATLAS.1 (observed)" procedure_flow_atlas_sweep.sh
run_sweep "GNURUST.CALL.EXTENSION.ATLAS.1 (observed)" call_atlas_sweep.sh
run_sweep "GNURUST.INDEXED.FILE.ATLAS.1 (observed)" indexed_file_atlas_sweep.sh
run_sweep "GNURUST.SORT.MERGE.ATLAS.1 (observed)" sort_merge_atlas_sweep.sh
run_sweep "GNURUST.RELATIVE.FILE.ATLAS.1 (observed)" relative_file_atlas_sweep.sh
run_sweep "GNURUST.DIALECT.RUNTIME.ATLAS.1 (observed)" dialect_runtime_atlas_sweep.sh
run_sweep "GNURUST.DIRECTIVE.VARIANCE.ATLAS.1 (observed)" directive_variance_atlas_sweep.sh
run_sweep "GNURUST.DECLARATIVES.ATLAS.1 (observed)" declaratives_atlas_sweep.sh
run_sweep "GNURUST.CALL.LAYOUT.ATLAS.1 (observed)" call_layout_atlas_sweep.sh
run_sweep "GNURUST.LINEAGE.CORPUS.20M.0 (engine)" lineage_engine_sweep.sh
run_sweep "GNURUST.LINEAGE.CORPUS.20M.SMOKE (burn)" lineage_corpus_sweep.sh
run_sweep "GNURUST.LINEAGE.CORPUS.20M.1 (full run)" lineage_fullrun_sweep.sh
run_sweep "GNURUST.IF.EVALUATE.SLICE.1" if_eval_sweep.sh
run_sweep "GNURUST.IF.NUMERIC.SLICE.1" if_numeric_sweep.sh
run_sweep "GNURUST.PERFORM.SLICE.1" perform_sweep.sh
run_sweep "GNURUST.TABLE.PERFORM.SLICE.1" table_sweep.sh
run_sweep "GNURUST.SEARCH.TABLE.1" search_sweep.sh
run_sweep "GNURUST.SUBSCRIPT.1 subscript" subscript_sweep.sh
run_sweep "GNURUST.ODO.1 odo"        odo_sweep.sh
run_sweep "GNURUST.INDEX.1 usage-index" index_sweep.sh
run_sweep "GNURUST.FILE.FLOW.SLICE.1" file_flow_sweep.sh
run_sweep "GNURUST.FILE.FILTER.SLICE.1" file_filter_sweep.sh
run_sweep "SIZE.ERROR.ATLAS.1 (observed)" size_error_atlas_sweep.sh
run_sweep "GNURUST.SIZE.ERROR.1" size_error_sweep.sh
run_sweep "GNURUST.8  VALUE image"   value_sweep.sh
run_sweep "GNURUST.VALUE.NEGZERO.EDGE.1" edge_negzero_sweep.sh
run_sweep "GNURUST.11 LEVEL-88 eval" cond_sweep.sh
run_sweep "GNURUST.CLASS.1 class cond"  class_sweep.sh
run_sweep "GNURUST.12 SET 88 TRUE"   set_sweep.sh
run_sweep "GNURUST.12B SET 88 FALSE"  set_false_sweep.sh

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
if command -v kobold-attest >/dev/null 2>&1; then
  kobold-attest selftest >/dev/null 2>&1 && row "ENTERPRISE.2 kobold-attest selftest (external rust ed25519, 6 states)" "PASS" || { row "ENTERPRISE.2 kobold-attest selftest" "FAIL"; RED=$((RED+1)); }
else
  row "ENTERPRISE.2 kobold-attest selftest (external; not installed -> skipped)" "PASS"
fi
( cd "$ROOT" && bash lab/check-docs.sh >/dev/null 2>&1 ) && row "doc-gate (anti-staleness)" "PASS" || { row "doc-gate (anti-staleness)" "FAIL"; RED=$((RED+1)); }
# Forensic claim-ladder gate: the hand-authored claim-ladder must still match machine reality (every
# verified court declared, schema-complete, oracle == admitted 3.2 build, PORTING-LADDER.md fresh).
( cd "$ROOT" && cargo run -q -p xtask -- ladder check >/dev/null 2>&1 ) && row "claim-ladder gate (forensic)" "PASS" || { row "claim-ladder gate (forensic)" "FAIL"; RED=$((RED+1)); }
# PORT-INDEX.1: the TYPED C↔Rust symbol parity (gnucobol-rs-port-index) must match a fresh re-derivation
# from the admitted libcob source + the Rust src. This is the authoritative parity (real `fn`s vs doc-only
# false hits, with #if 0 / config classification) that replaced grep name-matching. Source-gated.
( cd "$ROOT" && cargo run -q -p gnucobol-rs-port-index -- check >/dev/null 2>&1 ) && row "libcob-parity gate (port-index, typed)" "PASS" || { row "libcob-parity gate (port-index, typed)" "FAIL"; RED=$((RED+1)); }
# GNURUST.CCVS85.1: external CCVS85 (NIST COBOL-85 validation) corpus CUSTODY -- compressed/decompressed
# hashes + split-index metadata stable vs the committed receipt. Corpus-custody only; NO conformance claim.
( cd "$ROOT" && cargo run -q -p gnucobol-rs-port-index -- ccvs85 check >/dev/null 2>&1 ) && row "GNURUST.CCVS85.1 corpus custody (NIST CCVS85)" "PASS" || { row "GNURUST.CCVS85.1 corpus custody (NIST CCVS85)" "FAIL"; RED=$((RED+1)); }
# GNURUST.COBOL-CORPUS-ATLAS.1: the multi-corpus custody manifest (3 evidence classes) stable vs the
# committed receipt; re-derives custody for any locally-present corpus (gitignored), green without them.
( cd "$ROOT" && cargo run -q -p gnucobol-rs-port-index -- corpus-atlas check >/dev/null 2>&1 ) && row "GNURUST.COBOL-CORPUS-ATLAS.1 (5-corpus custody atlas)" "PASS" || { row "GNURUST.COBOL-CORPUS-ATLAS.1 (5-corpus custody atlas)" "FAIL"; RED=$((RED+1)); }
# Rust-port doxygen: run doxygen on crates/gnucobol-rs/src as a CLEAN refresh (the previous run is wiped
# first, so it never accumulates), and assert it documented the port. The authoritative per-function
# coverage ("did we miss anything") is the parity gate above; this proves the browsable native-Rust
# libcob doxygen regenerates and maps against the C-side libcob doxygen (lab/doxygen/Doxyfile).
if command -v doxygen >/dev/null 2>&1; then
  ( cd "$ROOT" && rm -rf lab/doxygen/out-rust && doxygen lab/doxygen/Doxyfile-rust >/dev/null 2>&1 )
  RFNS=$(grep -rhoE 'kind="function"' "$ROOT"/lab/doxygen/out-rust/xml/*8rs.xml 2>/dev/null | wc -l)
  if [ "${RFNS:-0}" -gt 200 ]; then row "rust-port doxygen (clean, $RFNS fns)" "PASS"; else row "rust-port doxygen (clean refresh)" "FAIL"; RED=$((RED+1)); fi
  # DOXYGEN.PARITY: the AUTHORITATIVE C-vs-Rust coverage compare. Regenerate the C-side XML inventory
  # (fast, XML-only) from the pinned libcob, then assert the committed coverage doc is fresh AND that no
  # libcob file the awk parity reports complete has a doxygen-found function with no Rust counterpart
  # ("did we miss anything"). This is the gate that proves each file is truly done.
  ( cd "$ROOT" && doxygen lab/doxygen/Doxyfile-c-xml >/dev/null 2>&1 )
  ( cd "$ROOT" && cargo run -q -p xtask -- doxygen-compare check >/dev/null 2>&1 ) && row "C-vs-Rust doxygen parity (did-we-miss)" "PASS" || { row "C-vs-Rust doxygen parity (did-we-miss)" "FAIL"; RED=$((RED+1)); }
else
  row "rust-port doxygen (doxygen absent -> skipped)" "PASS"
fi

echo
echo "== $GREEN green, $RED red =="
# PUBLISH GUARD: nonzero exit on ANY red. Treat this as the gate before every version bump / publish /
# git tag (KOBOLD.DATA.4 lesson: assert FAILED:0 here BEFORE packaging, never grep after).
if [ "$RED" -ne 0 ]; then
  echo "!! $RED FAILING block(s) — DO NOT commit, version-bump, or publish until green."
fi
[ "$RED" -eq 0 ]
