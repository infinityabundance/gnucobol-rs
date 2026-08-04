
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
       SYMBOLIC CONSTANT
         con-1  IS 1
         25156c   "25156c".
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  NUM2      PIC 9.
       01  SHORT-X   PIC X(5).
      *
       PROCEDURE DIVISION.
       MAIN.
           MOVE CON-1  TO NUM2.
           MOVE 25156C TO SHORT-X
           STOP RUN.
