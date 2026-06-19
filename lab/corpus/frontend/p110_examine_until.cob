      *> @std: ibm
      *> EXAMINE UNTIL FIRST: REPLACING UNTIL FIRST x BY y rewrites the chars before the
      *> first x; TALLYING UNTIL FIRST x REPLACING BY y counts then rewrites that span.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P110.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S1 PIC X(8) VALUE "AABXCCDX".
       01 S2 PIC X(8) VALUE "AABXCCDX".
       PROCEDURE DIVISION.
           EXAMINE S1 REPLACING UNTIL FIRST "X" BY "*".
           DISPLAY "ruf [" S1 "]".
           EXAMINE S2 TALLYING UNTIL FIRST "X" REPLACING BY "*".
           DISPLAY "tuf [" S2 "] TALLY=" TALLY.
           STOP RUN.
