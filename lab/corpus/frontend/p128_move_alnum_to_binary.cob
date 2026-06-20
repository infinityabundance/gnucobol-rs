       IDENTIFICATION DIVISION.
       PROGRAM-ID. P128.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 C1 PIC 99      COMP.
       01 C2 PIC 9(4)    COMP.
       01 P1 PIC 99      COMP-3.
       01 P2 PIC 9(3)V9  COMP-3.
       01 B5 PIC 9(4)    COMP-5.
       PROCEDURE DIVISION.
      *> MOVE of an alphanumeric literal to a binary/packed receiver (move.c indirect_move path)
           MOVE "12"   TO C1.
           MOVE "1234" TO C2.
           MOVE "12"   TO P1.
           MOVE "1234" TO P2.
           MOVE "99"   TO B5.
           DISPLAY "C1=" C1 " C2=" C2 " P1=" P1 " P2=" P2 " B5=" B5.
           STOP RUN.
