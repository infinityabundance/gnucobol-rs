       IDENTIFICATION DIVISION.
       PROGRAM-ID. P137.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 CNT PIC 9 VALUE 2.
       01 TBL.
          05 ENT OCCURS 1 TO 4 DEPENDING ON CNT.
             10 K PIC 99.
             10 V PIC X.
       PROCEDURE DIVISION.
      *> OCCURS DEPENDING ON on a group: live image is CNT*elem (built at MAX 4)
           MOVE 11 TO K(1). MOVE "a" TO V(1).
           MOVE 22 TO K(2). MOVE "b" TO V(2).
           DISPLAY "TBL=[" TBL "] LEN=" FUNCTION LENGTH(TBL).
           DISPLAY "K2=" K(2) " V1=" V(1).
           STOP RUN.
