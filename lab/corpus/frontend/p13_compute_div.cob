       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9(6) VALUE 22.
       01 B PIC 9(6) VALUE 7.
       01 R PIC 9.9(8).
       PROCEDURE DIVISION.
           COMPUTE R = A / B.
           DISPLAY "PI=" R.
           STOP RUN.
