      *> EXTERNAL -- run-unit-shared storage (by name), zero-filled, VALUE ignored. OUTER sets SHARED.SV,
      *> CALLs INNER which sees the same storage (1234) and writes it back (ADD 1 -> OUTER sees 1235).
      *> Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. OUTER.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SHARED EXTERNAL.
          05 SV PIC 9(4).
       PROCEDURE DIVISION.
           MOVE 1234 TO SV.
           DISPLAY "outer-sv=[" SV "]".
           CALL "INNER".
           DISPLAY "outer-after=[" SV "]".
           STOP RUN.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. INNER.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SHARED EXTERNAL.
          05 SV PIC 9(4).
       PROCEDURE DIVISION.
           DISPLAY "inner-sv=[" SV "]".
           ADD 1 TO SV.
           GOBACK.
       END PROGRAM INNER.
       END PROGRAM OUTER.
