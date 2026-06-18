      *> Trig / EXP10 / LOG10 / financial (ANNUITY, PRESENT-VALUE) / algebraic-bound intrinsics,
      *> each byte-identical to cobc via the ported cob_intr_* runtime.
       PROGRAM-ID. P70.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A   PIC 9V9(4) VALUE 0.5000.
       01 D8  PIC 9(8)   VALUE 20260618.
       01 DY  PIC 9(7)   VALUE 2026169.
       01 SS  PIC S9(4)V99.
       PROCEDURE DIVISION.
           DISPLAY "SIN=[" FUNCTION SIN(A) "]".
           DISPLAY "COS=[" FUNCTION COS(A) "]".
           DISPLAY "TAN=[" FUNCTION TAN(A) "]".
           DISPLAY "ASIN=[" FUNCTION ASIN(A) "]".
           DISPLAY "ACOS=[" FUNCTION ACOS(A) "]".
           DISPLAY "ATAN=[" FUNCTION ATAN(A) "]".
           DISPLAY "EXP10=[" FUNCTION EXP10(2) "]".
           DISPLAY "LOG10=[" FUNCTION LOG10(1000) "]".
           DISPLAY "IODY=[" FUNCTION INTEGER-OF-DAY(DY) "]".
           DISPLAY "DAYOI=[" FUNCTION DAY-OF-INTEGER(150000) "]".
           DISPLAY "TDATE=[" FUNCTION TEST-DATE-YYYYMMDD(D8) "]".
           DISPLAY "TDAY=[" FUNCTION TEST-DAY-YYYYDDD(DY) "]".
           DISPLAY "TNVC=[" FUNCTION TEST-NUMVAL-C("$1,2.50") "]".
           DISPLAY "TNVF=[" FUNCTION TEST-NUMVAL-F("1.5E2") "]".
           DISPLAY "NVF=[" FUNCTION NUMVAL-F("1.5E2") "]".
           DISPLAY "B2C=[" FUNCTION BIT-TO-CHAR("01000001") "]".
           DISPLAY "LOWA=[" FUNCTION LOWEST-ALGEBRAIC(SS) "]".
           DISPLAY "HIGHA=[" FUNCTION HIGHEST-ALGEBRAIC(SS) "]".
           DISPLAY "ANN=[" FUNCTION ANNUITY(0.05 10) "]".
           DISPLAY "PV=[" FUNCTION PRESENT-VALUE(0.05 100 200) "]".
           STOP RUN.
