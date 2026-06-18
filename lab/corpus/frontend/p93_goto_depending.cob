       IDENTIFICATION DIVISION.
       PROGRAM-ID. GD.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 K PIC 9 VALUE 0.
       PROCEDURE DIVISION.
       MAIN.
           ADD 1 TO K.
           GO TO L1 L2 L3 DEPENDING ON K.
           DISPLAY "K=" K " none".
           GO TO CHK.
       L1. DISPLAY "K=" K " L1". GO TO CHK.
       L2. DISPLAY "K=" K " L2". GO TO CHK.
       L3. DISPLAY "K=" K " L3". GO TO CHK.
       CHK.
           IF K < 5 GO TO MAIN.
           STOP RUN.
