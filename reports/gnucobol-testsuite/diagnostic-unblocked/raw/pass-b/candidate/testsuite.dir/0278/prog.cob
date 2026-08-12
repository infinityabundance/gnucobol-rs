
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 X             PIC X.
       PROCEDURE        DIVISION.
           ACCEPT X                       END-ACCEPT.
           ACCEPT X WITH    AUTO          END-ACCEPT.
           ACCEPT X WITH    AUTO-SKIP     END-ACCEPT.
           ACCEPT X WITH    AUTOTERMINATE END-ACCEPT.
           ACCEPT X WITH    TAB           END-ACCEPT.
           STOP RUN.
