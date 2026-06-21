      *> UNSTRING into binary (COMP/COMP-5) and packed (COMP-3) receivers: the delimited segment is sized by
      *> the receiver's DIGIT width and stored via the alnum->binary/packed conversion. Identical to cobc 3.2.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P159.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 S  PIC X(20) VALUE "12,345,6789,42,7".
       01 A  PIC 9(3) COMP.
       01 B  PIC S9(4) COMP-3.
       01 C  PIC 9(4) COMP-5.
       01 D  PIC 999.
       01 C1 PIC 99.
       PROCEDURE DIVISION.
           UNSTRING S DELIMITED BY "," INTO A COUNT C1 B C D.
           DISPLAY "A=" A " C1=" C1 " B=" B " C=" C " D=" D.
           STOP RUN.
