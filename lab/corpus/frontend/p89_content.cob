      *> FUNCTION CONTENT-OF / CONTENT-LENGTH -- dereference a USAGE POINTER set via SET ptr TO ADDRESS OF
      *> field, returning the target's bytes (optionally a prefix length) and its length. Identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P89.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 P USAGE POINTER.
       01 S PIC X(5) VALUE "hello".
       PROCEDURE DIVISION.
           SET P TO ADDRESS OF S.
           DISPLAY "COF=[" FUNCTION CONTENT-OF(P) "]".
           DISPLAY "COF3=[" FUNCTION CONTENT-OF(P 3) "]".
           DISPLAY "CLEN=[" FUNCTION CONTENT-LENGTH(P) "]".
           STOP RUN.
