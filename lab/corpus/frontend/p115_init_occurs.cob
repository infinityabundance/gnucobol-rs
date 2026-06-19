       IDENTIFICATION DIVISION.
       PROGRAM-ID. P115.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 T1.
          05 E OCCURS 3.
             10 N PIC 9(3) VALUE 111.
             10 A PIC X(2) VALUE "zz".
       PROCEDURE DIVISION.
           INITIALIZE T1.
           DISPLAY "p [" N(1) "][" A(1) "][" N(2) "][" A(3) "]".
           MOVE 111 TO N(1) N(2) N(3).
           MOVE "zz" TO A(1) A(2) A(3).
           INITIALIZE T1 REPLACING NUMERIC BY 7.
           DISPLAY "r [" N(1) "][" A(1) "][" N(2) "][" A(3) "]".
           STOP RUN.
