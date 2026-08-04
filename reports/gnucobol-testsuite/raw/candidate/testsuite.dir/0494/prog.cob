
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 WS-ONE    PIC 9(4) VALUE 37.
       01 WS-TWO    PIC A(4) VALUE 'HIGH'.
       01 WS-THREE  PIC X(4) VALUE 'BAR'.
       01 WS-FOUR            COMP-1 VALUE 37.
       01 WS-FIVE            COMP-2 VALUE 37.
       01 WS-SIX    PIC S999 COMP-3 VALUE -37.
       01 WS-SEVEN  PIC $$,$$$,$$9.99 VALUE ZERO.
       PROCEDURE        DIVISION.
           STOP RUN.
