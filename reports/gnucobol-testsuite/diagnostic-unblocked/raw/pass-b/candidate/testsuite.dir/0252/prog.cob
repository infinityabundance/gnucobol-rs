
        IDENTIFICATION DIVISION.
        PROGRAM-ID. prog.
        DATA DIVISION.
        WORKING-STORAGE SECTION.
        01 WRKN  PIC S999 VALUE 123.
           88 WRKN-IS-ZERO VALUE 0.
        01 WRKX  PIC X(8) VALUE 'House'.
        PROCEDURE DIVISION.
        MAIN.
            EVALUATE WRKN
              GREATER ZERO
              < 0
              > 0
            WHEN TRUE
              DISPLAY "WHAT IS IT?"
            END-EVALUATE.
            IF WRKN = 123 EQUAL 456
              DISPLAY "Strange brew! " WRKN
            END-IF.
            IF WRKN NOT EQUAL 123 NOT = 456
              DISPLAY "Strange brew! " WRKN
            END-IF.
            IF WRKN = 123 OR 456
              DISPLAY "Home brew! " WRKN
            END-IF.
            IF WRKX = "Red" OR "White"
              DISPLAY "Home wine! " WRKX
            END-IF.
            IF WRKX <= "Red" = "White"
              DISPLAY "Home wine! " WRKX
            END-IF.
            IF WRKX = "Red" NOT "White"
              DISPLAY "Home wine! " WRKX
            END-IF.
            IF WRKN = (123 - 12) OR
                   >= (456 + 16)
              DISPLAY "And another brew! " WRKN
            END-IF.
            IF ( WRKN-IS-ZERO + 3  ) > 0
	      DISPLAY "Weird if compiled"
	    END-IF.
            IF ( 3 + WRKN-IS-ZERO  ) > 0
	      DISPLAY "Weird if compiled"
	    END-IF.
            IF ( WRKN-IS-ZERO AND WRKN-IS-ZERO ) > 0
              DISPLAY "Weird if compiled"
            END-IF.
            STOP RUN.
