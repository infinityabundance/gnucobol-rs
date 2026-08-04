
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       ENVIRONMENT      DIVISION.
       CONFIGURATION    SECTION.
       SPECIAL-NAMES.
            SYMBOLIC NL IS 101
                     NL2   102
            CLASS    CHECK-VALID    'a' THRU 'z'
                                    'A' THRU 'Z'
                                    'cdef'
            .
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01 X             PIC X(8).
       PROCEDURE        DIVISION.
           IF X         IS CHECK-VALID
              DISPLAY "OK"
              END-DISPLAY
           END-IF.
           STOP RUN.
