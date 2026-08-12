
       IDENTIFICATION
           DIVISION
           .
           author.
           tester.
       PROGRAM-ID
           .
           prog1
           .
           REMARKS;. Should work.,,

       ENVIRONMENT
           DIVISION
           .
       CONFIGURATION
           SECTION
           .
       SOURCE-COMPUTER
           .
             whatever
           WITH
           DEBUGGING
           MODE
           .

      DDATA
      D   DIVISION
      D   .
       WORKING-STORAGE
          SECTION
          .
       01
           i
           PIC
           9
           .

       PROCEDURE
           DIVISION
           .
       >> SOURCE FORMAT IS FREE
IF
i
GREATER
THAN
OR
EQUAL

TO

5

THEN

GOBACK.
           STOP
           RUN
           .
