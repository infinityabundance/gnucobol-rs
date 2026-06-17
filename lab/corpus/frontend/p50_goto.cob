      *> GO TO between paragraphs: a forward jump that skips a statement, and a backward jump forming a
      *> loop (with a fall-through exit). Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P50.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 9 VALUE 0.
       PROCEDURE DIVISION.
       MAIN-PARA.
           DISPLAY "START".
           GO TO SKIP-PARA.
           DISPLAY "SHOULD-NOT-PRINT".
       SKIP-PARA.
           DISPLAY "AFTER-SKIP".
           ADD 1 TO N.
           IF N < 3 GO TO SKIP-PARA END-IF.
           DISPLAY "DONE N=" N.
           STOP RUN.
