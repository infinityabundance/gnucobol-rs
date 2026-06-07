#!/usr/bin/env bash
# Dialect DATA-LAYOUT delta: binary storage config (binary-size / byteorder / truncate) per -std,
# resolved through the dialect .conf 'include' chain. oracle_source (admitted config). The same COMP
# copybook can decode to DIFFERENT byte widths per dialect -- a read-fidelity scar. ROOT from path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; CONF="$ROOT/lab/oracle/prefix/share/gnucobol/config"
[ -d "$CONF" ] || { echo "oracle config absent"; exit 2; }
OUT="$ROOT/archaeology/atlases/G-gnucobol-dialect-axis"
val() { # key dialectconf -> value, following one include
  local k="$1" f="$CONF/$2.conf" v inc
  [ -f "$f" ] || { echo "?"; return; }
  v=$(grep -E "^$k:" "$f" | head -1 | awk '{print $2}')
  if [ -z "$v" ]; then inc=$(grep -E '^include ' "$f" | head -1 | awk '{print $2}' | tr -d '"'); [ -n "$inc" ] && v=$(grep -E "^$k:" "$CONF/$inc" 2>/dev/null | head -1 | awk '{print $2}'); fi
  echo "${v:-?}"
}
DIALECTS="default cobol85 cobol2002 cobol2014 xopen ibm mvs mf acu rm bs2000 gcos realia"
{
  echo '{'
  printf '  "schema": "gnucobol-atlas-dialect-layout-v1",\n  "axis": "G",\n  "evidence_kind": "oracle_source (dialect .conf)",\n'
  printf '  "note": "binary storage config per -std dialect, resolved through the .conf include chain. data_layout_delta: COMP/binary field WIDTHS differ by dialect (1-2-4-8 vs 2-4-8 vs 1--8), so the SAME copybook decodes to different bytes. gnucobol-rs GNURUST.14 sealed the default/cobol85 table (1-2-4-8).",\n'
  printf '  "dialects": {\n'
  first=1
  for d in $DIALECTS; do
    [ $first -eq 0 ] && echo ','; first=0
    printf '    "%s": {"binary_size": "%s", "binary_byteorder": "%s", "binary_truncate": "%s"}' "$d" "$(val binary-size $d)" "$(val binary-byteorder $d)" "$(val binary-truncate $d)"
  done
  echo ''
  printf '  },\n  "findings": [\n'
  printf '    "binary-size differs: default/cobol85/cobol2002/acu = 1-2-4-8; ibm/mvs/rm/bs2000 = 2-4-8 (NO 1-byte: PIC 9(1-4) COMP is 2 bytes, not 1-2); mf = 1--8 (every width 1-8).",\n'
  printf '    "Migration scar: the same `PIC 9(2) COMP` field is 1 byte under default but 2 bytes under IBM/MVS. A copybook alone does not fix binary layout -- the source dialect does.",\n'
  printf '    "gnucobol-rs non-claim refinement: GNURUST.14 binary sizes are the default/cobol85 1-2-4-8 table; IBM/MVS (2-4-8) and MF (1--8) binary layouts are NOT claimed."\n'
  printf '  ]\n}\n'
} > "$OUT/dialect-layout.json"
echo "wrote dialect-layout.json"
