#!/usr/bin/env bash
# cp500 EBCDIC zoned-numeric decode sweep (GNURUST.17). Build ONE program compiled `-fsign=EBCDIC`:
# each case MOVEs the cp500-translated ascii-overpunch bytes into a signed zoned field, then into an
# edited field, and DISPLAYs the edited bytes. The Rust mirror decodes the RAW EBCDIC via
# from_ebcdic_zoned and checks it equals both the expected value and GnuCOBOL's own decode (the edited
# output, via the sealed decode_edited). PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_ebcdic_num"
ROWS="$ROOT/target/release/examples/ebcdic_num_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/cases.tsv"

{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. EBNUM."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  i=0
  while IFS=$'\t' read -r label pic value raw ascii outpic; do
    # size of the zoned field = number of ascii bytes
    sz=${#ascii}
    echo "01 R-$i PIC X($sz)."
    echo "01 S-$i REDEFINES R-$i PIC $pic."
    echo "01 O-$i PIC $outpic."
    i=$((i+1))
  done < "$TMP/cases.tsv"
  echo "PROCEDURE DIVISION."
  i=0
  while IFS=$'\t' read -r label pic value raw ascii outpic; do
    echo "MOVE \"$ascii\" TO R-$i."
    echo "MOVE S-$i TO O-$i."
    echo "DISPLAY \"$label[\" O-$i \"]\"."
    i=$((i+1))
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/ebnum.cob"

if ! cobc -free -fsign=EBCDIC -x -o "$TMP/ebnum" "$TMP/ebnum.cob" 2> "$TMP/cobc.err"; then
  echo "compile failed:"; cat "$TMP/cobc.err"; exit 2
fi
"$TMP/ebnum" > "$TMP/out.txt" 2>/dev/null

# join: label pic value raw_hex out_pic oracle_out_hex
join_check() {
  while IFS=$'\t' read -r label pic value raw ascii outpic; do
    line=$(grep -m1 "^$label\[" "$TMP/out.txt")
    inner="${line#*[}"; inner="${inner%]*}"
    ohex=$(printf '%s' "$inner" | od -An -tx1 | tr -d ' \n')
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$label" "$pic" "$value" "$raw" "$outpic" "$ohex"
  done < "$TMP/cases.tsv"
}
join_check | "$ROWS"
