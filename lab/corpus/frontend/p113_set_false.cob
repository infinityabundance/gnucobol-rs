       IDENTIFICATION DIVISION.
       PROGRAM-ID. P113.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SW PIC 9 VALUE 1.
          88 RDY VALUE 1 WHEN SET TO FALSE 0.
       PROCEDURE DIVISION.
           DISPLAY "a " SW.
           SET RDY TO FALSE.
           DISPLAY "b " SW.
           SET RDY TO TRUE.
           DISPLAY "c " SW.
           STOP RUN.
