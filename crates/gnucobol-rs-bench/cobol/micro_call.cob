      *> Micro: module CALL dispatch, 50_000 iterations to a contained
      *> subprogram (same source); accumulates the returned values.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MICRO-CALL.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  I                 PIC 9(9).
       01  ARG               PIC 9(9).
       01  RES               PIC 9(9).
       01  ACC               PIC S9(12) VALUE 0.
       01  A-E               PIC 9(12).
       01  I-E               PIC 9(9).
       PROCEDURE DIVISION.
           PERFORM VARYING I FROM 1 BY 1 UNTIL I > 50000
               MOVE I TO ARG
               CALL "MICRO-SUB" USING ARG RES
               ADD RES TO ACC
           END-PERFORM
           SUBTRACT 1 FROM I
           MOVE ACC TO A-E
           MOVE I TO I-E
           DISPLAY "CALL-DONE " A-E " " I-E
           STOP RUN.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MICRO-SUB.
       DATA DIVISION.
       LINKAGE SECTION.
       01  ARG               PIC 9(9).
       01  RES               PIC 9(9).
       PROCEDURE DIVISION USING ARG RES.
           COMPUTE RES = ARG + 1
           GOBACK.
       END PROGRAM MICRO-SUB.
