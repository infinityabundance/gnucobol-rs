
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
       ALPHABET ALPHA IS ASCII.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01.
         02 X-POS       PIC 9 VALUE 3.
         02 X           PIC X(8) OCCURS 2.
       PROCEDURE        DIVISION.
           INSPECT X(1) REPLACING ALL SPACES BY "AA".
           INSPECT X(1) REPLACING ALL "ABC"  BY "AA".
           INSPECT X(1) REPLACING ALL "DEF"  BY SPACES.
           INSPECT X(1) CONVERTING SPACES TO "AA".
           INSPECT X(1) CONVERTING "ABC"  TO "AA".
           INSPECT X(1) (X-POS:2) CONVERTING "DEF"  TO SPACES.
           INSPECT X(1) CONVERTING "GHI"  TO ALPHA.
      *>   the following is allowed, see NC221A and ref-mod definition
           INSPECT X(1) CONVERTING "DEF"  TO X(2) (X-POS:3).
           INSPECT X(1) CONVERTING "DEF"  TO X(2) (X-POS:4).
           STOP RUN.
