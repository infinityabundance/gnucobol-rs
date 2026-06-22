      *> A multi-line arithmetic expression whose continuation line BEGINS with the division operator `/`
      *> (indented well past the indicator area). cobc treats a deeply-indented line-leading `/` as DIVISION,
      *> not a fixed-format page-eject comment -- so the divide must NOT be dropped. Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P173.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC S9(14)V99 VALUE 12024546.51.
       01 B PIC S9(14)V99 VALUE 11674330.16.
       01 R PIC S9(3)V99 VALUE ZERO COMP-3.
       PROCEDURE DIVISION.
           COMPUTE R ROUNDED = (A * 100)
                             / (B - 1.0)
           END-COMPUTE.
           DISPLAY "R=" R.
           STOP RUN.
