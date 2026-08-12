
       REPLACE   =="SOME"== BY =="MANY"==
                 =='SOME'== BY =="VERY MUCH"==
                 ==STUFF==  BY ==NEWSTUFF==.
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 NEWSTUFF PIC X(80).
       PROCEDURE        DIVISION.
           DISPLAY STUFF " BENEFITS SOME PARTS FROM "
                   "SOME" "STUFF" ", " 'SOME' "GOOD" STUFF "AND STUFF !"
                   NO ADVANCING
           END-DISPLAY.
           STOP RUN.
