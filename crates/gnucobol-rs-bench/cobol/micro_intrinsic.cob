      *> Micro: FUNCTION intrinsic dispatch (NUMVAL + INTEGER), 50_000
      *> iterations over a fixed numeric string; sum of the integer parts.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MICRO-INTRINSIC.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  I                 PIC 9(9).
       01  S                 PIC X(10).
       01  V                 PIC 9(9)V99.
       01  N                 PIC 9(9).
       01  INT-ACC           PIC S9(12) VALUE 0.
       01  A-E               PIC 9(12).
       01  I-E               PIC 9(9).
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 50000
               MOVE "001234.56" TO S
               COMPUTE V = FUNCTION NUMVAL(S)
               COMPUTE N = FUNCTION INTEGER(V)
               ADD N TO INT-ACC
           END-PERFORM
           SUBTRACT 1 FROM I
           MOVE INT-ACC TO A-E
           MOVE I TO I-E
           DISPLAY "INTRINSIC-DONE " A-E " " I-E
           STOP RUN.
