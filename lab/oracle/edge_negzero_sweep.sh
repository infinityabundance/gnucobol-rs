#!/usr/bin/env bash
# GNURUST.VALUE.NEGZERO.EDGE.1 -- oracle characterization + regression lock of negative-zero VALUE sign,
# harvested by GNURUST.LINEAGE.CORPUS.20M (1056 hits). The DISCRIMINATOR is the LITERAL FORM (integer vs
# decimal-point), NOT the field scale.
#
# NAMED oracle rule (each a locked assertion):
#   oracle_comp3_integer_negzero_canonicalizes_positive : COMP-3 + integer-form -0  -> sign 0C
#   oracle_comp3_decimal_negzero_preserves_negative      : COMP-3 + decimal-form -0.0 -> sign 0D (even V99)
#   oracle_display_negzero_preserves_overpunch           : DISPLAY any-form -0       -> overpunch 0x70
#   oracle_binary_negzero_collapses_to_zero_bytes        : COMP/COMP-5/COMP-X -0     -> all-zero (no sign nibble)
#   oracle_unsigned_negzero_rejected                     : unsigned 9(n) VALUE -0    -> COMPILE REJECT
# NAMED rust divergence lock:
#   rust_diverges_only_comp3_integer_form_negzero        : value_image != cobc ONLY on COMP-3 integer-form
#   rust_matches_display_negzero                         : DISPLAY cells MATCH
#   rust_matches_comp3_decimal_form_negzero              : decimal-form COMP-3 cells MATCH
# A new divergence, an oracle-rule change, or a rust change in a matching cell -> RED. The patch (when
# decided) flips ONLY the COMP-3 integer-form cells to match; if it touches DISPLAY/decimal cells -> RED.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=dumb
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
VR="$ROOT/target/release/examples/value_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# label|pic|usage_clause|value|group(comp3_int|comp3_dec|display|binary|unsigned)|rust_compared(1/0)
CELLS=(
 "comp3-int-0|S9(3)|USAGE COMP-3|-0|comp3_int|1"
 "comp3-int-00|S9(3)|USAGE COMP-3|-00|comp3_int|1"
 "comp3-int-000|S9(3)|USAGE COMP-3|-000|comp3_int|1"
 "comp3v99-int0|S9(3)V99|USAGE COMP-3|-0|comp3_int|1"
 "comp3-pos-0|S9(3)|USAGE COMP-3|0|comp3_int|1"
 "comp3v99-dec0|S9(3)V99|USAGE COMP-3|-0.0|comp3_dec|1"
 "comp3v99-dec00|S9(3)V99|USAGE COMP-3|-0.00|comp3_dec|1"
 "disp-int-0|S9(3)||-0|display|1"
 "disp-int-00|S9(3)||-00|display|1"
 "dispv99-dec0|S9(3)V99||-0.0|display|1"
 "disp-pos-0|S9(3)||0|display|1"
 "comp-int0|S9(4)|USAGE COMP|-0|binary|0"
 "comp5-int0|S9(4)|USAGE COMP-5|-0|binary|0"
 "compx-int0|S9(4)|USAGE COMP-X|-0|binary|0"
 "unsigned-comp3|9(3)|USAGE COMP-3|-0|unsigned|0"
 "unsigned-disp|9(3)||-0|unsigned|0"
)

oracle_bytes() { # pic usage_clause value
  cat > "$TMP/z.cob" <<COB
       IDENTIFICATION DIVISION.
       PROGRAM-ID. Z.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 REC.
          05 F PIC $1 $2 VALUE $3.
       PROCEDURE DIVISION.
           DISPLAY REC WITH NO ADVANCING.
           STOP RUN.
COB
  if cobc -free -x -o "$TMP/z" "$TMP/z.cob" 2>/dev/null; then "$TMP/z" | od -An -tx1 | tr -d ' \n'; else echo "REJECT"; fi
}

: > "$TMP/rows.txt"
{ printf '{'; first=1
  for cell in "${CELLS[@]}"; do
    IFS='|' read -r lbl pic uc val grp rc <<< "$cell"
    ob="$(oracle_bytes "$pic" "$uc" "$val")"
    [ $first -eq 1 ] || printf ','; printf '"%s":"%s"' "$lbl" "$ob"; first=0
    if [ "$rc" = "1" ]; then
      uchar="D"; [[ "$uc" == *COMP-3* ]] && uchar="C"
      echo "$lbl|01:REC::G:|05:F:$pic:$uchar:N$val" >> "$TMP/rows.txt"
    fi
  done
  printf '}'; } > "$TMP/oracle.json"
"$VR" < "$TMP/rows.txt" > "$TMP/rust.txt"

( cd "$ROOT" && ROOT="$ROOT" TMP="$TMP" cargo run -q -p xtask -- atlas-negzero "${CELLS[@]}" )
