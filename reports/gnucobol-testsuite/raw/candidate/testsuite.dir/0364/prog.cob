
       IDENTIFICATION DIVISION.
       PROGRAM-ID. prog.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  str.
           03  str-1  PIC XX BASED.
       01  str-2 BASED.
           03  str-2a PIC X SYNC.
           03  str-2b PIC 9.
       01  one        PIC XX BASED.
       77  seven      PIC 9 BASED.
       77  var        PIC 9.
       77  ptr        USAGE POINTER.
       SCREEN-STORAGE SECTION.
       01  scrn      BASED.
           03  scrn-field pic x.

       PROCEDURE DIVISION.
           ALLOCATE one
           ALLOCATE seven INITIALIZED
           ALLOCATE seven CHARACTERS
           ALLOCATE seven CHARACTERS RETURNING ptr
           ALLOCATE 1 + 2 * 3 CHARACTERS RETURNING ptr
           ALLOCATE 1 + one * 3 CHARACTERS RETURNING ptr
           ALLOCATE one   CHARACTERS RETURNING ptr
           ALLOCATE seven CHARACTERS INITIALIZED RETURNING ptr
           ALLOCATE var
           ALLOCATE 9 CHARACTERS INITIALIZED TO 9 RETURNING ptr.
           FREE var
           FREE ADDRESS OF var
           FREE one
           FREE ADDRESS OF seven
           MOVE ADDRESS OF seven TO ptr
           FREE ptr
           GOBACK.
