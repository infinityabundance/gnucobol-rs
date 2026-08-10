      *> Mixed business workflow: file input, validation, numeric computation,
      *> table processing, module calls, report, error handling.
      *> Input: "K0001 0042 00001234 01" (key, qty, price-cents, dept).
      *> For each record: validates qty>0, computes line total with a dept
      *> discount (table lookup), calls a module to apply a surcharge, and
      *> writes a report line. Output: per-record lines then "DONE <n> <sum>".
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MIXED.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT MX-IN ASSIGN TO "mixed.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT MX-OUT ASSIGN TO "mixed.out"
               ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD  MX-IN.
       01  MX-REC.
           05  MX-KEY        PIC X(8).
           05  MX-QTY        PIC 9(4).
           05  MX-PRICE-C    PIC 9(8).
           05  MX-DEPT       PIC 9(2).
       FD  MX-OUT.
       01  OUT-REC           PIC X(80).
       WORKING-STORAGE SECTION.
       01  DISCOUNT-TABLE.
           05  D-ENTRY OCCURS 5 TIMES.
               10  D-DEPT    PIC 9(2).
               10  D-PCT     PIC 9(2).
       01  TOTALS.
           05  T-SUM         PIC S9(16) COMP-3 VALUE 0.
           05  T-COUNT       PIC 9(9) COMP-3 VALUE 0.
           05  T-REJECTS     PIC 9(9) COMP-3 VALUE 0.
       01  LINE-TOTAL-C      PIC S9(14).
       01  DISC-C            PIC S9(14).
       01  NET-C             PIC S9(14).
       01  SURC-C            PIC S9(14).
       01  PCT               PIC 9(2).
       01  I                 PIC 9(2).
       01  K-E               PIC X(8).
       01  N-E               PIC 9(9).
       01  N-E2              PIC 9(9).
       01  V-E               PIC Z(13)9.
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
           MOVE 1 TO D-DEPT (1)
           MOVE 05 TO D-PCT (1)
           MOVE 2 TO D-DEPT (2)
           MOVE 10 TO D-PCT (2)
           MOVE 3 TO D-DEPT (3)
           MOVE 15 TO D-PCT (3)
           MOVE 4 TO D-DEPT (4)
           MOVE 20 TO D-PCT (4)
           MOVE 5 TO D-DEPT (5)
           MOVE 25 TO D-PCT (5)
           OPEN INPUT MX-IN OUTPUT MX-OUT.
           PERFORM UNTIL EXIT
               READ MX-IN
                   AT END EXIT PERFORM
               END-READ
               IF MX-QTY = 0
                   ADD 1 TO T-REJECTS
                   MOVE MX-KEY TO K-E
                   MOVE SPACES TO OUT-BUF
                   STRING "REJECT " K-E DELIMITED BY SIZE
                       INTO OUT-BUF END-STRING
                   MOVE OUT-BUF TO OUT-REC
                   WRITE OUT-REC
                   EXIT PERFORM CYCLE
               END-IF
               COMPUTE LINE-TOTAL-C = MX-QTY * MX-PRICE-C
               MOVE 0 TO PCT
               PERFORM VARYING I FROM 1 BY 1 UNTIL I > 5
                   IF D-DEPT (I) = MX-DEPT
                       MOVE D-PCT (I) TO PCT
                   END-IF
               END-PERFORM
               COMPUTE DISC-C ROUNDED = LINE-TOTAL-C * PCT / 100
               COMPUTE NET-C = LINE-TOTAL-C - DISC-C
               CALL "MIXMOD" USING NET-C SURC-C
               ADD SURC-C TO NET-C
               ADD NET-C TO T-SUM
               ADD 1 TO T-COUNT
               MOVE MX-KEY TO K-E
               MOVE NET-C TO V-E
               MOVE SPACES TO OUT-BUF
               STRING K-E " " V-E DELIMITED BY SIZE
                   INTO OUT-BUF END-STRING
               MOVE OUT-BUF TO OUT-REC
               WRITE OUT-REC
           END-PERFORM
           MOVE T-COUNT TO N-E
           MOVE T-SUM TO V-E
           MOVE T-REJECTS TO N-E2
           MOVE SPACES TO OUT-BUF
           STRING "DONE " N-E " " V-E " REJECTS " N-E2
               DELIMITED BY SIZE INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           CLOSE MX-IN MX-OUT.
           STOP RUN.
