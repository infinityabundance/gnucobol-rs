      *> A group with MULTIPLE FILLERs of DIFFERENT sizes receiving a group MOVE: each FILLER must occupy its
      *> own slot so the named children land at the right offsets (a date `2021 09 15` splits correctly, and
      *> all three numeric parts test NUMERIC). Regression for FILLER key collision. Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P176.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-IN PIC X(19) VALUE '2021 09 15'.
       01 WS-DATE.
          05 YYYY PIC 9(04) VALUE ZERO.
          05 FILLER PIC X(01) VALUE SPACES.
          05 MM   PIC 9(02) VALUE ZERO.
          05 FILLER PIC X(01) VALUE SPACES.
          05 DD   PIC 9(02) VALUE ZERO.
          05 FILLER PIC X(09) VALUE SPACES.
       PROCEDURE DIVISION.
           MOVE WS-IN TO WS-DATE.
           DISPLAY "Y=" YYYY " M=" MM " D=" DD.
           IF YYYY NUMERIC AND MM NUMERIC AND DD NUMERIC
              DISPLAY "VALID" ELSE DISPLAY "INVALID" END-IF.
           STOP RUN.
