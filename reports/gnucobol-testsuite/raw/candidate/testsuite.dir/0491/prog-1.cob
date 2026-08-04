
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog-1.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  blah   PIC x.

       PROCEDURE DIVISION.
           ACCEPT blah END-ACCEPT
           CALL "prog-2" USING blah END-CALL
           GO TO EX

           DISPLAY blah.

       EX. STOP RUN.
