#!/usr/bin/env bash
# PIC oracle: for each input PIC, ask the built cobc what cob_field_attr + storage size it
# computes (the compiler's own authoritative PICTURE -> field-attribute decision), by generating
# a tiny program, running `cobc -C`, and parsing the emitted `a_N`/`f_M` definitions.
#
# This uses GENERATED C as the witness for the COMPILER'S FIELD-ATTRIBUTE COMPUTATION only — not
# as a runtime semantic claim (GNURUST.GENC.0). The storage size is cross-checkable at runtime via
# `LENGTH OF` (GNURUST.PIC oracle note).
#
# Input  (stdin): one case per line, TAB-separated:  label <TAB> pic <TAB> usage <TAB> sign
#                 usage is empty, "DISPLAY", or "COMP-3"; sign is empty or a SIGN clause
#                 e.g. "SIGN LEADING SEPARATE".
# Output (stdout): label <TAB> type <TAB> digits <TAB> scale <TAB> flags <TAB> size
# ROOT derived from script path; pinned env.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib"
export COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" COB_COPY_DIR="$PREFIX/share/gnucobol/copy"
export LC_ALL=C.UTF-8

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
SRC="$TMP/picattr.cob"

labels=()
{
  echo '>>SOURCE FORMAT IS FREE'
  echo 'IDENTIFICATION DIVISION.'
  echo 'PROGRAM-ID. PICATTR.'
  echo 'DATA DIVISION.'
  echo 'WORKING-STORAGE SECTION.'
  i=0
  while IFS=$'\t' read -r label pic usage sign; do
    [ -z "${label:-}" ] && continue
    i=$((i + 1))
    name=$(printf 'F%05d' "$i")
    labels+=("$label")
    decl="01 $name PIC $pic"
    [ -n "${usage:-}" ] && decl="$decl USAGE $usage"
    [ -n "${sign:-}" ] && decl="$decl $sign"
    echo "$decl."
  done
  echo 'PROCEDURE DIVISION.'
  # Reference every field so the compiler emits its attr+field def.
  for n in $(seq 1 "$i"); do printf 'DISPLAY F%05d.\n' "$n"; done
  echo 'STOP RUN.'
} > "$SRC"

if ! cobc -C -free -o "$TMP/out.c" "$SRC" 2>"$TMP/err"; then
  echo "cobc -C failed:" >&2
  cat "$TMP/err" >&2
  exit 2
fi

HDR="$TMP/out.c.h"
LHDR="$TMP/out.c.l.h"

# Parse attrs: a_N = {type, digits, scale, flags, NULL};
awk '
  /cob_field_attr a_[0-9]+[[:space:]]*=/ {
    name=$0; sub(/.*a_/,"a_",name); sub(/[[:space:]]*=.*/,"",name);
    body=$0; sub(/.*\{/,"",body); sub(/\}.*/,"",body); gsub(/[[:space:]]/,"",body);
    n=split(body,f,","); print name, f[1], f[2], f[3], f[4];
  }' "$HDR" > "$TMP/attrs.txt"

# Parse fields in declaration order: f_M = {size, b_K, &a_N};  /* name */
awk '
  /cob_field f_[0-9]+[[:space:]]*=/ {
    body=$0; sub(/.*\{/,"",body); sub(/\}.*/,"",body); gsub(/[[:space:]]/,"",body);
    n=split(body,f,","); size=f[1]; attr=f[3]; sub(/&/,"",attr);
    print size, attr;
  }' "$LHDR" > "$TMP/fields.txt"

# Join: fields are in declaration order == labels order.
idx=0
while read -r size attr; do
  a=$(grep -m1 "^$attr " "$TMP/attrs.txt")
  set -- $a
  type=$2; digits=$3; scale=$4; flags=$5
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "${labels[$idx]}" "$type" "$digits" "$scale" "$flags" "$size"
  idx=$((idx + 1))
done < "$TMP/fields.txt"
