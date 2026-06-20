      *> UNSTRING DELIMITED BY [ALL] d1 [OR [ALL] d2]...: earliest match splits, DELIMITER IN captures it,
      *> ALL collapses consecutive delimiters. Identical cobc/cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P145.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SRC PIC X(12) VALUE "A,B;C,D".
       01 R1 PIC X(3). 01 R2 PIC X(3). 01 R3 PIC X(3). 01 R4 PIC X(3).
       01 D1 PIC X. 01 D2 PIC X.
       01 S2 PIC X(8) VALUE "A,,,B".
       01 Q1 PIC X(3). 01 Q2 PIC X(3).
       PROCEDURE DIVISION.
           UNSTRING SRC DELIMITED BY "," OR ";" INTO
               R1 DELIMITER IN D1
               R2 DELIMITER IN D2
               R3 R4.
           DISPLAY "R1=[" R1 "] D1=[" D1 "] R2=[" R2 "] D2=[" D2 "] R3=[" R3 "] R4=[" R4 "]".
           UNSTRING S2 DELIMITED BY ALL "," INTO Q1 Q2.
           DISPLAY "Q1=[" Q1 "] Q2=[" Q2 "]".
           STOP RUN.
