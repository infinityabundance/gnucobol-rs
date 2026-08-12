
       IDENTIFICATION DIVISION.
       PROGRAM-ID.    prog.
       ENVIRONMENT    DIVISION.
       CONFIGURATION  SECTION.
       SPECIAL-NAMES.
           stdin IS my-stdin
           .
       PROCEDURE      DIVISION.
           CALL "something" USING stdout
           CALL "something" USING stdin
           CALL "something" USING my-stdin
           .
