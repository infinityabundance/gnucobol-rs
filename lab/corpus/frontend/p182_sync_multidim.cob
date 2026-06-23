      *> SYNCHRONIZED descendant inside a MULTI-DIMENSION group-OCCURS table. Each table element aligns its
      *> SYNC field (slack before) and is padded to the largest SYNC alignment, so every occurrence stays
      *> aligned: X(1) + slack(3) + S9(9)COMP SYNC(4) + Y(1) -> 12-byte CELL element. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P182.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 T.
          05 ROW OCCURS 2.
             10 CELL OCCURS 2.
                15 X PIC X VALUE "x".
                15 S PIC S9(9) COMP SYNC.
                15 Y PIC X VALUE "y".
       PROCEDURE DIVISION.
           MOVE 5 TO S(1 1). MOVE 3 TO S(1 2). MOVE 9 TO S(2 2).
           DISPLAY "LEN=" FUNCTION LENGTH(T).
           DISPLAY "S11=" S(1 1) " S12=" S(1 2) " S22=" S(2 2).
           DISPLAY "X11=" X(1 1) " Y22=" Y(2 2).
           STOP RUN.
