      *> A qualified AND subscripted operand -- `name OF group (subscript)` -- in both a condition and a MOVE,
      *> over a REDEFINES group-OCCURS table. The subscript syntactically follows the qualifier; the
      *> canonicalizer must resolve `SL OF GRP-TBL (I)` to the table leaf subscripted by I (not glue the
      *> subscript onto the group name). Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P184.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  GRP-CTL.
           05  GRP-FLAT.
               10  P1  PIC X(3) VALUE "901".
               10  D1  PIC 9    VALUE 0.
               10  P2  PIC X(3) VALUE "902".
               10  D2  PIC 9    VALUE 3.
               10  P3  PIC X(3) VALUE "903".
               10  D3  PIC 9    VALUE 5.
           05  GRP-TBL REDEFINES GRP-FLAT.
               10  ENT OCCURS 3 TIMES.
                   15  PL  PIC X(3).
                   15  SL  PIC 9.
       01  I     PIC 9 VALUE 1.
       01  OUTP  PIC X(3) VALUE SPACES.
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1
               UNTIL SL OF GRP-TBL (I) NOT EQUAL ZERO
           END-PERFORM.
           MOVE PL OF GRP-TBL (I) TO OUTP.
           DISPLAY "OUT=" OUTP " AT=" I.
           STOP RUN.
