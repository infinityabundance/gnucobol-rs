
       IDENTIFICATION DIVISION.
       PROGRAM-ID.    prog.

       ENVIRONMENT    DIVISION.
       INPUT-OUTPUT   SECTION.
       FILE-CONTROL.
           SELECT f ASSIGN "f.txt"
               ORGANIZATION LINE SEQUENTIAL.

       DATA           DIVISION.
       FILE           SECTION.
       FD  f LINAGE 10.
       01  f-rec PIC X.

       PROCEDURE      DIVISION.
           WRITE f-rec FROM "x"
               NOT END-OF-PAGE
                   CONTINUE
               END-OF-PAGE
                   CONTINUE
           END-WRITE
           WRITE f-rec FROM "x"
               END-OF-PAGE
                   CONTINUE
               NOT END-OF-PAGE
                   CONTINUE
           END-WRITE
           WRITE f-rec FROM "x"
               NOT END-OF-PAGE
                   CONTINUE
           END-WRITE
           WRITE f-rec FROM "x"
               END-OF-PAGE
                   CONTINUE
           END-WRITE

           DISPLAY "blah"
               ON EXCEPTION
                   CALL "err"
                       NOT ON EXCEPTION
                           CONTINUE
                       ON EXCEPTION
                           CONTINUE.
           DISPLAY "blah"
               NOT ON EXCEPTION
                   CALL "err"
                       ON EXCEPTION
                           CONTINUE
                       NOT ON EXCEPTION
                           CONTINUE.
