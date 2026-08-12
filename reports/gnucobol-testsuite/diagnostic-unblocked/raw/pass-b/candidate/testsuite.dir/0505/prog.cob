           TITLE "GnuCOBOL lists IBM"
       IDENTIFICATION   DIVISION.
           SKIP1
       PROGRAM-ID.      prog.
           SKIP2
       DATA             DIVISION.
           SKIP3
       WORKING-STORAGE  SECTION.
       01  TITLE-01          PIC X(2).
       01  TITLE-02          PIC X(2).
           TITLE "here goes the code"
       PROCEDURE        DIVISION.
       EJECT
           MOVE SPACE TO TITLE-01
                         TITLE-02.
           STOP RUN.
