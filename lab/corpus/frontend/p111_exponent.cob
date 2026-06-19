       IDENTIFICATION DIVISION.
       PROGRAM-ID. P111.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 R PIC 9(6)V9(4).
       01 N PIC 9 VALUE 3.
       PROCEDURE DIVISION.
           COMPUTE R = 9 ** 0.5.
           DISPLAY "a " R.
           COMPUTE R = 16 ** 0.25.
           DISPLAY "b " R.
           COMPUTE R = 2 ** 10.
           DISPLAY "c " R.
           COMPUTE R = 27 ** N.
           DISPLAY "d " R.
           STOP RUN.
