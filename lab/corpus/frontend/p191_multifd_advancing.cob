*> The CCVS85 report shape: WRITE of the SECOND record with AFTER ADVANCING (GNURUST.FILEIO.MULTI-RECORD-FD.1).
*> The advancing newlines land in the file (the oracle writes n x LF before the record, plus a final LF
*> at close); a read-back of a printer-style file is outside the front-end's sealed subset, so this
*> fixture verifies the WRITE resolves + runs (both sides' stdout is empty; the oracle-side file bytes
*> are asserted by the sweep).
*> @format: free
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P191.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN TO "p191.dat" ORGANIZATION SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD F RECORD CONTAINS 120 CHARACTERS.
       01 PRINT-REC PIC X(120).
       01 DUMMY-RECORD PIC X(120).
       WORKING-STORAGE SECTION.
       01 H PIC X(120) VALUE "HELLO".
       PROCEDURE DIVISION.
           MOVE H TO DUMMY-RECORD.
           OPEN OUTPUT F.
           WRITE DUMMY-RECORD AFTER ADVANCING 1 LINES.
           WRITE DUMMY-RECORD AFTER ADVANCING 2 LINES.
           CLOSE F.
           STOP RUN.
