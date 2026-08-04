
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       01  valid-1       PIC 9.
       01  valid-2       BINARY-LONG.
       01  valid-3       PIC 99 COMP-3.

       01  invalid-1     PIC 99V99.
       01  invalid-2     FLOAT-LONG.
       01  invalid-3     PIC 99PP.
       01  invalid-4     PIC VPP99.
       01  invalid-5     PIC X(30).
           88  invalid-6 VALUE "abc".

       01  x             PIC X(9).
       01  y             PIC X(9).

       PROCEDURE       DIVISION.
           UNSTRING x INTO y COUNT valid-1
           UNSTRING x INTO y COUNT valid-2
           UNSTRING x INTO y COUNT valid-3

           UNSTRING x INTO y COUNT invalid-1
           UNSTRING x INTO y COUNT invalid-2
           UNSTRING x INTO y COUNT invalid-3
           UNSTRING x INTO y COUNT invalid-4
           UNSTRING x INTO y COUNT invalid-5
           UNSTRING x INTO y COUNT invalid-6
           .
