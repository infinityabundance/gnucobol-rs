       IDENTIFICATION DIVISION.
       PROGRAM-ID. P107.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 NUM PIC 9(3)  VALUE 111.
          05 ALN PIC X(3)  VALUE "aaa".
          05 ALB PIC A(3)  VALUE "bbb".
          05 NED PIC ZZ9   VALUE 222.
       01 SOLO PIC 9(4) VALUE 7777.
       PROCEDURE DIVISION.
           INITIALIZE G REPLACING NUMERIC DATA BY 7.
           DISPLAY "1 [" NUM "][" ALN "][" ALB "][" NED "]".
           INITIALIZE G REPLACING ALPHANUMERIC BY "Q".
           DISPLAY "2 [" NUM "][" ALN "][" ALB "][" NED "]".
           INITIALIZE G REPLACING ALPHABETIC BY "Z".
           DISPLAY "3 [" NUM "][" ALN "][" ALB "][" NED "]".
           INITIALIZE G REPLACING NUMERIC-EDITED BY 5.
           DISPLAY "4 [" NUM "][" ALN "][" ALB "][" NED "]".
           INITIALIZE G REPLACING NUMERIC BY ZERO
                                  ALPHANUMERIC BY SPACE.
           DISPLAY "5 [" NUM "][" ALN "][" ALB "][" NED "]".
           INITIALIZE SOLO REPLACING NUMERIC BY 3.
           DISPLAY "6 [" SOLO "]".
           STOP RUN.
