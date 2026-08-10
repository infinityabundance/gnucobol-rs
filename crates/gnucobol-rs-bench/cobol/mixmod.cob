      *> Mixed-workflow subprogram: applies a fixed 3% surcharge (rounded).
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MIXMOD.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  RATE-P            PIC 9(3) VALUE 003.
       LINKAGE SECTION.
       01  NET-C             PIC S9(14).
       01  SURC-C            PIC S9(14).
       PROCEDURE DIVISION USING NET-C SURC-C.
           COMPUTE SURC-C ROUNDED = NET-C * RATE-P / 100
           GOBACK.
