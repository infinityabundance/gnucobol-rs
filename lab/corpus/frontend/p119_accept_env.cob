      *> @env: P119VAR=hello
      *> ACCEPT id FROM ENVIRONMENT "name": read a (pinned) environment variable.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P119.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 V PIC X(10).
       PROCEDURE DIVISION.
           ACCEPT V FROM ENVIRONMENT "P119VAR".
           DISPLAY "[" V "]".
           STOP RUN.
