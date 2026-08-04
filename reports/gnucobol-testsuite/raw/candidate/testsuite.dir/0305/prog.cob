
       IDENTIFICATION DIVISION.
       PROGRAM-ID.    prog.
       PROCEDURE      DIVISION.
           CALL STATIC "stuff"
              ON EXCEPTION
                 CONTINUE
           END-CALL
           GOBACK.
