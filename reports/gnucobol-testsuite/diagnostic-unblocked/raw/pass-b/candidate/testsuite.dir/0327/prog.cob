
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.
       PROCEDURE     DIVISION.
           CALL EXTERN 'callee'.
           CALL 'callee' WITH C LINKAGE.
           CALL
              EXTERN
              'callee'
              WITH C LINKAGE
              .
           STOP RUN.
