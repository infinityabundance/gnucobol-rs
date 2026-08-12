
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      inside2.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 EXT-VAR       PIC X(5) EXTERNAL.
       01 CLA-VAR       PIC X(5).
       PROCEDURE        DIVISION.
           DISPLAY "Aloha2".
           CALL "inside".
           GOBACK. *> never gets here
       END PROGRAM inside2.
