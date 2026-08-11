      *> Micro: sequential-file read/write: 50_000 fixed records read from
      *> micro_seqfile.dat (same layout as the corpus seqfile workload) and
      *> echoed to stdout with VALID/INVALID classification.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MICRO-SEQFILE.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT MS-IN ASSIGN TO "micro_seqfile.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD  MS-IN.
       01  MS-REC.
           05  MS-KEY        PIC X(8).
           05  MS-AMOUNT     PIC 9(12).
           05  MS-CODE       PIC XX.
       WORKING-STORAGE SECTION.
       01  VALID-SUM         PIC S9(16) VALUE 0.
       01  VALID-N           PIC 9(9) VALUE 0.
       01  INVALID-N         PIC 9(9) VALUE 0.
       01  AMT-E             PIC 9(12).
       01  N-E               PIC 9(9).
       01  S-E               PIC Z(15)9.
       PROCEDURE DIVISION.
           OPEN INPUT MS-IN.
           PERFORM UNTIL EXIT
               READ MS-IN
                   AT END EXIT PERFORM
               END-READ
               IF MS-CODE = "OK"
                   ADD MS-AMOUNT TO VALID-SUM
                   ADD 1 TO VALID-N
               ELSE
                   ADD 1 TO INVALID-N
               END-IF
               MOVE MS-AMOUNT TO AMT-E
               DISPLAY MS-KEY " " AMT-E
           END-PERFORM
           CLOSE MS-IN.
           MOVE VALID-N TO N-E
           MOVE VALID-SUM TO S-E
           DISPLAY "VALID " N-E " " S-E
           MOVE INVALID-N TO N-E
           DISPLAY "INVALID " N-E
           STOP RUN.
