      *> Module workload subprogram: computes n * 3 + 1 and keeps an EXTERNAL
      *> accumulator that survives CALLs (re-initialized by CANCEL).
       IDENTIFICATION DIVISION.
       PROGRAM-ID. CALC-MOD.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  EXTERNAL-ACC      PIC 9(10) EXTERNAL.
       LINKAGE SECTION.
       01  ARG               PIC 9(10).
       01  RES               PIC 9(10).
       PROCEDURE DIVISION USING ARG RES.
           COMPUTE RES = ARG * 3 + 1
           ADD 1 TO EXTERNAL-ACC
           GOBACK.
