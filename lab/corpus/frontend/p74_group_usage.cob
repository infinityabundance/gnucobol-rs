      *> Group-level USAGE inheritance: a child with no stated USAGE inherits the enclosing group's
      *> (COMP-3 here). The packed byte image is exposed via FUNCTION LENGTH + a REDEFINES. Identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P74.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 GRP USAGE COMP-3.
          05 A PIC 9(3) VALUE 123.
          05 B PIC 9(3) VALUE 456.
       01 P USAGE POINTER.
       PROCEDURE DIVISION.
           DISPLAY "LEN=[" FUNCTION LENGTH(GRP) "]".
           DISPLAY "A=[" A "] B=[" B "]".
           COMPUTE A = A + B.
           DISPLAY "A2=[" A "]".
           STOP RUN.
