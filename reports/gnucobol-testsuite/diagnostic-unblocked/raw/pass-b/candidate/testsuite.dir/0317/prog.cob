
       IDENTIFICATION   DIVISION.
      *> Valid
       >> TURN ec-i-o f CHECKING ON
      *> Invalid
       >> TURN dd CHECKING ON
       >> TURN ec-all f CHECKING ON
       >> TURN EC-I-O-invalid-KEY not-exist CHECKING OFF
       PROGRAM-ID.      prog.
       ENVIRONMENT      DIVISION.
       INPUT-OUTPUT     SECTION.
       FILE-CONTROL.
           SELECT F ASSIGN "f.txt", SEQUENTIAL.
       DATA             DIVISION.
       FILE             SECTION.
       FD  f.
       01  f-rec        PIC X.
       PROCEDURE        DIVISION.
           STOP RUN.
