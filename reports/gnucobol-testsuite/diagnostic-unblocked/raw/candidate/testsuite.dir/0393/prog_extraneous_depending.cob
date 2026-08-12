
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog_extraneous_depending.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       1 L              COMP-1.
       1 F-DAT          PICTURE X(10) DEPENDING ON L.
       PROCEDURE        DIVISION.
           MOVE SPACES TO F-DAT
           STOP RUN.
