
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog-2.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  data-b  PIC 9.
       LINKAGE SECTION.
       01  stuff   PIC x.
       PROCEDURE DIVISION USING stuff.
       MAIN.
           MOVE FUNCTION NUMVAL (stuff) TO data-b
           DISPLAY data-b
           GO TO EX

           ACCEPT stuff.

       EX. STOP RUN.

