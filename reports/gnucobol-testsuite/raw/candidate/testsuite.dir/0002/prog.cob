
        IDENTIFICATION DIVISION.
        PROGRAM-ID. prog.
        DATA DIVISION.
        WORKING-STORAGE SECTION.
        77 var1    pic 9.
        77 var2    pic 99.
        PROCEDURE DIVISION.
        PERFORM UNTIL 0 = 1
           DISPLAY 'BLA'
        END-PERFORM.
        MOVE var1 to var2
        MOVE var2 to var1
        GOBACK.
