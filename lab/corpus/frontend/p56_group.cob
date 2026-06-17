      *> Group / sub-field items: a group reads as the concatenation of its leaves; a MOVE into the group
      *> distributes bytes across the leaves; sub-fields read/write independently. Identical under cobc/cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P56.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 REC.
          05 PART-A PIC X(3).
          05 PART-B PIC X(3).
          05 PART-C PIC X(3).
       01 PERSON.
          05 NAME-X PIC X(4).
          05 NUM-Y  PIC 9(3).
       PROCEDURE DIVISION.
           MOVE "AAA" TO PART-A.
           MOVE "BBB" TO PART-B.
           MOVE "CCC" TO PART-C.
           DISPLAY "REC=[" REC "]".
           MOVE "XYZ123QWE" TO REC.
           DISPLAY "A=[" PART-A "] B=[" PART-B "] C=[" PART-C "]".
           DISPLAY "REC2=[" REC "]".
           MOVE "BOB" TO NAME-X.
           MOVE 7 TO NUM-Y.
           DISPLAY "PERSON=[" PERSON "]".
           STOP RUN.
