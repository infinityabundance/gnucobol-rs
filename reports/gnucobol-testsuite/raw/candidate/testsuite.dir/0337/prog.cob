
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       ENVIRONMENT     DIVISION.
       CONFIGURATION   SECTION.
       SOURCE-COMPUTER.
            whatever   WITH DEBUGGING MODE.

       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       01  i           PIC 9.

       PROCEDURE DIVISION.
           PERFORM VARYING i FROM 1 BY 1
                   UNTIL i = 5
           END-PERFORM
           .
