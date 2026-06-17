      *> INITIALIZE (no REPLACING): numeric items become ZERO, alphanumeric items become SPACES,
      *> ignoring VALUE clauses -- matching cobc. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P46.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 CTR     PIC 9(4) VALUE 1234.
       01 SCTR    PIC S9(3) VALUE -42.
       01 NAMEF   PIC X(6) VALUE "ABCDEF".
       PROCEDURE DIVISION.
           DISPLAY "BEFORE C=" CTR " S=" SCTR " N=" NAMEF.
           INITIALIZE CTR.
           INITIALIZE SCTR.
           INITIALIZE NAMEF.
           DISPLAY "AFTER  C=" CTR " S=" SCTR " N=" NAMEF.
           STOP RUN.
