
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  a-variable PIC 9.

       PROCEDURE DIVISION.
           EVALUATE a-variable

              ALSO true

              WHEN 1

              ALSO a-variable
                 CONTINUE

              WHEN 1 OR 2
                 CONTINUE

           END-EVALUATE
           .
           EVALUATE a-variable

              ALSO true

              WHEN 3
                 CONTINUE

              WHEN 5
                 CONTINUE

           END-EVALUATE
           .
