      *> Table processing: OCCURS, subscripts, indexes, SORT, SEARCH ALL,
      *> aggregation. Sorts the input by v1, loads a table, searches.
      *> Input: "00000 001234 005678" (id, v1, v2).
      *> Output: "FOUND <n> <sum-v1> <sum-v2>" / "MISSED <n>" /
      *>         "TABLE <rows> <total-v1> <total-v2>"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. TABLES.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT TB-IN ASSIGN TO "tables.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT TB-SORTED ASSIGN TO "tables.sorted"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT TB-OUT ASSIGN TO "tables.out"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT TB-WORK ASSIGN TO "tables.work".
       DATA DIVISION.
       FILE SECTION.
       FD  TB-IN.
       01  TB-REC.
           05  TB-ID         PIC 9(5).
           05  TB-V1         PIC 9(6).
           05  TB-V2         PIC 9(6).
       FD  TB-SORTED.
       01  TS-REC            PIC X(17).
       FD  TB-OUT.
       01  OUT-REC           PIC X(80).
       SD  TB-WORK.
       01  TW-REC.
           05  TW-ID        PIC 9(5).
           05  TW-V1        PIC 9(6).
           05  TW-V2        PIC 9(6).
       WORKING-STORAGE SECTION.
       01  TBL.
           05  T-ENTRY OCCURS 1 TO 500000 TIMES
                   DEPENDING ON T-ROWS
                   ASCENDING KEY IS T-V1
                   INDEXED BY T-IDX.
               10  T-V1      PIC 9(6).
               10  T-V2      PIC 9(6).
       01  T-ROWS            PIC 9(9) VALUE 0.
       01  TOT-V1            PIC 9(15) VALUE 0.
       01  TOT-V2            PIC 9(15) VALUE 0.
       01  FOUND-N           PIC 9(9) VALUE 0.
       01  MISSED-N          PIC 9(9) VALUE 0.
       01  FOUND-V1          PIC 9(15) VALUE 0.
       01  FOUND-V2          PIC 9(15) VALUE 0.
       01  I                 PIC 9(9).
       01  S-V1              PIC 9(6).
       01  N-E               PIC 9(9).
       01  V-E1              PIC Z(14)9.
       01  V-E2              PIC Z(14)9.
       01  V-E3              PIC Z(14)9.
       01  V-E4              PIC Z(14)9.
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
      *>    sort the input by v1 through a sort work file
           SORT TB-WORK ON ASCENDING KEY TW-V1
               USING TB-IN GIVING TB-SORTED
           OPEN INPUT TB-SORTED OUTPUT TB-OUT.
           PERFORM UNTIL EXIT
               READ TB-SORTED
                   AT END EXIT PERFORM
               END-READ
               ADD 1 TO T-ROWS
               MOVE TS-REC(6:6) TO T-V1 (T-ROWS)
               MOVE TS-REC(12:6) TO T-V2 (T-ROWS)
               ADD T-V1 (T-ROWS) TO TOT-V1
               ADD T-V2 (T-ROWS) TO TOT-V2
           END-PERFORM
           CLOSE TB-SORTED
      *>    search for every 10th row's key
           PERFORM VARYING I FROM 1 BY 1
               UNTIL I > T-ROWS
               IF FUNCTION MOD(I, 10) = 0
                   MOVE T-V1 (I) TO S-V1
                   SEARCH ALL T-ENTRY
                       AT END
                           ADD 1 TO MISSED-N
                       WHEN T-V1 (T-IDX) = S-V1
                           ADD 1 TO FOUND-N
                           ADD T-V1 (T-IDX) TO FOUND-V1
                           ADD T-V2 (T-IDX) TO FOUND-V2
                   END-SEARCH
               END-IF
           END-PERFORM
           MOVE FOUND-N TO N-E
           MOVE FOUND-V1 TO V-E1
           MOVE FOUND-V2 TO V-E2
           MOVE SPACES TO OUT-BUF
           STRING "FOUND " N-E " " V-E1 " " V-E2 DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           MOVE MISSED-N TO N-E
           MOVE SPACES TO OUT-BUF
           STRING "MISSED " N-E DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           MOVE T-ROWS TO N-E
           MOVE TOT-V1 TO V-E3
           MOVE TOT-V2 TO V-E4
           MOVE SPACES TO OUT-BUF
           STRING "TABLE " N-E " " V-E3 " " V-E4 DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           CLOSE TB-OUT.
           STOP RUN.
