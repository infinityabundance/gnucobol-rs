      *> JSON GENERATE + XML GENERATE (group tree -> JSON/XML), and JSON PARSE accepted as a faithful no-op
      *> (GnuCOBOL 3.2 compiles JSON PARSE as "not implemented" and it does nothing). Identical cobc/cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P60.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 J-OUT PIC X(90).
       01 J-IN  PIC X(20) VALUE '{"AGE":77}'.
       01 REC.
          05 FNAME PIC X(5) VALUE "ANNA".
          05 AGE   PIC 9(3) VALUE 30.
          05 SUBG.
             10 CITY PIC X(4) VALUE "NYC".
             10 ZIP  PIC 9(5) VALUE 10001.
       PROCEDURE DIVISION.
           JSON GENERATE J-OUT FROM REC.
           DISPLAY "J=[" J-OUT "]".
           JSON PARSE J-IN INTO REC.
           DISPLAY "AGE-AFTER-PARSE=" AGE.
           XML GENERATE J-OUT FROM REC.
           DISPLAY "X=[" J-OUT "]".
           STOP RUN.
