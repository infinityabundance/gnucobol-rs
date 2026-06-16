       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC S9(4) VALUE 50.
       01 B PIC 9(4) VALUE 4.
       01 R PIC -ZZZ9.99.
       PROCEDURE DIVISION.
           COMPUTE R = 0 - A / B.
           DISPLAY "NEG=" R.
           STOP RUN.
