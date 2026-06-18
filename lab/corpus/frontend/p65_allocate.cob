      *> ALLOCATE based storage INITIALIZED (deterministic) + use it + FREE. The pointer address is a
      *> non-claim (not displayed). Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P65.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 P USAGE POINTER.
       01 R BASED.
          05 RN PIC 9(3).
          05 RX PIC X(3).
       PROCEDURE DIVISION.
           ALLOCATE R INITIALIZED.
           DISPLAY "INIT N=[" RN "] X=[" RX "]".
           MOVE 42 TO RN.
           MOVE "HI" TO RX.
           DISPLAY "SET  N=[" RN "] X=[" RX "]".
           FREE ADDRESS OF R.
           STOP RUN.
