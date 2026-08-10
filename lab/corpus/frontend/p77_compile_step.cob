      *> @env: SOURCE_DATE_EPOCH=1000000000
      *> @no312: SOURCE_DATE_EPOCH handling for WHEN-COMPILED/MODULE-DATE evolved 3.1.2 -> 3.2; port targets 3.2
      *> @no32: stable GnuCOBOL 3.2 oracle has the SOURCE_DATE_EPOCH MODULE-DATE off-by-one (upstream 946f3e638 fixed
      *>   it; the port deliberately targets the current-upstream civil-calendar conversion -- the drift is also
      *>   recorded in compile_tm's comment)
      *> The compile-stamp intrinsics, deterministic under a pinned SOURCE_DATE_EPOCH (the reproducible-
      *> builds standard cobc honours). The interpreter's compile step derives the same date/time exactly
      *> as libcob cob_set_date_from_epoch -- byte-identical to cobc.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P77.
       PROCEDURE DIVISION.
           DISPLAY "WC=[" FUNCTION WHEN-COMPILED "]".
           DISPLAY "MD=[" FUNCTION MODULE-DATE "]".
           DISPLAY "MT=[" FUNCTION MODULE-TIME "]".
           DISPLAY "MFD=[" FUNCTION MODULE-FORMATTED-DATE "]".
           STOP RUN.
