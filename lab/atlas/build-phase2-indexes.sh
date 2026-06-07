#!/usr/bin/env bash
# Phase-2 named indexes (spec): feature-token-index (SOURCE scan of admitted cobc/libcob -> WHERE each
# surface lives: grammar vs codegen vs runtime module), news-feature-index (NEWS mentions), and
# testsuite-feature-index (categories + sample titles). All oracle_source from the admitted 3.2 tree.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; SRC="$ROOT/lab/admit/gnucobol-3.2"
[ -d "$SRC/cobc" ] || { echo "admitted source absent"; exit 2; }
OUT="$ROOT/archaeology/atlases/A17-gnucobol-release-atlas"
g() { local n; n=$(grep -ic "$1" "$2" 2>/dev/null); echo "${n:-0}"; }

# surface : runtime-module  (where it would be IMPLEMENTED if functional)
declare -A MOD=( [json_generate]=mlio.c [xml]=mlio.c [screen_section]=screenio.c [report_writer]=reportio.c
  [file_io]=fileio.c [call]=call.c [inspect]=strings.c [string]=strings.c [examine]=__none__
  [alter]=__none__ [comp_6]=numeric.c [intrinsic]=intrinsic.c )
declare -A TOK=( [json_generate]='json' [xml]='xml' [screen_section]='screen' [report_writer]='report'
  [file_io]='cob_open\|cob_read' [call]='cob_call' [inspect]='inspect' [string]='cob_string'
  [examine]='examine' [alter]='alter' [comp_6]='comp.6\|COMP_6' [intrinsic]='cob_intr' )

{
  echo '{'
  printf '  "schema": "gnucobol-atlas-feature-token-index-v1",\n  "release_id": "gnucobol-3.2",\n  "evidence_kind": "oracle_source",\n'
  printf '  "method": "case-insensitive token counts in the admitted cobc/parser.y (grammar), cobc/codegen.c (codegen), and the surface runtime module. grammar-only + no-runtime => recognized-not-implemented; runtime-module hits => implemented (often backend-dependent).",\n'
  printf '  "tokens": {\n'
  first=1
  for k in $(echo "${!MOD[@]}" | tr ' ' '\n' | sort); do
    tok="${TOK[$k]}"; mod="${MOD[$k]}"
    gp=$(g "$tok" "$SRC/cobc/parser.y"); gc=$(g "$tok" "$SRC/cobc/codegen.c")
    if [ "$mod" = "__none__" ]; then gm=0; modname="(none)"; else gm=$(g "$tok" "$SRC/libcob/$mod"); modname="$mod"; fi
    if [ "$gm" -gt 0 ] && [ "$gc" -gt 0 ]; then cls="implemented"; elif [ "$gp" -gt 0 ] && [ "$gm" -eq 0 ]; then cls="recognized_grammar_only"; elif [ "$gm" -gt 0 ]; then cls="runtime_module_present"; else cls="absent_or_minimal"; fi
    [ $first -eq 0 ] && echo ','; first=0
    printf '    "%s": {"parser_y": %s, "codegen_c": %s, "runtime_module": "%s", "module_hits": %s, "classification": "%s"}' "$k" "$gp" "$gc" "$modname" "$gm" "$cls"
  done
  echo ''
  printf '  },\n  "note": "EXAMINE in parser.y but compiles under NO -std (see G/dialect-behavior.json) = grammar-present yet dialect-gated/obsolete: a sharp recognized-not-functional case."\n'
  echo '}'
} > "$OUT/feature-token-index.json"

# news-feature-index
{
  echo '{'
  printf '  "schema": "gnucobol-atlas-news-feature-index-v1",\n  "release_id": "gnucobol-3.2",\n  "evidence_kind": "oracle_source (NEWS)",\n  "mentions": {'
  first=1
  for t in JSON XML REPORT SCREEN EBCDIC COMP-5 COMP-6 OCCURS REDEFINES SYNCHRONIZED INSPECT EXAMINE ALTER; do
    n=$(g "$t" "$SRC/NEWS"); [ $first -eq 0 ] && printf ', '; first=0; printf '"%s": %s' "$t" "$n"
  done
  printf '},\n  "note": "Count of NEWS lines mentioning each token in the admitted 3.2 NEWS file."\n}\n'
} > "$OUT/news-feature-index.json"

# testsuite-feature-index (categories + sample titles)
{
  echo '{'
  printf '  "schema": "gnucobol-atlas-testsuite-feature-index-v1",\n  "release_id": "gnucobol-3.2",\n  "evidence_kind": "oracle_source (testsuite)",\n  "categories": {'
  first=1
  for f in "$SRC/tests/testsuite.src"/*.at; do
    b=$(basename "$f" .at); n=$(grep -cE 'AT_SETUP' "$f" 2>/dev/null || echo 0)
    [ $first -eq 0 ] && printf ', '; first=0; printf '"%s": %s' "$b" "$n"
  done
  printf '},\n  "data_domain_sample_titles": ['
  first=1
  for cat in data_packed data_binary data_display syn_redefines syn_occurs; do
    while IFS= read -r t; do [ $first -eq 0 ] && printf ', '; first=0; printf '"%s: %s"' "$cat" "$(echo "$t" | tr -d '"' | cut -c1-60)"; done < <(grep -hoE 'AT_SETUP\(\[[^]]*' "$SRC/tests/testsuite.src/$cat.at" 2>/dev/null | sed 's/AT_SETUP(\[//' | head -3)
  done
  printf ']\n}\n'
} > "$OUT/testsuite-feature-index.json"
echo "wrote feature-token-index.json + news-feature-index.json + testsuite-feature-index.json"
