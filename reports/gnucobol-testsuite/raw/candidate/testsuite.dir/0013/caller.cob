
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      caller.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 EXT-VAR       PIC X(5) EXTERNAL.
       01 WRK-VAR       PIC X(5).
       PROCEDURE        DIVISION.
           MOVE "Hello" TO EXT-VAR.
           CALL "callee"
           END-CALL.
           DISPLAY EXT-VAR NO ADVANCING
           END-DISPLAY.
           STOP RUN.
