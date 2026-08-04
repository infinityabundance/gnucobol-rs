
       IDENTIFICATION DIVISION.
       PROGRAM-ID.    prog2.
       PROCEDURE      DIVISION.
           CALL "stuff"
              ON EXCEPTION
                 CONTINUE
           END-CALL
           GOBACK.
