*> Two FDs each with multiple records: WRITE must resolve each record to its CORRECT owning file.
*> (Record names must be distinct -- GnuCOBOL rejects a duplicate record name across files as ambiguous
*> "needs qualification"; structurally IDENTICAL records under different FDs prove no cross-association.)
*> @format: free
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P190.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT FA ASSIGN TO "p190a.dat" ORGANIZATION SEQUENTIAL.
           SELECT FB ASSIGN TO "p190b.dat" ORGANIZATION SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD FA.
       01 REC-A PIC X(5).
       01 SAME-SHAPE-A PIC X(5).
       FD FB.
       01 REC-B PIC X(5).
       01 SAME-SHAPE-B PIC X(5).
       WORKING-STORAGE SECTION.
       01 EOF-FLAG PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT FA FB.
           MOVE "AAAAA" TO SAME-SHAPE-A.
           WRITE SAME-SHAPE-A.
           MOVE "BBBBB" TO SAME-SHAPE-B.
           WRITE SAME-SHAPE-B.
           CLOSE FA FB.
           OPEN INPUT FB.
           PERFORM UNTIL EOF-FLAG = "Y"
              READ FB AT END MOVE "Y" TO EOF-FLAG
                   NOT AT END DISPLAY "[" REC-B "]" END-READ
           END-PERFORM.
           CLOSE FB.
           STOP RUN.
