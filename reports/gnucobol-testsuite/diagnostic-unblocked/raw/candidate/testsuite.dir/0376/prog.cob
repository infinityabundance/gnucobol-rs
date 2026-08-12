
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       PROCEDURE        DIVISION.
       >>IF ACTIVATE DEFINED
       >>DISPLAY "NOTOK"
       >>ELIF ACTIVATE2 DEFINED
       >>DISPLAY "OK"
       >>ELSE
       >>DISPLAY "NOTOK"
       >>END-IF
           STOP RUN.
