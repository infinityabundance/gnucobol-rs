      *> USE in DECLARATIVES: OPEN INPUT a non-existent file -> status 35 -> the USE AFTER ERROR handler
      *> for that file runs. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P66.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN TO "nope66.dat" ORGANIZATION LINE SEQUENTIAL
               FILE STATUS IS ST.
       DATA DIVISION.
       FILE SECTION.
       FD F.
       01 FR PIC X(4).
       WORKING-STORAGE SECTION.
       01 ST PIC XX.
       PROCEDURE DIVISION.
       DECLARATIVES.
       H SECTION.
           USE AFTER STANDARD ERROR PROCEDURE ON F.
       HP.
           DISPLAY "USE-FIRED ST=" ST.
       END DECLARATIVES.
       MAIN-SECT SECTION.
       M.
           OPEN INPUT F.
           DISPLAY "AFTER-OPEN ST=" ST.
           STOP RUN.
