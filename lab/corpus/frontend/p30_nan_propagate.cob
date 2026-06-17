       IDENTIFICATION DIVISION.
       PROGRAM-ID. P30.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9(4) VALUE 100.
       01 Z PIC 9(4) VALUE 0.
       01 R PIC 9(4) VALUE 7777.
       PROCEDURE DIVISION.
           COMPUTE R = (A / Z) + 5
               ON SIZE ERROR DISPLAY "SE".
           DISPLAY "R1=" R.
           COMPUTE R = A + (10 / Z) - 2
               ON SIZE ERROR DISPLAY "SE2".
           DISPLAY "R2=" R.
           STOP RUN.
