
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.

       ENVIRONMENT      DIVISION.
       INPUT-OUTPUT     SECTION.
       FILE-CONTROL.
           SELECT f ASSIGN "f.dat" SEQUENTIAL
               FILE-LIMITS ARE 1 THRU 10, 100 THRU f-max
               TRACK-AREA 100 CHARACTERS
               TRACK-LIMIT 5 TRACKS.
           SELECT g ASSIGN "g.dat" RELATIVE
               RELATIVE KEY g-key
               ACTUAL KEY g-actual-key.

       I-O-CONTROL.
           APPLY RECORD-OVERFLOW f, g
           APPLY CORE-INDEX core-idx ON f
           APPLY CYL-INDEX TO 5 ON f
           APPLY CYL-OVERFLOW 10 TRACKS f
           APPLY EXTENDED-SEARCH g
           APPLY MASTER-INDEX TO 5 on g
           APPLY WRITE-VERIFY f, g
           APPLY REORG-CRITERIA f-rec, f
           RERUN ON "g2.dat" EVERY END REEL g
           .
       DATA             DIVISION.
       FILE             SECTION.
       FD  f.
       01  f-rec        PIC X(100).

       FD  g.
       01  g-rec        PIC 9(10).

       WORKING-STORAGE  SECTION.
       01  core-idx     PIC 999.
       01  f-max        PIC 9(5) VALUE 1000.
       01  g-key        PIC 999.
       01  g-actual-key PIC XXX.

       PROCEDURE        DIVISION.
           OPEN INPUT f DISP, INPUT g REREAD
           .
