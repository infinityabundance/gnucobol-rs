
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  one        PIC X.
       77  seven      PIC 9 BASED.
       77  var        PIC 9.
       77  ptr        USAGE POINTER.
       PROCEDURE DIVISION.
           CONTINUE
           PERFORM VARYING var FROM 1 BY 1 UNTIL var = 4
              CONTINUE
           END-PERFORM
           CONTINUE AFTER 42 SECONDS      *> COBOL 202x
           CONTINUE AFTER 4 + 2 SECONDS
           CONTINUE AFTER var + 2 SECONDS
           CONTINUE AFTER '1' SECONDS
           CONTINUE AFTER ptr SECONDS
           CONTINUE AFTER one SECONDS
           .
