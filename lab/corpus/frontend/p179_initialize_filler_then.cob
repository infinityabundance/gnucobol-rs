      *> INITIALIZE WITH FILLER and a leading THEN. A plain INITIALIZE leaves FILLER untouched (cobc); WITH
      *> FILLER also resets FILLER leaves; `items THEN REPLACING ...` == `items REPLACING ...` (only the named
      *> category is set, FILLER excluded). Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P179.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 N      PIC 9(3) VALUE 42.
          05 A      PIC X(3) VALUE "abc".
          05 FILLER PIC X(2) VALUE "##".
       PROCEDURE DIVISION.
           INITIALIZE G.
           DISPLAY "plain:       G=[" G "]".
           MOVE 42 TO N. MOVE "abc" TO A.
           INITIALIZE G WITH FILLER.
           DISPLAY "with filler: G=[" G "]".
           MOVE 42 TO N. MOVE "abc" TO A.
           INITIALIZE G THEN REPLACING NUMERIC DATA BY 7.
           DISPLAY "then repl:   G=[" G "]".
           STOP RUN.
