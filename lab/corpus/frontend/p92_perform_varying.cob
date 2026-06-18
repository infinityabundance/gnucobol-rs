       IDENTIFICATION DIVISION.
       PROGRAM-ID. P92.
      *> PERFORM VARYING (inline + out-of-line, TEST BEFORE default).
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 I    PIC 9(3) VALUE 0.
       01 SUM1 PIC 9(5) VALUE 0.
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 5
               ADD I TO SUM1
           END-PERFORM.
           DISPLAY "SUM=" SUM1 " I=" I.
           MOVE 0 TO SUM1.
           PERFORM ADDP VARYING I FROM 2 BY 2 UNTIL I > 8.
           DISPLAY "SUM2=" SUM1.
           STOP RUN.
       ADDP.
           ADD I TO SUM1.
