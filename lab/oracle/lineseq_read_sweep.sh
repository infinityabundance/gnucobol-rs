#!/usr/bin/env bash
# Line-sequential READ config-matrix sweep (GNURUST.FILEIO.LINESEQ.2). OPEN INPUT + READ NEXT over a
# LINE SEQUENTIAL file under the COB_LS_* matrix, logging FILE STATUS[2] + the 8-byte record area per
# READ, and check fileio::read_line_sequential == the oracle's per-read rows. PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/lineseq_read_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP" || exit 2

# READ harness: OPEN INPUT a LINE SEQUENTIAL file, READ NEXT repeatedly, log status+record (10 bytes/row).
cat > r.cob <<'COB'
>>SOURCE FORMAT FREE
IDENTIFICATION DIVISION.
PROGRAM-ID. R.
ENVIRONMENT DIVISION.
INPUT-OUTPUT SECTION.
FILE-CONTROL.
    SELECT FL ASSIGN "in.dat" ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS ST.
    SELECT FO ASSIGN "log.dat" ORGANIZATION IS RECORD SEQUENTIAL.
DATA DIVISION.
FILE SECTION.
FD FL.
01 RL PIC X(8).
FD FO.
01 ROW.
   05 R-ST PIC XX.
   05 R-DATA PIC X(8).
WORKING-STORAGE SECTION.
01 ST PIC XX.
01 DONE PIC X VALUE "N".
PROCEDURE DIVISION.
    OPEN INPUT FL. OPEN OUTPUT FO.
    PERFORM UNTIL DONE = "Y"
        READ FL NEXT RECORD AT END MOVE "Y" TO DONE
          NOT AT END MOVE ST TO R-ST MOVE RL TO R-DATA WRITE ROW END-READ
        IF ST = "10" MOVE "Y" TO DONE END-IF
    END-PERFORM.
    CLOSE FL FO. STOP RUN.
COB
cobc -free -x -o r r.cob 2>e || { echo "compile read harness failed"; cat e; exit 2; }

emit() { # tag  filehex  [env...]
  local tag="$1" fh="$2"; shift 2
  printf '%s' "$fh" | xxd -r -p > in.dat
  rm -f log.dat; env "$@" ./r >/dev/null 2>&1
  printf '%s=%s\n' "$tag" "$(xxd -p log.dat 2>/dev/null | tr -d '\n')"
}

{
  emit "basic"        "41420a43440a"
  emit "crlf"         "41420d0a43440a"
  emit "lonecr_def"   "410d420a"
  emit "lonecr_plain" "410d420a" COB_LS_VALIDATE=0
  emit "long_split"   "4142434445464748494a0a"
  emit "long_nosplit" "4142434445464748494a0a" COB_LS_SPLIT=0
  emit "exact8"       "41424344454647480a"
  emit "nulls"        "4100094200094200000a" COB_LS_VALIDATE=0 COB_LS_NULLS=1
  emit "tab_def"      "4109420a"
  emit "plain_ctrl"   "4109420a" COB_LS_VALIDATE=0
  emit "mid_empty"    "41420a0a43440a"
  emit "no_trail"     "4142"
} | "$ROWS"
