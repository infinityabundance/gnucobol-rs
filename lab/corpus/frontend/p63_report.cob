      *> Report writer: RD with a DETAIL group (COLUMN + SOURCE), GENERATE writes column-placed lines to
      *> the report file; a second SELECT on the same physical path reads them back. Identical cobc/cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P63.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT RPT ASSIGN TO "p63.out" ORGANIZATION LINE SEQUENTIAL.
           SELECT INF ASSIGN TO "p63.out" ORGANIZATION LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD RPT REPORT IS R1.
       FD INF.
       01 INREC PIC X(20).
       WORKING-STORAGE SECTION.
       01 ITM  PIC X(6) VALUE "WIDGET".
       01 QTY  PIC 9(3) VALUE 5.
       01 EOFF PIC X    VALUE "N".
       REPORT SECTION.
       RD R1.
       01 DETAIL-LINE TYPE DETAIL.
          05 LINE PLUS 1.
             10 COLUMN 1  PIC X(6) SOURCE ITM.
             10 COLUMN 10 PIC 9(3) SOURCE QTY.
       PROCEDURE DIVISION.
           OPEN OUTPUT RPT.
           INITIATE R1.
           GENERATE DETAIL-LINE.
           MOVE 42 TO QTY.
           GENERATE DETAIL-LINE.
           TERMINATE R1.
           CLOSE RPT.
           OPEN INPUT INF.
           PERFORM UNTIL EOFF = "Y"
               READ INF AT END MOVE "Y" TO EOFF END-READ
               IF EOFF = "N" DISPLAY "[" INREC "]" END-IF
           END-PERFORM.
           CLOSE INF.
           STOP RUN.
