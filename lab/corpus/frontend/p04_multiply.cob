       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9(3) VALUE 12.
       01 B PIC 9(3) VALUE 12.
       01 C PIC 9(5).
       01 R PIC ZZ,ZZ9.
       PROCEDURE DIVISION.
           MULTIPLY A BY B GIVING C.
           MOVE C TO R.
           DISPLAY "PROD=" R.
           STOP RUN.
