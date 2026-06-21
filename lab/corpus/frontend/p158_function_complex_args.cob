      *> FUNCTION arguments that are subscripted (A(i)) or reference-modified (S(s:l)), plus mixed literal +
      *> subscript args; LENGTH of a refmod is the BASE item's length (cobc ignores the refmod). Identical.
      *> @no312: FUNCTION LENGTH of a reference-modified item returns the base item length in 3.2 (it was the
      *> reference-modified length in 3.1.2) -- a behaviour that evolved across versions; the port targets 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P158.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 TB.
          05 A PIC S9(4) OCCURS 5 VALUE 0.
       01 S PIC X(9) VALUE "Hello-Wld".
       01 D PIC X(7) VALUE "-123.45".
       01 R PIC S9(7)V99 VALUE 0.
       01 RX PIC X(12).
       01 I PIC 9 VALUE 2.
       PROCEDURE DIVISION.
           MOVE 10 TO A(1). MOVE -20 TO A(2). MOVE 30 TO A(3). MOVE 5 TO A(4).
           COMPUTE R = FUNCTION MAX(A(1) A(2) A(3) A(4)). DISPLAY "MAX=[" R "]".
           COMPUTE R = FUNCTION SUM(A(1) A(3)). DISPLAY "SUM=[" R "]".
           COMPUTE R = FUNCTION MAX(A(I) 100 A(1)). DISPLAY "MIX=[" R "]".
           COMPUTE R = FUNCTION NUMVAL(D(1:6)). DISPLAY "NV=[" R "]".
           MOVE FUNCTION UPPER-CASE(S(1:5)) TO RX. DISPLAY "UC=[" RX "]".
           MOVE FUNCTION REVERSE(S(2:3)) TO RX. DISPLAY "RV=[" RX "]".
           COMPUTE R = FUNCTION LENGTH(S(2:4)). DISPLAY "LEN=[" R "]".
           STOP RUN.
