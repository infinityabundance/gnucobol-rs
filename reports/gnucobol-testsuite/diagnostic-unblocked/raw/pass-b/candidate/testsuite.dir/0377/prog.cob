
       IDENTIFICATION DIVISION.
       PROGRAM-ID.    prog.
       PROCEDURE      DIVISION.
       >> IF X DEFINED
         >>DISPLAY "X defined"
       >> ELSE
         >> DISPLAY "X not defined"
         >> DEFINE X 1
       >> END-IF
           CONTINUE
           .
