       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9(4) VALUE 10.
       01 B PIC 9(4) VALUE 3.
       01 C PIC 9(4) VALUE 5.
       01 R PIC ZZZZ9.
       PROCEDURE DIVISION.
           COMPUTE R = (A + B) * C.
           DISPLAY "R=" R.
           STOP RUN.
