      *> Out-of-line PERFORM: PERFORM para, PERFORM para N TIMES, PERFORM para UNTIL cond. The named
      *> paragraph range runs and returns. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P58.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 N   PIC 9  VALUE 0.
       01 TOT PIC 99 VALUE 0.
       PROCEDURE DIVISION.
       MAIN-PARA.
           PERFORM GREET.
           PERFORM ADD-ONE 3 TIMES.
           DISPLAY "TOT=" TOT.
           PERFORM COUNT-UP UNTIL N >= 3.
           STOP RUN.
       GREET.
           DISPLAY "HELLO".
       ADD-ONE.
           ADD 1 TO TOT.
       COUNT-UP.
           ADD 1 TO N.
           DISPLAY "N=" N.
