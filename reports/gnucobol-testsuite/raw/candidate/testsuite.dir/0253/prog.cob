
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.
       ENVIRONMENT DIVISION.
       DATA DIVISION.
      ******************************************************************
       WORKING-STORAGE SECTION.
      *
       01 FLD1 PIC 9 VALUE 1.
       01 FLD2 PIC 9 VALUE 2.
       01 FLDX PIC 9 VALUE 5.
       01 FLDY PIC 9 VALUE 6.

       PROCEDURE DIVISION.
           IF 1 AND 2 > 1 THEN
                DISPLAY 'Test 1 is WRONG'
           ELSE
                DISPLAY 'Test 1 is OK'.

           IF FLD1 AND FLDX > FLD2 THEN
                DISPLAY 'Test 2 is OK'
           ELSE
                DISPLAY 'Test 2 is Wrong'.

           IF FLD1 OR FLDX > FLD2 THEN
                DISPLAY 'Test 2 is OK'
           ELSE
                DISPLAY 'Test 2 is Wrong'.

           IF FLD1 > 2 AND FLDX > FLD2 THEN
                DISPLAY 'Test 2 is OK'
           ELSE
                DISPLAY 'Test 2 is Wrong'.

           IF FLD2 IS NUMERIC AND FLD1 THEN
                DISPLAY 'Test 2 is OK'
           ELSE
                DISPLAY 'Test 2 is Wrong'.

           IF FLDX > FLD2 AND FLD1 AND 8 THEN
                DISPLAY 'Test 3 is OK ' FLDX ' > ' FLD2 ' & ' FLD1
           ELSE
                DISPLAY 'Test 3 is Wrong'.

           IF FLDX > FLD2 OR FLD1 OR 8 THEN
                DISPLAY 'Test 3 is OK'
           ELSE
                DISPLAY 'Test 3 is Wrong'.
           STOP RUN.
