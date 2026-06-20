       IDENTIFICATION DIVISION.
       PROGRAM-ID. P126.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 9 VALUE 0.
       PROCEDURE DIVISION.
      *> bare inline PERFORM ... END-PERFORM runs the body exactly once
           PERFORM
              ADD 1 TO N
              DISPLAY "IN " N
           END-PERFORM.
           DISPLAY "OUT " N.
           STOP RUN.
