
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT f ASSIGN "A".

       DATA            DIVISION.
       FILE            SECTION.
       FD  f.
       01  f-rec PIC X.

       WORKING-STORAGE SECTION.
       01  not-display PIC 9(5) COMP.
       01  not-a-num   PIC X(5).

       PROCEDURE       DIVISION.
           INSPECT f TALLYING not-a-num FOR ALL 3
               REPLACING FIRST "abcde" BY not-display
           TRANSFORM f FROM 3 TO 2
           TRANSFORM function udf(f-rec) FROM SPACE TO ZERO
           TRANSFORM "yada" FROM SPACE TO ZERO
           INSPECT function trim(f-rec)
               TALLYING not-display FOR ALL SPACES *> valid with func
               REPLACING FIRST ZERO BY SPACE       *> not valid
           .
