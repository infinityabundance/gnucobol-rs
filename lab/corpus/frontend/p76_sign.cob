      *> SIGN IS LEADING / TRAILING / SEPARATE for signed DISPLAY numerics. The stored sign placement is
      *> exposed via a REDEFINES X view; byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P76.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC S9(4) SIGN IS LEADING VALUE -42.
       01 B PIC S9(4) SIGN IS TRAILING SEPARATE VALUE -42.
       01 C PIC S9(4) SIGN IS LEADING SEPARATE VALUE -42.
       01 D REDEFINES C PIC X(5).
       PROCEDURE DIVISION.
           DISPLAY "A=[" A "]".
           DISPLAY "B=[" B "]".
           DISPLAY "C=[" C "] raw=[" D "]".
           ADD 100 TO A.
           DISPLAY "A2=[" A "]".
           STOP RUN.
