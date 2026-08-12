
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01  CON          CONSTANT 10.
       01  V            PIC 9.
       78  C78          VALUE 'A'.
       PROCEDURE        DIVISION.
           IF FUNCTION TRIM (' ')
              CONTINUE
              CONTINUE
           END-IF.
           IF CON
              CONTINUE
              CONTINUE
           END-IF.
           IF V
              CONTINUE
              CONTINUE
           END-IF.
           IF C78
              CONTINUE
              CONTINUE
           END-IF.
           IF '2'
              CONTINUE
              CONTINUE
           END-IF.
           IF C78 OR V
               CONTINUE
               CONTINUE
           END-IF.
           EVALUATE TRUE
              WHEN FUNCTION TRIM (' ')
                 CONTINUE
                 CONTINUE
              WHEN CON
                 CONTINUE
                 CONTINUE
              WHEN V
                 CONTINUE
                 CONTINUE
              WHEN C78
                 CONTINUE
                 CONTINUE
              WHEN '2'
                 CONTINUE
              WHEN OTHER
                 CONTINUE
           END-EVALUATE.
           PERFORM VARYING V
                   FROM 1 BY 1
                   UNTIL FUNCTION TRIM (' ')
              CONTINUE
              CONTINUE
           END-PERFORM.
           PERFORM VARYING V
                   FROM 1 BY 1
                   UNTIL V
              CONTINUE
              CONTINUE
           END-PERFORM.
           PERFORM VARYING V
                   FROM 1 BY 1
                   UNTIL C78
              CONTINUE
              CONTINUE
           END-PERFORM.
           PERFORM VARYING V
                   FROM 1 BY 1
                   UNTIL '2'
              CONTINUE
              CONTINUE
           END-PERFORM.
           IF NOTDEFINED = 1 OR 2
              CONTINUE
           END-IF.

           IF (V = 1) AND V
              CONTINUE
           END-IF

           STOP RUN.
