
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 X             PIC X(8).
       PROCEDURE        DIVISION.
           INSPECT FUNCTION TRIM(X) REPLACING ALL "ABC" BY "DEF".
           INSPECT FUNCTION TRIM(X) CONVERTING "ABC" TO "AAA".
           STOP RUN.
