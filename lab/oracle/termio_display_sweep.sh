#!/usr/bin/env bash
# termio.c cob_display_common differential (GNURUST.DISPLAY.BYTES.1): the DISPLAY-bytes core. One source of
# truth (termio_display_rows): `--cob` emits the cobc oracle program (typed fields DISPLAYed with labels);
# no-arg builds the same storage in Rust and prints label=cob_display_common(bytes). Compare line-for-line.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/termio_display_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$ROWS" --cob > "$TMP/p.cob"
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
"$TMP/p" > "$TMP/c.out"
"$ROWS"  > "$TMP/rs.out"
TOTAL=$(wc -l < "$TMP/c.out")
PASS=0; FAIL=0
mapfile -t C < "$TMP/c.out"
mapfile -t R < "$TMP/rs.out"
for ((i=0;i<${#C[@]};i++)); do
  if [ "${C[$i]:-}" = "${R[$i]:-}" ]; then PASS=$((PASS+1)); else
    FAIL=$((FAIL+1)); [ "$FAIL" -le 14 ] && { echo "MISMATCH line $((i+1))" >&2; echo "  oracle: ${C[$i]:-}" >&2; echo "  rust:   ${R[$i]:-}" >&2; }
  fi
done
echo "total=$TOTAL  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ] && [ "$TOTAL" = "$(wc -l < "$TMP/rs.out")" ]
