       IDENTIFICATION DIVISION.
       PROGRAM-ID. P124.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 A PIC X(3) VALUE "abc".
          05 N PIC X(2).
          05 B PIC 99   VALUE 42.
          05 M PIC 99.
       PROCEDURE DIVISION.
           MOVE "ZZZZZZZZZ" TO G.
           INITIALIZE G ALL TO VALUE.
           DISPLAY "G1=[" G "]".
           MOVE "ZZZZZZZZZ" TO G.
           INITIALIZE G WITH FILLER ALL TO VALUE.
           DISPLAY "G2=[" G "]".
           STOP RUN.
