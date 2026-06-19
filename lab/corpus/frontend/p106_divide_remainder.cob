       IDENTIFICATION DIVISION.
       PROGRAM-ID. P106.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 Q  PIC 9(3).
       01 R  PIC S9(3).
       01 Q2 PIC 9(2)V9.
       01 R2 PIC 9(2)V99.
       PROCEDURE DIVISION.
           DIVIDE 17 BY 5 GIVING Q REMAINDER R.
           DISPLAY "a " Q "/" R.
           DIVIDE 5 INTO 17 GIVING Q REMAINDER R.
           DISPLAY "b " Q "/" R.
           DIVIDE 17 BY 5 GIVING Q2 REMAINDER R2.
           DISPLAY "c " Q2 "/" R2.
           DIVIDE 147 BY 10 GIVING Q ROUNDED REMAINDER R.
           DISPLAY "d " Q "/" R.
           DIVIDE -17 BY 5 GIVING Q REMAINDER R.
           DISPLAY "e " Q "/" R.
           STOP RUN.
