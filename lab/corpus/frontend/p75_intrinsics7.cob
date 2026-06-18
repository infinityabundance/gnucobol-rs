      *> Intrinsics that ARE deterministic under controlled conditions: MODULE-ID / MODULE-CALLER-ID
      *> (the running PROGRAM-ID + its caller) and the LOCALE conversions (under the pinned LC_ALL=C).
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P75.
       PROCEDURE DIVISION.
           DISPLAY "MID=[" FUNCTION MODULE-ID "]".
           DISPLAY "MCID=[" FUNCTION MODULE-CALLER-ID "]".
           DISPLAY "LD=[" FUNCTION LOCALE-DATE("20260618") "]".
           DISPLAY "LT=[" FUNCTION LOCALE-TIME("120000") "]".
           DISPLAY "LC=[" FUNCTION LOCALE-COMPARE("ABC" "ABD") "]".
           STOP RUN.
