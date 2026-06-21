      *> Space-separated multi-dimension subscripts -- CELL(1 1), CELL (3 2) (space before paren), and 3-D
      *> C(2 1 2). cobc accepts space OR comma separators; the gluer now keeps them distinct. Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P163.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 ROW OCCURS 3.
             10 CELL PIC 99 OCCURS 2.
       01 CUBE.
          05 PL OCCURS 2.
             10 RW OCCURS 2.
                15 C3 PIC 9 OCCURS 2.
       PROCEDURE DIVISION.
           MOVE 11 TO CELL(1 1). MOVE 12 TO CELL(1 2).
           MOVE 31 TO CELL (3 1). MOVE 32 TO CELL(3, 2).
           DISPLAY CELL(1 1) " " CELL(1 2) " " CELL (3 1) " " CELL(3, 2).
           DISPLAY "[" G "]".
           MOVE 7 TO C3(2 1 2). MOVE 4 TO C3(1 2 1).
           DISPLAY C3(2 1 2) " " C3(1 2 1).
           STOP RUN.
