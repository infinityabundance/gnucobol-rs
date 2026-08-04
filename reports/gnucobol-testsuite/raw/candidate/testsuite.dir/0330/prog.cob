
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  p                    PIC 99.
       01  o                    PIC X(5).
       PROCEDURE DIVISION.
          CALL "prog2"  USING p p o
                              p.
          CALL "prog3"  USING BY REFERENCE p
                              BY CONTENT   p o
                              BY REFERENCE p o(2:).
