      *> JSON/XML GENERATE content rendering: an all-spaces alphanumeric keeps ONE space (not empty), and XML
      *> element content escapes & < > and the double-quote (&quot;). Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P168.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 BL PIC X(4) VALUE SPACES.
          05 QT PIC X(3) VALUE "a""b".
          05 MK PIC X(5) VALUE "x<y>z".
          05 AM PIC X(3) VALUE "p&q".
       01 R PIC X(90).
       PROCEDURE DIVISION.
           JSON GENERATE R FROM G. DISPLAY "J=[" FUNCTION TRIM(R) "]".
           MOVE SPACES TO R.
           XML GENERATE R FROM G. DISPLAY "X=[" FUNCTION TRIM(R) "]".
           STOP RUN.
