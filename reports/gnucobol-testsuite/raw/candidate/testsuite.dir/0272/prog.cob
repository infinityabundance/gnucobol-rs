
       IDENTIFICATION   DIVISION.
       PROGRAM-ID.      prog.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
           SW1
             ON  IS SWIT1-ON
             OFF IS SWIT1-OFF
           .
           SWITCH B IS SWITCH-B
             ON  IS SWIT2-ON
             OFF IS SWIT2-OFF
           .
           SWITCH-25
             ON  IS SWIT25-ON
             OFF IS SWIT25-OFF
           .
           SWITCH-25
             ON  IS SWIT25-IS-ON
             OFF IS SWIT25-IS-OFF
           .
           SWITCH 25
             ON  IS SWIT25-SP-ON
             OFF IS SWIT25-SP-OFF
           .
           SWITCH Y
             ON  IS SWIT25-Y-ON
             OFF IS SWIT25-Y-OFF
           .
           SWITCH Z
             ON  IS SWIT26-ON
             ON  IS SWIT26-OFF
           .
           SWITCH-32
             ON  IS SWIT32-ON
           .
       DATA             DIVISION.
       WORKING-STORAGE  SECTION.
       01  SWITCH       PIC 99 VALUE 12.
       PROCEDURE        DIVISION.
           ADD SWITCH 1 GIVING SWITCH
           END-ADD.
           IF SWITCH NOT = 13
              DISPLAY "SWITCH (variable) WRONG: "
                      SWITCH
              END-DISPLAY
           END-IF.
           IF SWIT1-ON
              DISPLAY "ON"
              END-DISPLAY
           ELSE
              DISPLAY "OFF"
              END-DISPLAY
           END-IF.
           IF SWIT2-ON
              DISPLAY " ON"
              END-DISPLAY
           ELSE
              DISPLAY " OFF"
              END-DISPLAY
           END-IF.
           SET SWITCH-B TO OFF
           IF SWIT2-ON
              CONTINUE
           END-IF.
           IF SWIT25-ON
              CONTINUE
           END-IF.
           IF SWIT26-ON
              CONTINUE
           END-IF.
           IF SWIT32-ON
              CONTINUE
           END-IF.
           IF SWIT32-OFF
              CONTINUE
           END-IF.
           STOP RUN.
