
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       ENVIRONMENT     DIVISION.
       CONFIGURATION   SECTION.
       SOURCE-COMPUTER.
            whatever   WITH DEBUGGING MODE.

       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       01  i           PIC 9.

       PROCEDURE DIVISION.
       DECLARATIVES.
       test-DEBUG SECTION.
           USE FOR DEBUGGING ON ALL REFERENCES OF I
                                MAIN.
           DISPLAY DEBUG-ITEM.
           DISPLAY DEBUG-NAME " AT " DEBUG-LINE " WITH " DEBUG-CONTENTS.
           DISPLAY DEBUG-SUB-1 DEBUG-SUB-2 DEBUG-SUB-3.
       END DECLARATIVES.

       MAIN SECTION.
           MOVE 1 TO i
           STOP RUN.
