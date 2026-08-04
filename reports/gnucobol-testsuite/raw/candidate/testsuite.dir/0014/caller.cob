
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      caller.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 EXT-VAR       PIC X(5) EXTERNAL.
       PROCEDURE        DIVISION.
           MOVE "Hello" TO EXT-VAR.
           CALL "callee" END-CALL.
           DISPLAY EXT-VAR END-DISPLAY.
           STOP RUN.
       END PROGRAM caller.

       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      inside.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 EXT-VAR       PIC X(5) EXTERNAL.
       01 CLA-VAR       PIC X(5).
       PROCEDURE        DIVISION.
           MOVE "Aloha" TO EXT-VAR.
           CALL "callee" END-CALL.
           DISPLAY EXT-VAR END-DISPLAY.
           ACCEPT CLA-VAR FROM COMMAND-LINE END-ACCEPT.
           DISPLAY CLA-VAR END-DISPLAY.
           STOP RUN.
       END PROGRAM inside.
