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

ROOT="$ROOT" TMP="$TMP" python3 - "${CELLS[@]}" <<'PY'
import os, sys, json
ROOT, TMP = os.environ["ROOT"], os.environ["TMP"]
orc = json.load(open(os.path.join(TMP, "oracle.json")))
rust = {}
for ln in open(os.path.join(TMP, "rust.txt")):
    p = ln.split()
    if len(p) == 2: rust[p[0]] = p[1].lower()

cells = []
for spec in sys.argv[1:]:
    lbl, pic, uc, val, grp, rc = spec.split("|")
    o = orc.get(lbl, ""); r = rust.get(lbl) if rc == "1" else None
    has_dp = "." in val
    lit_scale = len(val.split(".", 1)[1]) if has_dp else 0
    field_scale = len(pic.split("V", 1)[1].replace("9", "9")) if "V" in pic else 0
    field_scale = (pic.count("9", pic.index("V")) if "V" in pic else 0)
    usage = "COMP-3" if "COMP-3" in uc else ("COMP-X" if "COMP-X" in uc else
            ("COMP-5" if "COMP-5" in uc else ("COMP" if "COMP" in uc else "DISPLAY")))
    signed = pic.startswith("S")
    classification = "match" if (r is not None and r == o) else (
        "known_diverge" if (r is not None and r != o) else
        ("compile_reject" if o == "REJECT" else "oracle_only"))
    cells.append({"cell": lbl, "group": grp, "literal_text": val, "literal_has_decimal_point": has_dp,
                  "literal_scale": lit_scale, "field_pic": pic, "field_scale": field_scale,
                  "usage": usage, "signedness": "signed" if signed else "unsigned",
                  "oracle_status": "reject" if o == "REJECT" else "ok", "oracle_hex": o,
                  "rust_hex": r, "classification": classification, "rust_compared": rc == "1"})

C = {c["cell"]: c for c in cells}
def g(group): return [c for c in cells if c["group"] == group]
asserts = []
def A(name, ok, detail): asserts.append({"name": name, "pass": bool(ok), "detail": detail})

A("oracle_comp3_integer_negzero_canonicalizes_positive",
  all(c["oracle_hex"].endswith("c") for c in g("comp3_int") if c["literal_text"].lstrip("-+").rstrip("0") == "" ),
  "COMP-3 integer-form -0/-00/-000 -> sign 0C")
A("oracle_comp3_decimal_negzero_preserves_negative",
  all(c["oracle_hex"].endswith("d") for c in g("comp3_dec")),
  "COMP-3 decimal-form -0.0/-0.00 -> sign 0D even in V99")
A("oracle_display_negzero_preserves_overpunch",
  all(c["oracle_hex"].endswith("70") for c in g("display") if c["literal_text"] != "0"),
  "DISPLAY any-form -0 -> trailing overpunch 0x70")
A("oracle_binary_negzero_collapses_to_zero_bytes",
  all(set(c["oracle_hex"]) <= {"0"} for c in g("binary")),
  "COMP/COMP-5/COMP-X -0 -> all-zero, no sign nibble")
A("oracle_unsigned_negzero_rejected",
  all(c["oracle_hex"] == "REJECT" for c in g("unsigned")),
  "unsigned 9(n) VALUE -0 -> compile reject")
div = [c["cell"] for c in cells if c["classification"] == "known_diverge"]
A("rust_diverges_only_comp3_integer_form_negzero",
  all(C[d]["group"] == "comp3_int" and C[d]["literal_text"] != "0" for d in div) and len(div) >= 1,
  f"divergence set = {div}")
A("rust_matches_display_negzero",
  all(c["classification"] == "match" for c in g("display")), "all DISPLAY cells match")
A("rust_matches_comp3_decimal_form_negzero",
  all(c["classification"] == "match" for c in g("comp3_dec")), "all decimal-form COMP-3 cells match")

fails = [a["name"] for a in asserts if not a["pass"]]
atlas = {
 "schema": "gnurust-negzero-edge-v2", "court": "GNURUST.VALUE.NEGZERO.EDGE.1",
 "harvested_by": "GNURUST.LINEAGE.CORPUS.20M (1056 hits)", "bounds": "GNURUST.8 (VALUE image)",
 "oracle": "cobc VALUE initial image (cobc/typeck.c + libcob packed/zoned encode)",
 "claim": "GnuCOBOL 3.2 canonicalizes signed COMP-3 VALUE integer-form negative-zero literals to the "
          "positive packed sign nibble, while preserving the negative sign for decimal-point negative-zero "
          "literals and for all DISPLAY zoned negative-zero; gnucobol-rs value_image currently diverges "
          "ONLY on the COMP-3 integer-form cells (oracle 0C vs rust 0D).",
 "discriminator": "literal form (integer vs decimal-point), NOT field scale",
 "named_assertions": asserts,
 "rust_divergence_set": div,
 "patch_scope_if_chosen": "canonicalize the PACKED sign nibble to positive ONLY when magnitude==0 AND the "
          "literal is integer-form (lit_scale==0); MUST NOT touch the zoned/DISPLAY path or decimal-form "
          "literals (a blanket parse_num fix already regressed value_sweep 391/392). After patch the COMP-3 "
          "integer cells flip known_diverge->match; DISPLAY + decimal-form MUST remain unchanged or this "
          "court goes RED.",
 "cells": cells,
}
json.dump(atlas, open(os.path.join(ROOT, "reports/negzero-edge-atlas.json"), "w"), indent=2)
print(f"PASS={len(asserts) - len(fails)} FAIL={len(fails)}")
for n in fails: print("  FAIL", n)
PY
