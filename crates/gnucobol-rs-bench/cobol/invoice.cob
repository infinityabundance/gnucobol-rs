      *> Invoice workload: item records, decimal multiplication, discounts,
      *> taxes, balances. Input: "0000 0042 00001234 05" (item, qty,
      *> unit price in cents, discount percent).
      *> Output: "0000 00000051816 00000027980 00000023836 00000003387"
      *>         (line-total-cents, discount-cents, taxable-cents, tax-cents)
      *> then "TOTAL <sum-cents> <sum-tax>"
       IDENTIFICATION DIVISION.
       PROGRAM-ID. INVOICE.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT INV-IN ASSIGN TO "invoice.dat"
               ORGANIZATION IS LINE SEQUENTIAL.
           SELECT INV-OUT ASSIGN TO "invoice.out"
               ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD  INV-IN.
       01  INV-REC.
           05  ITEM-NO       PIC 9(4).
           05  ITEM-QTY      PIC 9(4).
           05  ITEM-PRICE-C  PIC 9(8).
           05  ITEM-DISC     PIC 9(2).
       FD  INV-OUT.
       01  OUT-REC           PIC X(80).
       WORKING-STORAGE SECTION.
       01  LINE-TOTAL-C      PIC S9(14).
       01  DISC-C            PIC S9(14).
       01  TAXABLE-C         PIC S9(14).
       01  TAX-C             PIC S9(14).
       01  TAXRATE-P         PIC 9(3) VALUE 014.
       01  T-TOTAL-C         PIC S9(16) VALUE 0.
       01  T-TAX-C           PIC S9(16) VALUE 0.
       01  T-COUNT           PIC 9(9) VALUE 0.
       01  NUM-E1            PIC Z(15)9.
       01  NUM-E2            PIC Z(15)9.
       01  NUM-E3            PIC Z(15)9.
       01  NUM-E4            PIC Z(15)9.
       01  OUT-BUF           PIC X(80).
       PROCEDURE DIVISION.
           OPEN INPUT INV-IN OUTPUT INV-OUT.
           PERFORM UNTIL EXIT
               READ INV-IN
                   AT END EXIT PERFORM
               END-READ
      *>       line total = qty * price (cents)
               COMPUTE LINE-TOTAL-C = ITEM-QTY * ITEM-PRICE-C
      *>       discount = line-total * disc% / 100 (rounded)
               COMPUTE DISC-C ROUNDED = LINE-TOTAL-C * ITEM-DISC / 100
               COMPUTE TAXABLE-C = LINE-TOTAL-C - DISC-C
               COMPUTE TAX-C ROUNDED = TAXABLE-C * TAXRATE-P / 100
               ADD LINE-TOTAL-C TO T-TOTAL-C
               ADD TAX-C TO T-TAX-C
               ADD 1 TO T-COUNT
               MOVE LINE-TOTAL-C TO NUM-E1
               MOVE DISC-C TO NUM-E2
               MOVE TAXABLE-C TO NUM-E3
               MOVE TAX-C TO NUM-E4
               MOVE SPACES TO OUT-BUF
               STRING ITEM-NO " " NUM-E1 " " NUM-E2 " " NUM-E3
                   " " NUM-E4 DELIMITED BY SIZE
                   INTO OUT-BUF END-STRING
               MOVE OUT-BUF TO OUT-REC
               WRITE OUT-REC
           END-PERFORM
           MOVE T-TOTAL-C TO NUM-E1
           MOVE T-TAX-C TO NUM-E2
           MOVE SPACES TO OUT-BUF
           STRING "TOTAL " T-COUNT " " NUM-E1 " " NUM-E2
               DELIMITED BY SIZE INTO OUT-BUF END-STRING
           MOVE OUT-BUF TO OUT-REC
           WRITE OUT-REC
           CLOSE INV-IN INV-OUT.
           STOP RUN.
