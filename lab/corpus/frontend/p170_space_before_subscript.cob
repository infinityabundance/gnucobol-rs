      *> A space before a (balanced) subscript -- `E (I)`, `G (1)` -- as cobc accepts; the subscript gluer now
      *> joins `name (sub)` even when the lexer kept the subscript balanced (not only split ones). Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P170.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 T.
          05 E PIC 99 OCCURS 5 VALUE 0.
       01 G.
          05 ROW OCCURS 3.
             10 C PIC 99.
       01 I PIC 9 VALUE 3.
       PROCEDURE DIVISION.
           MOVE 11 TO E (1). MOVE 33 TO E (I). MOVE 22 TO E (2).
           IF E (I) = 33 DISPLAY "E: " E (1) " " E (2) " " E (3) END-IF.
           MOVE 77 TO C (2). MOVE 88 TO C (3).
           DISPLAY "C: " C (2) " " C (3).
           STOP RUN.
