
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       1 W-DATA.
         3 X            PIC X(1) VALUE "X".
         3 XX           PIC X(2) VALUE "XX".
         3 Z            PIC X(1) VALUE "Z".
         3 ZZ           PIC X(2) VALUE "ZZ".
         3 Y            PIC X(1) VALUE "Y".
       PROCEDURE        DIVISION.
           REPLACE LEADING "X" BY SPACES
                   TRAILING "Z" BY SPACES
                   LEADING "Y" BY "X".
       MAIN.
           DISPLAY "XX: *" XX "*"
           DISPLAY "X: *" X "*"
           DISPLAY "ZZ: *" ZZ "*"
           DISPLAY "Z: *" Z "*"
           DISPLAY "Y: *" Y "*"
           STOP RUN.
