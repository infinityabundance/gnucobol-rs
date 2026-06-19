       IDENTIFICATION DIVISION.
       PROGRAM-ID. P101.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC X(5) VALUE SPACES.
       01 Z PIC 9(4) VALUE ZEROS.
       01 N PIC 9(3).
       01 ZA PIC X(4).
       01 SEP PIC X(7).
       01 AB PIC X(5).
       PROCEDURE DIVISION.
           DISPLAY "A=[" A "]" " Z=[" Z "]".
           MOVE ZEROS TO N. MOVE ZEROS TO ZA.
           DISPLAY "N=[" N "] ZA=[" ZA "]".
           MOVE ALL "-" TO SEP. DISPLAY "SEP=[" SEP "]".
           MOVE ALL "ab" TO AB. DISPLAY "AB=[" AB "]".
           STOP RUN.
