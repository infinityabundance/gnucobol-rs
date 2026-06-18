      *> USAGE INDEX (a signed index item cobc DISPLAYs as S9(9); SET TO / SET UP BY) and GLOBAL/EXTERNAL
      *> (scope clauses that do not change the byte layout). Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P82.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 IX USAGE INDEX.
       01 G1 PIC X(4) GLOBAL VALUE "abcd".
       01 E1 PIC 9(3) EXTERNAL VALUE 7.
       PROCEDURE DIVISION.
           SET IX TO 2.    DISPLAY "IX=[" IX "]".
           SET IX UP BY 3. DISPLAY "IX2=[" IX "]".
           DISPLAY "G1=[" G1 "] E1=[" E1 "]".
           STOP RUN.
