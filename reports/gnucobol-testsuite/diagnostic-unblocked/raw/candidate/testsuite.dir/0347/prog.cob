
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
               IF IF.

           STOP RUN.
