       IDENTIFICATION DIVISION.
       PROGRAM-ID. P112.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S PIC X(8) VALUE "AABBAABB".
       01 C PIC 9(3) VALUE 0.
       PROCEDURE DIVISION.
           INSPECT S TALLYING C FOR ALL "A"
                     REPLACING ALL "B" BY "X".
           DISPLAY "[" S "] C=" C.
           STOP RUN.
