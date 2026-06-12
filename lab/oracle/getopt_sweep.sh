#!/usr/bin/env bash
# cobgetopt.c differential (cob_getopt_long_long): feed identical scenarios to the libcob oracle
# (getopt_harness.c, linked against the built libcob) and the Rust port (getopt_rows), compare the
# per-call result stream (return:optarg:optind:optopt). PASS=n FAIL=n. ROOT from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
H="$ROOT/lab/oracle/getopt_harness"
[ -x "$H" ] || gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/getopt_harness.c" -o "$H" -L"$PREFIX/lib" -lcob || exit 2
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_getopt"; ROWS="$ROOT/target/release/examples/getopt_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"
TOTAL=$(grep -cvE '^\s*(#|$)' "$TMP/cases.tsv")
"$H"    < "$TMP/cases.tsv" | sort > "$TMP/c.out"
"$ROWS" < "$TMP/cases.tsv" | sort > "$TMP/rs.out"
PASS=0; FAIL=0
while IFS= read -r rsline; do
  label="${rsline%% *}"
  cline=$(grep -m1 "^$label " "$TMP/c.out")
  if [ "$cline" = "$rsline" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1)); [ "$FAIL" -le 20 ] && { echo "MISMATCH $label" >&2; echo "  oracle: $cline" >&2; echo "  rust:   $rsline" >&2; }
  fi
done < "$TMP/rs.out"
echo "total=$TOTAL  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
