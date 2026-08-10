      *> Sequential-file batch: large fixed records, validation, aggregation,
      *> file-status checks. Input: "K00000000 000000123456 OK" (key, amount
      *> cents, code). Valid = code "OK". Output: one line per record with the
      *> running balance, then "VALID <n> <sum>" / "INVALID <n> <sum>".
       IDENTIFICATION DIVISION.
       PROGRAM-ID. SEQFILE.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT SF-IN ASSIGN TO "seqfile.dat"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS FS-IN.
           SELECT SF-OUT ASSIGN TO "seqfile.out"
               ORGANIZATION IS LINE SEQUENTIAL
               FILE STATUS IS FS-OUT.
       DATA DIVISION.
       FILE SECTION.
       FD  SF-IN.
       01  SF-REC.
           05  SF-KEY        PIC X(8).
           05  SF-AMOUNT     PIC 9(12).
           05  SF-CODE       PIC XX.
       FD  SF-OUT.
       01  OUT-REC           PIC X(80).
       WORKING-STORAGE SECTION.
       01  FS-IN             PIC XX.
       01  FS-OUT            PIC XX.
       01  BALANCE           PIC S9(16) VALUE 0.
       01  VALID-SUM         PIC S9(16) VALUE 0.
       01  INVALID-SUM       PIC S9(16) VALUE 0.
       01  VALID-N           PIC 9(9) VALUE 0.
       01  INVALID-N         PIC 9(9) VALUE 0.
       01  AMT-S             PIC 9(12).
       01  BAL-E             PIC Z(15)9.
       01  N-E               PIC 9(9).
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
           OPEN INPUT SF-IN OUTPUT SF-OUT.
           IF FS-IN NOT = "00" OR FS-OUT NOT = "00"
               DISPLAY "OPEN-FAIL " FS-IN " " FS-OUT
               STOP RUN RETURNING 1
           END-IF
           PERFORM UNTIL EXIT
               READ SF-IN
                   AT END EXIT PERFORM
               END-READ
               IF SF-CODE = "OK"
                   ADD SF-AMOUNT TO VALID-SUM
                   ADD 1 TO VALID-N
                   ADD SF-AMOUNT TO BALANCE
               ELSE
                   ADD SF-AMOUNT TO INVALID-SUM
                   ADD 1 TO INVALID-N
               END-IF
               MOVE SF-AMOUNT TO AMT-S
               MOVE BALANCE TO BAL-E
               MOVE SPACES TO OUT-BUF
               STRING SF-KEY " " AMT-S " " BAL-E DELIMITED BY SIZE
                   INTO OUT-BUF END-STRING
               MOVE OUT-BUF TO OUT-REC
               WRITE OUT-REC
           END-PERFORM
           MOVE VALID-N TO N-E
           MOVE VALID-SUM TO BAL-E
           MOVE SPACES TO OUT-BUF
           STRING "VALID " N-E " " BAL-E DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           MOVE INVALID-N TO N-E
           MOVE INVALID-SUM TO BAL-E
           MOVE SPACES TO OUT-BUF
           STRING "INVALID " N-E " " BAL-E DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           CLOSE SF-IN SF-OUT.
           STOP RUN.
