      *> Report-generation workload: grouping, totals, page formatting.
      *> Input: "D0 0000012345" per line (dept, amount cents).
      *> Output: per-department subtotal lines plus a grand total; the
      *> generated report is written to report.out as fixed lines.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. REPORTWORK.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT RP-IN ASSIGN TO "report.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT RP-OUT ASSIGN TO "report.out"
               ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD  RP-IN.
       01  RP-REC.
           05  RP-DEPT       PIC X(2).
           05  RP-AMOUNT     PIC 9(10).
       FD  RP-OUT.
       01  OUT-REC           PIC X(80).
       WORKING-STORAGE SECTION.
       01  DEPT-TOTAL        PIC 9(14) VALUE 0.
       01  GRAND-TOTAL       PIC 9(16) VALUE 0.
       01  CUR-DEPT          PIC X(2) VALUE SPACES.
       01  ROW-COUNT         PIC 9(9) VALUE 0.
       01  DEPT-N            PIC 9(9) VALUE 0.
       01  AMT-E             PIC Z(13)9.
       01  ROW-E             PIC 9(9).
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
           OPEN INPUT RP-IN OUTPUT RP-OUT.
           PERFORM UNTIL EXIT
               READ RP-IN
                   AT END EXIT PERFORM
               END-READ
               IF RP-DEPT NOT = CUR-DEPT AND CUR-DEPT NOT = SPACES
      *>          emit the previous department's subtotal
                   MOVE CUR-DEPT TO OUT-BUF(1:2)
                   MOVE DEPT-TOTAL TO AMT-E
                   MOVE SPACES TO OUT-BUF(3:78)
                   STRING "SUBTOTAL " AMT-E DELIMITED BY SIZE
                       INTO OUT-BUF(3:78) END-STRING
                   MOVE OUT-BUF TO OUT-REC
                   WRITE OUT-REC
                   ADD 1 TO DEPT-N
               END-IF
               MOVE RP-DEPT TO CUR-DEPT
               ADD RP-AMOUNT TO DEPT-TOTAL
               ADD RP-AMOUNT TO GRAND-TOTAL
               ADD 1 TO ROW-COUNT
           END-PERFORM
      *>    last department subtotal
           IF CUR-DEPT NOT = SPACES
               MOVE CUR-DEPT TO OUT-BUF(1:2)
               MOVE DEPT-TOTAL TO AMT-E
               MOVE SPACES TO OUT-BUF(3:78)
               STRING "SUBTOTAL " AMT-E DELIMITED BY SIZE
                   INTO OUT-BUF(3:78) END-STRING
               MOVE OUT-BUF TO OUT-REC
               WRITE OUT-REC
               ADD 1 TO DEPT-N
           END-IF
           MOVE GRAND-TOTAL TO AMT-E
           MOVE ROW-COUNT TO ROW-E
           MOVE SPACES TO OUT-BUF
           STRING "GRAND " ROW-E " " AMT-E DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           CLOSE RP-IN RP-OUT.
           STOP RUN.
