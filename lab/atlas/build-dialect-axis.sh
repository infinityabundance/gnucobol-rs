#!/usr/bin/env bash
# Generate the GnuCOBOL -std DIALECT axis (G-axis) of the COBOL Atlas from the ADMITTED ORACLE only
# (real, reproducible evidence — not curated prose). For each -std dialect it captures reserved-word /
# intrinsic / system-routine counts and the reserved-word delta vs `default`. ROOT from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
[ -x "$PREFIX/bin/cobc" ] || { echo "oracle not built"; exit 2; }
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
OUT="$ROOT/archaeology/atlases/G-gnucobol-dialect-axis"
mkdir -p "$OUT"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

VER=$(cobc --version 2>/dev/null | head -1)
cobc -std=default --list-reserved 2>/dev/null | grep -oE '^[A-Z][A-Z0-9-]*' | sort -u > "$TMP/default.rw"

DIALECTS="default cobol2014 cobol2002 cobol85 xopen ibm-strict ibm mvs-strict mvs mf-strict mf bs2000-strict bs2000 acu-strict acu rm-strict rm"
{ echo "# G-axis reserved-word delta vs default — generated from the admitted oracle ($VER)"
  printf 'dialect\treserved\tintrinsics\tsystem\tonly_in_dialect\tonly_in_default\n'; } > "$OUT/reserved-deltas.tsv"

json="$OUT/dialect-axis.json"
{
  echo '{'
  printf '  "schema": "gnucobol-atlas-dialect-axis-v1",\n  "axis": "G",\n'
  printf '  "oracle": "%s",\n' "$(echo "$VER" | sed 's/"/\\"/g')"
  printf '  "evidence_kind": "oracle_generated",\n'
  printf '  "note": "Counts and reserved-word deltas are produced by the admitted cobc -std=<dialect>. Reproduce with lab/atlas/build-dialect-axis.sh.",\n'
  printf '  "dialects": [\n'
  first=1
  for d in $DIALECTS; do
    cobc -std=$d --list-reserved 2>/dev/null | grep -oE '^[A-Z][A-Z0-9-]*' | sort -u > "$TMP/$d.rw"
    rw=$(wc -l < "$TMP/$d.rw" | tr -d ' ')
    intr=$(cobc -std=$d --list-intrinsics 2>/dev/null | grep -cE '^[A-Z]')
    sysr=$(cobc -std=$d --list-system 2>/dev/null | grep -cE '^[A-Z"]')
    only_d=$(comm -23 "$TMP/$d.rw" "$TMP/default.rw" | wc -l | tr -d ' ')
    only_def=$(comm -13 "$TMP/$d.rw" "$TMP/default.rw" | wc -l | tr -d ' ')
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$d" "$rw" "$intr" "$sysr" "$only_d" "$only_def" >> "$OUT/reserved-deltas.tsv"
    [ $first -eq 0 ] && echo '    },'
    first=0
    printf '    {"id": "%s", "reserved_words": %s, "intrinsics": %s, "system_routines": %s, "reserved_only_in_dialect": %s, "reserved_missing_vs_default": %s' "$d" "$rw" "$intr" "$sysr" "$only_d" "$only_def"
  done
  echo '    }'
  echo '  ]'
  echo '}'
} > "$json"
echo "wrote $json + reserved-deltas.tsv ($(echo "$DIALECTS" | wc -w) dialects)"

# Actual extension WORD lists each vendor dialect adds vs default (the hidden surface, not just counts).
DD="$OUT/deltas"; mkdir -p "$DD"
for d in ibm mvs mf acu rm bs2000; do
  comm -23 "$TMP/$d.rw" "$TMP/default.rw" > "$DD/$d-adds-vs-default.txt"
done
# Words reserved across mf AND ibm AND acu but NOT default (cross-vendor legacy extensions).
comm -12 <(sort "$DD/mf-adds-vs-default.txt") <(sort "$DD/ibm-adds-vs-default.txt")   | comm -12 - <(sort "$DD/acu-adds-vs-default.txt") > "$DD/cross-vendor-not-default.txt"
echo "wrote deltas/ ($(wc -l < "$DD/cross-vendor-not-default.txt" | tr -d ' ') cross-vendor words)"
