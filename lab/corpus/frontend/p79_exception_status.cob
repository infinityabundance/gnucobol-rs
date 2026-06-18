      *> FUNCTION EXCEPTION-STATUS -- the last raised arithmetic SIZE ERROR condition, STICKY (a clean
      *> statement does not clear it; only a new exception overwrites). Wiring it also fixed a real ON SIZE
      *> ERROR gap: the front-end now detects result-overflow, not just divide-by-zero. Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P79.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9(2) VALUE 10.
       01 B PIC 9(2) VALUE 0.
       01 C PIC 9(2) VALUE 99.
       01 R PIC 9(4).
       PROCEDURE DIVISION.
           DISPLAY "0=[" FUNCTION EXCEPTION-STATUS "]".
           COMPUTE R = A / B ON SIZE ERROR CONTINUE END-COMPUTE.
           DISPLAY "1=[" FUNCTION EXCEPTION-STATUS "]".
           ADD 1 TO C.
           DISPLAY "2=[" FUNCTION EXCEPTION-STATUS "] C=[" C "]".
           COMPUTE R = A + 5.
           DISPLAY "3=[" FUNCTION EXCEPTION-STATUS "]".
           COMPUTE R = 50 + 60 ON SIZE ERROR DISPLAY "size!" END-COMPUTE.
           DISPLAY "4=[" R "]".
           STOP RUN.
