      *> STRING concatenation (DELIMITED BY SIZE / by literal, preserving the target tail) and UNSTRING
      *> split (DELIMITED BY a literal into receiving fields). Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P48.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 OUT1  PIC X(11) VALUE "-----------".
       01 FNAME PIC X(5)  VALUE "JANE".
       01 LNAME  PIC X(6)  VALUE "DOE".
       01 SRC   PIC X(11) VALUE "RED,GREEN".
       01 P1    PIC X(5).
       01 P2    PIC X(6).
       PROCEDURE DIVISION.
           STRING FNAME DELIMITED BY " " "," DELIMITED BY SIZE
                  LNAME DELIMITED BY " " INTO OUT1.
           DISPLAY "STR=[" OUT1 "]".
           UNSTRING SRC DELIMITED BY "," INTO P1 P2.
           DISPLAY "P1=[" P1 "] P2=[" P2 "]".
           STOP RUN.
