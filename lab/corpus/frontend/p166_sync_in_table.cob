      *> SYNCHRONIZED descendant of a table element: slack before each SYNC field aligns it, and the element is
      *> padded up to the largest SYNC alignment so every occurrence stays aligned. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P166.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 ROW OCCURS 3.
             10 A PIC X.
             10 B PIC S9(4) COMP SYNC.
       01 H.
          05 E OCCURS 3.
             10 P PIC X.
             10 Q PIC S9(9) COMP SYNC.
             10 R PIC X.
       PROCEDURE DIVISION.
           DISPLAY "elem=" FUNCTION LENGTH(ROW(1)) " G=" FUNCTION LENGTH(G).
           MOVE 7 TO B(1). MOVE 1234 TO B(2). MOVE 9 TO B(3).
           MOVE "y" TO A(2).
           DISPLAY A(2) " " B(1) " " B(2) " " B(3).
           DISPLAY "helem=" FUNCTION LENGTH(E(1)) " H=" FUNCTION LENGTH(H).
           MOVE 123456 TO Q(2). MOVE "z" TO R(2).
           DISPLAY Q(2) " " R(2).
           STOP RUN.
