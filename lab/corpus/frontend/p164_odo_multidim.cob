      *> OCCURS DEPENDING ON the OUTER dimension of a multi-dimension group: the buffer is built at MAX, the
      *> element addressing uses fixed MAX strides, and the LIVE image / LENGTH is counter*stride. Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P164.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 9 VALUE 2.
       01 G.
          05 ROW OCCURS 1 TO 3 DEPENDING ON N.
             10 CELL PIC 99 OCCURS 2.
       PROCEDURE DIVISION.
           MOVE 11 TO CELL(1 1). MOVE 12 TO CELL(1 2).
           MOVE 21 TO CELL(2 1). MOVE 22 TO CELL(2 2).
           DISPLAY "len2=" FUNCTION LENGTH(G).
           DISPLAY "G2=[" G "]".
           DISPLAY CELL(1 1) " " CELL(2 2).
           MOVE 3 TO N.
           MOVE 31 TO CELL(3 1). MOVE 32 TO CELL(3 2).
           DISPLAY "len3=" FUNCTION LENGTH(G).
           DISPLAY "G3=[" G "]".
           STOP RUN.
