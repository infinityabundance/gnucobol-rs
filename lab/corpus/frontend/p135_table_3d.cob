       IDENTIFICATION DIVISION.
       PROGRAM-ID. P135.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 CUBE.
          05 PL OCCURS 2.
             10 RW OCCURS 2.
                15 C PIC 99 OCCURS 2.
       PROCEDURE DIVISION.
      *> three-dimensional table C(i,j,k)
           MOVE 11 TO C(1,1,1). MOVE 88 TO C(2,2,2). MOVE 55 TO C(2,1,2).
           DISPLAY "CUBE=[" CUBE "] C222=" C(2,2,2) " C212=" C(2,1,2).
           STOP RUN.
