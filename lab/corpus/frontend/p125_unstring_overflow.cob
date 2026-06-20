       IDENTIFICATION DIVISION.
       PROGRAM-ID. P125.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SRC PIC X(12) VALUE "A,B,C,D,E".
       01 R1 PIC X(3).
       01 R2 PIC X(3).
       PROCEDURE DIVISION.
      *> 5 segments, only 2 receivers -> overflow (chars remain unexamined)
           UNSTRING SRC DELIMITED BY "," INTO R1 R2
              ON OVERFLOW DISPLAY "OVF1"
              NOT ON OVERFLOW DISPLAY "OK1"
           END-UNSTRING.
           DISPLAY "R1=[" R1 "] R2=[" R2 "]".
      *> source fully consumed -> no overflow
           MOVE "X,Y" TO SRC.
           UNSTRING SRC DELIMITED BY "," INTO R1 R2
              ON OVERFLOW DISPLAY "OVF2"
              NOT ON OVERFLOW DISPLAY "OK2"
           END-UNSTRING.
           DISPLAY "R1=[" R1 "] R2=[" R2 "]".
           STOP RUN.
