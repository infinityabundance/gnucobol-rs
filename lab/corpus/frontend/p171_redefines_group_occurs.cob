      *> A group-OCCURS table that REDEFINES a VALUE-bearing group (the classic "table initialised via a
      *> redefinition" idiom): WS-ENTRIES OCCURS reads the literal entries through the redefinition, SEARCH
      *> finds one, and a write through the redefining table lands in the shared storage (E5 sees it). Identical.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P171.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-TABLE.
          05 WS-DETAIL.
             10 E1 PIC X(3) VALUE '901'.
             10 E2 PIC X(3) VALUE '902'.
             10 E3 PIC X(3) VALUE '903'.
             10 E4 PIC X(3) VALUE '904'.
             10 E5 PIC X(3) VALUE '905'.
       01 WS-TABLE-R REDEFINES WS-TABLE.
          05 WS-ENTRY OCCURS 5 INDEXED BY IX.
             10 WS-VAL PIC X(3).
       01 WS-KEY PIC X(3) VALUE '903'.
       01 WS-FOUND PIC X(3) VALUE SPACES.
       PROCEDURE DIVISION.
           DISPLAY "READ v2=" WS-VAL (2) " v5=" WS-VAL (5).
           SET IX TO 1.
           SEARCH WS-ENTRY
              AT END DISPLAY "NOTFOUND"
              WHEN WS-VAL (IX) = WS-KEY MOVE WS-VAL (IX) TO WS-FOUND
           END-SEARCH.
           DISPLAY "FOUND=" WS-FOUND.
           MOVE 'ZZZ' TO WS-VAL (5).
           DISPLAY "AFTER E5=" E5 " v5=" WS-VAL (5).
           STOP RUN.
