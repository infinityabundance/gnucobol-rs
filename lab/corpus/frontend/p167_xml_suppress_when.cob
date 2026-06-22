      *> XML GENERATE ... SUPPRESS id WHEN {ZERO|SPACE|...} -- the element is omitted only when its value
      *> matches the figurative (JSON GENERATE rejects SUPPRESS WHEN -- a cobc compile error). Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P167.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 A PIC 99 VALUE 0.
          05 B PIC 99 VALUE 7.
          05 C PIC 99 VALUE 0.
       01 R PIC X(60).
       PROCEDURE DIVISION.
           XML GENERATE R FROM G SUPPRESS A WHEN ZERO.
           DISPLAY "T1=[" FUNCTION TRIM(R) "]".
           MOVE 5 TO A.
           XML GENERATE R FROM G SUPPRESS A WHEN ZERO C WHEN ZERO.
           DISPLAY "T2=[" FUNCTION TRIM(R) "]".
           STOP RUN.
