
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  empty-pic PIC.
       01  too-long-pic PIC XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
      -XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.
       01  too-long-pic2 PIC XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
      -XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
      -XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
      -XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
      -XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX.
       01  multiple-symbols.
           03  PIC 9CRCR.
           03  PIC 9DBDB.
           03  PIC SS99S.
           03  PIC 99..9.
           03  PIC 99VV9.
           03  PIC +$99+.
           03  PIC $+99$-.
       01  non-symbols.
           03  PIC 9K.
           03  PIC 999C.
           03  PIC 999D.
       01  too-many-digits PIC 9(50).
       01  too-long-number-in-parens PIC 9(11111111111111).
       01  nested-parens PIC 9((100)).
       01  unbalanced-parens PIC 9(.
       01  multiple-pairs-of-parens PIC 9(5)(3).
       01  no-digit-in-parens PIC 9().
       01  mutually-exclusive-symbols.
           03  PIC P(3)9.9.
           03  PIC 9V.9.
           03  PIC Z*.
           03  PIC +(5)--.
           03  PIC $(4)Z(9).
           03  PIC $$B*(4).
           03  PIC NX.
           03  PIC AN.
           03  PIC AZ(3).
           03  PIC 99.99XXXXX.
           03  PIC SA.
           03  PIC $$$B+++B---.
           03  PIC +++9+.
           03  PIC +9(5)CR.
           03  PIC -9(5)DB.
       01 non-rightmost-leftmost-symbols.
           03  PIC BBB+BB99.
           03  PIC 99-B.
           03  PIC 9CRB.
           03  PIC DB9(5).
           03  PIC 99$$$.
           03  PIC 99$B.
           03  PIC 0$99.
           03  PIC PPPVP9.
       01  missing-symbols.
           03  PIC B(5).
           03  PIC +.
           03  PIC $.

       01  str-constant CONSTANT "hello".
       01  float-constant CONSTANT 1.0.
       01  signed-constant CONSTANT -1.
       01  invalid-constant.
           03  PIC X(str-constant).
           03  PIC X(float-constant).
           03  PIC X(signed-constant).
           03  PIC X(unseen-constant).

       01  integer-constant CONSTANT 5.
       01  valid-pics.
           03  PIC VP9B.
           03  PIC B9P(3).
           03  PIC B$$$.
           03  PIC 0000+B0+++0B,+.
           03  PIC +(5)P(3).
           03  PIC ++.++.
           03  PIC $(integer-constant).
           03  PIC $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
      -(integer-constant).   *> CHECKME: should this be really valid?


       01  PC-COLOR-BACKGROUND-TABLE.
           05  BIT-BACKGROUND-BLACK      PIC 1(8) BIT VALUE B"00000000".
           05  BIT-BACKGROUND-BLUE       PIC 1(8) BIT VALUE B"00010000".
           05  BIT-BACKGROUND-GREEN      PIC 1(8) BIT VALUE B"00100000".
           05  BIT-BACKGROUND-CYAN       PIC 1(8) BIT VALUE B"00110000".
           05  BIT-BACKGROUND-RED        PIC 1(8) BIT VALUE B"01000000".
           05  BIT-BACKGROUND-MAGENTA    PIC 1(8) BIT VALUE B"01010000".
           05  BIT-BACKGROUND-BROWN      PIC 1(8) BIT VALUE B"01100000".
           05  BIT-BACKGROUND-LIGHT-GRAY PIC 1(8) BIT VALUE B"01110000".
       01  FILLER REDEFINES PC-COLOR-BACKGROUND-TABLE.
           05  COLOR-BACKGROUND
               OCCURS 8 TIMES            PIC 1(8) BIT.
