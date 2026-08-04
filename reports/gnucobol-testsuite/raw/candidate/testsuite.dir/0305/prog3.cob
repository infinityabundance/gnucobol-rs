
       >> CALL-CONVENTION STATIC
       IDENTIFICATION DIVISION.
       PROGRAM-ID.    prog3.
       PROCEDURE      DIVISION.
           CALL "stuff"
              ON EXCEPTION
                 CONTINUE
           END-CALL
           GOBACK.
