      *> REWRITE under OPEN I-O (replace the last-read record), plus UNLOCK / COMMIT (no-ops). Record
      *> SEQUENTIAL, fixed length. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P53.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT WF ASSIGN TO "p53.dat"
               ORGANIZATION IS SEQUENTIAL
               FILE STATUS IS ST.
       DATA DIVISION.
       FILE SECTION.
       FD WF.
       01 REC PIC X(5).
       WORKING-STORAGE SECTION.
       01 ST   PIC XX.
       01 EOFF PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT WF.
           MOVE "AAAAA" TO REC.
           WRITE REC.
           MOVE "BBBBB" TO REC.
           WRITE REC.
           MOVE "CCCCC" TO REC.
           WRITE REC.
           CLOSE WF.
           OPEN I-O WF.
           READ WF AT END CONTINUE END-READ.
           READ WF AT END CONTINUE END-READ.
           MOVE "XXXXX" TO REC.
           REWRITE REC.
           UNLOCK WF.
           COMMIT.
           CLOSE WF.
           OPEN INPUT WF.
           PERFORM UNTIL EOFF = "Y"
               READ WF AT END MOVE "Y" TO EOFF END-READ
               IF EOFF = "N" DISPLAY "[" REC "]" END-IF
           END-PERFORM.
           CLOSE WF.
           DISPLAY "ST=" ST.
           STOP RUN.
