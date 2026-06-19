       IDENTIFICATION DIVISION.
       PROGRAM-ID. P103.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 P PIC S9(4)V99 COMP-3 VALUE 12.50.
       01 B PIC S9(5) COMP VALUE -300.
       01 C5 PIC 9(4) COMP-5 VALUE 1000.
       01 Z PIC S9(3) COMP-3 VALUE 0.
       PROCEDURE DIVISION.
           IF P = 12.50 DISPLAY "P-eq" END-IF.
           IF P > 12 DISPLAY "P-gt12" END-IF.
           IF B = -300 DISPLAY "B-eq" END-IF.
           IF B < 0 DISPLAY "B-neg" END-IF.
           IF C5 = 1000 DISPLAY "C5-eq" END-IF.
           IF Z = ZERO DISPLAY "Z-zero" END-IF.
           STOP RUN.
