      *> Micro: alphanumeric comparison (IF A = B), 50_000 iterations.
      *> B alternates between equal and unequal; matches = iters / 2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MICRO-COMPARE.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  I                 PIC 9(9).
       01  A                 PIC X(8) VALUE "ABCDEFGH".
       01  B                 PIC X(8) VALUE "ABCDEFGH".
       01  FLIP              PIC 9 VALUE 0.
       01  MATCH-N           PIC 9(9) VALUE 0.
       01  M-E               PIC 9(9).
       01  I-E               PIC 9(9).
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 50000
               IF FLIP = 0
                   MOVE "ABCDEFGH" TO B
                   MOVE 1 TO FLIP
               ELSE
                   MOVE "ABCDEFGX" TO B
                   MOVE 0 TO FLIP
               END-IF
               IF A = B
                   ADD 1 TO MATCH-N
               END-IF
           END-PERFORM
           SUBTRACT 1 FROM I
           MOVE MATCH-N TO M-E
           MOVE I TO I-E
           DISPLAY "COMPARE-DONE " M-E " " I-E
           STOP RUN.
