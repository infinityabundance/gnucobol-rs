
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       01  var         PIC 999.

       PROCEDURE       DIVISION.
           IF  var = 83  AND >  1 + 1
              CONTINUE
           END-IF
           IF  var = 83  AND >  2
              CONTINUE
           END-IF
           IF  var = 83  AND > (1 + 1)
              CONTINUE
           END-IF
           IF (var NOT = 1) OR (var NOT = 2)
               CONTINUE
           END-IF
           .
