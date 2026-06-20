      *> MOVE conversion completeness: binary/packed -> alphanumeric, packed -> packed, and binary/packed ->
      *> numeric-edited (previously blank / UnsupportedConversion / garbage). Identical cobc/cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P146.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 BC  PIC 9(4) COMP VALUE 1234.
       01 P3  PIC S9(4) COMP-3 VALUE -123.
       01 B5  PIC 9(4) COMP-5 VALUE 99.
       01 AX  PIC X(6).
       01 PP  PIC S9(6) COMP-3.
       01 ED  PIC ZZ9.9.
       01 E2  PIC ++9.99.
       PROCEDURE DIVISION.
           MOVE BC TO AX. DISPLAY "AX=[" AX "]".
           MOVE P3 TO PP. MOVE PP TO ED. DISPLAY "ED=[" ED "]".
           MOVE BC TO E2. DISPLAY "E2=[" E2 "]".
           MOVE P3 TO AX. DISPLAY "AX2=[" AX "]".
           STOP RUN.
