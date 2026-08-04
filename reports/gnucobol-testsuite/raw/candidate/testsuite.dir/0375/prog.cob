
       $SET CONSTANT DOGGY "Barky"
       $SET CONSTANT PONY "Blacky"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  THEDOG    PIC X(6) VALUE DOGGY.
       77  MYHORSE   PIC X(7) VALUE PONY.
       $SET CONSTANT PONY "White"
      *
      * ensure to test a numeric, too (not actually used):
       $SET CONSTANT C-UNUSED-NUM 0
       PROCEDURE DIVISION.
       MAIN.
           DISPLAY "Your Dog's name is " DOGGY ";".
           DISPLAY "The Dog's name is " THEDOG ";".
           DISPLAY "My Horse is " MYHORSE ";".
           DISPLAY "My little pony is " PONY ".".
           STOP RUN.
