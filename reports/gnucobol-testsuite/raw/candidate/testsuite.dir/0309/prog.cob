       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 DONE COMP-1 VALUE ZERO.

       PROCEDURE DIVISION.
       MAIN SECTION.
           PERFORM SEC-1.
           IF NOT DONE = 1
              DISPLAY "SEC-1 NOT EXECUTED"
           END-IF
           STOP RUN
       SEC-1 SECTION.
           MOVE 1 TO DONE
           GOBACK.
