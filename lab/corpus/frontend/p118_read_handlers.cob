      *> READ with AT END + NOT AT END handler clauses (the success/EOF branches in one statement).
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P118.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN "p118.dat" ORGANIZATION LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.
       FD F.
       01 FR PIC X(3).
       WORKING-STORAGE SECTION.
       01 E PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT F.
           MOVE "aaa" TO FR. WRITE FR.
           MOVE "bbb" TO FR. WRITE FR.
           CLOSE F.
           OPEN INPUT F.
           PERFORM UNTIL E = "Y"
              READ F
                 AT END MOVE "Y" TO E
                 NOT AT END DISPLAY "got " FR
              END-READ
           END-PERFORM.
           CLOSE F.
           STOP RUN.
