
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.
       DATA DIVISION.
       LINKAGE SECTION.
      * constants may occur anywhere but don't belong to LINKAGE
       78 NSIZE VALUE 1.
      *
      * two variables in USING
       77 A PIC X.
       01 B.
          03 B1 PIC X.
          03 B2 PIC 9(NSIZE).
      * variable not referenced anywhere - no warning
       77 C PIC X.
      * variable referenced by its REDEFINE
       01 D PIC XX.
       01 filler redefines D.
          03 D1 PIC X.
          03 D2 PIC 9(NSIZE).
      * variable referenced by its second REDEFINE
       01 E PIC XX.
       01 filler redefines E.
          03 Ea1 PIC X.
          03 Ea2 PIC 9(NSIZE).
       01 filler redefines E.
          03 Eb1 PIC X.
          03 Eb2 PIC 9(NSIZE).
      * variable referenced by its child
       01 F.
          03 F1 PIC X.
          03 F2 PIC 9(NSIZE).
      * variable referenced by level 88 (a validation entry)
       01 G.
          03 filler  PIC X.
             88 g-val-a value 'a'.
             88 g-val-b value 'b'.
       PROCEDURE       DIVISION USING A B.

           IF D2 OMITTED OR Eb2 OMITTED or F2 OMITTED
             set g-val-b to true
           END-IF
           .
