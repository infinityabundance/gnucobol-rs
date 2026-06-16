       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9(3) VALUE 3.
       01 R PIC ZZ9.
       PROCEDURE DIVISION.
           IF A > 5
               MOVE 99 TO R
           ELSE
               MOVE 11 TO R
           END-IF.
           DISPLAY "R=" R.
           STOP RUN.
