
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
           USE FOR DEBUGGING ON ALL REFERENCES OF I
                                TEST-debug
                                MAIN.
           DISPLAY DEBUG-LINE.
           PERFORM MAIN. PERFORM TEST-DEBUG2. GO TO TEST-DEBUG2.
       test-DEBUG2 SECTION.
           USE FOR DEBUGGING ON ALL OF I
                                TEST-debug.
           DISPLAY DEBUG-LINE.
       END DECLARATIVES.

       MAIN SECTION.
           MOVE 1 TO i, j
           PERFORM TesT-DebuG
           STOP RUN.
