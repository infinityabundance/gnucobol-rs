      *> MOVE CORRESPONDING moves only like-NAMED elementary leaves; FILLER never corresponds, so a separator
      *> FILLER in the target keeps its own value (the `-` in a yyyy-mm-dd trailer date survives). Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P172.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-SRC.
          05 YYYY PIC 9(4) VALUE 2021.
          05 FILLER PIC X VALUE SPACE.
          05 MM   PIC 9(2) VALUE 09.
          05 FILLER PIC X VALUE SPACE.
          05 DD   PIC 9(2) VALUE 01.
       01 WS-DST.
          05 YYYY PIC 9(4) VALUE ZERO.
          05 FILLER PIC X VALUE '-'.
          05 MM   PIC 9(2) VALUE ZERO.
          05 FILLER PIC X VALUE '-'.
          05 DD   PIC 9(2) VALUE ZERO.
       PROCEDURE DIVISION.
           MOVE CORRESPONDING WS-SRC TO WS-DST.
           DISPLAY "DST=[" WS-DST "]".
           STOP RUN.
