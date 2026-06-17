      *> RELATIVE file: random WRITE by RELATIVE KEY, DELETE a record, START to reposition, then a
      *> sequential READ that skips the deleted slot. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P54.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT RELFILE ASSIGN TO "p54.dat"
               ORGANIZATION IS RELATIVE
               ACCESS MODE IS DYNAMIC
               RELATIVE KEY IS RK
               FILE STATUS IS ST.
       DATA DIVISION.
       FILE SECTION.
       FD RELFILE.
       01 RREC PIC X(3).
       WORKING-STORAGE SECTION.
       01 RK   PIC 9(2).
       01 ST   PIC XX.
       01 EOFF PIC X VALUE "N".
       PROCEDURE DIVISION.
           OPEN OUTPUT RELFILE.
           MOVE 1 TO RK. MOVE "AAA" TO RREC. WRITE RREC.
           MOVE 2 TO RK. MOVE "BBB" TO RREC. WRITE RREC.
           MOVE 3 TO RK. MOVE "CCC" TO RREC. WRITE RREC.
           MOVE 4 TO RK. MOVE "DDD" TO RREC. WRITE RREC.
           CLOSE RELFILE.
           OPEN I-O RELFILE.
           MOVE 3 TO RK.
           DELETE RELFILE.
           MOVE 2 TO RK.
           START RELFILE KEY >= RK.
           DISPLAY "START-ST=" ST.
           PERFORM UNTIL EOFF = "Y"
               READ RELFILE NEXT AT END MOVE "Y" TO EOFF END-READ
               IF EOFF = "N" DISPLAY RK " " RREC END-IF
           END-PERFORM.
           CLOSE RELFILE.
           STOP RUN.
