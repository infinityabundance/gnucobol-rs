       IDENTIFICATION DIVISION.
       PROGRAM-ID. P29.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9(4) VALUE 100.
       01 Z PIC 9(4) VALUE 0.
       01 B PIC 9(4) VALUE 5.
       01 R PIC 9(4) VALUE 7777.
       PROCEDURE DIVISION.
           DIVIDE A BY Z GIVING R
               ON SIZE ERROR DISPLAY "SE1"
               NOT ON SIZE ERROR DISPLAY "OK1".
           DISPLAY "RA=" R.
           DIVIDE A BY B GIVING R
               ON SIZE ERROR DISPLAY "SE2"
               NOT ON SIZE ERROR DISPLAY "OK2".
           DISPLAY "RB=" R.
           COMPUTE R = A / Z
               ON SIZE ERROR DISPLAY "CSE".
           DISPLAY "RC=" R.
           DIVIDE A BY Z GIVING R.
           DISPLAY "RD=" R.
           DISPLAY "DONE".
           STOP RUN.
