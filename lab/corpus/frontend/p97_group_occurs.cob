       IDENTIFICATION DIVISION.
       PROGRAM-ID. P97.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 TBL.
          05 ENT OCCURS 3.
             10 EK PIC 9(2) VALUE 7.
             10 EV PIC XXX VALUE "zzz".
       01 WHOLE REDEFINES TBL PIC X(15).
       01 I PIC 9.
       PROCEDURE DIVISION.
           DISPLAY "INIT=[" WHOLE "]".
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 3
               MOVE I TO EK(I)
           END-PERFORM.
           MOVE "abc" TO EV(2).
           DISPLAY "WHOLE=[" WHOLE "]".
           DISPLAY "TBL=[" TBL "]".
           DISPLAY "ENT2=[" ENT(2) "]".
           STOP RUN.
