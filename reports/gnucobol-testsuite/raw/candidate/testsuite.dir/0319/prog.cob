
       IDENTIFICATION  DIVISION.
       PROGRAM-ID.     prog.

       DATA            DIVISION.
       WORKING-STORAGE SECTION.
       01  valid-ptr     PIC 99.
       01  valid-ptr-2   BINARY-LONG.
       01  valid-ptr-3   PIC 99 COMP-3.

       01  invalid-ptr   PIC 9.
       01  invalid-ptr-2 PIC 99V99.
       01  invalid-ptr-3 FLOAT-LONG.
       01  invalid-ptr-4 PIC 99PP.
       01  invalid-ptr-5 PIC VPP99.
       01  invalid-ptr-6 PIC X(30).
       01  invalid-ptr-7.
           03  n         PIC 99.
           03  n2        PIC 99.


       01  x             PIC X(9).
       01  y             PIC X(9).

       PROCEDURE       DIVISION.
           STRING "hello" INTO x POINTER valid-ptr
           STRING "hello" INTO x POINTER valid-ptr-2
           STRING "hello" INTO x POINTER valid-ptr-3

           STRING "hello" INTO x POINTER invalid-ptr
           STRING "hello" INTO x POINTER invalid-ptr-2
           STRING "hello" INTO x POINTER invalid-ptr-3
           STRING "hello" INTO x POINTER invalid-ptr-4
           STRING "hello" INTO x POINTER invalid-ptr-5
           STRING "hello" INTO x POINTER invalid-ptr-6
           STRING "hello" INTO x POINTER invalid-ptr-7

           UNSTRING y INTO x POINTER valid-ptr
           UNSTRING y INTO x POINTER valid-ptr-2
           UNSTRING y INTO x POINTER valid-ptr-3

           UNSTRING y INTO x POINTER invalid-ptr
           UNSTRING y INTO x POINTER invalid-ptr-2
           UNSTRING y INTO x POINTER invalid-ptr-3
           UNSTRING y INTO x POINTER invalid-ptr-4
           UNSTRING y INTO x POINTER invalid-ptr-5
           UNSTRING y INTO x POINTER invalid-ptr-6
           UNSTRING y INTO x POINTER invalid-ptr-7
           .
