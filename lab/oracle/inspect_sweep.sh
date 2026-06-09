#!/usr/bin/env bash
# INSPECT byte-effect sweep (GNURUST.INSPECT.1). For each case build an INSPECT statement, run it against
# cobc/libcob, capture the count receiver bytes (TALLYING) or the target bytes (REPLACING/CONVERTING) via a
# DISPLAY, and check inspect_* == the oracle bytes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_inspect"; ROWS="$ROOT/target/release/examples/inspect_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"

python3 - "$TMP/cases.tsv" "$TMP" <<'PY' | "$ROWS"
import subprocess, sys, os
tmp = sys.argv[2]; rows = []
def reg(spec):
    if spec.startswith("before:"): return f' BEFORE INITIAL "{spec[7:]}"'
    if spec.startswith("after:"):  return f' AFTER INITIAL "{spec[6:]}"'
    return ""
for line in open(sys.argv[1]):
    label, op, target, mode, a1, a2, rspec = line.rstrip("\n").split("\t")
    n = len(target)
    if op == "TALLY":
        m = {"all": f'ALL "{a1}"', "leading": f'LEADING "{a1}"', "chars": "CHARACTERS"}[mode]
        stmt = f'INSPECT T TALLYING C FOR {m}{reg(rspec)}'
        disp = 'DISPLAY "OUT[" CX "]"'
    elif op == "REPL":
        m = {"all": f'ALL "{a1}" BY "{a2}"', "leading": f'LEADING "{a1}" BY "{a2}"', "first": f'FIRST "{a1}" BY "{a2}"'}[mode]
        stmt = f'INSPECT T REPLACING {m}{reg(rspec)}'
        disp = 'DISPLAY "OUT[" T "]"'
    else:
        stmt = f'INSPECT T CONVERTING "{a1}" TO "{a2}"{reg(rspec)}'
        disp = 'DISPLAY "OUT[" T "]"'
    prog = f'''       >>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. INSP.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 T PIC X({n}).
01 C PIC 9(3).
01 CX REDEFINES C PIC X(3).
PROCEDURE DIVISION.
    MOVE "{target}" TO T. MOVE 0 TO C.
    {stmt}.
    {disp}.
    STOP RUN.
'''
    cob = os.path.join(tmp, "p.cob"); open(cob,"w").write(prog)
    exe = os.path.join(tmp, "p")
    r = subprocess.run(["cobc","-free","-x","-o",exe,cob], capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(f"compile {label} failed: {r.stderr[:200]}\n"); continue
    o = subprocess.run([exe], capture_output=True).stdout
    m = o.find(b"OUT[")
    cap = 3 if op == "TALLY" else n
    oraclehex = o[m+4:m+4+cap].hex() if m >= 0 else ""
    rows.append("\t".join([label, op, target, mode, a1, a2, rspec, oraclehex]))
print("\n".join(rows))
PY
