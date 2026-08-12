
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.
       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       78 Y            VALUE 'a'.
       PROCEDURE       DIVISION.
      $IF X DEFINED
      $DISPLAY X defined
      $ELIF Y DEFINED
      $DISPLAY X not defined, but Y via lvl 78
      $ELSE
      $DISPLAY X not defined
      $END
           CONTINUE
           .
