      *> EC-BOUND-SUBSCRIPT is OFF by default: an out-of-range subscript is NOT checked -- cobc reads
      *> adjacent storage and continues; the program runs to completion. (>>TURN ... CHECKING ON makes it
      *> abort -- exercised in the unit tests, not here, since an abort has no clean stdout to compare.)
      *> Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P42.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 E PIC 9 OCCURS 3.
       01 I PIC 9 VALUE 5.
       PROCEDURE DIVISION.
           MOVE 1 TO E(1).
           MOVE 2 TO E(2).
           MOVE 3 TO E(3).
           DISPLAY "IN-BOUNDS=" E(2).
           MOVE 9 TO E(I).
           DISPLAY "CONTINUED".
           STOP RUN.
