
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       ENVIRONMENT     DIVISION.
       CONFIGURATION   SECTION.
       SOURCE-COMPUTER.
            whatever   WITH DEBUGGING MODE.

       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       01  i           PIC 9.
       01  FILLER.
           03 x OCCURS 2 PIC 9.

       PROCEDURE DIVISION.
       DECLARATIVES.
       test-DEBUG SECTION.
           USE FOR DEBUGGING ON ALL OF I (1:1).
           DISPLAY DEBUG-LINE.
       test-DEBUGo SECTION.
           USE FOR DEBUGGING ON ALL OF X (1).
           DISPLAY DEBUG-LINE.
       END DECLARATIVES.

       MAIN SECTION.
           MOVE 1 TO i, x(2)
           STOP RUN.
