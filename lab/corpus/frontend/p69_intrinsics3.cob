      *> Transcendental / statistical / date / validator / bit intrinsics in FUNCTION references.
      *> The 2048-bit Mpf transcendental layer + date conversions are oracle-sealed; byte-identical to cobc.
       PROGRAM-ID. P69.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 X4   PIC 9(4)V99 VALUE 0016.00.
       01 D8   PIC 9(8)    VALUE 20260618.
       01 DI   PIC 9(7)    VALUE 0000001.
       01 NS   PIC X(8)    VALUE "12.5abc".
       PROCEDURE DIVISION.
           DISPLAY "SQRT=[" FUNCTION SQRT(X4) "]".
           DISPLAY "PI=[" FUNCTION PI "]".
           DISPLAY "E=[" FUNCTION E "]".
           DISPLAY "LOG=[" FUNCTION LOG(X4) "]".
           DISPLAY "EXP=[" FUNCTION EXP(2) "]".
           DISPLAY "VAR=[" FUNCTION VARIANCE(2 4 6) "]".
           DISPLAY "STDEV=[" FUNCTION STANDARD-DEVIATION(2 4 6) "]".
           DISPLAY "IOD=[" FUNCTION INTEGER-OF-DATE(D8) "]".
           DISPLAY "DOI=[" FUNCTION DATE-OF-INTEGER(150000) "]".
           DISPLAY "TNV=[" FUNCTION TEST-NUMVAL("12.5") "]".
           DISPLAY "TNVB=[" FUNCTION TEST-NUMVAL(NS) "]".
           DISPLAY "SCL=[" FUNCTION STORED-CHAR-LENGTH("hi  ") "]".
           DISPLAY "CAT=[" FUNCTION CONCATENATE("ab" "cd" "ef") "]".
           DISPLAY "BITOF=[" FUNCTION BIT-OF("A") "]".
           STOP RUN.
