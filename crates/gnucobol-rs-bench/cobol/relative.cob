      *> Relative-file workload: insert, lookup, update, delete, sequential
      *> traversal -- in phases (GnuCOBOL relative files behave deterministically
      *> across OPEN/CLOSE boundaries). Input: "0000000001 0000000123" (key,
      *> payload). Phase 1 inserts every even key; phase 2 updates (key%4==0,
      *> key%8!=0) and deletes (key%8==0); phase 3 traverses the survivors.
      *> Output: "WRITTEN <n>" / "UPDATED <n>" / "DELETED <n>" / "TRAVERSE <n> <sum>"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. RELATIVE.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT RL-IN ASSIGN TO "relative.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT RL-FILE ASSIGN TO "relative.idx"
               ORGANIZATION IS RELATIVE
               ACCESS MODE IS DYNAMIC
               RELATIVE KEY IS RL-KEY
               FILE STATUS IS FS.
           SELECT RL-OUT ASSIGN TO "relative.out"
               ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD  RL-IN.
       01  RL-REC.
           05  RL-KEY-IN     PIC 9(10).
           05  RL-PAYLOAD-IN PIC 9(10).
       FD  RL-FILE.
       01  RL-DATA          PIC X(20).
       FD  RL-OUT.
       01  OUT-REC           PIC X(80).
       WORKING-STORAGE SECTION.
       01  RL-KEY            PIC 9(10).
       01  FS                PIC XX.
       01  WRITTEN-N         PIC 9(9) VALUE 0.
       01  UPDATED-N         PIC 9(9) VALUE 0.
       01  DELETED-N         PIC 9(9) VALUE 0.
       01  TRAV-N            PIC 9(9) VALUE 0.
       01  TRAV-SUM          PIC 9(16) VALUE 0.
       01  PAYLOAD-STR       PIC X(10).
       01  PL                PIC 9(10).
       01  N-E               PIC 9(9).
       01  S-E               PIC Z(15)9.
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
      *>    phase 1: insert every even key
           OPEN INPUT RL-IN OUTPUT RL-FILE.
           PERFORM UNTIL EXIT
               READ RL-IN
                   AT END EXIT PERFORM
               END-READ
               IF FUNCTION MOD(RL-KEY-IN, 2) = 0
                   MOVE RL-KEY-IN TO RL-KEY
                   MOVE RL-PAYLOAD-IN TO PAYLOAD-STR
                   MOVE PAYLOAD-STR TO RL-DATA
                   WRITE RL-DATA
                   IF FS = "00"
                       ADD 1 TO WRITTEN-N
                   END-IF
               END-IF
           END-PERFORM
           CLOSE RL-IN RL-FILE
      *>    phase 2: update (key%4==0, key%8!=0) and delete (key%8==0)
           OPEN INPUT RL-IN I-O RL-FILE.
           PERFORM UNTIL EXIT
               READ RL-IN
                   AT END EXIT PERFORM
               END-READ
               IF FUNCTION MOD(RL-KEY-IN, 2) = 0
                   MOVE RL-KEY-IN TO RL-KEY
                   MOVE RL-PAYLOAD-IN TO PAYLOAD-STR
                   READ RL-FILE
                   IF FS = "00"
                       IF FUNCTION MOD(RL-KEY-IN, 8) = 0
                           DELETE RL-FILE
                           IF FS = "00"
                               ADD 1 TO DELETED-N
                           END-IF
                       ELSE
                           IF FUNCTION MOD(RL-KEY-IN, 4) = 0
                               MOVE PAYLOAD-STR TO RL-DATA
                               REWRITE RL-DATA
                               IF FS = "00"
                                   ADD 1 TO UPDATED-N
                               END-IF
                           END-IF
                       END-IF
                   END-IF
               END-IF
           END-PERFORM
           CLOSE RL-IN RL-FILE
      *>    phase 3: sequential traversal of the survivors
           OPEN INPUT RL-FILE OUTPUT RL-OUT.
           PERFORM UNTIL EXIT
               READ RL-FILE NEXT
                   AT END EXIT PERFORM
               END-READ
               MOVE RL-DATA(1:10) TO PL
               ADD 1 TO TRAV-N
               ADD PL TO TRAV-SUM
           END-PERFORM
           CLOSE RL-FILE
           MOVE WRITTEN-N TO N-E
           MOVE SPACES TO OUT-BUF
           STRING "WRITTEN " N-E DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           MOVE UPDATED-N TO N-E
           MOVE SPACES TO OUT-BUF
           STRING "UPDATED " N-E DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           MOVE DELETED-N TO N-E
           MOVE SPACES TO OUT-BUF
           STRING "DELETED " N-E DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           MOVE TRAV-N TO N-E
           MOVE TRAV-SUM TO S-E
           MOVE SPACES TO OUT-BUF
           STRING "TRAVERSE " N-E " " S-E DELIMITED BY SIZE
               INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           CLOSE RL-OUT.
           STOP RUN.
