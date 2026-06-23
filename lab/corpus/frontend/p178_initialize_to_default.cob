      *> INITIALIZE ... TO DEFAULT: the explicit form of the standard default behaviour -- every elementary
      *> leaf is reset to its category default (numeric -> zero, alphanumeric -> spaces), identical to a bare
      *> INITIALIZE. A trailing REPLACING after the (reduced) default still applies. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P178.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 N  PIC 9(3)  VALUE 42.
          05 A  PIC X(4)  VALUE "abcd".
          05 S  PIC S9(2) VALUE -7.
       01 H.
          05 P  PIC 9(2)  VALUE 11.
          05 Q  PIC X(3)  VALUE "xyz".
       PROCEDURE DIVISION.
           INITIALIZE G TO DEFAULT.
           DISPLAY "G: N=[" N "] A=[" A "] S=[" S "]".
           INITIALIZE H REPLACING ALPHANUMERIC DATA BY "**".
           DISPLAY "H: P=[" P "] Q=[" Q "]".
           STOP RUN.
