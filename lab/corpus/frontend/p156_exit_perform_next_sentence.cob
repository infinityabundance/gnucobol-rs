      *> EXIT PERFORM (break the inline PERFORM), EXIT PERFORM CYCLE (skip to its next iteration), and NEXT
      *> SENTENCE (transfer past the next period). Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P156.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 I PIC 99 VALUE 0.
       01 J PIC 99 VALUE 0.
       01 T PIC 999 VALUE 0.
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 9
              IF I = 5 EXIT PERFORM END-IF
              ADD I TO T
           END-PERFORM.
           DISPLAY "BRK T=" T " I=" I.
           MOVE 0 TO T.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 6
              COMPUTE J = FUNCTION MOD(I 2)
              IF J = 0 EXIT PERFORM CYCLE END-IF
              ADD I TO T
           END-PERFORM.
           DISPLAY "CYC T=" T.
           PERFORM 3 TIMES
              ADD 1 TO J
              PERFORM 9 TIMES
                 ADD 1 TO J
                 IF J > 100 EXIT PERFORM END-IF
                 EXIT PERFORM
              END-PERFORM
           END-PERFORM.
           DISPLAY "NEST J=" J.
           IF I > 0 NEXT SENTENCE ELSE DISPLAY "no" END-IF
           DISPLAY "skipped".
           DISPLAY "kept".
           STOP RUN.
