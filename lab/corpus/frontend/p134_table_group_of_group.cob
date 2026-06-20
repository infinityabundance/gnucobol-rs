       IDENTIFICATION DIVISION.
       PROGRAM-ID. P134.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 GOG.
          05 GRP OCCURS 2.
             10 SUB.
                15 A PIC X(2).
                15 B PIC 9.
       PROCEDURE DIVISION.
      *> group-of-group: outer group-OCCURS GRP over a sub-group SUB; leaves A(i)/B(i)
           MOVE "ab" TO A(1). MOVE 1 TO B(1).
           MOVE "cd" TO A(2). MOVE 2 TO B(2).
           DISPLAY "GOG=[" GOG "] A2=" A(2) " B1=" B(1).
           INITIALIZE GOG.
           DISPLAY "AFTER=[" GOG "]".
           STOP RUN.
