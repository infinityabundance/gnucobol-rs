
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT OLD-VERSION  ASSIGN TO "SYSUT1"
                               ORGANIZATION LINE SEQUENTIAL.
           SELECT NEW-VERSION  ASSIGN TO "SYSUT2"
                               ORGANIZATION LINE SEQUENTIAL.
           SELECT PRT-VERSION  ASSIGN TO "SYSUT2"
                               ORGANIZATION LINE SEQUENTIAL.
           SELECT MODIFICATION ASSIGN TO "SYSIN1"
                               ORGANIZATION LINE SEQUENTIAL.
           SELECT COMMENTARY   ASSIGN TO "SYSOU1"
                               ORGANIZATION LINE SEQUENTIAL.
       DATA DIVISION.
       FILE SECTION.

       FD  OLD-VERSION
           LABEL RECORDS ARE STANDARD
           BLOCK CONTAINS 80 CHARACTERS
           DATA RECORD IS OLD-RECORD.

       01  OLD-RECORD.
           02 OLD-STATEMENT       PICTURE X(75).
           02 OLD-NUMBER          PICTURE X(5).

       FD  NEW-VERSION
           LABEL RECORDS ARE STANDARD
           BLOCK CONTAINS 80 CHARACTERS
           DATA RECORD IS NEW-RECORD.

       01  NEW-RECORD.
           02 NEW-STATEMENT       PICTURE X(75).
           02 NEW-NUMBER          PICTURE X(5).

       FD  MODIFICATION
           LABEL RECORDS ARE OMITTED
           BLOCK CONTAINS 80 CHARACTERS
           DATA RECORD IS UPDATE-ORDER.

       01  UPDATE-ORDER.
           02 INSERTION.
              03 COMMAND          PICTURE X(6).
                 88 ENDJOB        VALUE "ENDJOB".
                 88 ENDSET        VALUE "ENDSET".
                 88 REMOVE        VALUE "REMOVE".
                 88 ADDNEW        VALUE "INSERT".
                 88 CHANGE        VALUE "CHANGE".
                 88 DISPLY        VALUE "DISPLY".
              03 FILLER           PICTURE X.
              03 A-FIELD          PICTURE 9(5).
              03 A-ALPHA REDEFINES A-FIELD    PICTURE X(5).
                 88 A-BLANK       VALUE SPACES.
              03 FILLER           PICTURE X(4).
              03 B-FIELD          PICTURE 9(5).
              03 B-ALPHA REDEFINES B-FIELD    PICTURE X(5).
                 88 B-BLANK       VALUE SPACES.
              03 FILLER           PICTURE X(54).
           02 FILLER              PICTURE X(5).

       FD  COMMENTARY
           LABEL RECORDS ARE OMITTED
           BLOCK CONTAINS 82 CHARACTERS
           DATA RECORD IS COMMENT-LINE.

       01  COMMENT-LINE.
           02 FILLER              PICTURE X(82).

       WORKING-STORAGE  SECTION.

       01  HEADINGS-LINE.
           02 FILLER              PICTURE X(15) VALUE "EDITOR VERSION".
           02 FILLER              PICTURE X(20) VALUE "1.1 - 206/72".
           02 PHASE               PICTURE X(17) VALUE "UPDATING AS OF".
           02 MONTH-RUN           PICTURE XX.
           02 FILLER              PICTURE X VALUE "/".
           02 DAY-RUN             PICTURE XX.
           02 FILLER              PICTURE X VALUE "/".
           02 YEAR-RUN            PICTURE XX.
           02 FILLER              PICTURE X(8) VALUE SPACES.
           02 FILLER              PICTURE X(8) VALUE "  PAGE: ".
           02 PAGE-NUMBER         PICTURE 9(4) VALUE 0.

       01  COMMAND-LISTING.
           02 FILLER              PICTURE X(2) VALUE SPACES.
           02 COMMAND-IMAGE       PICTURE X(80).

       01  ACTIVITIES-LISTING.
           02 DISPOSITION         PICTURE X(2).
           02 ACTIVE-IMAGE        PICTURE X(80).

       01  UPSI-BYTE.
           02 UPSI-BIT OCCURS 8 TIMES  PICTURE X.

       01  MESSAGE-LOG.
           02 FILLER              PICTURE X(2) VALUE SPACES.
           02 MESSAGE-TEXT        PICTURE X(80).

       01  DISPLAY-MESSAGE.
           02 FILLER              PICTURE X(2) VALUE SPACES.
           02 DISPLAY-TEMP        PICTURE X(6).
           02 FILLER              PICTURE X(2) VALUE SPACES.
           02 DISPLAY-TEXT        PICTURE X(60).

       PROCEDURE DIVISION.
           OPEN INPUT OLD-VERSION, MODIFICATION,
                OUTPUT NEW-VERSION, COMMENTARY.
           CLOSE OLD-VERSION, NEW-VERSION, MODIFICATION, COMMENTARY.
           STOP RUN.
