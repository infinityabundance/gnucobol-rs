       IDENTIFICATION DIVISION.
       PROGRAM-ID. P138.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 E PIC 99 OCCURS 3.
       01 R REDEFINES G PIC X(6).
       PROCEDURE DIVISION.
      *> REDEFINES alias over a group-OCCURS buffer: read AND write-through
           MOVE 12 TO E(1). MOVE 34 TO E(2). MOVE 56 TO E(3).
           DISPLAY "R=[" R "]".
           MOVE "999999" TO R.
           DISPLAY "E1=" E(1) " E2=" E(2) " E3=" E(3).
           STOP RUN.
