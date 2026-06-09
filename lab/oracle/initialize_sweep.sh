#!/usr/bin/env bash
# INITIALIZE byte-effect sweep (GNURUST.INITIALIZE.1). For each case: build a record, MOVE ALL "~" sentinel
# into it via a REDEFINES X view, INITIALIZE it, DISPLAY the raw bytes, and check initialize_record == the
# post-INITIALIZE bytes (which bytes are changed vs preserved).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_initialize"; ROWS="$ROOT/target/release/examples/initialize_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"

python3 - "$TMP/cases.tsv" "$TMP" <<'PY' | "$ROWS"
import subprocess, sys, os
tmp = sys.argv[2]
out_rows = []
for line in open(sys.argv[1]):
    label, reclen, lines = line.rstrip("\n").split("\t")
    decls = "\n".join("       " + d for d in lines.split("|"))
    prog = f"""       >>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. INIT.
DATA DIVISION.
WORKING-STORAGE SECTION.
01 REC.
{decls}
01 REDF REDEFINES REC PIC X({reclen}).
PROCEDURE DIVISION.
    MOVE ALL "~" TO REDF.
    INITIALIZE REC.
    DISPLAY "POST[" REDF "]".
    STOP RUN.
"""
    cob = os.path.join(tmp, "p.cob"); open(cob, "w").write(prog)
    exe = os.path.join(tmp, "p")
    r = subprocess.run(["cobc", "-free", "-x", "-o", exe, cob], capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(f"compile {label} failed: {r.stderr[:200]}\n"); continue
    o = subprocess.run([exe], capture_output=True).stdout
    m = o.find(b"POST["); 
    posthex = ""
    if m >= 0:
        s = m + 5; posthex = o[s:s+int(reclen)].hex()
    out_rows.append("\t".join([label, reclen, lines, posthex]))
print("\n".join(out_rows))
PY
