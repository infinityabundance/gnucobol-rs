      *> START ... INVALID KEY / NOT INVALID KEY handler clauses on an INDEXED file.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P120.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT IXF ASSIGN "p120.dat"
              ORGANIZATION INDEXED ACCESS DYNAMIC RECORD KEY IS K.
       DATA DIVISION.
       FILE SECTION.
       FD IXF.
       01 R.
          05 K PIC 9(2).
          05 V PIC X(3).
       WORKING-STORAGE SECTION.
       PROCEDURE DIVISION.
           OPEN OUTPUT IXF.
           MOVE 10 TO K. MOVE "aaa" TO V. WRITE R.
           MOVE 20 TO K. MOVE "bbb" TO V. WRITE R.
           CLOSE IXF.
           OPEN I-O IXF.
           MOVE 15 TO K.
           START IXF KEY >= K
              INVALID KEY DISPLAY "none"
              NOT INVALID KEY DISPLAY "ok"
           END-START.
           MOVE 99 TO K.
           START IXF KEY >= K
              INVALID KEY DISPLAY "none99"
              NOT INVALID KEY DISPLAY "ok99"
           END-START.
           CLOSE IXF.
           STOP RUN.
