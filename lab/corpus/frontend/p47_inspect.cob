      *> INSPECT TALLYING / REPLACING / CONVERTING byte effects (GNURUST.INSPECT.1 surface).
      *> Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P47.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 TXT   PIC X(11) VALUE "MISSISSIPPI".
       01 CNT   PIC 9(3) VALUE 0.
       01 LEAD  PIC X(6) VALUE "000123".
       01 LCNT  PIC 9(3) VALUE 0.
       01 CONV  PIC X(5) VALUE "abcde".
       PROCEDURE DIVISION.
           INSPECT TXT TALLYING CNT FOR ALL "S".
           DISPLAY "S-COUNT=" CNT.
           INSPECT LEAD TALLYING LCNT FOR LEADING "0".
           DISPLAY "LEAD0=" LCNT.
           INSPECT TXT REPLACING ALL "I" BY "*".
           DISPLAY "REPL=" TXT.
           INSPECT CONV CONVERTING "abc" TO "XYZ".
           DISPLAY "CONV=" CONV.
           STOP RUN.
