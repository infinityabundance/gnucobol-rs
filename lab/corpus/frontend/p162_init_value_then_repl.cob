      *> INITIALIZE ... [WITH FILLER] ALL TO VALUE [THEN] REPLACING cat BY val ... -- TO VALUE restores each
      *> leaf that HAS a VALUE; REPLACING then sets each leaf WITHOUT a VALUE whose category is named (a leaf
      *> with neither is left unchanged). Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P162.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 A PIC 99 VALUE 9.
          05 B PIC 99.
          05 C PIC XX VALUE "cc".
          05 D PIC XX.
       PROCEDURE DIVISION.
           MOVE 11 TO A. MOVE 22 TO B. MOVE "yy" TO C. MOVE "zz" TO D.
           INITIALIZE G ALL TO VALUE THEN REPLACING NUMERIC BY 3 ALPHANUMERIC BY "X".
           DISPLAY "T1=[" A "][" B "][" C "][" D "]".
           MOVE 11 TO A. MOVE 22 TO B. MOVE "yy" TO C. MOVE "zz" TO D.
           INITIALIZE G WITH FILLER ALL TO VALUE THEN REPLACING NUMERIC BY 3.
           DISPLAY "T2=[" A "][" B "][" C "][" D "]".
           STOP RUN.
