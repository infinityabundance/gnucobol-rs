       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9(3) VALUE 7.
       01 B PIC 9(3) VALUE 4.
       01 R PIC X(3).
       PROCEDURE DIVISION.
           IF A > 5 AND B < 10
               MOVE "YES" TO R
           ELSE
               MOVE "NO " TO R
           END-IF.
           DISPLAY R.
           STOP RUN.
