      *> The LENGTH OF / BYTE-LENGTH OF special register (no FUNCTION keyword) used as a numeric operand in
      *> arithmetic -- cobc folds it to the item's byte length. ADD LENGTH OF X TO N. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P185.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  WS-VARIABLE   PIC X(08) VALUE 'LENGTH 8'.
       01  N             PIC 9(4)  VALUE 1001.
       PROCEDURE DIVISION.
           ADD LENGTH OF WS-VARIABLE TO N.
           DISPLAY "N=" N.
           STOP RUN.
