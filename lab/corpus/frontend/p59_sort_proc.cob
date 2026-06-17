      *> SORT with INPUT PROCEDURE (RELEASE records) + OUTPUT PROCEDURE (RETURN sorted records), keyed on
      *> a sub-field. Exercises RELEASE, RETURN, out-of-line procedures. Identical stdout under cobc/cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P59.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT SORTWK ASSIGN TO "p59.dat".
       DATA DIVISION.
       FILE SECTION.
       SD SORTWK.
       01 SREC.
          05 S-KEY PIC 9(2).
          05 S-VAL PIC X(3).
       WORKING-STORAGE SECTION.
       01 EOFF PIC X VALUE "N".
       PROCEDURE DIVISION.
       MAIN-PARA.
           SORT SORTWK ON ASCENDING KEY S-KEY
               INPUT PROCEDURE IS FILL-IT
               OUTPUT PROCEDURE IS DRAIN-IT.
           STOP RUN.
       FILL-IT.
           MOVE 30 TO S-KEY. MOVE "CCC" TO S-VAL. RELEASE SREC.
           MOVE 10 TO S-KEY. MOVE "AAA" TO S-VAL. RELEASE SREC.
           MOVE 20 TO S-KEY. MOVE "BBB" TO S-VAL. RELEASE SREC.
       DRAIN-IT.
           PERFORM UNTIL EOFF = "Y"
               RETURN SORTWK AT END MOVE "Y" TO EOFF END-RETURN
               IF EOFF = "N" DISPLAY S-KEY " " S-VAL END-IF
           END-PERFORM.
