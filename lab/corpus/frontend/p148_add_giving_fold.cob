      *> ADD a b [c...] GIVING r (no TO) -- the multi-operand sum-into-GIVING fold. The sum is computed at
      *> full width before the store (was folded into the first operand's narrow width: 60+60 -> "20", and
      *> ON SIZE ERROR was then judged on the truncated value). Identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P148.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 R4 PIC 9(4) VALUE 0.
       01 R2 PIC 99   VALUE 0.
       PROCEDURE DIVISION.
           ADD 60 60 GIVING R4. DISPLAY "A=[" R4 "]".
           ADD 10 20 30 GIVING R4. DISPLAY "B=[" R4 "]".
           ADD 50 50 GIVING R2
               ON SIZE ERROR DISPLAY "SE"
           END-ADD.
           DISPLAY "C=[" R2 "]".
           STOP RUN.
