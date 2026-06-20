       IDENTIFICATION DIVISION.
       PROGRAM-ID. P122.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N   PIC 999 VALUE 17.
       01 D   PIC 999 VALUE 5.
       01 Q   PIC 999 VALUE 111.
       01 R   PIC 999 VALUE 222.
       01 QS  PIC 9   VALUE 7.
       01 RS  PIC 9   VALUE 8.
       PROCEDURE DIVISION.
      *> normal: NOT ON SIZE ERROR fires, Q=003 R=002
           DIVIDE N BY D GIVING Q REMAINDER R
              ON SIZE ERROR DISPLAY "SE1"
              NOT ON SIZE ERROR DISPLAY "OK1"
           END-DIVIDE.
           DISPLAY "Q=" Q " R=" R.
      *> divide by zero: ON SIZE ERROR fires, both receivers unchanged
           MOVE 0 TO D.
           DIVIDE N BY D GIVING Q REMAINDER R
              ON SIZE ERROR DISPLAY "SE2"
              NOT ON SIZE ERROR DISPLAY "OK2"
           END-DIVIDE.
           DISPLAY "Q=" Q " R=" R.
      *> quotient overflow: ON SIZE ERROR fires, QS unchanged (7), RS stored (0)
           DIVIDE 999 BY 1 GIVING QS REMAINDER RS
              ON SIZE ERROR DISPLAY "SE3"
              NOT ON SIZE ERROR DISPLAY "OK3"
           END-DIVIDE.
           DISPLAY "QS=" QS " RS=" RS.
      *> overflow with NO handler: truncate-store and continue (QS=9 RS=0)
           DIVIDE 888 BY 1 GIVING QS REMAINDER RS.
           DISPLAY "QS=" QS " RS=" RS.
           STOP RUN.
