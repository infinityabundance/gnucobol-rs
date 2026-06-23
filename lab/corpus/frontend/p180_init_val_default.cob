      *> INITIALIZE ALL TO VALUE [THEN] TO DEFAULT: TO VALUE restores each leaf that HAS a VALUE to its VALUE
      *> image; the trailing TO DEFAULT then sets each leaf WITHOUT a VALUE to its category default. (TO VALUE
      *> alone leaves no-VALUE leaves unchanged; THEN REPLACING sets them by category.) Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P180.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 N PIC 9(3) VALUE 42.
          05 A PIC X(3) VALUE "abc".
          05 Z PIC 9(2).
          05 T PIC X(2).
       PROCEDURE DIVISION.
           MOVE 99 TO Z. MOVE "qq" TO T.
           INITIALIZE G ALL TO VALUE.
           DISPLAY "value only:  N=[" N "] Z=[" Z "] T=[" T "]".
           MOVE 99 TO Z. MOVE "qq" TO T.
           INITIALIZE G ALL TO VALUE THEN TO DEFAULT.
           DISPLAY "then default:N=[" N "] Z=[" Z "] T=[" T "]".
           STOP RUN.
