      *> REDEFINES makes two items SHARE storage (the cob_field aliasable pointer): a MOVE into one is
      *> visible when the other is read, both directions, across categories. Identical stdout under cobc
      *> and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P41.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N   PIC 9(4) VALUE 1234.
       01 C   REDEFINES N PIC X(4).
       PROCEDURE DIVISION.
           DISPLAY "C0=" C.
           MOVE "9876" TO C.
           DISPLAY "N1=" N.
           DISPLAY "C1=" C.
           MOVE 5050 TO N.
           DISPLAY "C2=" C.
           STOP RUN.
