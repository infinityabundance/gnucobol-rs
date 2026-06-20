       IDENTIFICATION DIVISION.
       PROGRAM-ID. P141.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 A PIC X(3) VALUE "abc".
          05 B PIC 99   VALUE 42.
          05 C PIC X(2) VALUE "yz".
       01 TB.
          05 E PIC 99 VALUE 7 OCCURS 3.
       PROCEDURE DIVISION.
           MOVE "ZZZZZZ" TO G.
           INITIALIZE G NUMERIC TO VALUE.
           DISPLAY "NUM=[" G "]".
           MOVE "ZZZZZZ" TO G.
           INITIALIZE G ALPHANUMERIC TO VALUE.
           DISPLAY "ALN=[" G "]".
           MOVE 1 TO E(1). MOVE 2 TO E(2). MOVE 3 TO E(3).
           INITIALIZE TB ALL TO VALUE.
           DISPLAY "TB=" E(1) E(2) E(3).
           STOP RUN.
