       IDENTIFICATION DIVISION.
       PROGRAM-ID. P105.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 X  PIC 9(3) VALUE 4.
       01 Y  PIC 9(3) VALUE 10.
       01 Z  PIC 9(3) VALUE 20.
       01 C  PIC 9(3) VALUE 30.
       01 D  PIC 9(3) VALUE 61.
       PROCEDURE DIVISION.
           ADD 1 TO Y Z.
           DISPLAY "A " Y "/" Z.
           ADD 5 X GIVING C D.
           DISPLAY "B " C "/" D.
           MOVE 10 TO Y Z.
           ADD 2 X TO Y GIVING C D.
           DISPLAY "C " C "/" D.
           MULTIPLY 3 BY Y Z.
           DISPLAY "D " Y "/" Z.
           SUBTRACT 1 FROM Y Z.
           DISPLAY "E " Y "/" Z.
           MOVE 30 TO C.
           MOVE 61 TO D.
           DIVIDE 3 INTO C D.
           DISPLAY "F " C "/" D.
           STOP RUN.
