       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9(4) VALUE 100.
       01 B PIC 9(4) VALUE 250.
       01 R PIC ZZZ9.
       PROCEDURE DIVISION.
           ADD A TO B.
           MOVE B TO R.
           DISPLAY "SUM=" R.
           STOP RUN.
