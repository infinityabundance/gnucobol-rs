      *> FUNCTION MODULE-SOURCE -- the source-file path the host is running. cobc embeds the source name it
      *> was given; the interpreter knows the .cob it was invoked with. Same path under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P78.
       PROCEDURE DIVISION.
           DISPLAY "SRC=[" FUNCTION MODULE-SOURCE "]".
           STOP RUN.
