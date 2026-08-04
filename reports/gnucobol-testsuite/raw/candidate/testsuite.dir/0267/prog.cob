
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 SINGLE-ITEM   PIC X(9999999999).
       01 GROUP-ITEM1.
          05 FILLER     PIC X(9999999999).
       01 GROUP-ITEM2.
          05 FILLER     PIC X(199999999).
          05 FILLER     PIC X(199999999).
       >> IF P64 SET
          05 FILLER     PIC X(1999999999).
       >> END-IF
       PROCEDURE        DIVISION.
           STOP RUN.

