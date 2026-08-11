      *> Micro: binary ADD (COMP accumulator), 50_000 iterations.
      *> Output verifies the accumulated sum and the loop count.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MICRO-BINARY-ADD.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  I                 PIC 9(9).
       01  B-ACC             PIC S9(12) COMP VALUE 0.
       01  B-E               PIC 9(12).
       01  I-E               PIC 9(9).
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 50000
               ADD I TO B-ACC
           END-PERFORM
           SUBTRACT 1 FROM I
           MOVE B-ACC TO B-E
           MOVE I TO I-E
           DISPLAY "BINARY-ADD-DONE " B-E " " I-E
           STOP RUN.
