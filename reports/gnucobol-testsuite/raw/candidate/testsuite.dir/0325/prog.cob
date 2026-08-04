
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       01  valid-1     SIGN TRAILING.
           03  x       PIC S999.
           03  y       PIC S999.
       01  valid-2     PIC S99.

       01  invalid-1   PIC 99 SIGN LEADING.
       01  invalid-2   PIC S99 SIGN TRAILING, USAGE BINARY.
