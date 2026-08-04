
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      callee.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 WRK-VAR       PIC X(5).
       01 EXT-VAR       PIC X(5) EXTERNAL.
       PROCEDURE        DIVISION.
           DISPLAY EXT-VAR NO ADVANCING
           END-DISPLAY.
           MOVE "World" TO EXT-VAR.
           EXIT PROGRAM.
