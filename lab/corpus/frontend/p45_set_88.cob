      *> SET condition-name TO TRUE: constructs the parent's bytes from the 88's first VALUE (or a
      *> THRU range's lower bound). The write counterpart of the LEVEL-88 predicate. Identical stdout
      *> under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P45.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 GRADE PIC 99.
          88 PASSING VALUE 50 THRU 100.
          88 FAILING VALUE 0 THRU 49.
       01 FLAG PIC X.
          88 YES VALUE "Y".
          88 NOTYES VALUE "N".
       PROCEDURE DIVISION.
           SET PASSING TO TRUE.
           DISPLAY "GRADE=" GRADE.
           IF PASSING DISPLAY "PASS-OK" ELSE DISPLAY "PASS-BAD" END-IF.
           SET FAILING TO TRUE.
           DISPLAY "GRADE=" GRADE.
           SET YES TO TRUE.
           DISPLAY "FLAG=" FLAG.
           IF YES DISPLAY "YES-OK" ELSE DISPLAY "YES-BAD" END-IF.
           SET NOTYES TO TRUE.
           DISPLAY "FLAG=" FLAG.
           STOP RUN.
