
      *> Valid
           >>COBOL-WORDS RESERVE "CAT"
      *> Valid (GC-extension)
           >>COBOL-WORDS RESERVE  "doggy" "catty" "mouse"
      *> Valid
           >>COBOL-WORDS EQUATE "VALUE" WITH "VA"
      *> Invalid - Bread is not reserved.
           >>COBOL-WORDS EQUATE "BREAD" WITH "BARA"
      *> Invalid - ID is already reserved
           >>COBOL-WORDS EQUATE "IDENTIFICATION" WITH "ID"

      *> Valid, BREAD is not reserved.
           >>COBOL-WORDS SUBSTITUTE "program" BY "bread"
      *> Valid (GC-extension)
           >>COBOL-WORDS SUBSTITUTE "DIVISION" BY "DIV", "JUST" BY "JS"
      *> Invalid - Bread is not reserved.
           >>COBOL-WORDS SUBSTITUTE "BREAD" BY "BARA"
      *> Invalid - ID is already reserved
           >>COBOL-WORDS SUBSTITUTE "IDENTIFICATION" BY "ID"
      *> Invalid - needs BY, not WITH
      *> FIXME: error-recovery is bad, see below
      *> >>COBOL-WORDS SUBSTITUTE "INITIALIZE" WITH "INIT"

      *> Valid
           >>COBOL-WORDS UNDEFINE "BREAD"
      *> Valid (GC-extension)
           >>COBOL-WORDS UNDEFINE "DOGGY" "CATTY"
      *> Invalid in Standard COBOL, must be a defined word
           >>COBOL-WORDS UNDEFINE "BREAD"

      *> FIXME: error-recovery is bad, see below
      *>>COBOL-WORDS REMOVE "BREAD"

       IDENTIFICATION DIV.
       PROGRAM-ID. prog.

       DATA DIV.
       WORKING-STORAGE SECTION.
      *> Check EQUATE and SUBSTITUTE work correctly
       01  just PIC XX VA "1" JS.
      *> Check RESERVE
       01  cat PIC 9 VA 1.
      *> Check UNDEFINE
       01  dog PIC 9 VA 1.
