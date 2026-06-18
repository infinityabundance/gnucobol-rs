      *> @no312: EXCEPTION-LOCATION/STATEMENT formatting evolved 3.1.2 -> 3.2; port targets 3.2
      *> FUNCTION EXCEPTION-STATEMENT (spaces in the default dialect) + EXCEPTION-LOCATION ("<prog>; ; 0"
      *> once an exception is raised, a single space before). Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P80.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9 VALUE 9.
       PROCEDURE DIVISION.
       MAIN-PARA.
           DISPLAY "loc0=[" FUNCTION EXCEPTION-LOCATION "]".
           DISPLAY "stmt0=[" FUNCTION EXCEPTION-STATEMENT "]".
           ADD 5 TO A.
           DISPLAY "loc1=[" FUNCTION EXCEPTION-LOCATION "]".
           DISPLAY "stmt1=[" FUNCTION EXCEPTION-STATEMENT "]".
           DISPLAY "stat=[" FUNCTION EXCEPTION-STATUS "]".
           STOP RUN.
