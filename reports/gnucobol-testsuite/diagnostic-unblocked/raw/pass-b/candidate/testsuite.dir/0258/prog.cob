
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01  CON          CONSTANT 10.
       01  V            PIC 9.
       78  C78          VALUE 'A'.
       PROCEDURE DIVISION.
           INITIALIZE CON.
           INITIALIZE V.
           INITIALIZE V, 9.
           INITIALIZE C78, V.
