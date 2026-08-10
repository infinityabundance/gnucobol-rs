      *> Payroll workload: COMP-3 rates and totals, tax, rounding, report.
      *> Input:  payroll.dat "E0000 005072 004067 N" (id, hours in hundredths,
      *>         rate in cents, status) -- integer-exact, no embedded points.
      *> Output: payroll.out "E0000      2062.78       453.81      1608.97"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. PAYROLL.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT PAY-IN ASSIGN TO "payroll.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT PAY-OUT ASSIGN TO "payroll.out"
               ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD  PAY-IN.
       01  PAY-REC.
           05  EMP-ID       PIC X(5).
           05  EMP-HOURS-H  PIC 9(6).
           05  EMP-RATE-C   PIC 9(6).
           05  EMP-STATUS   PIC X.
               88 EXEMPT    VALUE "E".
       FD  PAY-OUT.
       01  OUT-REC          PIC X(80).
       WORKING-STORAGE SECTION.
       01  TOTALS.
           05  T-GROSS      PIC S9(12) COMP-3 VALUE 0.
           05  T-TAX        PIC S9(12) COMP-3 VALUE 0.
           05  T-NET        PIC S9(12) COMP-3 VALUE 0.
           05  T-COUNT      PIC 9(9) COMP-3 VALUE 0.
       01  GROSS-C           PIC S9(10).
       01  TAX-C             PIC S9(10).
       01  NET-C             PIC S9(10).
       01  GROSS             PIC S9(12)V99.
       01  TAX               PIC S9(12)V99.
       01  NET               PIC S9(12)V99.
       01  TAXRATE-P         PIC 9(3) VALUE 022.
       01  TAXEXEMPT-P       PIC 9(3) VALUE 010.
       01  GROSS-E           PIC Z(8)9.99.
       01  TAX-E             PIC Z(8)9.99.
       01  NET-E             PIC Z(8)9.99.
       01  COUNT-E           PIC 9(9).
       01  TOT-E             PIC Z(11)9.99.
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
           OPEN INPUT PAY-IN OUTPUT PAY-OUT.
           PERFORM UNTIL EXIT
               READ PAY-IN
                   AT END EXIT PERFORM
               END-READ
      *>       gross cents = round(hundredths * cents / 100)
               COMPUTE GROSS-C ROUNDED =
                   EMP-HOURS-H * EMP-RATE-C / 100
               IF EXEMPT
                   COMPUTE TAX-C ROUNDED = GROSS-C * TAXEXEMPT-P / 100
               ELSE
                   COMPUTE TAX-C ROUNDED = GROSS-C * TAXRATE-P / 100
               END-IF
               COMPUTE NET-C = GROSS-C - TAX-C
               COMPUTE GROSS = GROSS-C / 100
               COMPUTE TAX = TAX-C / 100
               COMPUTE NET = NET-C / 100
               ADD GROSS-C TO T-GROSS
               ADD TAX-C TO T-TAX
               ADD NET-C TO T-NET
               ADD 1 TO T-COUNT
               MOVE GROSS TO GROSS-E
               MOVE TAX TO TAX-E
               MOVE NET TO NET-E
               MOVE SPACES TO OUT-BUF
               STRING EMP-ID " " GROSS-E " " TAX-E " " NET-E
                   DELIMITED BY SIZE INTO OUT-BUF
               END-STRING
               MOVE OUT-BUF TO OUT-REC
               WRITE OUT-REC
           END-PERFORM
           MOVE T-COUNT TO COUNT-E
           COMPUTE GROSS = T-GROSS / 100
           MOVE GROSS TO TOT-E
           MOVE SPACES TO OUT-BUF
           STRING "TOTALS " COUNT-E " " TOT-E DELIMITED BY SIZE
               INTO OUT-BUF
           END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           COMPUTE TAX = T-TAX / 100
           MOVE TAX TO TOT-E
           MOVE SPACES TO OUT-BUF
           STRING "TAX " TOT-E DELIMITED BY SIZE INTO OUT-BUF
           END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           COMPUTE NET = T-NET / 100
           MOVE NET TO TOT-E
           MOVE SPACES TO OUT-BUF
           STRING "NET " TOT-E DELIMITED BY SIZE INTO OUT-BUF
           END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           CLOSE PAY-IN PAY-OUT.
           STOP RUN.
