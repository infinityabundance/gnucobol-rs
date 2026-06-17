      *> SORT USING/GIVING with a whole-record key: write unsorted records, SORT ascending into an
      *> output file, then read it back in order. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P55.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT SORTWK ASSIGN TO "p55s.dat".
           SELECT INF  ASSIGN TO "p55i.dat" ORGANIZATION IS LINE SEQUENTIAL.
           SELECT OUTF ASSIGN TO "p55o.dat" ORGANIZATION IS LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       SD SORTWK.
       01 SREC PIC X(4).
       FD INF.
       01 IREC PIC X(4).
       FD OUTF.
       01 OREC PIC X(4).
       WORKING-STORAGE SECTION.
       01 EOFF PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT INF.
           MOVE "MIKE" TO IREC. WRITE IREC.
           MOVE "ANNA" TO IREC. WRITE IREC.
           MOVE "ZACK" TO IREC. WRITE IREC.
           MOVE "BETH" TO IREC. WRITE IREC.
           CLOSE INF.
           SORT SORTWK ON ASCENDING KEY SREC USING INF GIVING OUTF.
           OPEN INPUT OUTF.
           PERFORM UNTIL EOFF = "Y"
               READ OUTF AT END MOVE "Y" TO EOFF END-READ
               IF EOFF = "N" DISPLAY "[" OREC "]" END-IF
           END-PERFORM.
           CLOSE OUTF.
           STOP RUN.
