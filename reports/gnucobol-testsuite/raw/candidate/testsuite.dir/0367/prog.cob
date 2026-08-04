
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT SRTFIL ASSIGN TO "SRTFIL"
           ORGANIZATION LINE SEQUENTIAL.
           SELECT STFILE ASSIGN TO "STFILE"
           ORGANIZATION LINE SEQUENTIAL.

       DATA DIVISION.
       FILE SECTION.
       FD SRTFIL.
       01 SRTREC               PIC X(256).
       SD STFILE.
       01 STFREC               PIC X(256).

       WORKING-STORAGE SECTION.
       01 G             VALUE "d4b2e1a3c5".
         02 TBL         OCCURS 5.
           03 X         PIC X.
           03 Y         PIC 9.
         02 TBL-ORD     OCCURS 5 ASCENDING YO.
           03 XO        PIC X.
           03 YO        PIC 9.

       PROCEDURE DIVISION.
           SORT TBL ASCENDING KEY X.
           SORT TBL DESCENDING.
           SORT TBL.
           SORT TBL-ORD ASCENDING.
           SORT TBL-ORD.
           SORT STFILE DESCENDING KEY SRTREC.
           SORT STFILE DESCENDING KEY SRTREC USING SRTFIL GIVING SRTFIL.
           SORT STFILE ASCENDING.
           SORT STFILE.
           STOP RUN.
