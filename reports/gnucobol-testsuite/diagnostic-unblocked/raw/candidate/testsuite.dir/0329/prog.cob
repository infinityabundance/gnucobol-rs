
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.
       DATA DIVISION.
       LINKAGE SECTION.
       01  p                    PIC 9.
       01  o                    PIC X(5).
       PROCEDURE DIVISION USING p p o.
       ENTRY "sub" USING o p o.
