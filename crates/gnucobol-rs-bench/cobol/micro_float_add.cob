      *> Micro: float ADD (COMP-1 f32 + COMP-2 f64), 50_000 iterations.
      *> Every add is exact (integer-valued, below 2^24); the outputs are
      *> edited Z(.)9.99 fields, byte-identical to the Rust expectation.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MICRO-FLOAT-ADD.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  I                 PIC 9(9).
       01  F32-ACC           COMP-1.
       01  F64-ACC           COMP-2.
       01  F32-E             PIC Z(6)9.99.
       01  F64-E             PIC Z(9)9.99.
       01  I-E               PIC 9(9).
       PROCEDURE DIVISION.
           COMPUTE F32-ACC = 0
           COMPUTE F64-ACC = 0
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 50000
               COMPUTE F32-ACC = F32-ACC + 1
               COMPUTE F64-ACC = F64-ACC + 1
           END-PERFORM
           SUBTRACT 1 FROM I
           MOVE F32-ACC TO F32-E
           MOVE F64-ACC TO F64-E
           MOVE I TO I-E
           DISPLAY "FLOAT-ADD-DONE " F32-E " " F64-E " " I-E
           STOP RUN.
