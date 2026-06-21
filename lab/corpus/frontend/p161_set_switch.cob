      *> SET <switch-mnemonic> TO ON|OFF -- toggle a SPECIAL-NAMES UPSI switch at runtime; the ON/OFF STATUS
      *> condition-names read the live state. Multi-target SET too. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P161.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
           SWITCH-1 IS SW1 ON STATUS IS S1ON OFF STATUS IS S1OFF
           SWITCH-2 IS SW2 ON STATUS IS S2ON.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 X PIC 9.
       PROCEDURE DIVISION.
           SET SW1 SW2 TO ON.
           IF S1ON DISPLAY "S1 on" END-IF.
           IF S2ON DISPLAY "S2 on" END-IF.
           SET SW1 TO OFF.
           IF S1OFF DISPLAY "S1 off" END-IF.
           IF S1ON DISPLAY "S1 still on" ELSE DISPLAY "S1 now off" END-IF.
           IF S2ON DISPLAY "S2 still on" END-IF.
           STOP RUN.
