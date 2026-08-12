
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 X             PIC X(8).
       PROCEDURE        DIVISION.
           INSPECT X REPLACING "AB" BY "CD".
           STOP RUN.
