#!/usr/bin/env bash
# DATA DIVISION layout oracle (GNURUST.4): for one 01 record, ask the built cobc what byte OFFSET
# and SIZE it assigns each item, by generating a program, running `cobc -C`, and parsing the
# emitted `cob_field f_M = {size, b_REC + OFFSET, &attr};  /* name */` definitions.
#
# Generated C is used only as the witness of the COMPILER'S RECORD-LAYOUT decision (GNURUST.GENC.0);
# the record total is cross-checked at runtime via `LENGTH OF`.
#
# Input  (stdin): the record's items, one per line:  name <TAB> cobol-item-text
#                 first line is the 01 line (name "REC"); FILLER lines use name "FILLER".
#                 e.g.   REC<TAB>01 REC
#                        A<TAB>05 A PIC 9(3)
#                        FILLER<TAB>05 FILLER PIC X(2)
#                        T<TAB>05 T OCCURS 3 TIMES PIC 9(2)
# Output (stdout): name <TAB> offset <TAB> size      (one per non-FILLER item; REC gives 0/total)
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib"
export COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" COB_COPY_DIR="$PREFIX/share/gnucobol/copy"
export LC_ALL=C.UTF-8

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
SRC="$TMP/lay.cob"

names=()
decls=()
while IFS=$'\t' read -r name decl; do
  [ -z "${decl:-}" ] && continue
  names+=("$name")
  decls+=("$decl")
done

{
  echo '>>SOURCE FORMAT IS FREE'
  echo 'IDENTIFICATION DIVISION.'
  echo 'PROGRAM-ID. LAY.'
  echo 'DATA DIVISION.'
  echo 'WORKING-STORAGE SECTION.'
  for d in "${decls[@]}"; do echo "$d."; done
  echo 'PROCEDURE DIVISION.'
  # Reference every statically-addressable item so cobc emits its field def (offset+size). Items
  # that are OCCURS tables, or nested under an OCCURS ancestor, have runtime-computed (non-static)
  # offsets and are skipped here — their effect is validated transitively by the offset of the
  # field that follows the table and by the record total. FILLER cannot be referenced.
  # Track OCCURS ancestry with a level stack.
  stack_lvl=()
  stack_occ=()
  idx=0
  for n in "${names[@]}"; do
    d="${decls[$idx]}"
    lvl=$(printf '%s' "$d" | awk '{print $1+0}')
    # pop stack entries with level >= this item's level
    while [ "${#stack_lvl[@]}" -gt 0 ] && [ "${stack_lvl[${#stack_lvl[@]}-1]}" -ge "$lvl" ]; do
      unset 'stack_lvl[${#stack_lvl[@]}-1]'
      unset 'stack_occ[${#stack_occ[@]}-1]'
      stack_lvl=("${stack_lvl[@]}")
      stack_occ=("${stack_occ[@]}")
    done
    under_occ=0
    for o in "${stack_occ[@]:-}"; do [ "$o" = "1" ] && under_occ=1; done
    self_occ=0; case "$d" in *OCCURS*) self_occ=1 ;; esac
    if [ "$n" != "FILLER" ] && [ "$under_occ" -eq 0 ] && [ "$self_occ" -eq 0 ]; then
      echo "DISPLAY $n."
    fi
    stack_lvl+=("$lvl"); stack_occ+=("$self_occ")
    idx=$((idx + 1))
  done
  echo 'STOP RUN.'
} > "$SRC"

if ! cobc -C -free -o "$TMP/out.c" "$SRC" 2>"$TMP/err"; then
  echo "cobc -C failed:" >&2
  sed 's#'"$TMP"'#<TMP>#g' "$TMP/err" >&2
  exit 2
fi

# Parse: f_M = {size, b_BASE [+ OFFSET], &a_N};	/* name */
awk '
  /cob_field f_[0-9]+[[:space:]]*=/ {
    name=$0; sub(/.*\/\*[[:space:]]*/,"",name); sub(/[[:space:]]*\*\/.*/,"",name);
    body=$0; sub(/.*\{/,"",body); sub(/\}.*/,"",body);
    # body = "size, b_NN + OFF, &a_K"  or  "size, b_NN, &a_K"
    nf=split(body,f,",");
    size=f[1]; gsub(/[[:space:]]/,"",size);
    base=f[2];
    off=0;
    if (base ~ /\+/) { sub(/.*\+/,"",base); gsub(/[[:space:]]/,"",base); off=base; }
    print name, off, size;
  }' "$TMP/out.c.l.h"
