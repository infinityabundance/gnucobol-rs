
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  a-variable PIC 9.

       PROCEDURE DIVISION.
           IF a-variable = 1
               ACCEPT a-variable, not-a-variable
                   ON EXCEPTION
                       CONTINUE
               END-ACCEPT
           ELSE
               CONTINUE
           END-IF
           .
