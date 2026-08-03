*> GROUP records beneath one FD: each WRITE emits the NAMED group's layout, and the record area
*> is shared (the stale tail of a shorter read stays visible, as libcob leaves it) (GNURUST.FILEIO.MULTI-RECORD-FD.1).
*> @format: free
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P189.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN TO "p189.dat" ORGANIZATION SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD F.
       01 DETAIL-REC.
          05 ITEM-CODE PIC 9(3).
          05 ITEM-TEXT PIC X(7).
       01 TOTAL-REC.
          05 ITEM-LABEL PIC X(6).
          05 ITEM-AMOUNT PIC 9(5).
       WORKING-STORAGE SECTION.
       01 EOF-FLAG PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT F.
           MOVE "123ABCDEFG" TO DETAIL-REC.
           WRITE DETAIL-REC.
           MOVE "TOTAL00042" TO TOTAL-REC.
           WRITE TOTAL-REC.
           CLOSE F.
           OPEN INPUT F.
           PERFORM UNTIL EOF-FLAG = "Y"
              READ F AT END MOVE "Y" TO EOF-FLAG
                   NOT AT END DISPLAY "[" DETAIL-REC "]" END-READ
           END-PERFORM.
           CLOSE F.
           STOP RUN.
