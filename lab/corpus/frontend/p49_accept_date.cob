      *> @env: COB_CURRENT_DATE=20260617123456.000000000
      *> ACCEPT FROM the system date/time registers, made deterministic by a pinned COB_CURRENT_DATE
      *> (the same override libcob honors). Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P49.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 D6   PIC 9(6).
       01 D8   PIC 9(8).
       01 DY5  PIC 9(5).
       01 T8   PIC 9(8).
       01 DOW  PIC 9.
       PROCEDURE DIVISION.
           ACCEPT D6  FROM DATE.
           DISPLAY "DATE=" D6.
           ACCEPT D8  FROM DATE YYYYMMDD.
           DISPLAY "DATE8=" D8.
           ACCEPT DY5 FROM DAY.
           DISPLAY "DAY=" DY5.
           ACCEPT T8  FROM TIME.
           DISPLAY "TIME=" T8.
           ACCEPT DOW FROM DAY-OF-WEEK.
           DISPLAY "DOW=" DOW.
           STOP RUN.
