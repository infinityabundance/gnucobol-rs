      *> @no312: EXCEPTION-FILE formatting evolved 3.1.2 -> 3.2; port targets 3.2
      *> FUNCTION EXCEPTION-FILE -- the last I/O operation's <status><SELECT-name> ("00" before any I/O,
      *> "35<name>" after a not-found OPEN, "00<name>" after a good one). Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P83.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT BADF ASSIGN TO "p83_nonexist.dat"
              ORGANIZATION IS LINE SEQUENTIAL FILE STATUS IS FS.
       DATA DIVISION.
       FILE SECTION.
       FD BADF.
       01 BR PIC X(4).
       WORKING-STORAGE SECTION.
       01 FS PIC XX.
       PROCEDURE DIVISION.
           DISPLAY "ef0=[" FUNCTION EXCEPTION-FILE "]".
           OPEN INPUT BADF.
           DISPLAY "fs=[" FS "] ef=[" FUNCTION EXCEPTION-FILE "]".
           STOP RUN.
