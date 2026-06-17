      *> A CALLed contained program's WORKING-STORAGE is STATIC -- it persists across CALLs (C=1,2),
      *> and CANCEL un-initializes it so the next CALL rebuilds from VALUE (C=1,2 again). An INITIAL
      *> program (ISUB) re-initializes every entry (I=1,1). Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P38MAIN.
       PROCEDURE DIVISION.
           CALL "P38SUB".
           CALL "P38SUB".
           CANCEL "P38SUB".
           CALL "P38SUB".
           CALL "P38INIT".
           CALL "P38INIT".
           STOP RUN.
       END PROGRAM P38MAIN.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P38SUB.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 C PIC 9 VALUE 0.
       PROCEDURE DIVISION.
           ADD 1 TO C.
           DISPLAY "C=" C.
       END PROGRAM P38SUB.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P38INIT IS INITIAL.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N PIC 9 VALUE 0.
       PROCEDURE DIVISION.
           ADD 1 TO N.
           DISPLAY "N=" N.
       END PROGRAM P38INIT.
