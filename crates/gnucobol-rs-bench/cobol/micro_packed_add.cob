      *> Micro: packed-decimal ADD (COMP-3 accumulator), 50_000 iterations.
      *> Output verifies the accumulated sum and the loop count.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MICRO-PACKED-ADD.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  I                 PIC 9(9).
       01  P-ACC             PIC S9(12) COMP-3 VALUE 0.
       01  P-E               PIC 9(12).
       01  I-E               PIC 9(9).
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 50000
               ADD I TO P-ACC
           END-PERFORM
           SUBTRACT 1 FROM I
           MOVE P-ACC TO P-E
           MOVE I TO I-E
           DISPLAY "PACKED-ADD-DONE " P-E " " I-E
           STOP RUN.
