
       $SET NOAREACHECK
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.
       DATA            DIVISION.
       $SET AREACHECK
       WORKING-STORAGE SECTION.
       01 X.
           02 Y PIC X.
           01 Z PIC X
       01 T PIC 9.
       $SET NOAREACHECK
       PROCEDURE       DIVISION.
       MAIN-1 SECTION.
       DISPLAY "SOMETHING"
       $SET AREACHECK
       MAIN-2 SECTION.
       DISPLAY "SOMETHING ELSE"
       STOP RUN.
