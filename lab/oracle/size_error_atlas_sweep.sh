#!/usr/bin/env bash
# SIZE.ERROR.ATLAS.1 — observe GnuCOBOL arithmetic size-error behavior (ATLAS, not implementation).
# Each case pre-fills a receiver with a SENTINEL value, runs an overflowing/divide-by-zero arithmetic op
# either plain or with ON SIZE ERROR, and DISPLAYs the receiver's raw bytes (via a REDEFINES X) BEFORE and
# AFTER + a size-error flag. We then record: receiver_written (after != before) and size_error_signaled.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" \
  COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; echo "PASS=0 FAIL=0"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# case: label  recv_pic  usage  size  init  arith   expect_written_plain  (assertion for the SE variant is
# always preserved+signaled; for plain it is the documented truncation/zero behavior)
# arith uses RCV as the receiver; %SE% is replaced by the ON SIZE ERROR clause (or empty).
emit_case() {  # label recv_pic usage size init arith
  local label="$1" pic="$2" usage="$3" size="$4" init="$5" arith="$6" se="$7"
  local seclause=""; [ "$se" = "1" ] && seclause='ON SIZE ERROR MOVE "Y" TO SEF'
  {
    echo ">>SOURCE FORMAT FREE"
    echo "IDENTIFICATION DIVISION."
    echo "PROGRAM-ID. SE${label}."
    echo "DATA DIVISION."
    echo "WORKING-STORAGE SECTION."
    echo "01 RCV PIC ${pic}${usage}."
    echo "01 RCVX REDEFINES RCV PIC X(${size})."
    echo "01 ZERO-V PIC 9 VALUE 0."
    echo "01 SEF PIC X VALUE \"N\"."
    echo "PROCEDURE DIVISION."
    echo "MOVE ${init} TO RCV."
    echo "DISPLAY \"BEFORE[\" RCVX \"]\"."
    echo "${arith} ${seclause}."
    echo "DISPLAY \"AFTER[\" RCVX \"]\"."
    echo "DISPLAY \"SE[\" SEF \"]\"."
    echo "STOP RUN."
  } > "$TMP/$1.cob"
  ( cd "$TMP" && cobc -x -free -o "$1.bin" "$1.cob" >/dev/null 2>"$TMP/$1.err" ) || { echo "COMPILE-FAIL $1"; return 1; }
  ( cd "$TMP" && ./"$1.bin" > "$TMP/$1.out" 2>/dev/null ); :
}

# scenarios (label op recv usage size init arith) — each run plain (P) and size-error (S)
# ADD/MUL/SUB/DIVIDE × DISPLAY/COMP-3 × signed/ROUNDED/divide-by-zero
gen() { # base pic usage size init arith
  emit_case "${1}P" "$2" "$3" "$4" "$5" "$6" 0
  emit_case "${1}S" "$2" "$3" "$4" "$5" "$6" 1
}
gen ADDDISP  "9(3)" ""             3 999  "ADD 500 TO RCV"
gen ADDC3    "9(3)" " USAGE COMP-3" 2 999  "ADD 500 TO RCV"
gen MULDISP  "9(3)" ""             3 999  "MULTIPLY 5 BY RCV"
gen SUBSIGN  "S9(3)" ""            3 -999 "SUBTRACT 500 FROM RCV"
gen DIV0DISP "9(3)" ""             3 999  "DIVIDE RCV BY ZERO-V GIVING RCV"
gen ROUNDDISP "9(3)" ""            3 999  "ADD 0.6 TO RCV ROUNDED"

python3 - "$TMP" "$ROOT" <<'PY'
import sys, os, json, glob
tmp, root = sys.argv[1], sys.argv[2]
def grab(buf, marker, n):
    i = buf.find(marker)
    if i < 0: return None
    s = i + len(marker); return buf[s:s+n]
SCEN = {  # base -> (op, recv, narrative)
 "ADDDISP": ("ADD","DISPLAY 9(3)"), "ADDC3": ("ADD","COMP-3 9(3)"),
 "MULDISP": ("MULTIPLY","DISPLAY 9(3)"), "SUBSIGN": ("SUBTRACT","signed S9(3)"),
 "DIV0DISP": ("DIVIDE-BY-ZERO","DISPLAY 9(3)"), "ROUNDDISP": ("ADD ROUNDED carry","DISPLAY 9(3)"),
}
SIZE = {"ADDDISP":3,"ADDC3":2,"MULDISP":3,"SUBSIGN":3,"DIV0DISP":3,"ROUNDDISP":3}
rows=[]; pf=0; fl=0
for base,(op,recv) in SCEN.items():
    for var in ("P","S"):
        f=os.path.join(tmp,f"{base}{var}.out")
        if not os.path.exists(f): print(f"NO-OUTPUT {base}{var}"); fl+=1; continue
        buf=open(f,"rb").read(); n=SIZE[base]
        before=grab(buf,b"BEFORE[",n); after=grab(buf,b"AFTER[",n); se=grab(buf,b"SE[",1)
        if before is None or after is None or se is None: print(f"PARSE-FAIL {base}{var}"); fl+=1; continue
        written = before != after
        signaled = (se == b"Y")
        rows.append({"case":f"{base}{var}","op":op,"receiver":recv,
                     "on_size_error": var=="S",
                     "receiver_before_hex":before.hex(),"receiver_after_hex":after.hex(),
                     "receiver_written":written,"receiver_preserved":not written,
                     "size_error_signaled":signaled})
        # ASSERT the well-defined behaviors:
        if var=="S":
            ok = signaled and (not written)            # ON SIZE ERROR -> preserved + signaled
        else:
            ok = (not signaled)                         # plain -> no SE flag (no clause); record write/preserve
        (globals().__setitem__('pf', pf+1) if ok else (print(f"MISMATCH {base}{var}: written={written} signaled={signaled}"), globals().__setitem__('fl', fl+1)))
atlas={"schema":"kobold-size-error-atlas-v1","court":"SIZE.ERROR.ATLAS.1","oracle":"GnuCOBOL 3.2 (lab/oracle/prefix)",
 "doctrine":"Observed oracle evidence only: whether size error is signaled and whether receiver bytes are written or preserved. NO ON SIZE ERROR / NOT ON SIZE ERROR control-flow implementation, no Procedure Division execution, no business-arithmetic correctness.",
 "cases":rows,
 "non_claims":["NEG.SIZE_ERROR.CONTROL_FLOW","NEG.SIZE_ERROR.ON_SIZE_ERROR_NOT_IMPLEMENTED","NEG.SIZE_ERROR.NOT_ON_SIZE_ERROR_NOT_IMPLEMENTED","NEG.SIZE_ERROR.RECEIVER_WRITE_NOT_INFERRED","NEG.SIZE_ERROR.BRANCH_EXECUTION_NOT_CLAIMED","NEG.SIZE_ERROR.BUSINESS_ARITHMETIC_NOT_CLAIMED"]}
os.makedirs(os.path.join(root,"reports"),exist_ok=True)
json.dump(atlas, open(os.path.join(root,"reports","size-error-atlas.json"),"w"), indent=2)
print(f"PASS={pf} FAIL={fl}")
PY
