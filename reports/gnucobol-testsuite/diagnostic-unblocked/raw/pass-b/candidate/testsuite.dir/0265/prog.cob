
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  x PIC X(10).

       PROCEDURE DIVISION.
           INSPECT x REPLACING CHARACTERS BY "x"
               BEFORE "A" BEFORE "b" AFTER "c"
           .
