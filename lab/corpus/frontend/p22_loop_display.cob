       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 I PIC 9 VALUE 1.
       01 R PIC Z9.
       PROCEDURE DIVISION.
           PERFORM UNTIL I > 3
               MOVE I TO R
               DISPLAY "LINE " R
               ADD 1 TO I
           END-PERFORM.
           STOP RUN.
