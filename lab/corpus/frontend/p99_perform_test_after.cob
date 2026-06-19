       IDENTIFICATION DIVISION.
       PROGRAM-ID. P99.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 9 VALUE 5.
       01 I PIC 9 VALUE 0.
       01 C PIC 99 VALUE 0.
       PROCEDURE DIVISION.
           PERFORM WITH TEST AFTER UNTIL N = 5
               DISPLAY "after-body N=" N
               ADD 1 TO N
           END-PERFORM.
           PERFORM WITH TEST AFTER VARYING I FROM 1 BY 1 UNTIL I >= 3
               ADD I TO C
               DISPLAY "v I=" I
           END-PERFORM.
           DISPLAY "C=" C " I=" I.
           STOP RUN.
