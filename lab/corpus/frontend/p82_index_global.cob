      *> USAGE INDEX (a signed index item cobc DISPLAYs as S9(9); SET TO / SET UP BY) and GLOBAL (a scope
      *> clause that does not change the byte layout). Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P82.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 IX USAGE INDEX.
       01 G1 PIC X(4) GLOBAL VALUE "abcd".
       PROCEDURE DIVISION.
           SET IX TO 2.    DISPLAY "IX=[" IX "]".
           SET IX UP BY 3. DISPLAY "IX2=[" IX "]".
           DISPLAY "G1=[" G1 "]".
           STOP RUN.
