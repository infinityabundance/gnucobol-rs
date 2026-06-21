      *> Reference modification base(start:len) / base(start:) -- alphanumeric substring, 1-based start, as a
      *> value (DISPLAY, IF, MOVE source) and as a RECEIVER (MOVE/INSPECT/STRING target), with literal,
      *> data-name, and subscripted bounds/bases. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P149.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S  PIC X(10) VALUE "ABCDEFGHIJ".
       01 P  PIC 9 VALUE 3.
       01 L  PIC 9 VALUE 4.
       01 TB.
          05 TE PIC XX OCCURS 3 VALUE "qq".
       PROCEDURE DIVISION.
           DISPLAY "A=[" S(1:3) "]".
           DISPLAY "B=[" S(P:L) "]".
           DISPLAY "C=[" S(8:) "]".
           IF S(4:2) = "DE" DISPLAY "EQ" ELSE DISPLAY "NE" END-IF.
           MOVE "XY" TO S(3:2).
           DISPLAY "D=[" S "]".
           MOVE "wz" TO TE(2).
           DISPLAY "E=[" TE(2)(2:1) "]".
           INSPECT S(1:5) REPLACING ALL "X" BY "0".
           DISPLAY "F=[" S "]".
           STOP RUN.
