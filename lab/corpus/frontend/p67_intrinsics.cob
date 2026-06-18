      *> FUNCTION references in DISPLAY / COMPUTE / MOVE: the front-end resolves each into a temp field
      *> evaluated by the ported cob_intr_* runtime, so every display form (cobc's constant-fold LENGTH,
      *> the 9-digit ORD/INTEGER/MOD, signed ABS, NUMVAL with a point) is byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P67.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 NAME-X PIC X(10) VALUE "hello".
       01 N1     PIC 9(3)V99 VALUE 012.50.
       01 N2     PIC S9(4)   VALUE -25.
       01 R      PIC 9(6).
       01 RS     PIC S9(6)V99.
       01 TXT    PIC X(20).
       PROCEDURE DIVISION.
           DISPLAY "LEN=[" FUNCTION LENGTH(NAME-X) "]".
           DISPLAY "UC=[" FUNCTION UPPER-CASE(NAME-X) "]".
           DISPLAY "LC=[" FUNCTION LOWER-CASE("WORLD") "]".
           DISPLAY "REV=[" FUNCTION REVERSE("abcd") "]".
           DISPLAY "NV=[" FUNCTION NUMVAL("  123.45 ") "]".
           DISPLAY "INT=[" FUNCTION INTEGER(N1) "]".
           DISPLAY "ABS=[" FUNCTION ABS(N2) "]".
           DISPLAY "MOD=[" FUNCTION MOD(17 5) "]".
           DISPLAY "MAX=[" FUNCTION MAX(3 9 2 7) "]".
           DISPLAY "MIN=[" FUNCTION MIN(3 9 2 7) "]".
           DISPLAY "ORD=[" FUNCTION ORD("A") "]".
           COMPUTE R = FUNCTION LENGTH(NAME-X) + 1.
           DISPLAY "CMP=[" R "]".
           COMPUTE RS = FUNCTION MAX(10 20 5) * 2.
           DISPLAY "CMP2=[" RS "]".
           MOVE FUNCTION UPPER-CASE(NAME-X) TO TXT.
           DISPLAY "MOV=[" TXT "]".
           STOP RUN.
