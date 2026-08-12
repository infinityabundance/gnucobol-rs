
       IDENTIFICATION DIVISION.
       PROGRAM-ID.    prog.
       PROCEDURE      DIVISION.
       >>IF A IS DEFINED
       >>IF B IS DEFINED
           CONTINUE
       >> DISPLAY "NOT OK (not both definitions)"
           .
       >>ELSE
           CONTINUE
       >> DISPLAY "NOT OK (no definitions)"
           .
       >>END-IF
       >>ELSE
           CONTINUE
       >> DISPLAY "OK"
           .
       >>END-IF
