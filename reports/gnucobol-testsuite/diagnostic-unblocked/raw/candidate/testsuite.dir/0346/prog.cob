
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA  DIVISION.
       WORKING-STORAGE SECTION.
       01  var       PIC X(200).
       78  var-len   value 115.

       PROCEDURE DIVISION.

           IF var-len < 16
               move spaces
                 to var (17 - var-len:var-len - 8)
           END-IF

           IF var-len < 16
              move notdefined to var
           END-IF

           IF var-len < 16
              perform notdefined
           END-IF

           IF var-len < 16
              if notdefined continue.

           STOP RUN.
