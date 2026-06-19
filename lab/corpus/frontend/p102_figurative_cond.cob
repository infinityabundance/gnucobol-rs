       IDENTIFICATION DIVISION.
       PROGRAM-ID. P102.
      *> figurative constants in IF comparison + DISPLAY (zoned numeric + alphanumeric).
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC X(3) VALUE SPACES.
       01 N PIC 9(3) VALUE 0.
       01 B PIC X(3) VALUE "abc".
       PROCEDURE DIVISION.
           IF A = SPACES DISPLAY "A-blank" ELSE DISPLAY "A-notblank" END-IF.
           IF N = ZERO DISPLAY "N-zero" END-IF.
           IF B > SPACES DISPLAY "B-gt-sp" END-IF.
           IF B NOT = SPACES DISPLAY "B-ne-sp" END-IF.
           DISPLAY "x" SPACES "y".
           STOP RUN.
