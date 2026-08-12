
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA  DIVISION.
       WORKING-STORAGE SECTION.
       01  A         PIC 9 VALUE 1.
       01  B         PIC 9 VALUE 2.

       PROCEDURE DIVISION.

           EVALUATE TRUE
           WHEN A = B
           WHEN OTHER
               DISPLAY 'other' END-DISPLAY
           END-EVALUATE

           STOP RUN.
