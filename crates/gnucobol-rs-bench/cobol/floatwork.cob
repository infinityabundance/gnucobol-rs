      *> Floating-point workload: COMP-1/COMP-2 arithmetic with deterministic
      *> result checks. Input: "57303 57136" integers per line (5+5 chars).
      *> Computes sum, product (COMP-1/COMP-2), and a SIZE ERROR check.
      *> Output: "<sum> <product> <size-error-flag>"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. FLOATWORK.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT FL-IN ASSIGN TO "float.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT FL-OUT ASSIGN TO "float.out"
               ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD  FL-IN.
       01  FL-REC.
           05  FL-A          PIC 9(5).
           05  FL-B          PIC 9(5).
       FD  FL-OUT.
       01  OUT-REC           PIC X(80).
       WORKING-STORAGE SECTION.
       01  S1                COMP-1.
       01  S2                COMP-2.
       01  P2                COMP-2.
       01  SIZE-FLAG         PIC X VALUE "N".
       01  S-E               PIC Z(5)9.99.
       01  P-E               PIC Z(9)9.99.
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
           OPEN INPUT FL-IN OUTPUT FL-OUT.
           PERFORM UNTIL EXIT
               READ FL-IN
                   AT END EXIT PERFORM
               END-READ
               MOVE "N" TO SIZE-FLAG
               COMPUTE S1 = FL-A + FL-B
               COMPUTE S2 = FL-A + FL-B
               COMPUTE P2 = FL-A * FL-B
                   ON SIZE ERROR MOVE "Y" TO SIZE-FLAG
               END-COMPUTE
               MOVE S2 TO S-E
               MOVE P2 TO P-E
               MOVE SPACES TO OUT-BUF
               STRING S-E " " P-E " " SIZE-FLAG DELIMITED BY SIZE
                   INTO OUT-BUF END-STRING
               MOVE OUT-BUF TO OUT-REC
               WRITE OUT-REC
           END-PERFORM
           CLOSE FL-IN FL-OUT.
           STOP RUN.
