
       IDENTIFICATION DIVISION.
       PROGRAM-ID.    prog.

       DATA           DIVISION.
       LINKAGE        SECTION.
      *> Valid
       01  valid-1    PIC X ANY LENGTH.
       01  valid-2    PIC N ANY LENGTH.
       01  valid-3    PIC 9 ANY NUMERIC.

      *> Invalid
       01  invalid-1  PIC A ANY LENGTH.
       01  invalid-2  PIC Z ANY LENGTH.
       01  invalid-3  PIC 9 ANY LENGTH.
       01  invalid-4  PIC X ANY NUMERIC.
       01  invalid-5  PIC XX ANY LENGTH.
       01  invalid-6  PIC NN ANY LENGTH.
