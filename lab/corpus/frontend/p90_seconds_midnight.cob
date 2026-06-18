      *> @clock: live-clock test -- retry-tolerant on a second-boundary straddle
      *> @no312: SECONDS-PAST-MIDNIGHT is a live-clock read; the 3.1.2 cross-check runs at a different moment
      *> FUNCTION SECONDS-PAST-MIDNIGHT reads the live wall clock exactly as libcob (it ignores
      *> COB_CURRENT_DATE). Under TZ=UTC0 the port's UTC computation equals libcob's localtime, so cobc and
      *> cobrun -- run back-to-back in the same second -- report the identical time-of-day in seconds.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P90.
       PROCEDURE DIVISION.
           DISPLAY "SPM=[" FUNCTION SECONDS-PAST-MIDNIGHT "]".
           STOP RUN.
