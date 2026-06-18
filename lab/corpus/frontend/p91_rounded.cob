       IDENTIFICATION DIVISION.
       PROGRAM-ID. P91.
      *> COMPUTE + arithmetic-verb ROUNDED (default mode: NEAREST, ties away from zero).
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC S9(5)V99 VALUE 0.
       01 B PIC S9(5)V9  VALUE 0.
       01 C PIC S9(6)    VALUE 0.
       01 N PIC S9(5)V99 VALUE 0.
       PROCEDURE DIVISION.
           COMPUTE A ROUNDED = 10 / 3.
           COMPUTE B ROUNDED = 10 / 3.
           COMPUTE C ROUNDED = 2.5.
           COMPUTE N ROUNDED = -10 / 3.
           DISPLAY "A=" A.
           DISPLAY "B=" B.
           DISPLAY "C=" C.
           DISPLAY "N=" N.
           DIVIDE 100 BY 7 GIVING B ROUNDED.
           DISPLAY "DIV=" B.
           MULTIPLY 1.005 BY 100 GIVING A ROUNDED.
           DISPLAY "MUL=" A.
           STOP RUN.
