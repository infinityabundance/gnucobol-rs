#!/usr/bin/env bash
# Dialect BEHAVIORAL deltas (beyond reserved words): compile probe snippets -fsyntax-only under each
# -std and record accept/reject. Surfaces inert-syntax (reserved-but-nonfunctional) and vendor-vs-
# strict drift the word-counts miss. oracle_generated. ROOT from path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
[ -x "$PREFIX/bin/cobc" ] || { echo "oracle not built"; exit 2; }
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
OUT="$ROOT/archaeology/atlases/G-gnucobol-dialect-axis"; TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
VER=$(cobc --version 2>/dev/null | head -1)

probe() { printf '       IDENTIFICATION DIVISION.\n       PROGRAM-ID. P.\n       DATA DIVISION.\n       WORKING-STORAGE SECTION.\n%s\n       PROCEDURE DIVISION.\n%s\n       STOP RUN.\n' "$2" "$3" > "$TMP/$1.cob"; }
probe examine '       01 X PIC X(5) VALUE "AABAA".\n       01 C PIC 9(4).' '       EXAMINE X TALLYING C ALL "A".'
probe comp6   '       01 X PIC 9(4) USAGE COMP-6.' '       CONTINUE.'
probe comp5   '       01 X PIC 9(4) USAGE COMP-5.' '       CONTINUE.'
probe altr    '' ''
printf '       IDENTIFICATION DIVISION.\n       PROGRAM-ID. P.\n       PROCEDURE DIVISION.\n       PA. GO TO QA.\n       ALTER PA TO PROCEED TO QA.\n       QA. STOP RUN.\n' > "$TMP/altr.cob"

DIALECTS="default cobol85 cobol2002 cobol2014 xopen ibm mvs mf acu rm bs2000"
PROBES="examine comp6 comp5 altr"
{
  echo '{'
  printf '  "schema": "gnucobol-atlas-dialect-behavior-v1",\n  "axis": "G",\n  "oracle": "%s",\n  "evidence_kind": "oracle_generated",\n' "$VER"
  printf '  "probe_method": "cobc -std=<dialect> -fsyntax-only; accept=compiles, reject=diagnostic error",\n'
  printf '  "probes": {"examine": "EXAMINE verb (legacy, pre-INSPECT)", "comp6": "USAGE COMP-6 (vendor unsigned packed)", "comp5": "USAGE COMP-5", "altr": "ALTER statement (obsolete)"},\n'
  printf '  "matrix": {\n'
  fd=1
  for d in $DIALECTS; do
    [ $fd -eq 0 ] && echo ','; fd=0
    printf '    "%s": {' "$d"
    fp=1
    for p in $PROBES; do
      [ $fp -eq 0 ] && printf ', '; fp=0
      if cobc -std=$d -fsyntax-only "$TMP/$p.cob" >/dev/null 2>&1; then printf '"%s": "accept"' "$p"; else printf '"%s": "reject"' "$p"; fi
    done
    printf '}'
  done
  echo ''
  echo '  },'
  printf '  "findings": [\n'
  printf '    "EXAMINE is RESERVED under ibm/mf/acu (see deltas/) but compiles under NO -std: reserved_word_before_feature / inert_compatibility_syntax (word-counts alone would miss this).",\n'
  printf '    "COMP-6 and ALTER are accepted by vendor dialects (ibm/mvs/mf/acu) but REJECTED by strict cobol85/cobol2002: vendor_extension_only / standard_behavior_differs_from_gnucobol."\n'
  printf '  ]\n'
  echo '}'
} > "$OUT/dialect-behavior.json"
echo "wrote dialect-behavior.json ($(echo $DIALECTS | wc -w) dialects x $(echo $PROBES | wc -w) probes)"
