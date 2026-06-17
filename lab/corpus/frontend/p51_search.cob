      *> SEARCH (serial) over an OCCURS ... INDEXED BY table: a WHEN that matches mid-table, and an
      *> AT END when no element matches. Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P51.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 TBL PIC X(3) OCCURS 4 INDEXED BY IX.
       PROCEDURE DIVISION.
           MOVE "CAT" TO TBL(1).
           MOVE "DOG" TO TBL(2).
           MOVE "FOX" TO TBL(3).
           MOVE "OWL" TO TBL(4).
           SET IX TO 1.
           SEARCH TBL
               AT END DISPLAY "MISS-FOX"
               WHEN TBL(IX) = "FOX" DISPLAY "FOUND-FOX"
           END-SEARCH.
           SET IX TO 1.
           SEARCH TBL
               AT END DISPLAY "MISS-ZZZ"
               WHEN TBL(IX) = "ZZZ" DISPLAY "FOUND-ZZZ"
           END-SEARCH.
           STOP RUN.
