*> Alternative FD records of DIFFERENT lengths: each WRITE emits the NAMED record's own length,
*> and a shorter view over a longer record shows the shared area's first bytes (GNURUST.FILEIO.MULTI-RECORD-FD.1).
*> @format: free
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P188.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN TO "p188.dat" ORGANIZATION SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD F.
       01 SHORT-REC PIC X(3).
       01 LONG-REC  PIC X(6).
       01 MID-REC   PIC X(4).
       WORKING-STORAGE SECTION.
       01 EOF-FLAG PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT F.
           MOVE "ABC" TO SHORT-REC.
           WRITE SHORT-REC.
           MOVE "123456" TO LONG-REC.
           WRITE LONG-REC.
           MOVE "WXYZ" TO MID-REC.
           WRITE MID-REC.
           CLOSE F.
           OPEN INPUT F.
           PERFORM UNTIL EOF-FLAG = "Y"
              READ F AT END MOVE "Y" TO EOF-FLAG
                   NOT AT END DISPLAY "[" SHORT-REC "]" END-READ
           END-PERFORM.
           CLOSE F.
           STOP RUN.
