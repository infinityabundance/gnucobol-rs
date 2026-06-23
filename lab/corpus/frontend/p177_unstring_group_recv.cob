      *> UNSTRING into a GROUP receiver: cobc treats the group as one alphanumeric field of its full byte
      *> length, storing the delimited segment left-justified and space-padded -- identical to a PIC X of the
      *> same size. COUNT/DELIMITER and a following elementary receiver work alongside it. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P177.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SRC PIC X(20) VALUE "ABC,DEFGH,IJ".
       01 G1.
          05 A PIC X(3).
          05 B PIC X(3).
       01 R2  PIC X(5).
       01 R3  PIC X(5).
       01 C1  PIC 99.
       PROCEDURE DIVISION.
           UNSTRING SRC DELIMITED BY "," INTO G1 COUNT C1 R2 R3.
           DISPLAY "G1=[" G1 "]".
           DISPLAY "A=[" A "] B=[" B "]".
           DISPLAY "C1=" C1.
           DISPLAY "R2=[" R2 "]".
           DISPLAY "R3=[" R3 "]".
           STOP RUN.
