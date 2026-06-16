       IDENTIFICATION DIVISION.
       PROGRAM-ID. P.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 I PIC 9(4) VALUE 1.
       01 F PIC 9(8) VALUE 1.
       01 R PIC ZZZZZZZ9.
       PROCEDURE DIVISION.
           PERFORM UNTIL I > 5
               MULTIPLY I BY F GIVING F
               ADD 1 TO I
           END-PERFORM.
           MOVE F TO R.
           DISPLAY "5!=" R.
           STOP RUN.
