       IDENTIFICATION DIVISION.
       PROGRAM-ID. P114.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S PIC X(8) VALUE "12,34,56".
       01 A PIC 9(3).
       01 B PIC 9(3).
       01 C PIC 9(3).
       PROCEDURE DIVISION.
           UNSTRING S DELIMITED BY "," INTO A B C.
           DISPLAY A "/" B "/" C.
           STOP RUN.
