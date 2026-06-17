      *> EVALUATE: the COBOL case statement -- subject-value match (WHEN value, WHEN value THRU value,
      *> WHEN OTHER) and EVALUATE TRUE (WHEN condition). Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P43.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 99 VALUE 7.
       PROCEDURE DIVISION.
           EVALUATE N
               WHEN 1 DISPLAY "ONE"
               WHEN 5 THRU 9 DISPLAY "FIVE-TO-NINE"
               WHEN OTHER DISPLAY "OTHER"
           END-EVALUATE.
           EVALUATE TRUE
               WHEN N > 50 DISPLAY "BIG"
               WHEN N < 50 DISPLAY "SMALL"
           END-EVALUATE.
           MOVE 99 TO N.
           EVALUATE N
               WHEN 1 DISPLAY "ONE"
               WHEN 5 THRU 9 DISPLAY "FIVE-TO-NINE"
               WHEN OTHER DISPLAY "OTHER"
           END-EVALUATE.
           STOP RUN.
