      *> JSON/XML GENERATE [ON EXCEPTION ...] [NOT ON EXCEPTION ...]. On success cobc runs NOT ON EXCEPTION
      *> ONLY when it is the sole handler; with both branches present it runs neither (a 3.2 quirk). Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P160.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G. 05 A PIC 99 VALUE 7.
       01 R PIC X(40).
       01 C PIC 99 VALUE 0.
       PROCEDURE DIVISION.
           JSON GENERATE R FROM G COUNT IN C
              NOT ON EXCEPTION DISPLAY "only-not C=" C
           END-JSON.
           DISPLAY "[" FUNCTION TRIM(R) "]".
           JSON GENERATE R FROM G
              ON EXCEPTION DISPLAY "exc"
              NOT ON EXCEPTION DISPLAY "both"
           END-JSON.
           DISPLAY "after-both".
           XML GENERATE R FROM G NOT ON EXCEPTION DISPLAY "xok" END-XML.
           DISPLAY "[" FUNCTION TRIM(R) "]".
           STOP RUN.
