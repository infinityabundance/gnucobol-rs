#!/usr/bin/env bash
# Sequential-file READ byte+status sweep (GNURUST.FILE.SEQUENTIAL.1). For each case: write the input file,
# run the org-specific cobc program (OPEN INPUT / READ NEXT / AT END), capture each record's raw bytes + file
# status, and check read_sequential == the oracle sequence.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_seqfile"; ROWS="$ROOT/target/release/examples/seqfile_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"

# one reader program per organization (the ORGANIZATION clause is compile-time)
prog() {
cat > "$TMP/$1.cob" <<COB
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. RDR.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT F ASSIGN TO "DDIN" ORGANIZATION IS $2 FILE STATUS IS WS.
DATA DIVISION.
FILE SECTION.
FD F.
01 R PIC X(8).
WORKING-STORAGE SECTION.
01 WS PIC XX.
01 EOFF PIC X VALUE "N".
PROCEDURE DIVISION.
    OPEN INPUT F.
    PERFORM UNTIL EOFF = "Y"
        READ F NEXT RECORD
            AT END DISPLAY "END[" WS "]" MOVE "Y" TO EOFF
            NOT AT END DISPLAY "REC[" R "][" WS "]"
        END-READ
    END-PERFORM.
    CLOSE F. STOP RUN.
COB
cobc -free -x -o "$TMP/$1" "$TMP/$1.cob" 2>"$TMP/$1.err" || { echo "compile $1 failed"; cat "$TMP/$1.err"; exit 2; }
}
prog rec "RECORD SEQUENTIAL"
prog lin "LINE SEQUENTIAL"

# per case: write file, run the org program, extract events, append to the row
python3 - "$TMP/cases.tsv" "$TMP" <<'PY' | "$ROWS"
import subprocess, sys, os
tmp = sys.argv[2]
for line in open(sys.argv[1]):
    label, org, reclen, filehex = line.rstrip("\n").split("\t")
    reclen = int(reclen)
    fb = bytes.fromhex(filehex)
    fp = os.path.join(tmp, "in.dat"); open(fp, "wb").write(fb)
    prog = os.path.join(tmp, "rec" if org == "RECORD" else "lin")
    env = dict(os.environ, DDIN=fp)
    out = subprocess.run([prog], capture_output=True, env=env).stdout
    # parse REC[<reclen bytes>][<2>] and END[<2>] events in order
    events = []
    i = 0
    while i < len(out):
        if out[i:i+4] == b"REC[":
            rec = out[i+4:i+4+reclen]; j = i+4+reclen
            assert out[j:j+2] == b"][", (label, out[i:i+40])
            st = out[j+2:j+4]
            events.append("R:" + rec.hex() + ":" + st.decode())
            i = j+5
        elif out[i:i+4] == b"END[":
            st = out[i+4:i+6]; events.append("E:" + st.decode()); i = i+7
        else:
            i += 1
    print("\t".join([label, org, str(reclen), filehex, ";".join(events)]))
PY
