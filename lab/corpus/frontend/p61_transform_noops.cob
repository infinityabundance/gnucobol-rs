      *> TRANSFORM (= INSPECT CONVERTING) plus verbs GnuCOBOL 3.2 accepts as no-ops (RAISE/VALIDATE/DESTROY/
      *> READY/RESET are "not implemented" or have no stdout effect). Identical stdout under cobc and cobrun.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P61.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 X PIC X(5) VALUE "ABCDE".
       01 N PIC 9 VALUE 5.
       PROCEDURE DIVISION.
           TRANSFORM X FROM "ABC" TO "XYZ".
           DISPLAY "T=[" X "]".
           RAISE EXCEPTION EC-ALL.
           VALIDATE N.
           DESTROY N.
           READY TRACE.
           RESET TRACE.
           DISPLAY "DONE N=" N.
           STOP RUN.
