      *> Abbreviated combined conditions: a term after AND/OR may omit the subject (reuse last) and the
      *> operator (reuse last too); a leading NOT negates a term. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P152.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9 VALUE 5.
       01 B PIC 9 VALUE 3.
       01 C PIC 9 VALUE 7.
       PROCEDURE DIVISION.
           IF A = 1 OR 2 OR 5 DISPLAY "T1 Y" ELSE DISPLAY "T1 N" END-IF.
           IF A > B AND < C DISPLAY "T2 Y" ELSE DISPLAY "T2 N" END-IF.
           IF A = B OR C DISPLAY "T3 Y" ELSE DISPLAY "T3 N" END-IF.
           IF NOT A = 5 DISPLAY "T4 Y" ELSE DISPLAY "T4 N" END-IF.
           IF A NOT = 1 AND 2 DISPLAY "T5 Y" ELSE DISPLAY "T5 N" END-IF.
           IF A >= 3 AND <= 7 DISPLAY "T6 Y" ELSE DISPLAY "T6 N" END-IF.
           IF A GREATER THAN 3 AND LESS THAN 8 DISPLAY "T7 Y" ELSE DISPLAY "T7 N" END-IF.
           STOP RUN.
