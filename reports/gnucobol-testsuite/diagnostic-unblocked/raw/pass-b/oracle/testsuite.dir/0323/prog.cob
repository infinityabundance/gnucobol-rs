
      *> Prototype and definition have different signatures.
       IDENTIFICATION DIVISION.
       FUNCTION-ID. invalid-1 PROTOTYPE.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SOURCE-COMPUTER. x.
       SPECIAL-NAMES.
           DECIMAL-POINT COMMA
           CURRENCY SIGN "F"
           .
       DATA DIVISION.
       LINKAGE SECTION.
       01  x.
           03  x1 PIC X(15).
           03  x2 PIC X(15) JUST.
       01  y PIC +9(30) BLANK ZERO.

       PROCEDURE DIVISION USING x returning y.
       END FUNCTION invalid-1.

      *> Prototype and definition have different signatures.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. invalid-2 PROTOTYPE.
       DATA DIVISION.
       LINKAGE SECTION.
       01  x PIC X.
       PROCEDURE DIVISION USING x.
       END PROGRAM invalid-2.

      *> Prototype and definition have matching signatures.
       IDENTIFICATION DIVISION.
       FUNCTION-ID. valid-1 PROTOTYPE.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
           DECIMAL-POINT COMMA
           .
       DATA DIVISION.
       LINKAGE SECTION.
       01  ret PIC 999V99.

       PROCEDURE DIVISION RETURNING ret.
       END FUNCTION valid-1.


       IDENTIFICATION DIVISION.
       PROGRAM-ID. blahblah AS "invalid-1".

       DATA DIVISION.
       LINKAGE SECTION.
       01  a.
           03  a-1 PIC X(15) JUST.
           03  a-2 PIC X(15).
       01  b PIC S9(10) SIGN LEADING.

       PROCEDURE DIVISION USING a RETURNING b.
           DISPLAY "yes"
           .
       END PROGRAM blahblah.


       IDENTIFICATION DIVISION.
       PROGRAM-ID. invalid-2.

       DATA DIVISION.
       LINKAGE SECTION.
       01  ret PIC X.

       PROCEDURE DIVISION RETURNING ret.
           MOVE "x" TO ret
           .
       END PROGRAM invalid-2.

       IDENTIFICATION DIVISION.
       FUNCTION-ID. valid-1.

       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
           DECIMAL-POINT COMMA
           .
       DATA DIVISION.
       LINKAGE SECTION.
       01  ret PIC 999V99.

       PROCEDURE DIVISION RETURNING ret.
           MOVE 1,5 TO ret
           .
       END FUNCTION valid-1.

      *> Prototypes must be at start of compilation group.
       IDENTIFICATION DIVISION.
       PROGRAM-ID. invalid-3.

       DATA DIVISION.
       LINKAGE SECTION.
       01  x PIC X(10).

       PROCEDURE DIVISION USING x.
       END PROGRAM invalid-3.

       IDENTIFICATION DIVISION.
       PROGRAM-ID. invalid-3 IS PROTOTYPE.

       DATA DIVISION.
       LINKAGE SECTION.
       01  x PIC X(10).

       PROCEDURE DIVISION USING x.
       END PROGRAM invalid-3.
