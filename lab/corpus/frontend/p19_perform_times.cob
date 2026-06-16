       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 9(4) VALUE 0.
       01 R PIC ZZZ9.
       PROCEDURE DIVISION.
           PERFORM 5 TIMES
               ADD 2 TO N
           END-PERFORM.
           MOVE N TO R.
           DISPLAY "N=" R.
           STOP RUN.
