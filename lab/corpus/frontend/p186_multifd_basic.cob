*> Two alternative 01 records beneath one FD (GNURUST.FILEIO.MULTI-RECORD-FD.1):
*> WRITE of either record emits its own bytes; read-back shows both, in source order.
*> @format: free
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P186.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN TO "p186.dat" ORGANIZATION SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD F.
       01 FIRST-REC PIC X(5).
       01 SECOND-REC PIC X(5).
       WORKING-STORAGE SECTION.
       01 EOF-FLAG PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT F.
           MOVE "FIRST" TO FIRST-REC.
           WRITE FIRST-REC.
           MOVE "OTHER" TO SECOND-REC.
           WRITE SECOND-REC.
           CLOSE F.
           OPEN INPUT F.
           PERFORM UNTIL EOF-FLAG = "Y"
              READ F AT END MOVE "Y" TO EOF-FLAG
                   NOT AT END DISPLAY "[" FIRST-REC "]" END-READ
           END-PERFORM.
           CLOSE F.
           STOP RUN.
