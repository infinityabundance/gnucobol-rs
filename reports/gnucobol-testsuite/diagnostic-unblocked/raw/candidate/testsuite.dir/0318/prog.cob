
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       77  a   PIC X.
       77  b   PIC X.
       77  c   PIC XXX.

       PROCEDURE DIVISION.
           STRING   DELIMITED BY SPACE INTO
           END-STRING
           STRING a DELIMITED BY SPACE c
           END-STRING
           STRING a DELIMITED BY SPACE INTO
           END-STRING
           STRING
                  DELIMITED BY SPACE
                  INTO c
           END-STRING
           STRING a DELIMITED BY SPACE

                  DELIMITED BY SIZE
                  INTO c
           END-STRING
           STRING a DELIMITED BY SPACE
                  b DELIMITED BY SIZE
                  INTO c
           END-STRING
      *
           UNSTRING   DELIMITED BY SPACE INTO a
           END-UNSTRING
           UNSTRING c DELIMITED BY SPACE a
           END-UNSTRING
           UNSTRING c DELIMITED BY SPACE INTO
           END-UNSTRING
           UNSTRING   DELIMITED BY SPACE INTO a
           END-UNSTRING
           UNSTRING c DELIMITED BY SPACE INTO a
           END-UNSTRING
           UNSTRING c DELIMITED BY SPACE
                      DELIMITED BY SIZE  INTO a
           END-UNSTRING
           .
