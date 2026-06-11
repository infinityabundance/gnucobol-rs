#!/usr/bin/env bash
# Edited-picture decode sweep (GNURUST.16c): numeric->edited ENCODE,: build ONE COBOL program that MOVEs each value into its
# edited field and DISPLAYs the field's bytes (bracketed); compile+run with the built cobc; then the
# Rust `edited_rows` mirror ENCODES the value and checks it reproduces those bytes exactly + size. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_edited"
ROWS="$ROOT/target/release/examples/edited_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/cases.tsv"

# Build a free-form program: one edited field + MOVE + bracketed DISPLAY per case.
{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. EDPROG."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  while IFS=$'\t' read -r label pic value; do
    echo "01 E-$label PIC $pic."
  done < "$TMP/cases.tsv"
  echo "PROCEDURE DIVISION."
  while IFS=$'\t' read -r label pic value; do
    echo "MOVE $value TO E-$label."
    echo "DISPLAY \"$label[\" E-$label \"]\"."
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/edprog.cob"

if ! cobc -free -x -o "$TMP/edprog" "$TMP/edprog.cob" 2> "$TMP/cobc.err"; then
  echo "compile failed:"; cat "$TMP/cobc.err"; exit 2
fi
"$TMP/edprog" > "$TMP/out.txt" 2>/dev/null

# Join: for each case, extract the oracle bytes between [ ] and hex-encode them.
join_and_check() {
  while IFS=$'\t' read -r label pic value; do
    line=$(grep -m1 "^$label\[" "$TMP/out.txt")
    inner="${line#*[}"; inner="${inner%]*}"
    hex=$(printf '%s' "$inner" | od -An -tx1 | tr -d ' \n')
    printf '%s\t%s\t%s\t%s\n' "$label" "$pic" "$value" "$hex"
  done < "$TMP/cases.tsv"
}
join_and_check | "$ROWS"
