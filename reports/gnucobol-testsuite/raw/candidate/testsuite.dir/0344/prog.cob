
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       DATA  DIVISION.
       WORKING-STORAGE SECTION.
       01  var       PIC X(200).
       01  othervar  PIC X(115).
       01  A         PIC 9 VALUE 1.
       01  B         PIC 9 VALUE 2.
       01  C         PIC 9 VALUE 3.
       78  var-len   value 115.

       PROCEDURE DIVISION.

           IF var-len < 16
               move othervar (1:var-len - 8)
                 to var (17 - var-len:var-len - 8)
           ELSE
               move othervar to var
           END-IF

           IF (2 = 3)
               move othervar to var
           ELSE
              IF 1 = 1
                 move var to othervar
              ELSE
                 move othervar (1:var-len - 8)
                   to var (17 - var-len:var-len - 8)
                 IF 1 = 1
                    move var to othervar
                 END-IF
              END-IF
              move A to B
              IF 1 = 1
                IF 2 = 1
                 move othervar (1:var-len - 8)
                   to var (17 - var-len:var-len - 8)
           .

           PERFORM WITH TEST BEFORE UNTIL 1 = 3
               move othervar to var
           END-PERFORM

           PERFORM WITH TEST BEFORE UNTIL 1 = 1
               move othervar (1:var-len - 8)
                 to var (17 - var-len:var-len - 8)
           END-PERFORM

           PERFORM WITH TEST AFTER UNTIL 1 = 3
               move othervar to var
           END-PERFORM

           PERFORM WITH TEST AFTER UNTIL 1 = 1
               move othervar to var
           END-PERFORM

           EVALUATE TRUE
           WHEN  var-len < 16
               move othervar (1:var-len - 8)
                 to var (17 - var-len:var-len - 8)
           WHEN  var-len > 16
               move othervar to var
           WHEN  A = B
               move var      to othervar
           WHEN OTHER
               CONTINUE
           END-EVALUATE

           EVALUATE FALSE
           WHEN  var-len < 16
               move othervar (1:var-len - 9)
                 to var (16 - var-len:var-len - 9)
           WHEN  var-len > 16
               move othervar to var
           WHEN  A = B
               move var      to othervar
           WHEN OTHER
               CONTINUE
           END-EVALUATE.

           IF 15 = var-len
               move othervar to var.

           IF var-len = 15
               move var      to othervar.

           STOP RUN.
