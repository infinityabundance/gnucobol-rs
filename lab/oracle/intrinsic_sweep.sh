#!/usr/bin/env bash
# intrinsic.c differential (cob_intr_*): the real exported intrinsics linked against the built libcob
# (intrinsic_harness.c, after cob_init) vs the Rust port (cob_intr_rows), over a fixed battery; compare the
# result-field bytes line-for-line. PASS=n FAIL=n. ROOT from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
H="$ROOT/lab/oracle/intrinsic_harness"
[ -x "$H" ] || gcc -O2 -I"$PREFIX/include" "$ROOT/lab/oracle/intrinsic_harness.c" -o "$H" -L"$PREFIX/lib" -lcob || exit 2
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/cob_intr_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$H"    > "$TMP/c.out"
"$ROWS" > "$TMP/rs.out"
TOTAL=$(wc -l < "$TMP/c.out")
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
