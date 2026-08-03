*> FD record descriptions share ONE record area (GnuCOBOL union): a MOVE into one record is
*> visible through the other, and WRITE of a record emits the shared bytes (GNURUST.FILEIO.MULTI-RECORD-FD.1).
*> @format: free
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P187.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN TO "p187.dat" ORGANIZATION SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD F.
       01 FIRST-REC PIC X(5).
       01 SECOND-REC PIC X(5).
       WORKING-STORAGE SECTION.
       01 EOF-FLAG PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT F.
           MOVE "11111" TO FIRST-REC.
           WRITE SECOND-REC.
           MOVE "22222" TO SECOND-REC.
           WRITE FIRST-REC.
           CLOSE F.
           OPEN INPUT F.
           PERFORM UNTIL EOF-FLAG = "Y"
              READ F AT END MOVE "Y" TO EOF-FLAG
                   NOT AT END DISPLAY "[" FIRST-REC ";" SECOND-REC "]" END-READ
           END-PERFORM.
           CLOSE F.
           STOP RUN.
