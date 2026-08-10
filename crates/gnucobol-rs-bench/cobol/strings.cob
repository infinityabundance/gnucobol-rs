      *> String processing: STRING, UNSTRING, INSPECT, reference modification.
      *> Input: "ABCDEFGH,1234,042" per line. UNSTRINGs into three fields,
      *> INSPECTs for digits, builds an output line with STRING + refmod.
      *> Output: "<a>|<b>|<c>|<digits-in-a>|<a(1:3)>"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. STRINGS.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT ST-IN ASSIGN TO "strings.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT ST-OUT ASSIGN TO "strings.out"
               ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD  ST-IN.
       01  ST-REC            PIC X(20).
       FD  ST-OUT.
       01  OUT-REC           PIC X(80).
       WORKING-STORAGE SECTION.
       01  A                 PIC X(8).
       01  B                 PIC X(4).
       01  C                 PIC X(3).
       01  DIGIT-COUNT       PIC 9(4).
       01  DIGIT-E           PIC 9(4).
       01  HEAD3             PIC X(3).
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
           OPEN INPUT ST-IN OUTPUT ST-OUT.
           PERFORM UNTIL EXIT
               READ ST-IN
                   AT END EXIT PERFORM
               END-READ
               MOVE SPACES TO A B C
               UNSTRING ST-REC DELIMITED BY ","
                   INTO A B C
               END-UNSTRING
               MOVE ZERO TO DIGIT-COUNT
               INSPECT A TALLYING DIGIT-COUNT FOR ALL "0" "1" "2"
                   "3" "4" "5" "6" "7" "8" "9"
               MOVE A(1:3) TO HEAD3
               MOVE DIGIT-COUNT TO DIGIT-E
               MOVE SPACES TO OUT-BUF
               STRING A "|" B "|" C "|" DIGIT-E "|" HEAD3
                   DELIMITED BY SIZE INTO OUT-BUF END-STRING
               MOVE OUT-BUF TO OUT-REC
               WRITE OUT-REC
           END-PERFORM
           CLOSE ST-IN ST-OUT.
           STOP RUN.
