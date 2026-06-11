#!/usr/bin/env bash
# GNURUST.DECLARATIVES.ATLAS.1 -- observe the runtime behavior of a USE AFTER STANDARD ERROR PROCEDURE
# declarative on file I/O. DECLARATIVES are one of COBOL's most under-specified corners; this maps the
# witnessed runtime semantics under gnucobol-3.2.0-default:
#   (1) a FAILING file op invokes that file's USE declarative (OPEN missing -> status 35; CLOSE-not-open
#       -> status 42), a SUCCESSFUL op invokes nothing,
#   (2) the binding is PER-FILE (file F's errors run F's section, never G's),
#   (3) FILE STATUS is VISIBLE inside the declarative (the per-op code),
#   (4) after the declarative, execution RESUMES at the statement after the failed I/O (rc=0, normal end).
# OBSERVED court: gnucobol-rs executes NO Procedure Division and runs NO declaratives (that is the L8
# multi-statement-execution summit, NOT CLAIMED). This MAPS the surface from real cobc/libcob output.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TERM=dumb
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
cd "$TMP"
rm -f missing.dat exists.dat
cat > decl.cob <<'COB'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. DECL.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN TO "missing.dat"
               ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS FSF.
           SELECT G ASSIGN TO "exists.dat"
               ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS FSG.
       DATA DIVISION.
       FILE SECTION.
       FD F.
       01 RECF PIC X(10).
       FD G.
       01 RECG PIC X(10).
       WORKING-STORAGE SECTION.
       01 FSF PIC XX VALUE "00".
       01 FSG PIC XX VALUE "00".
       PROCEDURE DIVISION.
       DECLARATIVES.
       ERR-F SECTION.
           USE AFTER STANDARD ERROR PROCEDURE ON F.
           DISPLAY "DECL-F fs=" FSF.
       ERR-G SECTION.
           USE AFTER STANDARD ERROR PROCEDURE ON G.
           DISPLAY "DECL-G fs=" FSG.
       END DECLARATIVES.
       MAIN SECTION.
           OPEN INPUT F.
           OPEN OUTPUT G.
           MOVE "HELLO" TO RECG.
           WRITE RECG.
           CLOSE F.
           CLOSE G.
           DISPLAY "REACHED-END".
           STOP RUN.
COB
cobc -free -x -o decl decl.cob 2>/dev/null || { echo "compile failed"; exit 2; }
OUT="$(./decl 2>/dev/null)"; RC=$?

( cd "$ROOT" && OUT="$OUT" RC="$RC" cargo run -q -p xtask -- atlas-declaratives )
