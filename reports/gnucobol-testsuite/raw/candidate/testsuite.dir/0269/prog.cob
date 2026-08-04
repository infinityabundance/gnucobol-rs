
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
           CRT STATUS IS MY-CRT-STATUS.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 X             PIC X.
       PROCEDURE        DIVISION.
           ACCEPT X END-ACCEPT.
           STOP RUN.
