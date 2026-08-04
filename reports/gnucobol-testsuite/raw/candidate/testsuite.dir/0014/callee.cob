
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      callee.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 EXT-VAR       PIC X(5) EXTERNAL.
       PROCEDURE        DIVISION.
           DISPLAY EXT-VAR END-DISPLAY.
           MOVE "World" TO EXT-VAR.
           EXIT PROGRAM.
