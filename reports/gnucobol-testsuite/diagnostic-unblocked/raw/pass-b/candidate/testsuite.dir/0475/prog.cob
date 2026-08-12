
      * some comments go here
                 *> and here
                        *> and finally... here
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 NEWSTUFF PIC X(80).
       PROCEDURE        DIVISION.
           DISPLAY NEWSTUFF " BENEFITS SOME PARTS FROM "
           "MANY" "STUFF" ", " "VERY MUCH" "GOOD" NEWSTUFF
           "AND STUFF !"
                   NO ADVANCING
           END-DISPLAY.
           STOP RUN.
