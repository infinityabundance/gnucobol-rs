       IDENTIFICATION DIVISION.
       PROGRAM-ID. P129.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SRC PIC X(8) VALUE "12,34,56".
       01 E1 PIC ZZ9.
       01 N1 PIC 9V9.
       01 A1 PIC XX.
       PROCEDURE DIVISION.
      *> UNSTRING into a numeric-edited (ZZ9) and a scaled DISPLAY (9V9) receiver
           UNSTRING SRC DELIMITED BY "," INTO E1 N1 A1.
           DISPLAY "E1=[" E1 "] N1=" N1 " A1=[" A1 "]".
           STOP RUN.
