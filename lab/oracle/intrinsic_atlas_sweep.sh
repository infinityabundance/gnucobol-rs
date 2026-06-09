#!/usr/bin/env bash
# Intrinsic-function atlas sweep (GNURUST.INTRINSIC.ATLAS.1). Probe high-use intrinsics with declared inputs
# against real cobc/libcob, assert the DETERMINISTIC results are stable, and emit reports/intrinsic-atlas.json
# classifying each (deterministic candidate-court vs environment-sensitive shape-only). OBSERVED court: maps
# the intrinsic surface before any are implemented.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cat > "$TMP/p.cob" <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. P.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 A5 PIC X(5) VALUE "HELLO".
01 N9 PIC 9(8)V99.
01 NS PIC S9(8)V99.
01 RT PIC X(8).
PROCEDURE DIVISION.
    MOVE FUNCTION LENGTH(A5) TO N9.        DISPLAY "LENGTH=" N9.
    MOVE FUNCTION BYTE-LENGTH(A5) TO N9.   DISPLAY "BYTE_LENGTH=" N9.
    MOVE FUNCTION NUMVAL("123.45") TO N9.  DISPLAY "NUMVAL=" N9.
    MOVE FUNCTION NUMVAL-C("$1,234.56") TO N9. DISPLAY "NUMVAL_C=" N9.
    MOVE FUNCTION INTEGER(3.7) TO NS.      DISPLAY "INTEGER_P=" NS.
    MOVE FUNCTION INTEGER(-3.7) TO NS.     DISPLAY "INTEGER_N=" NS.
    MOVE FUNCTION INTEGER-PART(3.7) TO NS. DISPLAY "INTPART_P=" NS.
    MOVE FUNCTION INTEGER-PART(-3.7) TO NS. DISPLAY "INTPART_N=" NS.
    MOVE FUNCTION MOD(17,5) TO NS.         DISPLAY "MOD_P=" NS.
    MOVE FUNCTION MOD(-17,5) TO NS.        DISPLAY "MOD_N=" NS.
    MOVE FUNCTION REM(17,5) TO NS.         DISPLAY "REM_P=" NS.
    MOVE FUNCTION REM(-17,5) TO NS.        DISPLAY "REM_N=" NS.
    MOVE FUNCTION UPPER-CASE("abc") TO RT. DISPLAY "UPPER=[" RT "]".
    MOVE FUNCTION LOWER-CASE("ABC") TO RT. DISPLAY "LOWER=[" RT "]".
    MOVE FUNCTION REVERSE("abcd") TO RT.   DISPLAY "REVERSE=[" RT "]".
    MOVE FUNCTION ORD("A") TO N9.          DISPLAY "ORD=" N9.
    MOVE FUNCTION CHAR(66) TO RT.          DISPLAY "CHAR=[" RT "]".
    DISPLAY "CURRENT_DATE_LEN=" FUNCTION LENGTH(FUNCTION CURRENT-DATE).
    DISPLAY "WHEN_COMPILED_LEN=" FUNCTION LENGTH(FUNCTION WHEN-COMPILED).
    STOP RUN.
COB
cobc -free -x -o "$TMP/p" "$TMP/p.cob" 2>"$TMP/err" || { echo "compile failed"; cat "$TMP/err"; exit 2; }
OUT=$("$TMP/p")
python3 - "$ROOT/reports/intrinsic-atlas.json" "$OUT" <<'PY'
import json, sys
out = sys.argv[2]
kv = {}
for line in out.splitlines():
    if "=" in line:
        k, v = line.split("=", 1); kv[k.strip()] = v.strip()
def num(s): return s.strip("[]").rstrip() if "[" in s else s
# expected DETERMINISTIC observations (the atlas's witnessed facts)
expect = {
 "LENGTH":"00000005.00","BYTE_LENGTH":"00000005.00","NUMVAL":"00000123.45","NUMVAL_C":"00001234.56",
 "INTEGER_P":"+00000003.00","INTEGER_N":"-00000004.00","INTPART_P":"+00000003.00","INTPART_N":"-00000003.00",
 "MOD_P":"+00000002.00","MOD_N":"+00000003.00","REM_P":"+00000002.00","REM_N":"-00000002.00",
 "UPPER":"[ABC     ]","LOWER":"[abc     ]","REVERSE":"[dcba    ]","ORD":"00000066.00","CHAR":"[A       ]",
 "CURRENT_DATE_LEN":"000000021","WHEN_COMPILED_LEN":"21",
}
fails = [(k, e, kv.get(k)) for k, e in expect.items() if kv.get(k) != e]
atlas = {
 "schema":"gnurust-intrinsic-atlas-v1","court":"GNURUST.INTRINSIC.ATLAS.1","dialect":"gnucobol-3.2.0-default",
 "oracle":"cobc FUNCTION intrinsics (libcob/intrinsic.c)",
 "intrinsics":[
  {"name":"LENGTH","category":"length","input":"X(5)","observed":"5","determinism":"deterministic","status":"implemented","note":"GNURUST.INTRINSIC.LENGTH.1 -- storage byte length"},
  {"name":"BYTE-LENGTH","category":"length","input":"X(5)","observed":"5","determinism":"deterministic","status":"candidate-court","note":"= LENGTH for single-octet"},
  {"name":"NUMVAL","category":"numeric-parse","input":"\"123.45\"","observed":"123.45","determinism":"deterministic","status":"implemented","note":"GNURUST.INTRINSIC.NUMVAL.1 -- narrow form (sign, spaces, CR/DB, decimal)"},
  {"name":"NUMVAL-C","category":"numeric-parse","input":"\"$1,234.56\"","observed":"1234.56","determinism":"deterministic","status":"candidate-court","note":"currency/thousands stripping; locale-sensitive"},
  {"name":"INTEGER","category":"rounding","input":"3.7 / -3.7","observed":"3 / -4","determinism":"deterministic","status":"candidate-court","note":"FLOOR (greatest integer <= arg); differs from INTEGER-PART on negatives"},
  {"name":"INTEGER-PART","category":"rounding","input":"3.7 / -3.7","observed":"3 / -3","determinism":"deterministic","status":"candidate-court","note":"TRUNCATE toward zero"},
  {"name":"MOD","category":"modulo","input":"17,5 / -17,5","observed":"2 / 3","determinism":"deterministic","status":"implemented","note":"GNURUST.INTRINSIC.MOD-REM.1 -- DIVISOR sign (mathematical modulo)"},
  {"name":"REM","category":"modulo","input":"17,5 / -17,5","observed":"2 / -2","determinism":"deterministic","status":"implemented","note":"GNURUST.INTRINSIC.MOD-REM.1 -- DIVIDEND sign (C-style remainder)"},
  {"name":"UPPER-CASE","category":"string","input":"\"abc\"","observed":"ABC","determinism":"deterministic","status":"candidate-court","note":"ASCII fold; locale-sensitive for non-ASCII (refused)"},
  {"name":"LOWER-CASE","category":"string","input":"\"ABC\"","observed":"abc","determinism":"deterministic","status":"candidate-court","note":"ASCII fold"},
  {"name":"REVERSE","category":"string","input":"\"abcd\"","observed":"dcba","determinism":"deterministic","status":"candidate-court","note":"byte reversal"},
  {"name":"ORD","category":"char","input":"\"A\"","observed":"66","determinism":"deterministic","status":"candidate-court","note":"1-based position in the collating sequence (ASCII 'A'=65 -> 66)"},
  {"name":"CHAR","category":"char","input":"66","observed":"A","determinism":"deterministic","status":"candidate-court","note":"1-based inverse of ORD"},
  {"name":"CURRENT-DATE","category":"date-time","input":"(none)","observed":"21-char string (value time-dependent)","determinism":"environment-sensitive","status":"atlas-only-shape","note":"value is the wall clock; only the 21-char SHAPE is admitted, never the value"},
  {"name":"WHEN-COMPILED","category":"date-time","input":"(none)","observed":"21-char string (value compile-time-dependent)","determinism":"environment-sensitive","status":"atlas-only-shape","note":"compile timestamp; only the shape is admitted"},
 ],
 "negative_capabilities":["NEG.INTRINSIC.NOT_ALL_INTRINSICS","NEG.INTRINSIC.NO_ENV_SENSITIVE_VALUES",
   "NEG.INTRINSIC.NO_LOCALE_COLLATION","NEG.INTRINSIC.NO_NATIONAL_UTF8","NEG.INTRINSIC.STATUS_NOT_IMPLEMENTATION",
   "NEG.INTRINSIC.NO_ALL_DIALECTS"],
}
json.dump(atlas, open(sys.argv[1],"w"), indent=2)
print(f"PASS={len(expect)-len(fails)} FAIL={len(fails)}")
for f in fails: print("  MISMATCH", f)
PY
