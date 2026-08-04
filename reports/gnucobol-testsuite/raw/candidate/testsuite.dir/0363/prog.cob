
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  str.
           03  str-1 PIC XX.
           03  str-2 PIC X.
       66  renames-item RENAMES str-1 THRU str-2.
       01  bool-area.
           03  bool-item PIC 1(30) USAGE BIT.
           03  zoned-decimal PIC 99V99 PACKED-DECIMAL.
       01  just-item PIC X(30) JUST.
       01  table-area.
           03  table-entry PIC X(30) OCCURS 2 TIMES.
       01  long-str PIC X(200).

       01  float-item FLOAT-SHORT.
       01  pic-p-item PIC 99P(3).

       01  rec.
           03  child-1 PIC X(30).
           03  child-1a REDEFINES child-1 PIC 9(30).
           03  child-2 PIC X(30).
           03  child-3.
               05  child-3-1 PIC X OCCURS 5 TIMES.

       01  all-filler-rec.
           03  FILLER PIC XXXX.
           03  FILLER PIC 9999.

       01  invalid-sub-elt-rec.
           03  non-unique-name PIC X.
           03  valid-sub-rec.
               05  non-unique-name PIC X.
           03  bit-item PIC 1 USAGE BIT.
           03  ptr-item USAGE POINTER.

       01  with-attrs-does-nothing.
           03  FILLER PIC X.
           03  table-elt PIC X OCCURS 2 TIMES.
           03  with-attrs-group.
               05  with-attrs-group-child PIC X.
           03  with-attrs-child PIC X.

       PROCEDURE DIVISION.
      *> Receiving area is not alphanumeric or national.
           XML GENERATE bool-item FROM str
      *> Receiving area is JUSTIFIED RIGHT.
           XML GENERATE just-item FROM str
      *> Receiving area is subscripted or ref-mod'd.
           XML GENERATE table-entry (1) FROM str
           XML GENERATE long-str (1:100) FROM str

      *> Input record cannot be function identifier.
           XML GENERATE long-str FROM FUNCTION CHAR(4)
      *> Input record cannot be ref-mod'd.
           XML GENERATE long-str FROM str (2:1)
      *>  "       "   is not RENAMES (children may be RENAMES).
           XML GENERATE long-str FROM renames-item
      *> Non-ignored items of the input record must:
      *>  * alphabetic, alphanumeric, national, numeric or index.
      *>  * there must be at least one item.
      *>  * each non-FILLER name must be unique within the input record.
           XML GENERATE long-str FROM invalid-sub-elt-rec *> XXXXXXXXX ptr element is invalid
           XML GENERATE long-str FROM all-filler-rec

      *> COUNT IN field must be an integer.
           XML GENERATE long-str FROM str COUNT float-item
      *> COUNT IN field must not have P in PIC.
           XML GENERATE long-str FROM str COUNT pic-p-item

      *> ENCODING codepage must be unsigned integer.
      *> If receiving area is national, codepage must be 1200.
      *> "        "       "   alphanumeric, codepage must be 1208 or EBCDIC
      *>   page supported with XML.

      *> WITH ATTRIBUTES, generated immediate children must be
      *>  * elementary
      *>  * be non-FILLER
      *>  * not be OCCURS
      *>  * not be subject of a TYPE phrase.
           XML GENERATE long-str FROM with-attrs-does-nothing
               WITH ATTRIBUTES
               TYPE OF with-attrs-child IS ELEMENT

      *> NAMESPACE must be a valid URI.
           XML GENERATE long-str FROM str NAMESPACE "<>"
      *> NAMESPACE and -PREFIX must be alphanumeric or national.
           XML GENERATE long-str FROM str
              NAMESPACE bool-item NAMESPACE-PREFIX bool-item
      *>   "        "     "    may not be figurative constants.
           XML GENERATE long-str FROM str
               NAMESPACE SPACES NAMESPACE-PREFIX QUOTES
      *> NAMESPACE-PREFIX must be a valid XML name.
           XML GENERATE long-str FROM str
               NAMESPACE "http://www.w3.org/xml" NAMESPACE-PREFIX X"00"

      *> NAME items must reference input record or its children.
           XML GENERATE long-str FROM rec
               NAME OF child-1 IS "c1", long-str IS "c2", rec IS "r"
      *> NAME items cannot be reference modified or subscripted.
           XML GENERATE long-str FROM rec
               NAME OF child-1 (1:2) IS "c1"
      *> NAME items may not be ignored by the statement.
           XML GENERATE long-str FROM rec
               NAME OF child-1a IS "c1a"
      *> NAME literals must be valid XML names.
           XML GENERATE long-str FROM rec
               NAME OF child-1 IS X"00"

      *> TYPE items must be elementary and children of input record.
           XML GENERATE long-str FROM rec
               TYPE OF child-3 IS ELEMENT, long-str IS CONTENT,
                   rec IS CONTENT
      *> TYPE items cannot be ref-mod'd or subscripted.
           XML GENERATE long-str FROM rec
               TYPE OF child-1 (1:3) IS ATTRIBUTE,
                   child-3-1 (1) IS CONTENT
      *> TYPE items may not be ignored by the statement
           XML GENERATE long-str FROM rec
               TYPE OF child-1a IS ELEMENT
      *> TYPE ATTRIBUTE items must satisfy the conditions for WITH
      *>  ATTRIBUTES. (Covered by the above.)

      *> SUPPRESS WHEN items must be:
      *>  * elementary
      *>  * not ignored
      *>  * child of input record.
           XML GENERATE long-str FROM rec
               SUPPRESS child-3 WHEN SPACES, child-1a WHEN SPACES,
                   rec WHEN SPACES
      *> All SUPPRESS items must not be functions
           XML GENERATE long-str FROM rec
               SUPPRESS FUNCTION CHAR(5) WHEN SPACE
      *> All SUPPRESS items must not be ref-mod'd or subscripted.
           XML GENERATE long-str FROM rec
               SUPPRESS child-1 (1:3) WHEN ZERO,
                   child-3-1 (1) WHEN SPACES
      *> If non-WHEN SUPPRESS items may be groups. (No error message here.)
           XML GENERATE long-str FROM rec SUPPRESS child-3
      *> If SUPPRESS WHEN ZEROES, item is not DISPLAY-1.
      *> If SUPPRESS WHEN SPACES, item must be USAGE DISPLAY, DISPLAY-1 or
      *>   NATIONAL
           XML GENERATE long-str FROM bool-area
               SUPPRESS bool-item WHEN SPACES
      *> If SUPPRESS WHEN LOW-/HIGH-VALUES, item must be USAGE DISPLAY or
      *>   NATIONAL. If item is a zoned/national decimal item, it must be
      *>   an integer.
           XML GENERATE long-str FROM bool-area
               SUPPRESS bool-item WHEN LOW-VALUES,
                   zoned-decimal WHEN HIGH-VALUE
      *> (For generic WHEN phrases, invalid items above are ignored.)
           GOBACK.
