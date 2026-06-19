       IDENTIFICATION DIVISION.
       PROGRAM-ID. P96.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 I PIC 9.
       01 J PIC 9.
       01 K PIC 9.
       01 N PIC 99 VALUE 0.
       PROCEDURE DIVISION.
      *> inline nested: i=1..2, j=1..3 (j fastest)
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 2
                   AFTER J FROM 1 BY 1 UNTIL J > 3
               DISPLAY "I=" I " J=" J
           END-PERFORM.
      *> triple nest + count
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 2
                   AFTER J FROM 1 BY 1 UNTIL J > 2
                   AFTER K FROM 1 BY 1 UNTIL K > 2
               ADD 1 TO N
           END-PERFORM.
           DISPLAY "N=" N.
           STOP RUN.
