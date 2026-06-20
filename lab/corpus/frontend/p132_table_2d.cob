       IDENTIFICATION DIVISION.
       PROGRAM-ID. P132.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 T1.
          05 ROW OCCURS 2.
             10 CEL PIC 99 OCCURS 3.
       PROCEDURE DIVISION.
      *> two-dimensional table: outer group-OCCURS ROW, inner elementary-OCCURS CEL -> CEL(i,j)
           MOVE 11 TO CEL(1,1). MOVE 12 TO CEL(1,2). MOVE 13 TO CEL(1,3).
           MOVE 21 TO CEL(2,1). MOVE 22 TO CEL(2,2). MOVE 23 TO CEL(2,3).
           DISPLAY "CEL22=" CEL(2,2) " CEL13=" CEL(1,3).
           DISPLAY "T1=[" T1 "]".
           STOP RUN.
