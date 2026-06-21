      *> REDEFINES descendant inside a table element (OCCURS group) -- the redefining item overlays its target
      *> at the same element offset, across all occurrences. Flat group-OCCURS and multi-dimension. Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P165.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 E OCCURS 3.
             10 FULL PIC 9(4).
             10 PARTS REDEFINES FULL.
                15 HI PIC 99.
                15 LO PIC 99.
       01 H.
          05 ROW OCCURS 2.
             10 CELL PIC 99 OCCURS 2.
             10 RAW REDEFINES CELL PIC X(4).
       PROCEDURE DIVISION.
           MOVE 1234 TO FULL(1). MOVE 5678 TO FULL(3).
           DISPLAY "E1 " HI(1) " " LO(1) " E3 " HI(3) " " LO(3).
           MOVE 12 TO CELL(1 1). MOVE 34 TO CELL(1 2).
           MOVE 56 TO CELL(2 1). MOVE 78 TO CELL(2 2).
           DISPLAY "RAW1=[" RAW(1) "] RAW2=[" RAW(2) "]".
           STOP RUN.
