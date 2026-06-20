       IDENTIFICATION DIVISION.
       PROGRAM-ID. P123.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 H  PIC X(2).
          05 L  PIC X(3).
       01 C  PIC 99 VALUE 0.
       PROCEDURE DIVISION.
           MOVE "AB" TO H.
           MOVE LOW-VALUES TO L.
           INSPECT G TALLYING C FOR ALL LOW-VALUE.
           DISPLAY "C=" C.
           INSPECT G REPLACING ALL LOW-VALUE BY "Z".
           INSPECT G REPLACING ALL "Z" BY QUOTE.
           DISPLAY "G=[" G "]".
           STOP RUN.
