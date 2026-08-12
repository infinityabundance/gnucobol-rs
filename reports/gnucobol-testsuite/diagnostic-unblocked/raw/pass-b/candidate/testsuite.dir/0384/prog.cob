
       IDENTIFICATION DIVISION.
       PROGRAM-ID. DEMOUNSTRING.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  Full-Name               PIC X(40).
       01  SUB1                    USAGE BINARY-SHORT.
       PROCEDURE DIVISION.
           MOVE 1                  TO SUB1
           PERFORM LENGTH(Full-Name) TIMES
           IF Full-Name(SUB2:1) = SPACE
              IF Full-Name(SUB2 + 1:1) = SPACE
                 DISPLAY 'IN'
                 EXIT PERFORM CYCLE
              END-IF
           END-IF
           ADD 1 TO SUB1.
