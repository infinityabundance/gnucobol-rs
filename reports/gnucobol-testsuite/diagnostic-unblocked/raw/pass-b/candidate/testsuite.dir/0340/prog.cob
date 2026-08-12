
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  NUM2      CONSTANT 3.
       01  CON3      CONSTANT ((1 + 2) * NUM2 - 4.
       01  CON4      CONSTANT (1 + 2) * NUM2 - 4).
       01  CON5      CONSTANT (1 + 2) // NUM2.
       01  CON6      CONSTANT (1 + 2 + 3 + (4)) / (NUM2).
      *
       PROCEDURE DIVISION.
       MAIN.
           STOP RUN.
