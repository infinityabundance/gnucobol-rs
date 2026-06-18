      *> USAGE forms in the front-end: COMP-3 (packed), COMP/BINARY, COMP-5, COMP-6, COMP-X. DISPLAY of
      *> the decoded value, COMPUTE and in-place ADD on binary/packed receivers -- all byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P73.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 P3   PIC S9(5)V99 COMP-3 VALUE -123.45.
       01 B4   PIC S9(4)    COMP   VALUE -42.
       01 B5   PIC 9(9)     COMP-5 VALUE 1000000.
       01 C6   PIC 9(4)     COMP-6 VALUE 1234.
       01 CX   PIC 9(4)     COMP-X VALUE 255.
       01 R    PIC S9(7)V99 VALUE 0.
       PROCEDURE DIVISION.
           DISPLAY "P3=[" P3 "]".
           DISPLAY "B4=[" B4 "] B5=[" B5 "]".
           DISPLAY "C6=[" C6 "] CX=[" CX "]".
           COMPUTE R = P3 * 2.
           DISPLAY "R=[" R "]".
           ADD 8 TO B4.
           DISPLAY "B4b=[" B4 "]".
           STOP RUN.
