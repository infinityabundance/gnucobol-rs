      *> @env: COB_CURRENT_DATE=20260618123456.00+0000
      *> CURRENT-DATE + the year-window conversions honour the pinned COB_CURRENT_DATE (an explicit
      *> offset makes the timezone deterministic); byte-identical to cobc under the same pin.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P72.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 CDV PIC X(21).
       PROCEDURE DIVISION.
           MOVE FUNCTION CURRENT-DATE TO CDV.
           DISPLAY "CD=[" CDV "]".
           DISPLAY "FCD=[" FUNCTION FORMATTED-CURRENT-DATE("YYYY-MM-DDThh:mm:ss") "]".
           DISPLAY "Y2Y=[" FUNCTION YEAR-TO-YYYY(40) "]".
           DISPLAY "Y2Yb=[" FUNCTION YEAR-TO-YYYY(40 20) "]".
           DISPLAY "D2YMD=[" FUNCTION DATE-TO-YYYYMMDD(400618) "]".
           DISPLAY "D2YDD=[" FUNCTION DAY-TO-YYYYDDD(40169) "]".
           STOP RUN.
