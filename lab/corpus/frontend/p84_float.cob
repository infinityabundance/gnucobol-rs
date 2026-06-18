      *> USAGE COMP-1 (32-bit IEEE float) / COMP-2 (64-bit double): MOVE, DISPLAY (cob_display_common reads
      *> the f32/f64 directly) and arithmetic (operands round-trip through the wide-decimal intermediate;
      *> the receiver store rounds to the nearest IEEE value). Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P84.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 F1 COMP-1.
       01 A COMP-2 VALUE 12.5.
       01 B COMP-2 VALUE 4.
       01 R COMP-2.
       PROCEDURE DIVISION.
           MOVE 3.5 TO F1.   DISPLAY "F1=[" F1 "]".
           MOVE 0.1 TO R.    DISPLAY "R01=[" R "]".
           COMPUTE R = A + B. DISPLAY "ADD=[" R "]".
           COMPUTE R = A * B. DISPLAY "MUL=[" R "]".
           COMPUTE R = 100 / 3. DISPLAY "DIV=[" R "]".
           STOP RUN.
