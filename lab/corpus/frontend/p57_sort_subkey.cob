      *> SORT on a SUB-FIELD key of a group record (now that group items exist): records are ordered by
      *> the AGE field, not the whole record. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P57.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT SORTWK ASSIGN TO "p57s.dat".
           SELECT INF  ASSIGN TO "p57i.dat" ORGANIZATION IS LINE SEQUENTIAL.
           SELECT OUTF ASSIGN TO "p57o.dat" ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       SD SORTWK.
       01 SREC.
          05 S-NAME PIC X(4).
          05 S-AGE  PIC 9(2).
       FD INF.
       01 IREC PIC X(6).
       FD OUTF.
       01 OREC PIC X(6).
       WORKING-STORAGE SECTION.
       01 EOFF PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT INF.
           MOVE "MIKE45" TO IREC. WRITE IREC.
           MOVE "ANNA20" TO IREC. WRITE IREC.
           MOVE "ZACK30" TO IREC. WRITE IREC.
           MOVE "BETH10" TO IREC. WRITE IREC.
           CLOSE INF.
           SORT SORTWK ON ASCENDING KEY S-AGE USING INF GIVING OUTF.
           OPEN INPUT OUTF.
           PERFORM UNTIL EOFF = "Y"
               READ OUTF AT END MOVE "Y" TO EOFF END-READ
               IF EOFF = "N" DISPLAY "[" OREC "]" END-IF
           END-PERFORM.
           CLOSE OUTF.
           STOP RUN.
