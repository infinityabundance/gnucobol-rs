      *> INDEXED file: WRITE out of key order, READ NEXT in ascending RECORD KEY order, random READ by
      *> key, START KEY >=, DELETE by key. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P117.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT IXF ASSIGN TO "p117.dat"
              ORGANIZATION IS INDEXED ACCESS DYNAMIC RECORD KEY IS K.
       DATA DIVISION.
       FILE SECTION.
       FD IXF.
       01 R.
          05 K PIC 9(2).
          05 V PIC X(3).
       WORKING-STORAGE SECTION.
       01 EOFF PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT IXF.
           MOVE 30 TO K. MOVE "ccc" TO V. WRITE R.
           MOVE 10 TO K. MOVE "aaa" TO V. WRITE R.
           MOVE 20 TO K. MOVE "bbb" TO V. WRITE R.
           CLOSE IXF.
           OPEN INPUT IXF.
           PERFORM UNTIL EOFF = "Y"
              READ IXF NEXT AT END MOVE "Y" TO EOFF END-READ
              IF EOFF = "N" DISPLAY "seq " K " " V END-IF
           END-PERFORM.
           CLOSE IXF.
           OPEN I-O IXF.
           MOVE 20 TO K. READ IXF. DISPLAY "key " K " " V.
           MOVE 15 TO K. START IXF KEY >= K.
           READ IXF NEXT. DISPLAY "ge  " K " " V.
           MOVE 10 TO K. DELETE IXF.
           CLOSE IXF.
           MOVE "N" TO EOFF.
           OPEN INPUT IXF.
           PERFORM UNTIL EOFF = "Y"
              READ IXF NEXT AT END MOVE "Y" TO EOFF END-READ
              IF EOFF = "N" DISPLAY "aft " K " " V END-IF
           END-PERFORM.
           CLOSE IXF.
           STOP RUN.
