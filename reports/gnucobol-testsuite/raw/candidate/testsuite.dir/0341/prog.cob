
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       78  NUM2      VALUE   3.
       78  NEG1      VALUE   -1.
       78  CON3      VALUE   ((1 + 2) * NUM2 - 4.
       78  CON4      VALUE   (1 + 2) * NUM2 - 4).
       78  CON5      VALUE   (1 + 2) // NUM2.
       78  CON6      VALUE   (1 + 2 + 3 + (4)) / (NUM2).
      *
       PROCEDURE DIVISION.
       MAIN.
           STOP RUN.
