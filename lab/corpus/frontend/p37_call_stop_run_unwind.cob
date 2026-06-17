      *> STOP RUN inside a CALLed contained program unwinds the WHOLE run to the run boundary (libcob
      *> longjmp stop_run): the post-CALL statement in the caller must NOT execute. GOBACK / EXIT PROGRAM
      *> instead return to the caller. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P37MAIN.
       PROCEDURE DIVISION.
           DISPLAY "MAIN-BEFORE-CALL".
           CALL "P37SUB".
           DISPLAY "MAIN-AFTER-CALL-NEVER".
           STOP RUN.
       END PROGRAM P37MAIN.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P37SUB.
       PROCEDURE DIVISION.
           DISPLAY "SUB-BEFORE-STOP".
           STOP RUN.
           DISPLAY "SUB-AFTER-STOP-NEVER".
       END PROGRAM P37SUB.
