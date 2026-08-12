
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01  XX  PIC 9.
       01  X            CONSTANT AS XX OF XX.
       PROCEDURE        DIVISION.
           MOVE X TO XX.
           DISPLAY XX NO ADVANCING
           END-DISPLAY.
           STOP RUN.
