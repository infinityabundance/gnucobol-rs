
       SOURCE-COMPUTER. GNU-LINUX.
       SPECIAL-NAMES.
            SYMBOLIC NL IS 101
                     NL2   102
            NUMERIC SIGN TRAILING SEPARATE
            DECIMAL-POINT IS COMMA
            .

            SELECT PRINT-FILE ASSIGN "EXTRXW"
            ORGANIZATION LINE SEQUENTIAL
            .
       DATA DIVISION.
       FD  PRINT-FILE EXTERNAL.
       01  PRINT-REC          PIC X(64).

           DISPLAY "X"
           END-DISPLAY
           ACCEPT OMITTED
           END-ACCEPT
           GOBACK
           .
