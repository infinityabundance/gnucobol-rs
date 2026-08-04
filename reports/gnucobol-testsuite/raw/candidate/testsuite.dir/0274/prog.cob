
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.
       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       77 var          PIC x.
       PROCEDURE       DIVISION.
           ACCEPT  var FROM SYSOUT
           DISPLAY var UPON SYSIN
           ACCEPT  var FROM SYSIN
           DISPLAY var UPON SYSOUT
           .
