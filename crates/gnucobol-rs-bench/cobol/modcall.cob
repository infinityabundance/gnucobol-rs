      *> Module workload: repeated dynamic CALL, EXTERNAL data, CANCEL, reload.
      *> Caller reads "0000000001" per line (n), calls CALC-MOD n times with
      *> EXTERNAL accumulator state, CANCELs on every 5th, reloads.
      *> Output: "CALL <n> <result> <external-total>"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MODCALL.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT MD-IN ASSIGN TO "modules.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT MD-OUT ASSIGN TO "modules.out"
               ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD  MD-IN.
       01  MD-REC.
           05  MD-N          PIC 9(10).
       FD  MD-OUT.
       01  OUT-REC           PIC X(80).
       WORKING-STORAGE SECTION.
       01  ARG               PIC 9(10).
       01  RES               PIC 9(10).
       01  CALL-N            PIC 9(9) VALUE 0.
       01  N-E               PIC 9(9).
       01  R-E               PIC 9(10).
       01  X-E               PIC 9(10).
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
           OPEN INPUT MD-IN OUTPUT MD-OUT.
           PERFORM UNTIL EXIT
               READ MD-IN
                   AT END EXIT PERFORM
               END-READ
               MOVE MD-N TO ARG
               CALL "CALC-MOD" USING ARG RES
               ADD 1 TO CALL-N
               IF FUNCTION MOD(CALL-N, 5) = 0
                   CANCEL "CALC-MOD"
               END-IF
               MOVE CALL-N TO N-E
               MOVE RES TO R-E
               MOVE SPACES TO OUT-BUF
               STRING "CALL " N-E " " R-E DELIMITED BY SIZE
                   INTO OUT-BUF END-STRING
               MOVE OUT-BUF TO OUT-REC
               WRITE OUT-REC
           END-PERFORM
           CLOSE MD-IN MD-OUT.
           STOP RUN.
