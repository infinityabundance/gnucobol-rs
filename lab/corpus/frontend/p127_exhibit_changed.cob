       IDENTIFICATION DIVISION.
       PROGRAM-ID. P127.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 A PIC 9   VALUE 3.
       01 B PIC X(2) VALUE "hi".
       PROCEDURE DIVISION.
      *> cobc 3.2 runs EXHIBIT CHANGED as plain EXHIBIT (CHANGED suppression unimplemented).
      *> Only CHANGED without NAMED drops the "name = " prefix.
           EXHIBIT CHANGED A B.
           EXHIBIT CHANGED NAMED A B.
           EXHIBIT A.
           EXHIBIT NAMED B.
           STOP RUN.
