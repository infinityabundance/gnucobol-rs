      *> DISPLAY ... UPON ENVIRONMENT-NAME / ENVIRONMENT-VALUE set the runtime environment (no stdout);
      *> ACCEPT ... FROM ENVIRONMENT-VALUE / ENVIRONMENT "name" read it back. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P157.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 V PIC X(8) VALUE SPACES.
       PROCEDURE DIVISION.
           DISPLAY "MYVAR" UPON ENVIRONMENT-NAME.
           DISPLAY "hello" UPON ENVIRONMENT-VALUE.
           ACCEPT V FROM ENVIRONMENT-VALUE.
           DISPLAY "A=[" V "]".
           ACCEPT V FROM ENVIRONMENT "MYVAR".
           DISPLAY "B=[" V "]".
           STOP RUN.
