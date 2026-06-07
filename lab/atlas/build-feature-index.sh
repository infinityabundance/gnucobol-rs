#!/usr/bin/env bash
# Phase 2 of the atlas (GnuCOBOL release axis): extract FEATURE CLUES from the ADMITTED 3.2 oracle —
# the full cobc --list-* tables (oracle_generated), NEWS feature mentions and testsuite coverage
# (oracle_source). All evidence is from the admitted tree; nothing is curated here. ROOT from path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"; SRC="$ROOT/lab/admit/gnucobol-3.2"
[ -x "$PREFIX/bin/cobc" ] || { echo "oracle not built"; exit 2; }
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
OUT="$ROOT/archaeology/atlases/A17-gnucobol-release-atlas"; RAW="$OUT/raw"; mkdir -p "$RAW"
VER=$(cobc --version 2>/dev/null | head -1)

# 1. Full --list-* tables (oracle_generated) -> raw/
for l in reserved intrinsics system mnemonics exceptions; do
  cobc --list-$l 2>/dev/null > "$RAW/cobc-list-$l.txt"
done
rc=$(grep -cE '^[A-Z]' "$RAW/cobc-list-reserved.txt"); ic=$(grep -cE '^[A-Z]' "$RAW/cobc-list-intrinsics.txt")
sc=$(grep -cE '^[A-Z"]' "$RAW/cobc-list-system.txt"); mc=$(grep -cE '^[A-Z]' "$RAW/cobc-list-mnemonics.txt")
ec=$(grep -cE '^[A-Z]' "$RAW/cobc-list-exceptions.txt")

# 2. Surface presence in the reserved-word table (does the WORD exist? not whether it is functional).
present() { grep -qiE "^$1( |\$|\()" "$RAW/cobc-list-reserved.txt" && echo true || echo false; }
J=$(present JSON); X=$(present XML); RPT=$(present REPORT); SCR=$(present SCREEN); OO=$(present "INVOKE")

# 3. NEWS feature mentions (oracle_source): count lines naming each surface token.
news_n() { grep -ciE "$1" "$SRC/NEWS" 2>/dev/null || echo 0; }

# 4. Testsuite coverage (oracle_source): AT_SETUP titles per category file.
declare -A cat
if [ -d "$SRC/tests/testsuite.src" ]; then
  for f in "$SRC/tests/testsuite.src"/*.at; do
    b=$(basename "$f" .at); n=$(grep -cE 'AT_SETUP' "$f" 2>/dev/null || echo 0); cat[$b]=$n
  done
fi
tstotal=0; for k in "${!cat[@]}"; do tstotal=$((tstotal + cat[$k])); done

# Emit feature-index.json
{
  echo '{'
  printf '  "schema": "gnucobol-atlas-feature-index-v1",\n  "release_id": "gnucobol-3.2",\n'
  printf '  "oracle": "%s",\n  "evidence_kind": "oracle_generated (--list-*) + oracle_source (NEWS/testsuite)",\n' "$VER"
  printf '  "lists": {"reserved": %s, "intrinsics": %s, "system_routines": %s, "mnemonics": %s, "exceptions": %s},\n' "$rc" "$ic" "$sc" "$mc" "$ec"
  printf '  "reserved_word_present": {"JSON": %s, "XML": %s, "REPORT": %s, "SCREEN": %s, "INVOKE_OO": %s},\n' "$J" "$X" "$RPT" "$SCR" "$OO"
  printf '  "news_mentions": {"JSON": %s, "XML": %s, "REPORT": %s, "SCREEN": %s, "EBCDIC": %s, "COMP-5": %s, "OCCURS": %s, "REDEFINES": %s},\n' \
    "$(news_n 'JSON')" "$(news_n 'XML')" "$(news_n 'REPORT')" "$(news_n 'SCREEN')" "$(news_n 'EBCDIC')" "$(news_n 'COMP-5')" "$(news_n 'OCCURS')" "$(news_n 'REDEFINES')"
  printf '  "testsuite_total_cases": %s,\n' "$tstotal"
  printf '  "testsuite_by_category": {'
  first=1; for k in $(echo "${!cat[@]}" | tr ' ' '\n' | sort); do
    [ $first -eq 0 ] && printf ', '; first=0; printf '"%s": %s' "$k" "${cat[$k]}"
  done
  echo '},'
  printf '  "note": "reserved_word_present means the WORD is in the table, NOT that the feature is runtime-functional (the inert-syntax distinction). news_mentions/testsuite are coverage signals from the admitted 3.2 tree. Reproduce: lab/atlas/build-feature-index.sh"\n'
  echo '}'
} > "$OUT/feature-index.json"
echo "wrote feature-index.json + raw/ (reserved=$rc intrinsics=$ic system=$sc mnemonics=$mc exceptions=$ec; testsuite=$tstotal)"
