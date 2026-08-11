      *> Micro: decimal MOVE (display -> display), 50_000 iterations.
      *> Loop body is pure MOVE; output verifies the loop count and the
      *> final moved value (deterministic, byte-exact).
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MICRO-MOVE.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  I                 PIC 9(9).
       01  W                 PIC 9(9)V99.
       01  V                 PIC 9(9)V99.
       01  V-E               PIC 9(11).
       01  I-E               PIC 9(9).
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 50000
               MOVE I TO W
               MOVE W TO V
           END-PERFORM
           SUBTRACT 1 FROM I
           MOVE V TO V-E
           MOVE I TO I-E
           DISPLAY "MOVE-DONE " V-E " " I-E
           STOP RUN.
