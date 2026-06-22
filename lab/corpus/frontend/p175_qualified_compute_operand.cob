      *> A qualified name `X OF Y` as a COMPUTE target and as a parenthesised operand `(X OF Z * .08)` -- the
      *> qualified-name collapser strips the lexer-glued leading `(` so the inner name resolves. Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P175.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 SALES-RECORD.
          05 SALES-AMOUNT PIC 9(4)V99 VALUE 200.00.
       01 DETAIL-LINE.
          05 SALES-AMOUNT PIC 9(4)V99 VALUE ZERO.
       PROCEDURE DIVISION.
           COMPUTE SALES-AMOUNT OF DETAIL-LINE =
                  (SALES-AMOUNT OF SALES-RECORD * .08).
           DISPLAY "AMT=" SALES-AMOUNT OF SALES-RECORD.
           DISPLAY "TAX=" SALES-AMOUNT OF DETAIL-LINE.
           STOP RUN.
