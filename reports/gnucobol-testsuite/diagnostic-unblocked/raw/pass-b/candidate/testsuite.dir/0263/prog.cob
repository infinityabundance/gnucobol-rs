
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  x PIC 9(5).

       PROCEDURE DIVISION.
      *> Ok
           INSPECT "abcde" TALLYING x FOR CHARACTERS CHARACTERS

      *> Not ok
           INSPECT "abcde" TALLYING x FOR ALL LEADING
               TRAILING ALL ALL ALL TRAILING
           INSPECT "abcde" TALLYING x FOR x FOR LEADING "a"
           .
