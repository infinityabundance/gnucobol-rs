       IDENTIFICATION DIVISION.
       PROGRAM-ID. P130.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G1.
          05 A PIC 99   VALUE 11.
          05 B PIC X(3) VALUE "abc".
          05 C PIC 99   VALUE 5.
       01 G2.
          05 A PIC 99   VALUE 99.
          05 B PIC X(3) VALUE "zzz".
          05 D PIC 99   VALUE 7.
       PROCEDURE DIVISION.
           MOVE CORRESPONDING G1 TO G2.
           DISPLAY "MOVE A2=" A OF G2 " B2=" B OF G2 " D2=" D OF G2.
           ADD CORRESPONDING G1 TO G2.
           DISPLAY "ADD A2=" A OF G2.
           SUBTRACT CORRESPONDING G1 FROM G2.
           DISPLAY "SUB A2=" A OF G2.
           STOP RUN.
