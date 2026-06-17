      *> 88-level condition-names: a condition-name is true when its parent's value equals any listed
      *> value or falls in a THRU range. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P44.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 GRADE PIC 99.
          88 PASSING VALUE 50 THRU 100.
          88 FAILING VALUE 0 THRU 49.
       01 FLAG PIC X.
          88 YES VALUE "Y" "y".
       PROCEDURE DIVISION.
           MOVE 75 TO GRADE.
           IF PASSING DISPLAY "PASS" ELSE DISPLAY "FAIL" END-IF.
           MOVE 30 TO GRADE.
           IF FAILING DISPLAY "FAIL" ELSE DISPLAY "PASS" END-IF.
           MOVE "y" TO FLAG.
           IF YES DISPLAY "YES" ELSE DISPLAY "NO" END-IF.
           STOP RUN.
