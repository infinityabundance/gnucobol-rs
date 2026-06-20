       IDENTIFICATION DIVISION.
       PROGRAM-ID. P136.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 T1.
          05 ROW OCCURS 2.
             10 CEL PIC 99 OCCURS 3.
       PROCEDURE DIVISION.
           MOVE 11 TO CEL(1,1). MOVE 99 TO CEL(2,3).
           DISPLAY "BEFORE=[" T1 "]".
           INITIALIZE T1.
           DISPLAY "AFTER=[" T1 "]".
           STOP RUN.
