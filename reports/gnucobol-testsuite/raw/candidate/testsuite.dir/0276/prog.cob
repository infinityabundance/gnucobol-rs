
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.
       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       77 var          PIC x.
       PROCEDURE       DIVISION.
           ACCEPT var  FROM COMMAND-LINE
           DISPLAY var UPON COMMAND-LINE
           .
