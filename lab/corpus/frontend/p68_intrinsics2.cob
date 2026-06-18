      *> More intrinsics + FUNCTION inside an IF condition: TRIM / INTEGER-PART / FRACTION-PART /
      *> FACTORIAL / SIGN / CHAR and the statistical list functions, each byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P68.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 PADDED PIC X(12) VALUE "  hi there  ".
       01 N1     PIC S9(3)V99 VALUE -12.34.
       01 FL     PIC 9(2)     VALUE 05.
       PROCEDURE DIVISION.
           DISPLAY "TRIM=[" FUNCTION TRIM(PADDED) "]".
           DISPLAY "IPART=[" FUNCTION INTEGER-PART(N1) "]".
           DISPLAY "FPART=[" FUNCTION FRACTION-PART(N1) "]".
           DISPLAY "FACT=[" FUNCTION FACTORIAL(FL) "]".
           DISPLAY "SIGN=[" FUNCTION SIGN(N1) "]".
           DISPLAY "CHAR=[" FUNCTION CHAR(66) "]".
           DISPLAY "SUM=[" FUNCTION SUM(10 20 30) "]".
           DISPLAY "MEAN=[" FUNCTION MEAN(10 20 30) "]".
           DISPLAY "MEDIAN=[" FUNCTION MEDIAN(7 3 9 1 5) "]".
           DISPLAY "RANGE=[" FUNCTION RANGE(7 3 9 1 5) "]".
           DISPLAY "MIDR=[" FUNCTION MIDRANGE(7 3 9 1 5) "]".
           DISPLAY "OMAX=[" FUNCTION ORD-MAX(7 3 9 1 5) "]".
           DISPLAY "OMIN=[" FUNCTION ORD-MIN(7 3 9 1 5) "]".
           DISPLAY "REM=[" FUNCTION REM(17 5) "]".
           DISPLAY "NVC=[" FUNCTION NUMVAL-C("$1,234.50") "]".
           IF FUNCTION LENGTH(PADDED) = 12
               DISPLAY "IF-LEN-OK"
           END-IF.
           IF FUNCTION UPPER-CASE("abc") = "ABC"
               DISPLAY "IF-UC-OK"
           END-IF.
           STOP RUN.
