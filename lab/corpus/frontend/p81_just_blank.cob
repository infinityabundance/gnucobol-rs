      *> JUSTIFIED RIGHT (right-align, left-truncate) + BLANK WHEN ZERO (the field becomes spaces when the
      *> value is zero) on both numeric-edited and plain numeric receivers. Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P81.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 J  PIC X(6) JUSTIFIED RIGHT.
       01 JL PIC X(3) JUSTIFIED RIGHT.
       01 Z  PIC ZZZ9 BLANK WHEN ZERO.
       01 ZN PIC 9(4) BLANK WHEN ZERO.
       PROCEDURE DIVISION.
           MOVE "AB" TO J.    DISPLAY "J=[" J "]".
           MOVE "HELLO" TO JL. DISPLAY "JL=[" JL "]".
           MOVE 0 TO Z.  DISPLAY "Z0=[" Z "]".
           MOVE 42 TO Z. DISPLAY "Z42=[" Z "]".
           MOVE 0 TO ZN. DISPLAY "ZN0=[" ZN "]".
           MOVE 7 TO ZN. DISPLAY "ZN7=[" ZN "]".
           STOP RUN.
