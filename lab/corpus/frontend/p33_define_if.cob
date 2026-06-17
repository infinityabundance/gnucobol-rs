       IDENTIFICATION DIVISION.
       PROGRAM-ID. P33.
       PROCEDURE DIVISION.
       >>DEFINE FEAT AS 1
       >>IF FEAT DEFINED
           DISPLAY "FEAT-ON".
       >>ELSE
           DISPLAY "FEAT-OFF".
       >>END-IF
       >>IF OTHER DEFINED
           DISPLAY "OTHER-ON".
       >>ELSE
           DISPLAY "OTHER-OFF".
       >>END-IF
       >>IF FEAT = 1
           DISPLAY "FEAT-IS-1".
       >>END-IF
           DISPLAY "DONE".
           STOP RUN.
