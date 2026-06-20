       IDENTIFICATION DIVISION.
       PROGRAM-ID. P121.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SRC PIC X(20) VALUE "AA,BBB,CC,DDD".
       01 R1 PIC X(5).
       01 R2 PIC X(5).
       01 P  PIC 99 VALUE 4.
       01 TC PIC 99 VALUE 0.
       PROCEDURE DIVISION.
           UNSTRING SRC DELIMITED BY ","
              INTO R1 R2
              WITH POINTER P TALLYING IN TC.
           DISPLAY "R1=[" R1 "]".
           DISPLAY "R2=[" R2 "]".
           DISPLAY "P=" P " TC=" TC.
           STOP RUN.
