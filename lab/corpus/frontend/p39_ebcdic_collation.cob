      *> @no312: EBCDIC collating sequence is ignored in GnuCOBOL 3.1.2 but applied in 3.2
      *> PROGRAM COLLATING SEQUENCE IS <ebcdic-alphabet> orders alphanumeric comparisons in EBCDIC
      *> code-point order: lowercase < uppercase < digits (the opposite of ASCII for case, and letters
      *> below digits). Identical stdout under cobc 3.2 and cobrun. (String literals keep their case.)
       IDENTIFICATION DIVISION.
       PROGRAM-ID. P39.
       ENVIRONMENT DIVISION.
       CONFIGURATION SECTION.
       SPECIAL-NAMES.
           ALPHABET EB IS EBCDIC.
       OBJECT-COMPUTER.
           PROGRAM COLLATING SEQUENCE IS EB.
       PROCEDURE DIVISION.
           IF "a" < "A"
               DISPLAY "lower-before-upper"
           ELSE
               DISPLAY "upper-before-lower"
           END-IF.
           IF "Z" < "0"
               DISPLAY "letters-before-digits"
           ELSE
               DISPLAY "digits-before-letters"
           END-IF.
           STOP RUN.
