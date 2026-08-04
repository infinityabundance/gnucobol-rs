
      *> Valid
           $SET ADDRSV"DOG""CAT"
           $SET ADD-RSV  "doggy" "catty"
      *> Valid
           $SET ADD-SYN "VALUE" = "VA"
      *> Invalid - Bread is not reserved.
           $SET ADDSYN "BREAD" = "BARA"
      *> Invalid - ID is already reserved
           $SET ADDSYN "IDENTIFICATION" = "ID"

      *> Valid
           $SET MAKESYN(PROGRAM) = (FUNCTION)
      *> Invalid - BREAD is not reserved.
           $SET MAKESYN "BREAD" = "PROGRAM"
           $SET MAKE-SYN "program" = "bread"

      *> Valid
           $SET OVERRIDE "DIVISION" = "DIV" "JUST" = "JS"
      *> Invalid - Bread is not reserved
           $SET OVERRIDE "BREAD" = "BARA"
      *>Invalid - ID is already reserved; note: MF documents this rule but
      *>  does not check it and applies the line; we do it better on purpose :-)
           $SET OVERRIDE "IDENTIFICATION" = "ID"

      *> Valid - note: MF rules does allow reserving not reserved words
           $SET REMOVE "BREAD" (BARA)REMOVE(DOG)

       IDENTIFICATION DIV.
       PROGRAM-ID. prog.

       DATA DIV.
       WORKING-STORAGE SECTION.
      *> Check ADDSYN and OVERRIDE work correctly
       01  just PIC XX VA "1" JS.
      *> Check ADDRSV
       01  cat PIC 9 VA 1.
      *> Check REMOVE
       01  dog PIC 9 VA 1.
