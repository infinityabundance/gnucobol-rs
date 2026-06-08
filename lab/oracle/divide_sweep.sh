#!/usr/bin/env bash
# DIVIDE receiving-field byte sweep (GNURUST.19). Build ONE program: each case MOVEs literal operands,
# DIVIDEs into a GIVING receiver (BY/INTO, optional ROUNDED), and DISPLAYs the receiver's raw bytes via a
# REDEFINES. The Rust mirror builds the same operands via cob_move and checks cob_divide == receiver bytes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_divide"; ROWS="$ROOT/target/release/examples/divide_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"

{
  echo ">>SOURCE FORMAT FREE"
  echo "IDENTIFICATION DIVISION."
  echo "PROGRAM-ID. DIVSWEEP."
  echo "DATA DIVISION."
  echo "WORKING-STORAGE SECTION."
  i=0
  while IFS=$'\t' read -r label form rounded a apic ause b bpic buse cpic cuse csz; do
    au=""; [ "$ause" = "COMP-3" ] && au=" USAGE COMP-3"
    bu=""; [ "$buse" = "COMP-3" ] && bu=" USAGE COMP-3"
    cu=""; [ "$cuse" = "COMP-3" ] && cu=" USAGE COMP-3"
    echo "01 A-$i PIC $apic$au."
    echo "01 B-$i PIC $bpic$bu."
    echo "01 C-$i PIC $cpic$cu."
    echo "01 X-$i REDEFINES C-$i PIC X($csz)."
    i=$((i+1))
  done < "$TMP/cases.tsv"
  echo "PROCEDURE DIVISION."
  i=0
  while IFS=$'\t' read -r label form rounded a apic ause b bpic buse cpic cuse csz; do
    rnd=""; [ "$rounded" = "1" ] && rnd=" ROUNDED"
    echo "MOVE $a TO A-$i."
    echo "MOVE $b TO B-$i."
    if [ "$form" = "BY" ]; then
      echo "DIVIDE A-$i BY B-$i GIVING C-$i$rnd."
    else
      echo "DIVIDE A-$i INTO B-$i GIVING C-$i$rnd."
    fi
    echo "DISPLAY \"$label[\" X-$i \"]\"."
    i=$((i+1))
  done < "$TMP/cases.tsv"
  echo "STOP RUN."
} > "$TMP/div.cob"

if ! cobc -free -x -o "$TMP/div" "$TMP/div.cob" 2> "$TMP/cobc.err"; then
  echo "compile failed:"; head -20 "$TMP/cobc.err"; exit 2
fi
"$TMP/div" > "$TMP/out.txt" 2>/dev/null

# Join in Python: NUL bytes + a packed sign byte == ']' make delimiter/shell capture unsafe. We know
# each receiver's exact size, so we take exactly csz bytes after each "label[" marker (searched in order).
python3 - "$TMP/cases.tsv" "$TMP/out.txt" <<'PYJOIN' | "$ROWS"
import sys
cases = [l.rstrip("\n").split("\t") for l in open(sys.argv[1])]
out = open(sys.argv[2], "rb").read()
cur = 0
for c in cases:
    label, csz = c[0], int(c[11])
    m = out.find(b"%b[" % label.encode(), cur)
    ohex = ""
    if m >= 0:
        start = m + len(label) + 1
        ohex = out[start:start + csz].hex()
        cur = start + csz
    print("\t".join(c) + "\t" + ohex)
PYJOIN
