       IDENTIFICATION DIVISION.
       PROGRAM-ID. P109.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S  PIC X(12) VALUE "AB,CDE,F".
       01 P1 PIC X(4).
       01 P2 PIC X(4).
       01 P3 PIC X(4).
       01 D1 PIC X.
       01 C1 PIC 9.
       01 N  PIC 9 VALUE 5.
       PROCEDURE DIVISION.
           UNSTRING S DELIMITED BY ","
              INTO P1 DELIMITER IN D1 COUNT IN C1
                   P2
                   P3
              TALLYING IN N.
           DISPLAY "[" P1 "][" P2 "][" P3 "][" D1 "][" C1 "] N=" N.
           STOP RUN.
