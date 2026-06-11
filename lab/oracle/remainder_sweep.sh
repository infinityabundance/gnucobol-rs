#!/usr/bin/env bash
# DIVIDE...REMAINDER receiving-field byte sweep (GNURUST.REMAINDER.1). ONE program: each case MOVEs operands,
# DIVIDEs into a GIVING quotient + REMAINDER receiver (BY/INTO), and DISPLAYs BOTH receivers' raw bytes via
# REDEFINES. The Rust mirror builds the same operands via cob_move and checks cob_divide_remainder == both.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_remainder"; ROWS="$ROOT/target/release/examples/remainder_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"

{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. REMSWEEP."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  i=0
  while IFS=$'\t' read -r label form a apic ause b bpic buse cpic cuse csz dpic duse dsz; do
    au=""; [ "$ause" = "COMP-3" ] && au=" USAGE COMP-3"
    bu=""; [ "$buse" = "COMP-3" ] && bu=" USAGE COMP-3"
    cu=""; [ "$cuse" = "COMP-3" ] && cu=" USAGE COMP-3"
    du=""; [ "$duse" = "COMP-3" ] && du=" USAGE COMP-3"
    echo "01 A-$i PIC $apic$au."
    echo "01 B-$i PIC $bpic$bu."
    echo "01 C-$i PIC $cpic$cu."
    echo "01 X-$i REDEFINES C-$i PIC X($csz)."
    echo "01 D-$i PIC $dpic$du."
    echo "01 Y-$i REDEFINES D-$i PIC X($dsz)."
    i=$((i+1))
  done < "$TMP/cases.tsv"
  echo "PROCEDURE DIVISION."
  i=0
  while IFS=$'\t' read -r label form a apic ause b bpic buse cpic cuse csz dpic duse dsz; do
    echo "MOVE $a TO A-$i."
    echo "MOVE $b TO B-$i."
    if [ "$form" = "BY" ]; then
      echo "DIVIDE A-$i BY B-$i GIVING C-$i REMAINDER D-$i."
    else
      echo "DIVIDE A-$i INTO B-$i GIVING C-$i REMAINDER D-$i."
    fi
    echo "DISPLAY \"${label}Q[\" X-$i \"]\"."
    echo "DISPLAY \"${label}R[\" Y-$i \"]\"."
    i=$((i+1))
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/rem.cob"

if ! cobc -free -x -o "$TMP/rem" "$TMP/rem.cob" 2> "$TMP/cobc.err"; then
  echo "compile failed:"; head -20 "$TMP/cobc.err"; exit 2
fi
"$TMP/rem" > "$TMP/out.txt" 2>/dev/null

( cd "$ROOT" && cargo run -q -p xtask -- sweep-remainder "$TMP/cases.tsv" "$TMP/out.txt" ) | "$ROWS"
