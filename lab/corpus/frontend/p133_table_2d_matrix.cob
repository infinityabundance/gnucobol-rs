       IDENTIFICATION DIVISION.
       PROGRAM-ID. P133.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 GRID.
          05 R OCCURS 3.
             10 TAG PIC X.
             10 N   PIC 99 OCCURS 2.
       01 I PIC 9.
       01 J PIC 9.
       01 S PIC 999 VALUE 0.
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 3
              MOVE "R" TO TAG(I)
              PERFORM VARYING J FROM 1 BY 1 UNTIL J > 2
                 COMPUTE N(I, J) = I * 10 + J
              END-PERFORM
           END-PERFORM.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 3
              PERFORM VARYING J FROM 1 BY 1 UNTIL J > 2
                 ADD N(I, J) TO S
              END-PERFORM
           END-PERFORM.
           DISPLAY "GRID=[" GRID "]".
           DISPLAY "N32=" N(3,2) " TAG2=" TAG(2) " SUM=" S.
           STOP RUN.
