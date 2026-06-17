      *> Sequential file I/O: OPEN OUTPUT + WRITE records, CLOSE, OPEN INPUT + READ loop with AT END,
      *> LINE SEQUENTIAL (trailing spaces trimmed on write, padded on read), with FILE STATUS.
      *> Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P52.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT WORKFILE ASSIGN TO "p52.dat"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS WS-STAT.
       DATA DIVISION.
       FILE SECTION.
       FD WORKFILE.
       01 REC PIC X(10).
       WORKING-STORAGE SECTION.
       01 WS-STAT  PIC XX.
       01 EOF-FLAG PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT WORKFILE.
           MOVE "ALPHA" TO REC.
           WRITE REC.
           MOVE "BETA" TO REC.
           WRITE REC.
           MOVE "GAMMA" TO REC.
           WRITE REC.
           CLOSE WORKFILE.
           OPEN INPUT WORKFILE.
           PERFORM UNTIL EOF-FLAG = "Y"
               READ WORKFILE
                   AT END MOVE "Y" TO EOF-FLAG
               END-READ
               IF EOF-FLAG = "N" DISPLAY "REC=[" REC "]" END-IF
           END-PERFORM.
           CLOSE WORKFILE.
           DISPLAY "STAT=" WS-STAT.
           STOP RUN.
