      *> UNSTRING with DELIMITER / COUNT phrases WITHOUT the optional IN (cobc accepts both forms). Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P151.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S  PIC X(12) VALUE "A,BB,,CCC,D".
       01 F1 PIC X(4).
       01 F2 PIC X(4).
       01 F3 PIC X(4).
       01 C1 PIC 99.
       01 D1 PIC X.
       01 TLY PIC 99 VALUE 0.
       PROCEDURE DIVISION.
           UNSTRING S DELIMITED BY ","
              INTO F1 DELIMITER D1 COUNT C1 F2 F3
              TALLYING IN TLY
           END-UNSTRING.
           DISPLAY "[" F1 "][" F2 "][" F3 "] C1=" C1 " D1=[" D1 "] T=" TLY.
           STOP RUN.
