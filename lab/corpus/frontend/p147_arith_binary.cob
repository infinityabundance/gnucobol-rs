      *> Arithmetic with binary/packed operands + uninitialized binary/packed zero. DIVIDE with a COMP/COMP-3
      *> operand (was InvalidAttr); an uninitialized COMP/COMP-3 reads as 0 (was '0'-byte garbage). Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P147.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A  PIC S9(6) COMP VALUE 100.
       01 B  PIC S9(5)V9 COMP-3 VALUE 7.
       01 G  PIC S9(4)V99.
       01 UZ PIC S9(6) COMP.
       01 WP PIC S9(4) COMP-3.
       PROCEDURE DIVISION.
           DIVIDE A INTO B GIVING G ROUNDED.
           DISPLAY "G=[" G "]".
           DISPLAY "UZ=[" UZ "] WP=[" WP "]".
           DIVIDE UZ INTO A GIVING G.
           DISPLAY "DZ=[" G "]".
           STOP RUN.
