      *> OCCURS min TO max DEPENDING ON counter -- a variable-length table. The field is built at MAX; its
      *> live image and FUNCTION LENGTH reflect the DEPENDING counter (and LENGTH of a variable item is a
      *> runtime call -> the 9-digit form, not cobc's constant fold). Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P85.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 9 VALUE 2.
       01 T.
          05 E PIC 9(2) OCCURS 1 TO 4 DEPENDING ON N.
       PROCEDURE DIVISION.
           MOVE 11 TO E(1). MOVE 22 TO E(2).
           DISPLAY "LEN=[" FUNCTION LENGTH(T) "]".
           DISPLAY "T=[" T "]".
           MOVE 3 TO N.
           MOVE 33 TO E(3).
           DISPLAY "LEN2=[" FUNCTION LENGTH(T) "]".
           DISPLAY "T2=[" T "]".
           STOP RUN.
