      *> FUNCTION TRIM(x [LEADING | TRAILING]) -- the direction keyword is a modifier, not an argument.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P153.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S PIC X(8) VALUE "  hi   ".
       01 R PIC X(10).
       PROCEDURE DIVISION.
           MOVE FUNCTION TRIM(S) TO R. DISPLAY "B=[" R "]".
           MOVE FUNCTION TRIM(S LEADING) TO R. DISPLAY "L=[" R "]".
           MOVE FUNCTION TRIM(S TRAILING) TO R. DISPLAY "T=[" R "]".
           STOP RUN.
