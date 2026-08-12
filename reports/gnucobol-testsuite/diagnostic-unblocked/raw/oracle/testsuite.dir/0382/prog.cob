
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.
       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       78 Y            VALUE 1.
       78 Y2           VALUE 1.
       78 X            VALUE 2.
       78 Z            VALUE 354.
       PROCEDURE       DIVISION.
      $IF Y = Y2X
      $DISPLAY correct Y = Y2
      $ELSE
      $DISPLAY bad: Y should be = Y2
      $END
      $IF Y > X
      $DISPLAY BAD - Y is not > X
      $ELIF Y < X
      $DISPLAY correct Y < X
      $ELSE
      $DISPLAY BROKEN
      $END

      $IF X > Y
      $DISPLAY correct X > Y
      $ELIF X < Y
      $DISPLAY BAD - X is not < Y
      $ELSE
      $DISPLAY BROKEN
      $END
           CONTINUE
           .
