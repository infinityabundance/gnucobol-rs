      *> DECIMAL-POINT IS COMMA: an alphanumeric "12,34" MOVEd to a numeric receiver must treat the
      *> comma as the decimal point (store 12.34, not 1234->34), and numeric DISPLAY must render the
      *> decimal point as a comma -- both identical under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P36.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
           DECIMAL-POINT IS COMMA.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A   PIC X(5) VALUE "12,34".
       01 N   PIC 9(2)V99.
       01 M   PIC S9(3)V9 VALUE -7,5.
       01 E   PIC ZZ9,99.
       PROCEDURE DIVISION.
           MOVE A TO N.
           MOVE N TO E.
           DISPLAY "N=" N.
           DISPLAY "M=" M.
           DISPLAY "E=" E.
           STOP RUN.
