      *> SYNCHRONIZED -- a binary item aligned to its natural boundary (COMP S9(4)->2, S9(9)->4), with a
      *> synthetic slack FILLER inserted before it in the group layout. FUNCTION LENGTH(G)=12 (1 + 1 slack
      *> + 2 + 1 + 3 slack + 4). Byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P87.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 G.
          05 S1 PIC X.
          05 S2 PIC S9(4) COMP SYNC.
          05 S3 PIC X.
          05 S4 PIC S9(9) COMP SYNC.
       PROCEDURE DIVISION.
           MOVE 1 TO S2. MOVE 2 TO S4.
           DISPLAY "LEN=[" FUNCTION LENGTH(G) "]".
           DISPLAY "S2=[" S2 "] S4=[" S4 "]".
           STOP RUN.
