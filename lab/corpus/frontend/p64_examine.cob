      *> @std: ibm
      *> EXAMINE (the COBOL-68 precursor of INSPECT, an OS/VS dialect verb): TALLYING sets TALLY,
      *> TALLYING ... REPLACING BY counts + replaces, REPLACING LEADING. Identical stdout under cobc/cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P64.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC X(7) VALUE "AABAABA".
       01 B PIC X(7) VALUE "AABAABA".
       01 C PIC X(7) VALUE "000123X".
       PROCEDURE DIVISION.
           EXAMINE A TALLYING ALL "A".
           DISPLAY "TAL-ALL=" TALLY " A=[" A "]".
           EXAMINE B TALLYING ALL "A" REPLACING BY "Z".
           DISPLAY "TAL-REP=" TALLY " B=[" B "]".
           EXAMINE C REPLACING LEADING "0" BY "*".
           DISPLAY "C=[" C "]".
           STOP RUN.
