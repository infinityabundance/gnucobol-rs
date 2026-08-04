
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       ENVIRONMENT     DIVISION.
       CONFIGURATION   SECTION.
       SOURCE-COMPUTER.
            whatever   WITH DEBUGGING MODE.

       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       01  i           PIC 9.
       01  j           PIC 9.

       PROCEDURE DIVISION.
       DECLARATIVES.
       test-DEBUG SECTION.
           USE FOR DEBUGGING ON MAIN MAIN.
           DISPLAY DEBUG-LINE.
       test-DEBUG2 SECTION.
           USE FOR DEBUGGING ON ALL I
                                ALL REFERENCES OF J
                                ALL PROCEDURES.
           DISPLAY DEBUG-LINE.
       test-DEBUG3 SECTION.
           USE FOR DEBUGGING ON ALL PROCEDURES
                                J
                                ALL OF I.
           DISPLAY DEBUG-LINE.
       END DECLARATIVES.

       MAIN SECTION.
           MOVE 1 TO i
           STOP RUN.
