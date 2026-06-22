      *> JSON/XML GENERATE over an elementary OCCURS table emits only the FIRST occurrence (cobc 3.2). Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P169.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 N PIC 99 VALUE 9.
          05 T PIC 99 OCCURS 3.
          05 S PIC X(3) OCCURS 2.
       01 R PIC X(90).
       PROCEDURE DIVISION.
           MOVE 11 TO T(1). MOVE 22 TO T(2). MOVE 33 TO T(3).
           MOVE "ab" TO S(1). MOVE "cd" TO S(2).
           JSON GENERATE R FROM G. DISPLAY "J=[" FUNCTION TRIM(R) "]".
           MOVE SPACES TO R.
           XML GENERATE R FROM G. DISPLAY "X=[" FUNCTION TRIM(R) "]".
           STOP RUN.
