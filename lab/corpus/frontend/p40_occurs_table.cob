      *> 01-level OCCURS table: subscripted element access NAME(i) for MOVE, DISPLAY, arithmetic and a
      *> variable subscript NAME(I). Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P40.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 E PIC 99 OCCURS 4 TIMES.
       01 I PIC 9 VALUE 3.
       01 S PIC 999.
       PROCEDURE DIVISION.
           MOVE 10 TO E(1).
           MOVE 20 TO E(2).
           MOVE 30 TO E(3).
           MOVE 40 TO E(4).
           DISPLAY "E1=" E(1) " E3=" E(3).
           DISPLAY "EI=" E(I).
           ADD E(1) E(2) E(I) GIVING S.
           DISPLAY "SUM134=" S.
           COMPUTE S = (E(1) + E(3)) * 2 - E(I).
           DISPLAY "EXPR=" S.
           IF E(2) < E(I)
               DISPLAY "E2-LESS-E3"
           ELSE
               DISPLAY "E2-GE-E3"
           END-IF.
           STOP RUN.
