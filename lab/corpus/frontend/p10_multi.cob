       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 X PIC 9(3) VALUE 5.
       01 Y PIC 9(3) VALUE 3.
       01 Z PIC 9(4).
       01 R PIC ZZZ9.
       PROCEDURE DIVISION.
           ADD X TO Y.
           MULTIPLY Y BY X GIVING Z.
           MOVE Z TO R.
           DISPLAY "R=" R " DONE".
           STOP RUN.
