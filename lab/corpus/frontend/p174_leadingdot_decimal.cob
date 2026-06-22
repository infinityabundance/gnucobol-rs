      *> A numeric literal written with no leading zero -- `.08`, `.5` -- (cobc accepts it as 0.08 / 0.5). The
      *> leading `.` must NOT be lexed as a sentence terminator. Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P174.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 U PIC 9(4)V99 VALUE 200.00.
       01 W PIC 9(4)V99 VALUE ZERO.
       PROCEDURE DIVISION.
           COMPUTE W = U * .08.
           DISPLAY "TAX=" W.
           COMPUTE W = U * .5 + .25.
           DISPLAY "HALF=" W.
           STOP RUN.
