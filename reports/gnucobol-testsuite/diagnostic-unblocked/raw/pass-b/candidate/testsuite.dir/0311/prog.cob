
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 FLAG          PIC 9 VALUE ZERO.
       PROCEDURE        DIVISION.
       MAIN
       .
         PERFORM SEC-1
         IF NOT FLAG = 1
              DISPLAY "SEC-1 NOT EXECUTED"
         END-IF
         STOP RUN
         .
       SEC-1 SECTION.
         MOVE 1 TO FLAG
         .
