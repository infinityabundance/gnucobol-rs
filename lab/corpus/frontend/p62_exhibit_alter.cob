      *> EXHIBIT NAMED (OS/VS debug display) and ALTER (retarget an alterable paragraph's GO TO).
      *> Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P62.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N    PIC 9(3) VALUE 42.
       01 S    PIC X(4) VALUE "HI".
       01 FLAG PIC 9    VALUE 1.
       PROCEDURE DIVISION.
       MAIN-PARA.
           EXHIBIT NAMED N.
           EXHIBIT NAMED N S.
       SW.
           GO TO FIRST-P.
       FIRST-P.
           DISPLAY "FIRST".
           ALTER SW TO PROCEED TO SECOND-P.
           IF FLAG = 1
               MOVE 2 TO FLAG
               GO TO SW
           END-IF.
           GO TO DONE-P.
       SECOND-P.
           DISPLAY "SECOND".
           GO TO DONE-P.
       DONE-P.
           STOP RUN.
