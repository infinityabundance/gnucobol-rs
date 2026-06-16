       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 I   PIC 9(3) VALUE 1.
       01 TOT PIC 9(5) VALUE 0.
       01 R   PIC ZZZZ9.
       PROCEDURE DIVISION.
           PERFORM UNTIL I > 100
               ADD I TO TOT
               ADD 1 TO I
           END-PERFORM.
           MOVE TOT TO R.
           DISPLAY "TOT=" R.
           STOP RUN.
