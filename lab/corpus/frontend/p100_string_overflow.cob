       IDENTIFICATION DIVISION.
       PROGRAM-ID. P100.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC X(4) VALUE "ABCD".
       01 B PIC X(4) VALUE "WXYZ".
       01 SMALL PIC X(6).
       01 BIG   PIC X(20).
       PROCEDURE DIVISION.
      *> overflow: 8 source chars into a 6-char target
           STRING A DELIMITED BY SIZE B DELIMITED BY SIZE INTO SMALL
               ON OVERFLOW DISPLAY "OVERFLOW small=[" SMALL "]"
               NOT ON OVERFLOW DISPLAY "OK"
           END-STRING.
      *> no overflow: fits in 20
           STRING A DELIMITED BY SIZE B DELIMITED BY SIZE INTO BIG
               ON OVERFLOW DISPLAY "OF2"
               NOT ON OVERFLOW DISPLAY "NOOF big=[" BIG "]"
           END-STRING.
           STOP RUN.
