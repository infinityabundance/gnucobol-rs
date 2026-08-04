
       IDENTIFICATION DIVISION.

       PROGRAM-ID. prog.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  x PIC 9.
        88 never-true value 99.
       01  y PIC 9.
        88 never-truen value -9.
       01  xx pic x.
        88 some-not values 'a', 'b', 'cd'.
        88 some-not-s values '00', 'a', 'b', 'cd'.

       PROCEDURE DIVISION.
           if never-true
            continue
           end-if
           if never-truen
            continue
           end-if
           if some-not-s
            set some-not   to true
            set some-not-s to true
           end-if
           set never-true  to true
           goback.
